use crate::{get_str, opt_str, NodeError, NodeResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

/// Reject non-HTTPS or RFC-1918/link-local targets to prevent SSRF via webhook_url.
fn validate_webhook_url(url: &str) -> Result<(), NodeError> {
    if !url.starts_with("https://") {
        return Err(NodeError::Other("webhook_url must use https".to_owned()));
    }
    // Extract the host (strip "https://", take up to first '/', '?', or '#').
    let rest = &url[8..];
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host_port = &rest[..host_end];
    // Strip optional port.
    let host = match host_port.rfind(':') {
        Some(i) if host_port[i + 1..].chars().all(|c| c.is_ascii_digit()) => &host_port[..i],
        _ => host_port,
    };

    // Reject IP literals that are loopback/private/link-local to block SSRF.
    use std::net::IpAddr;
    if let Ok(ip) = host.parse::<IpAddr>() {
        let blocked = match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified()
            }
            IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
        };
        if blocked {
            return Err(NodeError::Other(format!(
                "webhook_url: disallowed IP address: {ip}"
            )));
        }
    }

    Ok(())
}

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
    validate_webhook_url(&url)?;
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
    validate_webhook_url(&url)?;
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
