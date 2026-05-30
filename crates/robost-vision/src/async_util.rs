use crate::prepared::PreparedTemplate;
use crate::template_match::{MatchError, MatchResult, Result, TemplateMatcher};
use crate::types::{MaskRegion, ScreenPoint};
use image::RgbaImage;
use std::sync::Arc;
use std::time::Duration;

impl TemplateMatcher {
    /// Async version of [`find`] — runs NCC on the blocking thread pool.
    ///
    /// Takes owned images so they can be moved into `spawn_blocking`.
    pub async fn find_async(
        &self,
        haystack: RgbaImage,
        template: RgbaImage,
        origin: ScreenPoint,
    ) -> Result<MatchResult> {
        let m = self.clone();
        tokio::task::spawn_blocking(move || m.find(&haystack, &template, origin))
            .await
            .unwrap_or_else(|e| panic!("template match task panicked: {e}"))
    }

    /// Async version of [`find_with_masks`].
    pub async fn find_with_masks_async(
        &self,
        haystack: RgbaImage,
        template: RgbaImage,
        origin: ScreenPoint,
        masks: Vec<MaskRegion>,
    ) -> Result<MatchResult> {
        let m = self.clone();
        tokio::task::spawn_blocking(move || m.find_with_masks(&haystack, &template, origin, &masks))
            .await
            .unwrap_or_else(|e| panic!("template match task panicked: {e}"))
    }

    /// Async version of [`find_all`].
    pub async fn find_all_async(
        &self,
        haystack: RgbaImage,
        template: RgbaImage,
        origin: ScreenPoint,
    ) -> Vec<MatchResult> {
        let m = self.clone();
        tokio::task::spawn_blocking(move || m.find_all(&haystack, &template, origin))
            .await
            .unwrap_or_else(|e| panic!("find_all task panicked: {e}"))
    }

