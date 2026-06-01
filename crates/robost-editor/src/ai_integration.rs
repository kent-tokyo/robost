// ---- AI integration ---------------------------------------------------------

use crate::settings::{AiProvider, AppSettings};
use crate::step_templates::STEP_TEMPLATES;
use crate::types::AiMessage;

pub(crate) fn build_system_prompt() -> String {
    let steps: Vec<String> = STEP_TEMPLATES
        .iter()
        .map(|t| format!("\"{}\"", t.name))
        .collect();
    format!(
        "あなたはrobost RPAツールのシナリオ作成アシスタントです。\n\
         利用可能なステップ: {steps}\n\
         YAMLを提案する際は必ず```yamlブロックで囲んでください。\n\
         変数参照は {{{{ var_name }}}} 形式です。",
        steps = steps.join(", ")
    )
}

/// Extracts fenced ```yaml blocks from text.
/// Returns `(blocks, has_unclosed)` where `has_unclosed` is true if a block
/// was opened but never closed (partial block is still returned).
pub(crate) fn extract_yaml_blocks(text: &str) -> (Vec<String>, bool) {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut buf = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```yaml") {
            in_block = true;
            buf.clear();
        } else if in_block && line.trim() == "```" {
            let trimmed = buf.trim().to_owned();
            if !trimmed.is_empty() {
                blocks.push(trimmed);
            }
            in_block = false;
        } else if in_block {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    let unclosed = if in_block {
        let partial = buf.trim().to_owned();
        if !partial.is_empty() {
            blocks.push(partial);
        }
        true
    } else {
        false
    };
    (blocks, unclosed)
}

pub(crate) fn call_ai_api(
    settings: &AppSettings,
    history: &[AiMessage],
    input: &str,
    system: &str,
) -> anyhow::Result<String> {
    match settings.provider {
        AiProvider::Anthropic => call_anthropic(settings, history, input, system),
        AiProvider::OpenAI => call_openai(settings, history, input, system),
    }
}

pub(crate) fn call_anthropic(
    settings: &AppSettings,
    history: &[AiMessage],
    input: &str,
    system: &str,
) -> anyhow::Result<String> {
    if settings.api_key.is_empty() {
        anyhow::bail!("Anthropic APIキーが設定されていません。「設定」から入力してください。");
    }
    let mut messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();
    messages.push(serde_json::json!({ "role": "user", "content": input }));
    let body = serde_json::json!({
        "model": settings.model,
        "max_tokens": 2048,
        "system": system,
        "messages": messages,
    });
    let resp_str = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", settings.api_key.trim())
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(body)?
        .into_string()?;
    let resp: serde_json::Value = serde_json::from_str(&resp_str)?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("Anthropic API error: {}", err);
    }
    resp.get("content")
        .and_then(|c| c.get(0))
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Unexpected Anthropic response format: {resp_str}"))
        .map(str::to_owned)
}

pub(crate) fn call_openai(
    settings: &AppSettings,
    history: &[AiMessage],
    input: &str,
    system: &str,
) -> anyhow::Result<String> {
    if settings.api_key.is_empty() {
        anyhow::bail!("OpenAI APIキーが設定されていません。「設定」から入力してください。");
    }
    let mut msgs = vec![serde_json::json!({ "role": "system", "content": system })];
    msgs.extend(
        history
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content })),
    );
    msgs.push(serde_json::json!({ "role": "user", "content": input }));
    let body = serde_json::json!({ "model": settings.model, "messages": msgs });
    let resp_str = ureq::post("https://api.openai.com/v1/chat/completions")
        .set(
            "Authorization",
            &format!("Bearer {}", settings.api_key.trim()),
        )
        .set("content-type", "application/json")
        .send_json(body)?
        .into_string()?;
    let resp: serde_json::Value = serde_json::from_str(&resp_str)?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("OpenAI API error: {}", err);
    }
    resp.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|first| first.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Unexpected OpenAI response format: {resp_str}"))
        .map(str::to_owned)
}

pub(crate) fn test_ai_connection(settings: &AppSettings) -> (bool, String) {
    let result = match settings.provider {
        AiProvider::Anthropic => {
            let body = serde_json::json!({
                "model": settings.model,
                "max_tokens": 8,
                "messages": [{ "role": "user", "content": "ping" }],
            });
            ureq::post("https://api.anthropic.com/v1/messages")
                .set("x-api-key", settings.api_key.trim())
                .set("anthropic-version", "2023-06-01")
                .set("content-type", "application/json")
                .send_json(body)
                .map(|_| "Connection OK".to_owned())
                .map_err(friendly_api_error)
        }
        AiProvider::OpenAI => {
            let body = serde_json::json!({
                "model": settings.model,
                "max_tokens": 8,
                "messages": [{ "role": "user", "content": "ping" }],
            });
            ureq::post("https://api.openai.com/v1/chat/completions")
                .set(
                    "Authorization",
                    &format!("Bearer {}", settings.api_key.trim()),
                )
                .set("content-type", "application/json")
                .send_json(body)
                .map(|_| "Connection OK".to_owned())
                .map_err(friendly_api_error)
        }
    };
    match result {
        Ok(msg) => (true, msg),
        Err(e) => (false, e),
    }
}

pub(crate) fn friendly_api_error(e: ureq::Error) -> String {
    let msg = e.to_string();
    if msg.contains("401") {
        "401 Unauthorized — API key is invalid or expired. Check your key at console.anthropic.com"
            .to_owned()
    } else if msg.contains("403") {
        "403 Forbidden — API key lacks permission for this model".to_owned()
    } else if msg.contains("429") {
        "429 Rate limit exceeded — try again later".to_owned()
    } else {
        msg
    }
}
