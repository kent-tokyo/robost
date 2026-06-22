// ML-based text region detection.
//
// Two backends are available:
//   - ONNX (feature = "ml"): DBNet-family models via ONNX Runtime.
//   - Vision (feature = "ml-vision", macOS only): Apple Vision framework.
//
// The shared types (DetectionError, TextBox) are always compiled when either
// feature is active; backend-specific code is gated individually.

use thiserror::Error;

// ---- Shared types -----------------------------------------------------------

#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("failed to load model: {0}")]
    ModelLoad(String),
    #[error("inference failed: {0}")]
    Inference(String),
}

pub type Result<T> = std::result::Result<T, DetectionError>;

/// A detected text region (position only; see [`vision::VisionTextBox`] for text content).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextBox {
    /// Bounding rectangle in screen-global coordinates.
    pub bounds: crate::types::Rect,
    /// Mean probability score over the detected pixels in this region.
    pub confidence: f32,
}

// ---- ONNX backend (feature = "ml") -----------------------------------------
//
// Compatible with DBNet-family models (PP-OCRv4 detection, DBNet-MobileNetV3, etc.).
// The model must accept float32 input [1, 3, H, W] (ImageNet-normalised) and return
// a probability map [1, 1, H, W].
//
// ONNX Runtime is downloaded automatically by the `ort` crate at build time.
//
// # Getting a model
//
// Download the PP-OCRv4 detection model and convert to ONNX:
//
// ```sh
// pip install paddle2onnx paddlepaddle
// wget https://paddleocr.bj.bcebos.com/PP-OCRv4/chinese/ch_PP-OCRv4_det_infer.tar
// tar xf ch_PP-OCRv4_det_infer.tar
// paddle2onnx --model_dir ch_PP-OCRv4_det_infer \
//             --model_filename inference.pdmodel \
//             --params_filename inference.pdiparams \
//             --save_file ppocr_det.onnx --opset_version 11
// ```

#[cfg(feature = "ml")]
use {
    crate::types::{Rect, ScreenPoint},
    image::RgbaImage,
    ndarray::Array4,
    ort::{
        session::Session,
        value::{TensorRef, ValueType},
    },
    std::path::Path,
    tracing::instrument,
};

// ImageNet normalisation constants used by PaddleOCR and most DBNet models.
#[cfg(feature = "ml")]
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
#[cfg(feature = "ml")]
const STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Minimum side length (in model-space pixels) to accept a detected component.
#[cfg(feature = "ml")]
const MIN_SIDE: u32 = 3;

/// ONNX-based text region detector.
///
/// Wraps an ONNX Runtime session for DBNet-family text detection models.
/// After detection, combine with [`crate::ocr::OcrEngine::extract_text_in_region`]
/// to read the text at each detected position.
///
/// ## Why `&mut self` on [`detect`]?
///
/// ONNX Runtime requires exclusive access during inference, so [`Session::run`]
/// takes `&mut self`.
#[cfg(feature = "ml")]
pub struct TextDetector {
    session: Session,
    /// Fixed model input size (W, H). `None` = dynamic (padded to multiple of 32).
    input_size: Option<(u32, u32)>,
    threshold: f32,
    min_area: u32,
}

