/// ML-based object detection — requires the `ml` feature and an ONNX model.
///
/// This is a Phase 2 stub. The API is stable so callers can be wired up now,
/// but `detect()` always returns `MlError::NotImplemented` until the ort
/// inference backend is filled in.
use image::RgbaImage;
use thiserror::Error;

use crate::types::Rect;

#[derive(Debug, Error)]
pub enum MlError {
    #[error("ML detection not yet implemented (planned for Phase 2)")]
    NotImplemented,
    #[error("model load error: {0}")]
    ModelLoad(String),
    #[error("inference error: {0}")]
    Inference(String),
}

/// A single detected object.
#[derive(Debug, Clone)]
pub struct Detection {
    /// Class label predicted by the model.
    pub label: String,
    /// Confidence score in [0, 1].
    pub score: f32,
    /// Bounding box in the image's local coordinates.
    pub bbox: Rect,
}

/// Runs an ONNX model against a captured image to detect objects.
///
/// Construction is cheap; the model is not loaded until the first `detect` call
/// (Phase 2 will cache the session across calls).
pub struct MlDetector {
    pub model_path: String,
    pub threshold: f32,
}

impl MlDetector {
    pub fn new(model_path: impl Into<String>, threshold: f32) -> Self {
        Self {
            model_path: model_path.into(),
            threshold,
        }
    }

    /// Detect objects in `img`.
    ///
    /// Returns `Err(MlError::NotImplemented)` until the Phase 2 ort backend
    /// is wired up.
    pub fn detect(&self, _img: &RgbaImage) -> Result<Vec<Detection>, MlError> {
        Err(MlError::NotImplemented)
    }
}
