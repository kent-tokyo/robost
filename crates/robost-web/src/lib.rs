//! WebDriver-based browser automation.
//!
//! Wraps [fantoccini](https://docs.rs/fantoccini) to provide async browser
//! control via any W3C WebDriver endpoint (ChromeDriver, GeckoDriver, etc.).
//!
//! # Prerequisites
//!
//! Start a WebDriver server before running scenarios with `web_open`:
//!
//! ```sh
//! # Chrome
//! chromedriver --port=4444
//!
//! # Firefox
//! geckodriver --port=4444
//! ```

use fantoccini::{Client, ClientBuilder, Locator};
use thiserror::Error;
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug, Error)]
pub enum WebError {
    #[error("webdriver connection failed: {0}")]
    Connection(String),
    #[error("element not found: {0}")]
    NotFound(String),
    #[error("webdriver command failed: {0}")]
    Command(String),
    #[error("screenshot decode failed: {0}")]
    Screenshot(String),
    #[error("no active browser session — call web_open first")]
    NoSession,
}

pub type Result<T> = std::result::Result<T, WebError>;

/// An active browser session managed by the RPA engine.
pub struct WebSession {
    client: Client,
}

impl WebSession {
    /// Connect to a WebDriver endpoint and open `url`.
    pub async fn new(driver_url: &str) -> Result<Self> {
        let client = ClientBuilder::native()
            .connect(driver_url)
            .await
            .map_err(|e| WebError::Connection(e.to_string()))?;
        Ok(Self { client })
    }

    /// Navigate to `url`.
    pub async fn open(&self, url: &str) -> Result<()> {
        self.client
            .goto(url)
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Find an element by CSS selector (waits up to `timeout_ms`).
    pub async fn find(
        &self,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<fantoccini::elements::Element> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            match self.client.find(Locator::Css(selector)).await {
                Ok(el) => return Ok(el),
                Err(_) if Instant::now() < deadline => {
                    sleep(Duration::from_millis(200)).await;
                }
                Err(_) => return Err(WebError::NotFound(selector.to_owned())),
            }
        }
    }