#[cfg(feature = "ml")]
impl TextDetector {
    /// Load a text detection ONNX model from `path`.
    pub fn from_model_path(path: impl AsRef<Path>) -> Result<Self> {
        let session = Session::builder()
            .map_err(|e| DetectionError::ModelLoad(e.to_string()))?
            .commit_from_file(path)
            .map_err(|e| DetectionError::ModelLoad(e.to_string()))?;
        let input_size = fixed_input_size(&session);
        Ok(Self {
            session,
            input_size,
            threshold: 0.3,
            min_area: 10,
        })
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_min_area(mut self, min_area: u32) -> Self {
        self.min_area = min_area;
        self
    }

    #[instrument(name = "text_detect", skip(self, image))]
    pub fn detect(&mut self, image: &RgbaImage, origin: ScreenPoint) -> Result<Vec<TextBox>> {
        let orig_w = image.width();
        let orig_h = image.height();

        let (input_w, input_h) = self.input_size.unwrap_or_else(|| {
            let w = ((orig_w + 31) / 32 * 32).min(1920);
            let h = ((orig_h + 31) / 32 * 32).min(1920);
            (w, h)
        });

        let array = preprocess(image, input_w, input_h);
        let tensor = TensorRef::from_array_view(array.view())
            .map_err(|e| DetectionError::Inference(e.to_string()))?;
        let outputs = self
            .session
            .run(ort::inputs![tensor])
            .map_err(|e| DetectionError::Inference(e.to_string()))?;

        let (shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| DetectionError::Inference(e.to_string()))?;

        if shape.len() < 4 {
            return Err(DetectionError::Inference(format!(
                "unexpected output rank {} (expected 4)",
                shape.len()
            )));
        }
        let map_h = shape[2] as u32;
        let map_w = shape[3] as u32;
        let scale_x = orig_w as f32 / map_w as f32;
        let scale_y = orig_h as f32 / map_h as f32;

        tracing::debug!(
            map_w,
            map_h,
            scale_x,
            scale_y,
            threshold = self.threshold,
            "text detection map"
        );

        let n = (map_h * map_w) as usize;
        let mut mask = vec![false; n];
        let mut conf = vec![0.0f32; n];
        for i in 0..n {
            conf[i] = data[i];
            mask[i] = data[i] >= self.threshold;
        }

        Ok(extract_boxes(
            &mask,
            &conf,
            map_w,
            map_h,
            scale_x,
            scale_y,
            origin,
            self.min_area,
        ))
    }

    pub fn detect_in_region(
        &mut self,
        image: &RgbaImage,
        origin: ScreenPoint,
        region: Rect,
    ) -> Result<Vec<TextBox>> {
        let x0 = region.x.max(0) as u32;
        let y0 = region.y.max(0) as u32;
        let x1 = (region.x + region.width as i32).min(image.width() as i32) as u32;
        let y1 = (region.y + region.height as i32).min(image.height() as i32) as u32;
        let w = x1.saturating_sub(x0);
        let h = y1.saturating_sub(y0);
        let roi = image::imageops::crop_imm(image, x0, y0, w, h).to_image();
        let roi_origin = ScreenPoint {
            x: origin.x + x0 as i32,
            y: origin.y + y0 as i32,
        };
        self.detect(&roi, roi_origin)
    }
}

// ---- ONNX helpers -----------------------------------------------------------

#[cfg(feature = "ml")]
fn fixed_input_size(session: &Session) -> Option<(u32, u32)> {
    let input = session.inputs().first()?;
    if let ValueType::Tensor { shape, .. } = input.dtype() {
        if shape.len() == 4 && shape[2] > 0 && shape[3] > 0 {
            return Some((shape[3] as u32, shape[2] as u32));
        }
    }
    None
}

#[cfg(feature = "ml")]
fn preprocess(image: &RgbaImage, input_w: u32, input_h: u32) -> Array4<f32> {
    use image::imageops;
    let resized = imageops::resize(image, input_w, input_h, imageops::FilterType::Lanczos3);
    let mut tensor = Array4::<f32>::zeros((1, 3, input_h as usize, input_w as usize));
    for y in 0..input_h as usize {
        for x in 0..input_w as usize {
            let p = resized.get_pixel(x as u32, y as u32);
            for c in 0..3usize {
                tensor[[0, c, y, x]] = (p[c] as f32 / 255.0 - MEAN[c]) / STD[c];
            }
        }
    }
    tensor
}

#[cfg(feature = "ml")]
fn extract_boxes(
    mask: &[bool],
    conf: &[f32],
    w: u32,
    h: u32,
    scale_x: f32,
    scale_y: f32,
    origin: crate::types::ScreenPoint,
    min_area: u32,
) -> Vec<TextBox> {
    let mut visited = vec![false; mask.len()];
    let mut boxes = Vec::new();

    for sy in 0..h {
        for sx in 0..w {
            let sidx = (sy * w + sx) as usize;
            if !mask[sidx] || visited[sidx] {
                continue;
            }

            let mut queue = std::collections::VecDeque::new();
            queue.push_back((sx, sy));
            visited[sidx] = true;

            let (mut min_x, mut max_x) = (sx, sx);
            let (mut min_y, mut max_y) = (sy, sy);
            let mut sum_conf = 0.0f32;
            let mut count = 0u32;

            while let Some((cx, cy)) = queue.pop_front() {
                sum_conf += conf[(cy * w + cx) as usize];
                count += 1;
                min_x = min_x.min(cx);
                max_x = max_x.max(cx);
                min_y = min_y.min(cy);
                max_y = max_y.max(cy);

                for (nx, ny) in four_neighbors(cx, cy, w, h) {
                    let nidx = (ny * w + nx) as usize;
                    if mask[nidx] && !visited[nidx] {
                        visited[nidx] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }

            let box_w = max_x - min_x + 1;
            let box_h = max_y - min_y + 1;
            if box_w < MIN_SIDE || box_h < MIN_SIDE || box_w * box_h < min_area {
                continue;
            }

            let bx = (min_x as f32 * scale_x).round() as i32 + origin.x;
            let by = (min_y as f32 * scale_y).round() as i32 + origin.y;
            let bw = (box_w as f32 * scale_x).round() as u32;
            let bh = (box_h as f32 * scale_y).round() as u32;

            boxes.push(TextBox {
                bounds: crate::types::Rect {
                    x: bx,
                    y: by,
                    width: bw,
                    height: bh,
                },
                confidence: sum_conf / count as f32,
            });
        }
    }

    boxes
}

#[cfg(feature = "ml")]
fn four_neighbors(x: u32, y: u32, w: u32, h: u32) -> impl Iterator<Item = (u32, u32)> {
    let mut v = [None; 4];
    if x > 0 {
        v[0] = Some((x - 1, y));
    }
    if x + 1 < w {
        v[1] = Some((x + 1, y));
    }
    if y > 0 {
        v[2] = Some((x, y - 1));
    }
    if y + 1 < h {
        v[3] = Some((x, y + 1));
    }
    v.into_iter().flatten()
}

// ---- ONNX tests -------------------------------------------------------------

#[cfg(all(test, feature = "ml"))]
mod tests {
    use super::*;
    use crate::types::ScreenPoint;
    use image::{Rgba, RgbaImage};

    fn white_image(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn preprocess_output_shape() {
        let img = white_image(100, 80);
        let t = preprocess(&img, 64, 64);
        assert_eq!(t.shape(), &[1, 3, 64, 64]);
    }

    #[test]
    fn preprocess_normalises_white() {
        let img = white_image(4, 4);
        let t = preprocess(&img, 4, 4);
        let val = t[[0, 0, 0, 0]];
        assert!((val - 2.249).abs() < 0.01, "expected ~2.249, got {val}");
    }

    #[test]
    fn preprocess_normalises_black() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([0, 0, 0, 255]));
        let t = preprocess(&img, 4, 4);
        let val = t[[0, 0, 0, 0]];
        assert!((val - (-2.118)).abs() < 0.01, "expected ~-2.118, got {val}");
    }

    #[test]
    fn extract_boxes_single_blob() {
        let w = 10u32;
        let h = 10u32;
        let mut mask = vec![false; (w * h) as usize];
        let conf = vec![0.9f32; (w * h) as usize];
        for y in 2..6u32 {
            for x in 2..6u32 {
                mask[(y * w + x) as usize] = true;
            }
        }
        let origin = ScreenPoint { x: 0, y: 0 };
        let boxes = extract_boxes(&mask, &conf, w, h, 1.0, 1.0, origin, 1);
        assert_eq!(boxes.len(), 1);
        let b = &boxes[0].bounds;
        assert_eq!((b.x, b.y, b.width, b.height), (2, 2, 4, 4));
    }

    #[test]
    fn extract_boxes_filters_small_side() {
        let w = 10u32;
        let h = 10u32;
        let mut mask = vec![false; (w * h) as usize];
        let conf = vec![0.9f32; (w * h) as usize];
        for x in 0..10u32 {
            mask[x as usize] = true;
        }
        let origin = ScreenPoint { x: 0, y: 0 };
        assert!(extract_boxes(&mask, &conf, w, h, 1.0, 1.0, origin, 1).is_empty());
    }

    #[test]
    fn extract_boxes_applies_origin_offset() {
        let w = 10u32;
        let h = 10u32;
        let mut mask = vec![false; (w * h) as usize];
        let conf = vec![0.9f32; (w * h) as usize];
        for y in 0..5u32 {
            for x in 0..5u32 {
                mask[(y * w + x) as usize] = true;
            }
        }
        let origin = ScreenPoint { x: 100, y: 200 };
        let boxes = extract_boxes(&mask, &conf, w, h, 1.0, 1.0, origin, 1);
        assert_eq!(boxes.len(), 1);
        assert_eq!((boxes[0].bounds.x, boxes[0].bounds.y), (100, 200));
    }

    #[test]
    fn extract_boxes_two_blobs() {
        let w = 20u32;
        let h = 10u32;
        let mut mask = vec![false; (w * h) as usize];
        let conf = vec![0.8f32; (w * h) as usize];
        for y in 0..5u32 {
            for x in 0..4u32 {
                mask[(y * w + x) as usize] = true;
            }
        }
        for y in 0..5u32 {
            for x in 15..19u32 {
                mask[(y * w + x) as usize] = true;
            }
        }
        let origin = ScreenPoint { x: 0, y: 0 };
        assert_eq!(
            extract_boxes(&mask, &conf, w, h, 1.0, 1.0, origin, 1).len(),
            2
        );
    }

    #[test]
    fn extract_boxes_scales_to_original_coords() {
        let w = 8u32;
        let h = 8u32;
        let mut mask = vec![false; (w * h) as usize];
        let conf = vec![0.9f32; (w * h) as usize];
        for y in 1..4u32 {
            for x in 1..4u32 {
                mask[(y * w + x) as usize] = true;
            }
        }
        let origin = ScreenPoint { x: 0, y: 0 };
        let boxes = extract_boxes(&mask, &conf, w, h, 2.0, 2.0, origin, 1);
        assert_eq!(boxes.len(), 1);
        let b = &boxes[0].bounds;
        assert_eq!((b.x, b.y, b.width, b.height), (2, 2, 6, 6));
    }

    #[test]
    fn extract_boxes_confidence_is_mean() {
        let w = 8u32;
        let h = 8u32;
        let mut mask = vec![false; (w * h) as usize];
        let mut conf = vec![0.0f32; (w * h) as usize];
        for y in 1..4u32 {
            for x in 1..4u32 {
                mask[(y * w + x) as usize] = true;
            }
        }
        let corners = [(1u32, 1u32), (3, 1), (1, 3), (3, 3)];
        let corner_values = [0.4f32, 0.6, 0.8, 1.0];
        for (i, &(x, y)) in corners.iter().enumerate() {
            conf[(y * w + x) as usize] = corner_values[i];
        }
        let origin = ScreenPoint { x: 0, y: 0 };
        let boxes = extract_boxes(&mask, &conf, w, h, 1.0, 1.0, origin, 1);
        assert_eq!(boxes.len(), 1);
        let expected = corner_values.iter().sum::<f32>() / 9.0;
        assert!((boxes[0].confidence - expected).abs() < 1e-5);
    }
}

// ---- macOS Vision framework backend -----------------------------------------

// Link Vision.framework so that class!(VNRecognizeTextRequest) resolves at runtime.
#[cfg(target_os = "macos")]
#[link(name = "Vision", kind = "framework")]
extern "C" {}

/// macOS Vision framework backend — detects **and** recognises text in one pass.
///
/// Requires macOS 10.15+. Enable with the `ml-vision` Cargo feature.
#[cfg(all(feature = "ml-vision", target_os = "macos"))]
pub mod vision {
    use super::DetectionError;
    use crate::types::{Rect, ScreenPoint};
    use image::RgbaImage;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, NSObject};
    use objc2::{extern_class, msg_send, AnyThread};
    use objc2_core_foundation::CGRect;
    use objc2_core_graphics::{
        CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage,
        CGImageAlphaInfo,
    };
    use objc2_foundation::{NSArray, NSDictionary, NSError, NSString};
    use std::ffi::c_void;
    use tracing::instrument;

    pub type Result<T> = std::result::Result<T, DetectionError>;

    // ---- extern Vision types ------------------------------------------------

    extern_class!(
        #[unsafe(super(NSObject))]
        #[derive(Debug, PartialEq, Eq, Hash)]
        struct VNRequest;
    );

    extern_class!(
        #[unsafe(super(VNRequest, NSObject))]
        #[derive(Debug, PartialEq, Eq, Hash)]
        struct VNRecognizeTextRequest;
    );

    extern_class!(
        #[unsafe(super(NSObject))]
        #[derive(Debug, PartialEq, Eq, Hash)]
        struct VNImageRequestHandler;
    );

    // ---- Public types -------------------------------------------------------

    /// Recognition accuracy level.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum RecognitionLevel {
        /// Accurate recognition (VNRequestTextRecognitionLevelAccurate = 0, default).
        #[default]
        Accurate,
        /// Fast recognition (VNRequestTextRecognitionLevelFast = 1).
        Fast,
    }

