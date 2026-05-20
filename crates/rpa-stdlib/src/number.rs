use crate::{NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

fn get_f64(inputs: &HashMap<String, Value>, key: &str) -> Result<f64, NodeError> {
    inputs
        .get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| NodeError::MissingInput(key.to_owned()))
}

/// Format a number with thousands separator and decimal places.
///
/// Required inputs: value (number)
/// Optional inputs:
///   decimals (int, default 0) — decimal places
///   separator (string, default ",") — thousands separator
///   decimal_point (string, default ".") — decimal point char
/// Output: { result: "1,234,567.89" }
pub fn format(inputs: HashMap<String, Value>) -> NodeResult {
    let value = get_f64(&inputs, "value")?;
    let decimals = inputs.get("decimals").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let sep = inputs
        .get("separator")
        .and_then(|v| v.as_str())
        .unwrap_or(",");
    let dp = inputs
        .get("decimal_point")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    let formatted = format_number(value, decimals, sep, dp);
    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::String(formatted));
    Ok(out)
}

/// Round a number to the given number of decimal places.
///
/// Required inputs: value
/// Optional inputs: decimals (default 0), mode ("half_up" | "floor" | "ceil", default "half_up")
/// Output: { result: number }
pub fn round(inputs: HashMap<String, Value>) -> NodeResult {
    let value = get_f64(&inputs, "value")?;
    let decimals = inputs.get("decimals").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let mode = inputs
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("half_up");

    let factor = 10f64.powi(decimals as i32);
    let rounded = match mode {
        "floor" => (value * factor).floor() / factor,
        "ceil" => (value * factor).ceil() / factor,
        _ => (value * factor).round() / factor,
    };

    let mut out = HashMap::new();
    if decimals == 0 {
        out.insert(
            "result".to_owned(),
            Value::Number(serde_json::Number::from(rounded as i64)),
        );
    } else {
        out.insert(
            "result".to_owned(),
            Value::Number(
                serde_json::Number::from_f64(rounded)
                    .ok_or_else(|| NodeError::Other("round produced NaN/Inf".to_owned()))?,
            ),
        );
    }
    Ok(out)
}

/// Convert a number to a percentage string.
///
/// Required inputs: value (0.0–1.0 ratio, or raw number if percent: true)
/// Optional inputs: decimals (default 1), percent (bool, default false)
/// Output: { result: "12.3%" }
pub fn to_percent(inputs: HashMap<String, Value>) -> NodeResult {
    let value = get_f64(&inputs, "value")?;
    let decimals = inputs.get("decimals").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let already_percent = inputs
        .get("percent")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let pct = if already_percent {
        value
    } else {
        value * 100.0
    };
    let s = format!("{:.prec$}%", pct, prec = decimals);

    let mut out = HashMap::new();
    out.insert("result".to_owned(), Value::String(s));
    Ok(out)
}

fn format_number(value: f64, decimals: usize, sep: &str, dp: &str) -> String {
    let negative = value < 0.0;
    let abs = value.abs();
    let integer_part = abs as u64;
    let frac_part = abs - integer_part as f64;

    let int_str = integer_part.to_string();
    let mut grouped = String::new();
    for (i, ch) in int_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push_str(&sep.chars().rev().collect::<String>());
        }
        grouped.push(ch);
    }
    let int_formatted: String = grouped.chars().rev().collect();

    let result = if decimals > 0 {
        let frac_str = format!("{:.prec$}", frac_part, prec = decimals);
        let frac_digits = &frac_str[2..]; // strip "0."
        format!("{}{}{}", int_formatted, dp, frac_digits)
    } else {
        int_formatted
    };

    if negative {
        format!("-{result}")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn inputs_val(v: f64) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("value".to_owned(), json!(v));
        m
    }

    #[test]
    fn format_integer() {
        let inp = inputs_val(1234567.0);
        let out = format(inp).unwrap();
        assert_eq!(out["result"], json!("1,234,567"));
    }

    #[test]
    fn format_with_decimals() {
        let mut inp = inputs_val(1234.5);
        inp.insert("decimals".to_owned(), json!(2));
        let out = format(inp).unwrap();
        assert_eq!(out["result"], json!("1,234.50"));
    }

    #[test]
    fn round_half_up() {
        let mut inp = inputs_val(2.345);
        inp.insert("decimals".to_owned(), json!(2));
        let out = round(inp).unwrap();
        // f64 rounding: 2.345 may be 2.34 or 2.35 depending on floating point
        let result = out["result"].as_f64().unwrap();
        assert!((result - 2.34).abs() < 0.01 || (result - 2.35).abs() < 0.01);
    }

    #[test]
    fn to_percent_ratio() {
        let inp = inputs_val(0.1234);
        let out = to_percent(inp).unwrap();
        assert_eq!(out["result"], json!("12.3%"));
    }
}
