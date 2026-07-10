use std::path::Path;

use calamine::{open_workbook_auto, Reader};
use thiserror::Error;

/// Errors that can occur while loading a [`DataSource`](crate::scenario::DataSource) file.
#[derive(Debug, Error)]
pub enum DataSourceError {
    /// The file at the given path doesn't exist.
    #[error("data source file not found: {0}")]
    NotFound(String),
    /// The file failed to parse as CSV.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    /// The file failed to parse as XLSX.
    #[error("XLSX error: {0}")]
    Xlsx(#[from] calamine::Error),
    /// The requested sheet name doesn't exist in the workbook.
    #[error("sheet '{0}' not found in workbook")]
    SheetNotFound(String),
    /// The file has no header row to derive column names from.
    #[error("data source has no header row")]
    EmptyHeaders,
}

/// A single data row, keyed by column header name.
pub type DataRow = std::collections::HashMap<String, serde_json::Value>;

/// Load a CSV or XLSX file and return rows as maps keyed by header names.
pub fn load(path: &Path, sheet: Option<&str>) -> Result<Vec<DataRow>, DataSourceError> {
    if !path.exists() {
        return Err(DataSourceError::NotFound(path.display().to_string()));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "csv" {
        load_csv(path)
    } else {
        load_xlsx(path, sheet)
    }
}

fn load_csv(path: &Path) -> Result<Vec<DataRow>, DataSourceError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_owned()).collect();

    if headers.is_empty() {
        return Err(DataSourceError::EmptyHeaders);
    }

    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let row: DataRow = headers
            .iter()
            .zip(record.iter())
            .map(|(h, v)| (h.clone(), serde_json::Value::String(v.to_owned())))
            .collect();
        rows.push(row);
    }
    Ok(rows)
}

fn load_xlsx(path: &Path, sheet: Option<&str>) -> Result<Vec<DataRow>, DataSourceError> {
    let mut workbook = open_workbook_auto(path)?;

    let sheet_name = match sheet {
        Some(s) => s.to_owned(),
        None => workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| DataSourceError::SheetNotFound("(first)".into()))?,
    };

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| DataSourceError::SheetNotFound(sheet_name.clone()))?;

    let mut iter = range.rows();

    let headers: Vec<String> = match iter.next() {
        Some(row) => row.iter().map(|c| c.to_string()).collect(),
        None => return Err(DataSourceError::EmptyHeaders),
    };

    if headers.is_empty() {
        return Err(DataSourceError::EmptyHeaders);
    }

    let mut rows = Vec::new();
    for data_row in iter {
        let row: DataRow = headers
            .iter()
            .zip(data_row.iter())
            .map(|(h, cell)| {
                let v = match cell {
                    calamine::Data::Int(i) => serde_json::Value::Number((*i).into()),
                    calamine::Data::Float(f) => serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(f.to_string())),
                    calamine::Data::Bool(b) => serde_json::Value::Bool(*b),
                    calamine::Data::Empty => serde_json::Value::Null,
                    other => serde_json::Value::String(other.to_string()),
                };
                (h.clone(), v)
            })
            .collect();
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn csv_tempfile(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        write!(f, "{content}").unwrap();
        f
    }

    #[test]
    fn csv_load_basic() {
        let f = csv_tempfile("name,age\nAlice,30\nBob,25\n");
        let rows = load(f.path(), None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], serde_json::json!("Alice"));
        assert_eq!(rows[1]["age"], serde_json::json!("25"));
    }

    #[test]
    fn csv_not_found() {
        let err = load(std::path::Path::new("/nonexistent/path.csv"), None).unwrap_err();
        assert!(matches!(err, DataSourceError::NotFound(_)));
    }

    #[test]
    fn csv_empty_file() {
        let f = csv_tempfile("");
        let err = load(f.path(), None).unwrap_err();
        assert!(matches!(
            err,
            DataSourceError::Csv(_) | DataSourceError::EmptyHeaders
        ));
    }
}