    /// A text region detected by the macOS Vision framework.
    ///
    /// Unlike [`TextBox`](super::TextBox), this includes the recognized text string.
    ///
    /// ## Coordinate space
    ///
    /// `bounds` is in **image-pixel** coordinates offset by the `origin` passed to
    /// [`VisionDetector::detect`]. No Retina scaling is applied — that is the
    /// caller's responsibility when converting to screen coordinates.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct VisionTextBox {
        /// Bounding rectangle in image-pixel coordinates (offset by origin).
        pub bounds: Rect,
        /// Recognition confidence in \[0, 1\].
        pub confidence: f32,
        /// Recognized text string.
        pub text: String,
    }

    impl VisionTextBox {
        /// Drop the text field and return a plain [`TextBox`](super::TextBox).
        pub fn into_text_box(self) -> super::TextBox {
            super::TextBox {
                bounds: self.bounds,
                confidence: self.confidence,
            }
        }
    }

    // ---- VisionDetector -----------------------------------------------------

    /// macOS Vision framework text detector.
    ///
    /// Wraps `VNRecognizeTextRequest` to detect and recognise text in one pass.
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "ml-vision", target_os = "macos"))]
    /// # {
    /// use robost_vision::ml::vision::VisionDetector;
    /// use robost_vision::{load_rgba, ScreenPoint};
    ///
    /// let detector = VisionDetector::new();
    /// let image = load_rgba("screenshot.png").unwrap();
    /// let boxes = detector.detect(&image, ScreenPoint { x: 0, y: 0 }).unwrap();
    /// for b in &boxes {
    ///     println!("{:?} → {:?}", b.bounds, b.text);
    /// }
    /// # }
    /// ```
    pub struct VisionDetector {
        languages: Vec<String>,
        level: RecognitionLevel,
    }

    impl Default for VisionDetector {
        fn default() -> Self {
            Self::new()
        }
    }

    impl VisionDetector {
        /// Create a detector with Japanese + English at accurate mode.
        pub fn new() -> Self {
            Self {
                languages: vec!["ja-JP".to_string(), "en-US".to_string()],
                level: RecognitionLevel::Accurate,
            }
        }

        /// Set the recognition languages (BCP 47 tags, e.g. `"ja-JP"`, `"zh-Hans"`).
        pub fn with_languages(
            mut self,
            langs: impl IntoIterator<Item = impl Into<String>>,
        ) -> Self {
            self.languages = langs.into_iter().map(Into::into).collect();
            self
        }

        /// Set recognition accuracy (default: [`RecognitionLevel::Accurate`]).
        pub fn with_recognition_level(mut self, level: RecognitionLevel) -> Self {
            self.level = level;
            self
        }

        /// Detect and recognise all text in `image`.
        ///
        /// Returns bounding boxes in image-pixel coordinates offset by `origin`.
        #[instrument(name = "vision_detect", skip(self, image))]
        pub fn detect(&self, image: &RgbaImage, origin: ScreenPoint) -> Result<Vec<VisionTextBox>> {
            unsafe { self.run_vision(image, origin) }
        }

        /// Detect text only within `region` of `image`.
        pub fn detect_in_region(
            &self,
            image: &RgbaImage,
            origin: ScreenPoint,
            region: Rect,
        ) -> Result<Vec<VisionTextBox>> {
            let x0 = region.x.max(0) as u32;
            let y0 = region.y.max(0) as u32;
            let x1 = (region.x + region.width as i32).min(image.width() as i32) as u32;
            let y1 = (region.y + region.height as i32).min(image.height() as i32) as u32;
            let w = x1.saturating_sub(x0);
            let h = y1.saturating_sub(y0);
            let roi = image::imageops::crop_imm(image, x0, y0, w, h).to_image();
            let roi_origin = ScreenPoint {
                x: origin.x + x0 as i32,
                y: origin.y + y0 as i32,
            };
            self.detect(&roi, roi_origin)
        }

        /// Find the first text region whose text contains `query`.
        pub fn find_text(
            &self,
            image: &RgbaImage,
            origin: ScreenPoint,
            query: &str,
        ) -> Result<Option<VisionTextBox>> {
            Ok(self
                .detect(image, origin)?
                .into_iter()
                .find(|b| b.text.contains(query)))
        }

        unsafe fn run_vision(
            &self,
            image: &RgbaImage,
            origin: ScreenPoint,
        ) -> Result<Vec<VisionTextBox>> {
            let w = image.width();
            let h = image.height();

            // 1. Build CGImage from RGBA pixel data.
            //    `image` outlives `cg_image` since both are stack-bound here.
            let color_space = CGColorSpace::new_device_rgb().ok_or_else(|| {
                DetectionError::Inference("CGColorSpaceCreateDeviceRGB failed".into())
            })?;
            let provider = CGDataProvider::with_data(
                std::ptr::null_mut(),
                image.as_raw().as_ptr() as *const c_void,
                (w * h * 4) as usize,
                None, // no release callback — data borrowed from `image`
            )
            .ok_or_else(|| {
                DetectionError::Inference("CGDataProviderCreateWithData failed".into())
            })?;
            // kCGImageAlphaLast (3) = non-premultiplied RGBA, alpha last
            let bitmap_info = CGBitmapInfo(CGImageAlphaInfo::Last.0 as u32);
            let cg_image = CGImage::new(
                w as usize,
                h as usize,
                8,
                32,
                (w * 4) as usize,
                Some(&color_space),
                bitmap_info,
                Some(&provider),
                std::ptr::null(),
                false,
                CGColorRenderingIntent::RenderingIntentDefault,
            )
            .ok_or_else(|| DetectionError::Inference("CGImageCreate failed".into()))?;

            // 2. Create VNImageRequestHandler
            let handler_alloc = VNImageRequestHandler::alloc();
            let opts: Retained<NSDictionary<NSString, NSObject>> =
                NSDictionary::<NSString, NSObject>::from_slices::<NSString>(&[], &[]);
            let handler: Retained<VNImageRequestHandler> = msg_send![
                handler_alloc,
                initWithCGImage: &*cg_image,
                options: &*opts,
            ];

            // 3. Create VNRecognizeTextRequest (plain init; read .results after perform)
            let request: Retained<VNRecognizeTextRequest> =
                msg_send![VNRecognizeTextRequest::alloc(), init];

            // 4. Configure
            // VNRequestTextRecognitionLevelAccurate = 0, VNRequestTextRecognitionLevelFast = 1
            let level_raw: i64 = match self.level {
                RecognitionLevel::Accurate => 0,
                RecognitionLevel::Fast => 1,
            };
            let _: () = msg_send![&*request, setRecognitionLevel: level_raw];
            let _: () = msg_send![&*request, setUsesLanguageCorrection: false];

            let lang_strs: Vec<Retained<NSString>> = self
                .languages
                .iter()
                .map(|s| NSString::from_str(s))
                .collect();
            let lang_refs: Vec<&NSString> = lang_strs.iter().map(|s| s.as_ref()).collect();
            let langs: Retained<NSArray<NSString>> = NSArray::from_slice(&lang_refs);
            let _: () = msg_send![&*request, setRecognitionLanguages: &*langs];

            // 5. Perform
            let req_any: &AnyObject = &*request;
            let req_array: Retained<NSArray<AnyObject>> = NSArray::from_slice(&[req_any]);
            let mut error: Option<Retained<NSError>> = None;
            let success: bool =
                msg_send![&*handler, performRequests: &*req_array, error: &mut error];

            if !success {
                let msg = error
                    .as_deref()
                    .map(|e| {
                        let desc: Retained<NSString> = msg_send![e, localizedDescription];
                        desc.to_string()
                    })
                    .unwrap_or_else(|| "performRequests:error: failed".into());
                return Err(DetectionError::Inference(msg));
            }

            // 6. Extract results
            // NSArray::iter() yields Retained<T>, not &T — use &*item in msg_send!
            let results: Option<Retained<NSArray<AnyObject>>> = msg_send![&*request, results];
            let results = match results {
                Some(r) => r,
                None => return Ok(vec![]),
            };

            let img_w = w as f64;
            let img_h = h as f64;
            let mut boxes = Vec::with_capacity(results.len());

            for obs in results.iter() {
                // obs: Retained<AnyObject> — dereference for msg_send receiver
                let bbox: CGRect = msg_send![&*obs, boundingBox];

                let candidates: Retained<NSArray<AnyObject>> =
                    msg_send![&*obs, topCandidates: 1usize];
                // iter() yields Retained<AnyObject>
                let Some(candidate) = candidates.iter().next() else {
                    continue;
                };

                let text_ns: Retained<NSString> = msg_send![&*candidate, string];
                let text = text_ns.to_string();
                let confidence: f32 = msg_send![&*candidate, confidence];

                // Convert Vision normalized coords (bottom-left origin) → image pixels (top-left)
                let bx = (bbox.origin.x * img_w).round() as i32 + origin.x;
                let by =
                    ((1.0 - bbox.origin.y - bbox.size.height) * img_h).round() as i32 + origin.y;
                let bw = (bbox.size.width * img_w).round() as u32;
                let bh = (bbox.size.height * img_h).round() as u32;

                tracing::debug!(bx, by, bw, bh, confidence, text = %text, "vision text box");

                boxes.push(VisionTextBox {
                    bounds: Rect {
                        x: bx,
                        y: by,
                        width: bw,
                        height: bh,
                    },
                    confidence,
                    text,
                });
            }

            Ok(boxes)
        }
    }
}
