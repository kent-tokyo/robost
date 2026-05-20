use crate::{get_str, NodeError, NodeResult};
use lettre::{
    message::header::ContentType as LettreContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use serde_json::Value;
use std::collections::HashMap;

/// Send an email via SMTP.
///
/// Required inputs: host, from, to, subject, body
/// Optional inputs: port (default 587), user, password, cc, bcc
pub fn smtp_send(inputs: HashMap<String, Value>) -> NodeResult {
    let host = get_str(&inputs, "host")?;
    let from = get_str(&inputs, "from")?;
    let to = get_str(&inputs, "to")?;
    let subject = get_str(&inputs, "subject")?;
    let body = get_str(&inputs, "body")?;

    let port = inputs
        .get("port")
        .and_then(|v| v.as_u64())
        .unwrap_or(587) as u16;

    let cc = inputs.get("cc").and_then(|v| v.as_str()).map(str::to_owned);
    let bcc = inputs.get("bcc").and_then(|v| v.as_str()).map(str::to_owned);

    let mut builder = Message::builder()
        .from(from.parse().map_err(|e| NodeError::Other(format!("invalid from address: {e}")))?)
        .to(to.parse().map_err(|e| NodeError::Other(format!("invalid to address: {e}")))?)
        .subject(subject);

    if let Some(cc_addr) = cc {
        builder = builder.cc(cc_addr.parse().map_err(|e| NodeError::Other(format!("invalid cc address: {e}")))?);
    }
    if let Some(bcc_addr) = bcc {
        builder = builder.bcc(bcc_addr.parse().map_err(|e| NodeError::Other(format!("invalid bcc address: {e}")))?);
    }

    let message = builder
        .header(LettreContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| NodeError::Other(format!("message build failed: {e}")))?;

    let user = inputs.get("user").and_then(|v| v.as_str()).map(str::to_owned);
    let password = inputs.get("password").and_then(|v| v.as_str()).map(str::to_owned);

    let transport = if let (Some(u), Some(p)) = (user, password) {
        SmtpTransport::starttls_relay(&host)
            .map_err(|e| NodeError::Other(format!("smtp relay error: {e}")))?
            .port(port)
            .credentials(Credentials::new(u, p))
            .build()
    } else {
        SmtpTransport::builder_dangerous(&host)
            .port(port)
            .build()
    };

    transport
        .send(&message)
        .map_err(|e| NodeError::Other(format!("smtp send failed: {e}")))?;

    tracing::info!(to, host, "mail.smtp_send: sent");
    Ok(HashMap::new())
}

