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
}
