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
        r#"あなたは robost RPA ツールのシナリオ作成アシスタントです。

## 利用可能なステップ
{steps}

## 出力ルール
- YAML を提案する際は必ず ```yaml ブロックで囲む
- 変数参照は {{{{ var_name }}}} 形式
- コードブロックは必ず閉じる

## ステップ選択の優先順位（重要）

### 1. OS 操作・アプリ操作は shell / script を優先
macOS/Windows の OS 標準操作はコマンドで実行できる。画像認識より確実。

例:
- ゴミ箱を空にする(macOS): shell: cmd: osascript args: ["-e", "tell application \"Finder\" to empty trash"]
- ゴミ箱を空にする(Windows): shell: cmd: powershell args: ["-Command", "Clear-RecycleBin -Force"]
- アプリを開く(macOS): shell: cmd: open args: ["-a", "Calculator"]
- テキストファイルを読む: file_read
- Web 操作: web_open, web_click, web_type など

### 2. 画像ステップはテンプレート採取が必要
click_image / wait_image / find_image 等を使う場合、template に実際の画像ファイルが必要。
ユーザーは Snip ツール(📸)でキャプチャする必要がある。

template フィールドには必ず `__CAPTURE_NEEDED__` プレフィックスを付ける:
```yaml
click_image:
  template: __CAPTURE_NEEDED__ゴミ箱アイコン.png
  timeout_ms: 5000
```
これにより何をキャプチャすればよいかユーザーに伝わる。

### 3. テキスト・座標ベースを検討
- OCR でテキスト検索: ocr_match
- 固定座標クリック: mouse_click_xy
- UIA (Windows アクセシビリティ): uia_click, uia_find

## 例: 「ゴミ箱を空にして」への回答
```yaml
- shell:
    cmd: osascript
    args: ["-e", "tell application \"Finder\" to empty trash"]
```
macOS では Finder コマンドが最も確実。画像認識不要。

## 例: 画像認識が必要な場合
```yaml
- click_image:
    template: __CAPTURE_NEEDED__ログインボタン.png
    timeout_ms: 5000
- type: "username"
```
"__CAPTURE_NEEDED__" で始まるテンプレートは「Snip ツールで採取が必要」という印。"#,
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