    /// Click an element identified by CSS selector.
    pub async fn click(&self, selector: &str, timeout_ms: u64) -> Result<()> {
        let el = self.find(selector, timeout_ms).await?;
        el.click()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Type text into an element (optionally clearing it first).
    pub async fn type_text(
        &self,
        selector: &str,
        text: &str,
        clear: bool,
        timeout_ms: u64,
    ) -> Result<()> {
        let el = self.find(selector, timeout_ms).await?;
        if clear {
            el.clear()
                .await
                .map_err(|e| WebError::Command(e.to_string()))?;
        }
        el.send_keys(text)
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Get the visible text of an element.
    pub async fn get_text(&self, selector: &str, timeout_ms: u64) -> Result<String> {
        let el = self.find(selector, timeout_ms).await?;
        el.text()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Get an attribute of an element.
    pub async fn get_attr(
        &self,
        selector: &str,
        attr: &str,
        timeout_ms: u64,
    ) -> Result<Option<String>> {
        let el = self.find(selector, timeout_ms).await?;
        el.attr(attr)
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Take a screenshot and save it as PNG.
    pub async fn screenshot(&self, path: &str) -> Result<()> {
        let png_bytes = self
            .client
            .screenshot()
            .await
            .map_err(|e| WebError::Command(e.to_string()))?;
        std::fs::write(path, &png_bytes).map_err(|e| WebError::Screenshot(e.to_string()))
    }

    /// Close the browser session.
    pub async fn close(self) -> Result<()> {
        self.client
            .close()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Get the current page URL.
    pub async fn current_url(&self) -> Result<String> {
        self.client
            .current_url()
            .await
            .map(|u| u.to_string())
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Get the page title.
    pub async fn title(&self) -> Result<String> {
        self.client
            .title()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Select an option in a `<select>` element by visible text or value attribute.
    pub async fn select(&self, selector: &str, item: &str, timeout_ms: u64) -> Result<()> {
        let el = self.find(selector, timeout_ms).await?;
        let el_json =
            serde_json::to_value(&el).map_err(|e| WebError::Command(e.to_string()))?;
        let script = "var s=arguments[0],v=arguments[1];\
            for(var i=0;i<s.options.length;i++){\
                if(s.options[i].text===v||s.options[i].value===v){\
                    s.selectedIndex=i;\
                    s.dispatchEvent(new Event('change',{bubbles:true}));\
                    return;\
                }\
            }\
            throw new Error('option not found: '+v);";
        self.client
            .execute(script, vec![el_json, serde_json::Value::String(item.to_owned())])
            .await
            .map_err(|e| WebError::Command(e.to_string()))?;
        Ok(())
    }

    /// Execute JavaScript in the browser and return the result.
    pub async fn execute_js(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        self.client
            .execute(script, args)
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Switch into an iframe identified by CSS selector.
    pub async fn switch_frame_selector(&self, selector: &str, timeout_ms: u64) -> Result<()> {
        let el = self.find(selector, timeout_ms).await?;
        el.enter_frame()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Switch into an iframe by zero-based index.
    pub async fn switch_frame_index(&self, index: u16) -> Result<()> {
        self.client
            .enter_frame(Some(index))
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Switch back to the top-level browsing context (exit all frames).
    pub async fn switch_frame_main(&self) -> Result<()> {
        self.client
            .enter_frame(None)
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Scroll an element (or the window if `selector` is `None`) by `(x, y)` pixels.
    pub async fn scroll(
        &self,
        selector: Option<&str>,
        x: i32,
        y: i32,
        timeout_ms: u64,
    ) -> Result<()> {
        if let Some(sel) = selector {
            let el = self.find(sel, timeout_ms).await?;
            let el_json =
                serde_json::to_value(&el).map_err(|e| WebError::Command(e.to_string()))?;
            let script =
                "arguments[0].scrollLeft+=arguments[1];arguments[0].scrollTop+=arguments[2];";
            self.client
                .execute(script, vec![el_json, x.into(), y.into()])
                .await
                .map_err(|e| WebError::Command(e.to_string()))?;
        } else {
            self.client
                .execute(&format!("window.scrollBy({x},{y});"), vec![])
                .await
                .map_err(|e| WebError::Command(e.to_string()))?;
        }
        Ok(())
    }

    /// Return the text of the currently displayed JS alert/confirm/prompt.
    pub async fn alert_text(&self) -> Result<String> {
        self.client
            .get_alert_text()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Accept (click OK on) the current JS alert/confirm/prompt.
    pub async fn accept_alert(&self) -> Result<()> {
        self.client
            .accept_alert()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Dismiss (click Cancel on) the current JS alert/confirm.
    pub async fn dismiss_alert(&self) -> Result<()> {
        self.client
            .dismiss_alert()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Navigate back in the browser history.
    pub async fn back(&self) -> Result<()> {
        self.client
            .back()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Navigate forward in the browser history.
    pub async fn forward(&self) -> Result<()> {
        self.client
            .forward()
            .await
            .map_err(|e| WebError::Command(e.to_string()))
    }

    /// Return all elements matching `selector` as a list of strings (text or attribute).
    pub async fn find_all(&self, selector: &str, attr: Option<&str>) -> Result<Vec<String>> {
        let els = self
            .client
            .find_all(Locator::Css(selector))
            .await
            .map_err(|e| WebError::Command(e.to_string()))?;
        let mut result = Vec::with_capacity(els.len());
        for el in els {
            let s = match attr {
                Some(a) => el
                    .attr(a)
                    .await
                    .map_err(|e| WebError::Command(e.to_string()))?
                    .unwrap_or_default(),
                None => el.text().await.map_err(|e| WebError::Command(e.to_string()))?,
            };
            result.push(s);
        }
        Ok(result)
    }

    /// Wait until `selector` element's text contains `text` (polls every 200 ms).
    pub async fn wait_text(&self, selector: &str, text: &str, timeout_ms: u64) -> Result<()> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            if let Ok(el) = self.client.find(Locator::Css(selector)).await {
                if let Ok(t) = el.text().await {
                    if t.contains(text) {
                        return Ok(());
                    }
                }
            }
            if Instant::now() >= deadline {
                return Err(WebError::NotFound(format!(
                    "{selector} containing '{text}'"
                )));
            }
            sleep(Duration::from_millis(200)).await;
        }
    }
}