    /// Poll `capture_fn` until the template is found or `timeout` elapses.
    ///
    /// `capture_fn` is called on the blocking thread pool each iteration.
    /// Sleeps for `interval` between attempts.
    /// Returns `Err(MatchError::Timeout)` when the deadline is exceeded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use robost_vision::{TemplateMatcher, ScreenPoint};
    /// # use image::open;
    /// # use std::time::Duration;
    /// # async fn example() -> robost_vision::template_match::Result<()> {
    /// let template = open("button.png").unwrap().into_rgba8();
    /// let matcher = TemplateMatcher::default();
    ///
    /// let result = matcher
    ///     .wait_for_match(
    ///         || open("screenshot.png").unwrap().into_rgba8(),
    ///         template,
    ///         ScreenPoint { x: 0, y: 0 },
    ///         Duration::from_secs(10),
    ///         Duration::from_millis(500),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_match<F>(
        &self,
        capture_fn: F,
        template: RgbaImage,
        origin: ScreenPoint,
        timeout: Duration,
        interval: Duration,
    ) -> Result<MatchResult>
    where
        F: Fn() -> RgbaImage + Send + Sync + 'static,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        let capture_fn = Arc::new(capture_fn);
        let template = Arc::new(template);

        loop {
            let cf = Arc::clone(&capture_fn);
            let tmpl = Arc::clone(&template);
            let m = self.clone();

            let result = tokio::task::spawn_blocking(move || {
                let haystack = cf();
                m.find(&haystack, &tmpl, origin)
            })
            .await
            .unwrap_or_else(|e| panic!("capture task panicked: {e}"));

            if let Ok(r) = result {
                return Ok(r);
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(MatchError::Timeout);
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Like [`wait_for_match`] but reuses the pre-computed grayscale in `tmpl`,
    /// avoiding the RGB→gray conversion on every poll iteration.
    ///
    /// Pass `Arc<PreparedTemplate>` so the template can be moved into each
    /// `spawn_blocking` call without cloning the image data.
    pub async fn wait_for_match_prepared<F>(
        &self,
        capture_fn: F,
        tmpl: Arc<PreparedTemplate>,
        origin: ScreenPoint,
        timeout: Duration,
        interval: Duration,
    ) -> Result<MatchResult>
    where
        F: Fn() -> RgbaImage + Send + Sync + 'static,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        let capture_fn = Arc::new(capture_fn);

        loop {
            let cf = Arc::clone(&capture_fn);
            let t = Arc::clone(&tmpl);
            let m = self.clone();

            let result = tokio::task::spawn_blocking(move || {
                let haystack = cf();
                m.find_prepared(&haystack, &t, origin)
            })
            .await
            .unwrap_or_else(|e| panic!("capture task panicked: {e}"));

            if let Ok(r) = result {
                return Ok(r);
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(MatchError::Timeout);
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Poll `capture_fn` until **any** template in `templates` is found above the
    /// configured threshold, or until `timeout` elapses.
    ///
    /// Returns `(template_index, MatchResult)` for the highest-scoring template on the
    /// successful frame. Wrap templates in `Arc` to avoid cloning large image data each
    /// iteration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use robost_vision::{TemplateMatcher, ScreenPoint, load_rgba};
    /// # use std::{sync::Arc, time::Duration};
    /// # async fn example() -> robost_vision::template_match::Result<()> {
    /// let templates = Arc::new(vec![
    ///     load_rgba("success.png").unwrap(),
    ///     load_rgba("error.png").unwrap(),
    /// ]);
    /// let matcher = TemplateMatcher::default();
    /// let (idx, result) = matcher
    ///     .wait_for_any_match(
    ///         || load_rgba("screenshot.png").unwrap(),
    ///         templates,
    ///         ScreenPoint { x: 0, y: 0 },
    ///         Duration::from_secs(30),
    ///         Duration::from_millis(500),
    ///     )
    ///     .await?;
    /// println!("template {idx} matched at {:?}", result.center);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_any_match<F>(
        &self,
        capture_fn: F,
        templates: Arc<Vec<RgbaImage>>,
        origin: ScreenPoint,
        timeout: Duration,
        interval: Duration,
    ) -> Result<(usize, MatchResult)>
    where
        F: Fn() -> RgbaImage + Send + Sync + 'static,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        let capture_fn = Arc::new(capture_fn);

        loop {
            let cf = Arc::clone(&capture_fn);
            let tmpls = Arc::clone(&templates);
            let m = self.clone();

            let result = tokio::task::spawn_blocking(move || {
                let haystack = cf();
                m.find_best_of(&haystack, &tmpls, origin)
            })
            .await
            .unwrap_or_else(|e| panic!("capture task panicked: {e}"));

            if let Some(hit) = result {
                return Ok(hit);
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(MatchError::Timeout);
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Poll `capture_fn` until `template` is **no longer found** in the captured frame,
    /// or until `timeout` elapses.
    ///
    /// Returns `Ok(())` as soon as a capture returns anything other than a match (i.e.
    /// [`MatchError::BelowThreshold`] or [`MatchError::TemplateTooLarge`]).
    /// Returns `Err(Timeout)` if the template is still visible when the deadline passes.
    ///
    /// Typical use: wait for a loading spinner or dialog to disappear.
    pub async fn wait_for_no_match<F>(
        &self,
        capture_fn: F,
        template: RgbaImage,
        origin: ScreenPoint,
        timeout: Duration,
        interval: Duration,
    ) -> Result<()>
    where
        F: Fn() -> RgbaImage + Send + Sync + 'static,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        let capture_fn = Arc::new(capture_fn);
        let template = Arc::new(template);

        loop {
            let cf = Arc::clone(&capture_fn);
            let tmpl = Arc::clone(&template);
            let m = self.clone();

            let found = tokio::task::spawn_blocking(move || {
                let haystack = cf();
                m.find(&haystack, &tmpl, origin).is_ok()
            })
            .await
            .unwrap_or_else(|e| panic!("capture task panicked: {e}"));

            if !found {
                return Ok(());
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(MatchError::Timeout);
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Like [`wait_for_no_match`] but reuses the pre-computed grayscale in `tmpl`.
    pub async fn wait_for_no_match_prepared<F>(
        &self,
        capture_fn: F,
        tmpl: Arc<PreparedTemplate>,
        origin: ScreenPoint,
        timeout: Duration,
        interval: Duration,
    ) -> Result<()>
    where
        F: Fn() -> RgbaImage + Send + Sync + 'static,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        let capture_fn = Arc::new(capture_fn);

        loop {
            let cf = Arc::clone(&capture_fn);
            let t = Arc::clone(&tmpl);
            let m = self.clone();

            let found = tokio::task::spawn_blocking(move || {
                let haystack = cf();
                m.find_prepared(&haystack, &t, origin).is_ok()
            })
            .await
            .unwrap_or_else(|e| panic!("capture task panicked: {e}"));

            if !found {
                return Ok(());
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(MatchError::Timeout);
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Like [`wait_for_any_match`] but uses pre-converted [`PreparedTemplate`] slices
    /// to avoid repeated RGB→gray conversion on every poll iteration.
    pub async fn wait_for_any_match_prepared<F>(
        &self,
        capture_fn: F,
        templates: Arc<Vec<PreparedTemplate>>,
        origin: ScreenPoint,
        timeout: Duration,
        interval: Duration,
    ) -> Result<(usize, MatchResult)>
    where
        F: Fn() -> RgbaImage + Send + Sync + 'static,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        let capture_fn = Arc::new(capture_fn);

        loop {
            let cf = Arc::clone(&capture_fn);
            let tmpls = Arc::clone(&templates);
            let m = self.clone();

            let result = tokio::task::spawn_blocking(move || {
                let haystack = cf();
                m.find_best_of_prepared(&haystack, &tmpls, origin)
            })
            .await
            .unwrap_or_else(|e| panic!("capture task panicked: {e}"));

            if let Some(hit) = result {
                return Ok(hit);
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(MatchError::Timeout);
            }

            tokio::time::sleep(interval).await;
        }
    }
}

/// Poll `capture_fn` until the captured frame differs from `reference` by at least
/// `min_changed_ratio`, or until `timeout` elapses.
///
/// Returns `(new_frame, DiffResult)` on success so the caller can use the captured
/// image without re-capturing.
///
/// Requires the `async` feature. `diff` computation runs on the blocking thread pool.
pub async fn wait_for_diff<F>(
    capture_fn: F,
    reference: RgbaImage,
    pixel_threshold: u8,
    min_changed_ratio: f32,
    timeout: Duration,
    interval: Duration,
) -> std::result::Result<(RgbaImage, crate::diff::DiffResult), MatchError>
where
    F: Fn() -> RgbaImage + Send + Sync + 'static,
{
    let deadline = tokio::time::Instant::now() + timeout;
    let capture_fn = Arc::new(capture_fn);
    let reference = Arc::new(reference);

    loop {
        let cf = Arc::clone(&capture_fn);
        let ref_img = Arc::clone(&reference);

        let (new_frame, d) = tokio::task::spawn_blocking(move || {
            let new_frame = cf();
            let d = crate::diff::diff(&ref_img, &new_frame, pixel_threshold);
            (new_frame, d)
        })
        .await
        .unwrap_or_else(|e| panic!("diff task panicked: {e}"));

        if d.changed_ratio >= min_changed_ratio {
            return Ok((new_frame, d));
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(MatchError::Timeout);
        }

        tokio::time::sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ScreenPoint;
    use image::{Rgba, RgbaImage};

    fn solid(w: u32, h: u32, c: Rgba<u8>) -> RgbaImage {
        RgbaImage::from_pixel(w, h, c)
    }

    #[tokio::test]
    async fn find_async_matches() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl = solid(10, 10, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let r = m.find_async(hay, tmpl, ScreenPoint { x: 0, y: 0 }).await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn find_async_too_large() {
        let hay = solid(5, 5, Rgba([100, 100, 100, 255]));
        let tmpl = solid(10, 10, Rgba([100, 100, 100, 255]));
        let m = TemplateMatcher::default();
        let r = m.find_async(hay, tmpl, ScreenPoint { x: 0, y: 0 }).await;
        assert!(matches!(r, Err(MatchError::TemplateTooLarge)));
    }

    #[tokio::test]
    async fn wait_for_match_finds_immediately() {
        let tmpl = solid(10, 10, Rgba([180, 180, 180, 255]));
        let m = TemplateMatcher::default();
        let r = m
            .wait_for_match(
                || solid(100, 100, Rgba([180, 180, 180, 255])),
                tmpl,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_secs(1),
                Duration::from_millis(50),
            )
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn wait_for_match_times_out() {
        // Capture always returns a haystack smaller than the template → TemplateTooLarge every
        // iteration → match never succeeds → Timeout is returned when the deadline passes.
        let tmpl = solid(20, 20, Rgba([255, 0, 0, 255]));
        let m = TemplateMatcher::default();
        let r = m
            .wait_for_match(
                || solid(10, 10, Rgba([0, 0, 255, 255])),
                tmpl,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_millis(200),
                Duration::from_millis(40),
            )
            .await;
        assert!(matches!(r, Err(MatchError::Timeout)));
    }

    #[tokio::test]
    async fn wait_for_match_prepared_finds_immediately() {
        use crate::prepared::PreparedTemplate;
        let tmpl = Arc::new(PreparedTemplate::new(solid(
            10,
            10,
            Rgba([180, 180, 180, 255]),
        )));
        let m = TemplateMatcher::default();
        let r = m
            .wait_for_match_prepared(
                || solid(100, 100, Rgba([180, 180, 180, 255])),
                tmpl,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_secs(1),
                Duration::from_millis(50),
            )
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn wait_for_diff_detects_change() {
        use super::wait_for_diff;
        let reference = solid(10, 10, Rgba([100, 100, 100, 255]));
        let r = wait_for_diff(
            || solid(10, 10, Rgba([0, 0, 0, 255])), // always differs
            reference,
            10,
            0.5,
            Duration::from_secs(1),
            Duration::from_millis(50),
        )
        .await;
        assert!(r.is_ok());
        let (_, d) = r.unwrap();
        assert!(d.changed_ratio >= 0.5);
    }

    #[tokio::test]
    async fn wait_for_no_match_returns_ok_when_gone() {
        // haystack is 10x10, template is 20x20 → TemplateTooLarge every time → "not found"
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let r = m
            .wait_for_no_match(
                || solid(10, 10, Rgba([200, 200, 200, 255])),
                tmpl,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_secs(1),
                Duration::from_millis(50),
            )
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn wait_for_no_match_times_out_while_visible() {
        // Template always matches (same color) → should time out.
        let tmpl = solid(10, 10, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let r = m
            .wait_for_no_match(
                || solid(100, 100, Rgba([200, 200, 200, 255])),
                tmpl,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_millis(200),
                Duration::from_millis(40),
            )
            .await;
        assert!(matches!(r, Err(MatchError::Timeout)));
    }

    #[tokio::test]
    async fn wait_for_any_match_finds_one() {
        let m = TemplateMatcher::default();
        let templates = Arc::new(vec![
            solid(10, 10, Rgba([180, 180, 180, 255])), // matches
        ]);
        let r = m
            .wait_for_any_match(
                || solid(100, 100, Rgba([180, 180, 180, 255])),
                templates,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_secs(1),
                Duration::from_millis(50),
            )
            .await;
        assert!(r.is_ok());
        assert_eq!(r.unwrap().0, 0);
    }

    #[tokio::test]
    async fn wait_for_any_match_times_out() {
        let m = TemplateMatcher::default();
        // Template is too large → never matches.
        let templates = Arc::new(vec![solid(200, 200, Rgba([0, 0, 0, 255]))]);
        let r = m
            .wait_for_any_match(
                || solid(10, 10, Rgba([0, 0, 0, 255])),
                templates,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_millis(200),
                Duration::from_millis(40),
            )
            .await;
        assert!(matches!(r, Err(MatchError::Timeout)));
    }

    #[tokio::test]
    async fn wait_for_any_match_prepared_finds_one() {
        use crate::prepared::PreparedTemplate;
        let m = TemplateMatcher::default();
        let templates = Arc::new(vec![PreparedTemplate::new(solid(
            10,
            10,
            Rgba([180, 180, 180, 255]),
        ))]);
        let r = m
            .wait_for_any_match_prepared(
                || solid(100, 100, Rgba([180, 180, 180, 255])),
                templates,
                ScreenPoint { x: 0, y: 0 },
                Duration::from_secs(1),
                Duration::from_millis(50),
            )
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn wait_for_diff_times_out_when_no_change() {
        use super::wait_for_diff;
        let reference = solid(10, 10, Rgba([100, 100, 100, 255]));
        // capture_fn returns the same image as reference → no change
        let r = wait_for_diff(
            || solid(10, 10, Rgba([100, 100, 100, 255])),
            reference,
            10,
            0.5,
            Duration::from_millis(200),
            Duration::from_millis(40),
        )
        .await;
        assert!(matches!(r, Err(MatchError::Timeout)));
    }
}
