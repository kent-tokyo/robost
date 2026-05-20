use crate::{get_str, NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

/// Extract all text from a PDF file.
///
/// Required: file
/// Output: { text: "...", pages: N }
pub fn extract_text(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;

    let bytes = std::fs::read(&file)
        .map_err(|e| NodeError::Other(format!("read pdf failed: {e}")))?;

    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| NodeError::Other(format!("pdf extract failed: {e}")))?;

    // Count pages by parsing the /Pages count header heuristically.
    let page_count = count_pages(&bytes);

    tracing::info!(file, pages = page_count, "pdf.extract_text");
    let mut out = HashMap::new();
    out.insert("text".to_owned(), Value::String(text));
    out.insert("pages".to_owned(), Value::Number(page_count.into()));
    Ok(out)
}

/// Return the number of pages in a PDF.
///
/// Required: file
/// Output: { pages: N }
pub fn page_count(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;

    let bytes = std::fs::read(&file)
        .map_err(|e| NodeError::Other(format!("read pdf failed: {e}")))?;

    let count = count_pages(&bytes);

    tracing::info!(file, pages = count, "pdf.page_count");
    let mut out = HashMap::new();
    out.insert("pages".to_owned(), Value::Number(count.into()));
    Ok(out)
}

fn count_pages(bytes: &[u8]) -> u64 {
    // Search for "/Count N" in the PDF byte stream (approximate).
    let haystack = std::str::from_utf8(bytes).unwrap_or("");
    let mut max_count: u64 = 0;
    for pos in haystack.match_indices("/Count").map(|(i, _)| i) {
        let after = &haystack[pos + 6..];
        let n: String = after
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(n) = n.parse::<u64>() {
            max_count = max_count.max(n);
        }
    }
    max_count
}
