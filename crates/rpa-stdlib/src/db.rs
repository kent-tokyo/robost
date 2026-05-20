use crate::{get_str, NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

fn open_sqlite(url: &str) -> Result<rusqlite::Connection, NodeError> {
    let path = url
        .strip_prefix("sqlite://")
        .or_else(|| url.strip_prefix("sqlite:"))
        .unwrap_or(url);
    rusqlite::Connection::open(path).map_err(|e| NodeError::Other(format!("db open failed: {e}")))
}

fn bind_params(stmt: &mut rusqlite::Statement<'_>, params: &[Value]) -> Result<(), NodeError> {
    for (i, v) in params.iter().enumerate() {
        let idx = i + 1;
        match v {
            Value::Null => stmt.raw_bind_parameter(idx, rusqlite::types::Null),
            Value::Bool(b) => stmt.raw_bind_parameter(idx, b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    stmt.raw_bind_parameter(idx, i)
                } else {
                    stmt.raw_bind_parameter(idx, n.as_f64().unwrap_or(0.0))
                }
            }
            Value::String(s) => stmt.raw_bind_parameter(idx, s.as_str()),
            other => stmt.raw_bind_parameter(idx, other.to_string()),
        }
        .map_err(|e| NodeError::Other(format!("bind param {idx}: {e}")))?;
    }
    Ok(())
}

fn row_to_map(
    row: &rusqlite::Row<'_>,
    cols: &[String],
) -> Result<serde_json::Map<String, Value>, rusqlite::Error> {
    let mut map = serde_json::Map::new();
    for (i, col) in cols.iter().enumerate() {
        let val: rusqlite::types::Value = row.get(i)?;
        let json_val = match val {
            rusqlite::types::Value::Null => Value::Null,
            rusqlite::types::Value::Integer(n) => Value::Number(n.into()),
            rusqlite::types::Value::Real(f) => {
                Value::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
            }
            rusqlite::types::Value::Text(s) => Value::String(s),
            rusqlite::types::Value::Blob(b) => Value::String(base64_encode(&b)),
        };
        map.insert(col.clone(), json_val);
    }
    Ok(map)
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let _ = write!(out, "{}", TABLE[b0 >> 2] as char);
        let _ = write!(out, "{}", TABLE[((b0 & 3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            let _ = write!(out, "{}", TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            let _ = write!(out, "{}", TABLE[b2 & 0x3f] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Execute a SELECT query and return all rows.
///
/// Required: url, sql
/// Optional: params (JSON array of bind values)
/// Output: { rows: [{col: val, ...}, ...] }
pub fn query(inputs: HashMap<String, Value>) -> NodeResult {
    let url = get_str(&inputs, "url")?;
    let sql = get_str(&inputs, "sql")?;
    let params: Vec<Value> = inputs
        .get("params")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let conn = open_sqlite(&url)?;
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| NodeError::Other(format!("db prepare failed: {e}")))?;

    let cols: Vec<String> = stmt.column_names().into_iter().map(str::to_owned).collect();

    bind_params(&mut stmt, &params)?;

    let rows: Result<Vec<Value>, rusqlite::Error> = stmt
        .raw_query()
        .mapped(|row| row_to_map(row, &cols).map(Value::Object))
        .collect();

    let rows = rows.map_err(|e| NodeError::Other(format!("db query failed: {e}")))?;

    tracing::info!(url, sql, count = rows.len(), "db.query");
    let mut out = HashMap::new();
    out.insert("rows".to_owned(), Value::Array(rows));
    Ok(out)
}

/// Execute a single SELECT query and return the first row (or null).
///
/// Required: url, sql
/// Optional: params
/// Output: { row: {col: val, ...} | null }
pub fn query_one(inputs: HashMap<String, Value>) -> NodeResult {
    let url = get_str(&inputs, "url")?;
    let sql = get_str(&inputs, "sql")?;
    let params: Vec<Value> = inputs
        .get("params")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let conn = open_sqlite(&url)?;
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| NodeError::Other(format!("db prepare failed: {e}")))?;

    let cols: Vec<String> = stmt.column_names().into_iter().map(str::to_owned).collect();

    bind_params(&mut stmt, &params)?;

    let row_val = stmt
        .raw_query()
        .mapped(|row| row_to_map(row, &cols).map(Value::Object))
        .next()
        .transpose()
        .map_err(|e| NodeError::Other(format!("db query_one failed: {e}")))?
        .unwrap_or(Value::Null);

    tracing::info!(url, sql, "db.query_one");
    let mut out = HashMap::new();
    out.insert("row".to_owned(), row_val);
    Ok(out)
}

/// Execute an INSERT/UPDATE/DELETE statement.
///
/// Required: url, sql
/// Optional: params
/// Output: { rows_affected: N, last_insert_id: N }
pub fn execute(inputs: HashMap<String, Value>) -> NodeResult {
    let url = get_str(&inputs, "url")?;
    let sql = get_str(&inputs, "sql")?;
    let params: Vec<Value> = inputs
        .get("params")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let conn = open_sqlite(&url)?;
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| NodeError::Other(format!("db prepare failed: {e}")))?;

    bind_params(&mut stmt, &params)?;

    let rows_affected = stmt
        .raw_execute()
        .map_err(|e| NodeError::Other(format!("db execute failed: {e}")))?;

    let last_insert_id = conn.last_insert_rowid();

    tracing::info!(url, sql, rows_affected, "db.execute");
    let mut out = HashMap::new();
    out.insert(
        "rows_affected".to_owned(),
        Value::Number(rows_affected.into()),
    );
    out.insert(
        "last_insert_id".to_owned(),
        Value::Number(last_insert_id.into()),
    );
    Ok(out)
}
