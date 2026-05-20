use crate::{get_str, NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

/// Add a new sheet to an existing XLSX file.
///
/// Required inputs: file, name
pub fn add_sheet(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;
    let name = get_str(&inputs, "name")?;

    let mut book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    book.new_sheet(&name)
        .map_err(|e| NodeError::Other(format!("add_sheet failed: {e}")))?;

    umya_spreadsheet::writer::xlsx::write(&book, &file)
        .map_err(|e| NodeError::Other(format!("excel save failed: {e}")))?;

    tracing::info!(file, name, "excel.add_sheet");
    Ok(HashMap::new())
}

/// Delete a sheet from an XLSX file.
///
/// Required inputs: file, name
pub fn delete_sheet(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;
    let name = get_str(&inputs, "name")?;

    let mut book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    let idx = book
        .get_sheet_collection()
        .iter()
        .position(|s| s.get_name() == name)
        .ok_or_else(|| NodeError::Other(format!("sheet not found: {name}")))?;

    book.remove_sheet(idx)
        .map_err(|e| NodeError::Other(format!("delete_sheet failed: {e}")))?;

    umya_spreadsheet::writer::xlsx::write(&book, &file)
        .map_err(|e| NodeError::Other(format!("excel save failed: {e}")))?;

    tracing::info!(file, name, "excel.delete_sheet");
    Ok(HashMap::new())
}

/// Copy a sheet within an XLSX file.
///
/// Required inputs: file, from_name, to_name
pub fn copy_sheet(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;
    let from_name = get_str(&inputs, "from_name")?;
    let to_name = get_str(&inputs, "to_name")?;

    let mut book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    let cloned = book
        .get_sheet_by_name(&from_name)
        .ok_or_else(|| NodeError::Other(format!("source sheet not found: {from_name}")))?
        .clone();

    let mut new_sheet = cloned;
    new_sheet.set_name(to_name.clone());
    book.add_sheet(new_sheet)
        .map_err(|e| NodeError::Other(format!("copy_sheet add failed: {e}")))?;

    umya_spreadsheet::writer::xlsx::write(&book, &file)
        .map_err(|e| NodeError::Other(format!("excel save failed: {e}")))?;

    tracing::info!(file, from = from_name, to = to_name, "excel.copy_sheet");
    Ok(HashMap::new())
}

/// Save an XLSX file to a new path (save-as).
///
/// Required inputs: file, dest
pub fn save_as(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;
    let dest = get_str(&inputs, "dest")?;

    let book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    umya_spreadsheet::writer::xlsx::write(&book, &dest)
        .map_err(|e| NodeError::Other(format!("excel save_as failed: {e}")))?;

    tracing::info!(file, dest, "excel.save_as");
    Ok(HashMap::new())
}

/// List sheet names in an XLSX file.
///
/// Required inputs: file
/// Output: { sheets: ["Sheet1", ...] }
pub fn list_sheets(inputs: HashMap<String, Value>) -> NodeResult {
    let file = get_str(&inputs, "file")?;

    let book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    let names: Vec<Value> = book
        .get_sheet_collection()
        .iter()
        .map(|s| Value::String(s.get_name().to_owned()))
        .collect();

    let mut out = HashMap::new();
    out.insert("sheets".to_owned(), Value::Array(names));
    Ok(out)
}

// ── write_range / set_formula ──────────────────────────────────────────────

/// Parse "A1" → (col, row) as 1-indexed. Returns None on invalid input.
fn parse_cell_ref(s: &str) -> Option<(u32, u32)> {
    let s = s.trim().to_uppercase();
    let col_str: String = s.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let row_str = &s[col_str.len()..];
    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }
    let col = col_str
        .chars()
        .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));
    let row = row_str.parse::<u32>().ok()?;
    Some((col, row))
}

fn col_to_letters(mut col: u32) -> String {
    let mut s = String::new();
    while col > 0 {
        col -= 1;
        s.insert(0, (b'A' + (col % 26) as u8) as char);
        col /= 26;
    }
    s
}

fn set_cell(cell: &mut umya_spreadsheet::Cell, v: &Value) {
    match v {
        Value::Null          => {}
        Value::Bool(b)       => { cell.set_value_bool(*b); }
        Value::Number(n)     => {
            if let Some(f) = n.as_f64() { cell.set_value_number(f); }
        }
        Value::String(s)     => { cell.set_value_string(s); }
        other                => { cell.set_value_string(other.to_string()); }
    }
}

/// Write a 2D array of values into a sheet starting at the given cell.
///
/// Required: file, sheet, start (e.g. "A1"), data ([[row values]])
pub fn write_range(inputs: HashMap<String, Value>) -> NodeResult {
    let file  = get_str(&inputs, "file")?;
    let sheet = get_str(&inputs, "sheet")?;
    let start = get_str(&inputs, "start")?;

    let data = inputs
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| NodeError::MissingInput("data".to_owned()))?;

    let (start_col, start_row) = parse_cell_ref(&start)
        .ok_or_else(|| NodeError::Other(format!("invalid cell reference: {start}")))?;

    let mut book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    let ws = book
        .get_sheet_by_name_mut(&sheet)
        .ok_or_else(|| NodeError::Other(format!("sheet not found: {sheet}")))?;

    let row_count = data.len();
    for (r, row_val) in data.iter().enumerate() {
        if let Some(cols) = row_val.as_array() {
            for (c, cell_val) in cols.iter().enumerate() {
                let col = start_col + c as u32;
                let row = start_row + r as u32;
                set_cell(ws.get_cell_mut((col, row)), cell_val);
            }
        }
    }

    umya_spreadsheet::writer::xlsx::write(&book, &file)
        .map_err(|e| NodeError::Other(format!("excel save failed: {e}")))?;

    tracing::info!(file, sheet, start, rows = row_count, "excel.write_range");
    Ok(HashMap::new())
}

/// Write a formula to a single cell.
///
/// Required: file, sheet, cell (e.g. "C2"), formula (e.g. "=SUM(A1:A10)")
pub fn set_formula(inputs: HashMap<String, Value>) -> NodeResult {
    let file    = get_str(&inputs, "file")?;
    let sheet   = get_str(&inputs, "sheet")?;
    let cell    = get_str(&inputs, "cell")?;
    let formula = get_str(&inputs, "formula")?;

    let (col, row) = parse_cell_ref(&cell)
        .ok_or_else(|| NodeError::Other(format!("invalid cell reference: {cell}")))?;

    let formula_str = formula.trim_start_matches('=');

    let mut book = umya_spreadsheet::reader::xlsx::read(&file)
        .map_err(|e| NodeError::Other(format!("excel open failed: {e}")))?;

    let ws = book
        .get_sheet_by_name_mut(&sheet)
        .ok_or_else(|| NodeError::Other(format!("sheet not found: {sheet}")))?;

    ws.get_cell_mut((col, row)).set_formula(formula_str);

    umya_spreadsheet::writer::xlsx::write(&book, &file)
        .map_err(|e| NodeError::Other(format!("excel save failed: {e}")))?;

    let cell_label = format!("{}{}", col_to_letters(col), row);
    tracing::info!(file, sheet, cell = cell_label, "excel.set_formula");
    Ok(HashMap::new())
}
