use crate::{NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

fn get_list(inputs: &HashMap<String, Value>, key: &str) -> Result<Vec<Value>, NodeError> {
    match inputs.get(key) {
        Some(Value::Array(v)) => Ok(v.clone()),
        Some(_) => Err(NodeError::Other(format!("'{key}' must be an array"))),
        None => Err(NodeError::MissingInput(key.to_owned())),
    }
}

/// Sort a list. Supports lists of strings, numbers, or objects (sort by a key).
///
/// Required inputs: list
/// Optional inputs: key (object key to sort by), order ("asc" | "desc", default "asc")
/// Output: { result: [...] }
pub fn sort(inputs: HashMap<String, Value>) -> NodeResult {
    let mut list = get_list(&inputs, "list")?;
    let order = inputs
        .get("order")
        .and_then(|v| v.as_str())
        .unwrap_or("asc");
    let key = inputs.get("key").and_then(|v| v.as_str()).map(str::to_owned);

    list.sort_by(|a, b| {
        let va = key.as_deref().and_then(|k| a.get(k)).unwrap_or(a);
        let vb = key.as_deref().and_then(|k| b.get(k)).unwrap_or(b);
        compare_values(va, vb)
    });

    if order == "desc" {
        list.reverse();
    }

    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::Array(list));
    Ok(out)
}

/// Remove duplicate values from a list (preserves first occurrence).
///
/// Required inputs: list
/// Optional inputs: key (for object lists, deduplicate by this field)
/// Output: { result: [...] }
pub fn unique(inputs: HashMap<String, Value>) -> NodeResult {
    let list = get_list(&inputs, "list")?;
    let key = inputs.get("key").and_then(|v| v.as_str()).map(str::to_owned);

    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in list {
        let repr = key
            .as_deref()
            .and_then(|k| item.get(k))
            .unwrap_or(&item)
            .to_string();
        if seen.insert(repr) {
            result.push(item);
        }
    }

    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::Array(result));
    Ok(out)
}

/// Reverse a list.
///
/// Required inputs: list
/// Output: { result: [...] }
pub fn reverse(inputs: HashMap<String, Value>) -> NodeResult {
    let mut list = get_list(&inputs, "list")?;
    list.reverse();
    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::Array(list));
    Ok(out)
}

/// Return the number of items in a list.
///
/// Required inputs: list
/// Output: { result: number }
pub fn length(inputs: HashMap<String, Value>) -> NodeResult {
    let list = get_list(&inputs, "list")?;
    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::Number(list.len().into()));
    Ok(out)
}

/// Get an item at a given index (negative indices count from end).
///
/// Required inputs: list, index
/// Output: { result: value }
pub fn get(inputs: HashMap<String, Value>) -> NodeResult {
    let list = get_list(&inputs, "list")?;
    let idx = inputs
        .get("index")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| NodeError::MissingInput("index".to_owned()))?;

    let real_idx = if idx < 0 {
        list.len() as i64 + idx
    } else {
        idx
    };

    let item = list
        .get(real_idx as usize)
        .cloned()
        .ok_or_else(|| NodeError::Other(format!("index {idx} out of range (len={})", list.len())))?;

    let mut out = HashMap::new();
    out.insert("result".to_owned(), item);
    Ok(out)
}

/// Slice a list (Python-style, exclusive end).
///
/// Required inputs: list, start
/// Optional inputs: end (exclusive, default = end of list)
/// Output: { result: [...] }
pub fn slice(inputs: HashMap<String, Value>) -> NodeResult {
    let list = get_list(&inputs, "list")?;
    let start = inputs
        .get("start")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| NodeError::MissingInput("start".to_owned()))? as usize;
    let end = inputs
        .get("end")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(list.len());

    let sliced = list
        .get(start..end.min(list.len()))
        .unwrap_or_default()
        .to_vec();

    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::Array(sliced));
    Ok(out)
}

/// Flatten one level of nesting.
///
/// Required inputs: list
/// Output: { result: [...] }
pub fn flatten(inputs: HashMap<String, Value>) -> NodeResult {
    let list = get_list(&inputs, "list")?;
    let mut result = Vec::new();
    for item in list {
        match item {
            Value::Array(inner) => result.extend(inner),
            other => result.push(other),
        }
    }
    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::Array(result));
    Ok(out)
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(na), Value::Number(nb)) => na
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&nb.as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => a.to_string().cmp(&b.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn inputs(list: Value) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("list".to_owned(), list);
        m
    }

    #[test]
    fn sort_strings() {
        let inp = inputs(json!(["banana", "apple", "cherry"]));
        let out = sort(inp).unwrap();
        assert_eq!(out["result"], json!(["apple", "banana", "cherry"]));
    }

    #[test]
    fn sort_desc() {
        let mut inp = inputs(json!([3, 1, 2]));
        inp.insert("order".to_owned(), json!("desc"));
        let out = sort(inp).unwrap();
        assert_eq!(out["result"], json!([3, 2, 1]));
    }

    #[test]
    fn unique_preserves_order() {
        let inp = inputs(json!(["a", "b", "a", "c", "b"]));
        let out = unique(inp).unwrap();
        assert_eq!(out["result"], json!(["a", "b", "c"]));
    }

    #[test]
    fn length_count() {
        let inp = inputs(json!([1, 2, 3]));
        let out = length(inp).unwrap();
        assert_eq!(out["result"], json!(3));
    }

    #[test]
    fn slice_basic() {
        let mut inp = inputs(json!([10, 20, 30, 40]));
        inp.insert("start".to_owned(), json!(1));
        inp.insert("end".to_owned(), json!(3));
        let out = slice(inp).unwrap();
        assert_eq!(out["result"], json!([20, 30]));
    }

    #[test]
    fn flatten_one_level() {
        let inp = inputs(json!([[1, 2], [3, 4], 5]));
        let out = flatten(inp).unwrap();
        assert_eq!(out["result"], json!([1, 2, 3, 4, 5]));
    }
}
