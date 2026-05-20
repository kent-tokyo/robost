use crate::{get_str, opt_str, NodeResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

fn webhook_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(15))
            .build()
    })
}

/// Send a message to a Slack incoming webhook.
///
/// Required: webhook_url, text
/// Optional: username, icon_emoji, channel
pub fn slack_send(inputs: HashMap<String, Value>) -> NodeResult {
    let url = get_str(&inputs, "webhook_url")?;
    let text = get_str(&inputs, "text")?;

    let mut payload = json!({ "text": text });
    if let Some(u) = opt_str(&inputs, "username") {
        payload["username"] = json!(u);
    }
    if let Some(e) = opt_str(&inputs, "icon_emoji") {
        payload["icon_emoji"] = json!(e);
    }
    if let Some(c) = opt_str(&inputs, "channel") {
        payload["channel"] = json!(c);
    }

    let status = match webhook_agent()
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(payload)
    {
        Ok(r) => r.status(),
        Err(ureq::Error::Status(s, _)) => s,
        Err(e) => {
            return Err(crate::NodeError::Other(format!(
                "slack webhook failed: {e}"
            )))
        }
    };

    if status != 200 {
        return Err(crate::NodeError::Other(format!(
            "slack webhook returned HTTP {status}"
        )));
    }

    tracing::info!(url, "notify.slack_send");
    Ok(HashMap::new())
}

/// Send a message to a Microsoft Teams incoming webhook (Legacy MessageCard format).
///
/// Required: webhook_url, text
/// Optional: title, color (hex without #, e.g. "00ff00")
pub fn teams_send(inputs: HashMap<String, Value>) -> NodeResult {
    let url = get_str(&inputs, "webhook_url")?;
    let text = get_str(&inputs, "text")?;

    let title = opt_str(&inputs, "title").unwrap_or_default();
    let color = opt_str(&inputs, "color").unwrap_or_else(|| "0078D4".to_owned());

    let payload = json!({
        "@type":      "MessageCard",
        "@context":   "https://schema.org/extensions",
        "summary":    &title,
        "themeColor": color,
        "title":      title,
        "text":       text,
    });

    let status = match webhook_agent()
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(payload)
    {
        Ok(r) => r.status(),
        Err(ureq::Error::Status(s, _)) => s,
        Err(e) => {
            return Err(crate::NodeError::Other(format!(
                "teams webhook failed: {e}"
            )))
        }
    };

    if status != 200 {
        return Err(crate::NodeError::Other(format!(
            "teams webhook returned HTTP {status}"
        )));
    }

    tracing::info!(url, "notify.teams_send");
    Ok(HashMap::new())
}
