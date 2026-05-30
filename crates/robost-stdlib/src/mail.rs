use crate::{get_str, opt_str, NodeError, NodeResult};
use imap::types::Flag;
use lettre::{
    message::header::ContentType as LettreContentType,
    transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport,
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

    let port = inputs.get("port").and_then(|v| v.as_u64()).unwrap_or(587) as u16;

    let cc = inputs.get("cc").and_then(|v| v.as_str()).map(str::to_owned);
    let bcc = inputs
        .get("bcc")
        .and_then(|v| v.as_str())
        .map(str::to_owned);

    let mut builder = Message::builder()
        .from(
            from.parse()
                .map_err(|e| NodeError::Other(format!("invalid from address: {e}")))?,
        )
        .to(to
            .parse()
            .map_err(|e| NodeError::Other(format!("invalid to address: {e}")))?)
        .subject(subject);

    if let Some(cc_addr) = cc {
        builder = builder.cc(cc_addr
            .parse()
            .map_err(|e| NodeError::Other(format!("invalid cc address: {e}")))?);
    }
    if let Some(bcc_addr) = bcc {
        builder = builder.bcc(
            bcc_addr
                .parse()
                .map_err(|e| NodeError::Other(format!("invalid bcc address: {e}")))?,
        );
    }

    let message = builder
        .header(LettreContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| NodeError::Other(format!("message build failed: {e}")))?;

    let u = get_str(&inputs, "user")?;
    let p = get_str(&inputs, "password")?;
    let transport = SmtpTransport::starttls_relay(&host)
        .map_err(|e| NodeError::Other(format!("smtp relay error: {e}")))?
        .port(port)
        .credentials(Credentials::new(u, p))
        .build();

    transport
        .send(&message)
        .map_err(|e| NodeError::Other(format!("smtp send failed: {e}")))?;

    tracing::info!(to, host, "mail.smtp_send: sent");
    Ok(HashMap::new())
}

/// Receive emails via IMAP (TLS, port 993 by default).
///
/// Required inputs: host, user, password
/// Optional inputs: port (default 993), folder (default "INBOX"),
///                  count (default 10), only_unseen (default false)
/// Output: messages = [{subject, from, date, body, seen}]
pub fn imap_receive(inputs: HashMap<String, Value>) -> NodeResult {
    let host = get_str(&inputs, "host")?;
    let user = get_str(&inputs, "user")?;
    let password = get_str(&inputs, "password")?;
    let port = inputs.get("port").and_then(|v| v.as_u64()).unwrap_or(993) as u16;
    let folder = opt_str(&inputs, "folder").unwrap_or_else(|| "INBOX".to_owned());
    let count = inputs.get("count").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
    let only_unseen = inputs
        .get("only_unseen")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| NodeError::Other(format!("TLS init: {e}")))?;

    let client = imap::connect((&*host, port), &host, &tls)
        .map_err(|e| NodeError::Other(format!("IMAP connect: {e}")))?;

    let mut session = client
        .login(&user, &password)
        .map_err(|(e, _)| NodeError::Other(format!("IMAP login: {e}")))?;

    let mailbox = session
        .select(&folder)
        .map_err(|e| NodeError::Other(format!("IMAP select: {e}")))?;

    let exists = mailbox.exists;
    let messages: Vec<Value> = if exists == 0 {
        vec![]
    } else {
        let seq: String = if only_unseen {
            let uids = session
                .search("UNSEEN")
                .map_err(|e| NodeError::Other(format!("IMAP search: {e}")))?;
            if uids.is_empty() {
                let _ = session.logout();
                let mut out = HashMap::new();
                out.insert("messages".to_owned(), Value::Array(vec![]));
                return Ok(out);
            }
            let mut sorted: Vec<u32> = uids.into_iter().collect();
            sorted.sort_unstable();
            sorted
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(",")
        } else {
            let start = exists.saturating_sub(count.saturating_sub(1)).max(1);
            format!("{start}:{exists}")
        };

        let fetches = session
            .fetch(&seq, "(FLAGS ENVELOPE BODY.PEEK[TEXT]<0.8192>)")
            .map_err(|e| NodeError::Other(format!("IMAP fetch: {e}")))?;

        fetches
            .iter()
            .map(|msg| {
                let subject = msg
                    .envelope()
                    .and_then(|env| env.subject.as_ref().map(|s| s.as_ref()))
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .unwrap_or("")
                    .to_owned();

                let from = msg
                    .envelope()
                    .and_then(|env| env.from.as_ref())
                    .and_then(|addrs| addrs.first())
                    .map(|a| {
                        let mb = a
                            .mailbox
                            .as_ref()
                            .map(|b| b.as_ref())
                            .and_then(|b| std::str::from_utf8(b).ok())
                            .unwrap_or("");
                        let h = a
                            .host
                            .as_ref()
                            .map(|b| b.as_ref())
                            .and_then(|b| std::str::from_utf8(b).ok())
                            .unwrap_or("");
                        format!("{mb}@{h}")
                    })
                    .unwrap_or_default();

                let date = msg
                    .envelope()
                    .and_then(|env| env.date.as_ref().map(|d| d.as_ref()))
                    .and_then(|d| std::str::from_utf8(d).ok())
                    .unwrap_or("")
                    .to_owned();

                let body = msg
                    .text()
                    .and_then(|b| std::str::from_utf8(b).ok())
                    .unwrap_or("")
                    .to_owned();

                let seen = msg.flags().iter().any(|f| f == &Flag::Seen);

                serde_json::json!({
                    "subject": subject,
                    "from": from,
                    "date": date,
                    "body": body,
                    "seen": seen,
                })
            })
            .collect()
    };

    let _ = session.logout();

    tracing::info!(host, folder, count = messages.len(), "mail.imap_receive");
    let mut out = HashMap::new();
    out.insert("messages".to_owned(), Value::Array(messages));
    Ok(out)
}
