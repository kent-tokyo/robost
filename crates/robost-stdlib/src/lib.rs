// Built-in scenario node library.
// Called via `library: <name>` in scenario YAML.
// All functions share: inputs map → outputs map.

#[cfg(feature = "archive")]
mod archive;
#[cfg(feature = "db")]
mod db;
mod excel;
#[cfg(feature = "ftp")]
mod ftp;
#[cfg(feature = "keychain")]
mod keychain;
mod list;
#[cfg(feature = "mail")]
mod mail;
mod notify;
mod number;
#[cfg(feature = "pdf")]
mod pdf;

use serde_json::Value;
use std::collections::HashMap;

pub type NodeResult = Result<HashMap<String, Value>, NodeError>;

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("missing required input: {0}")]
    MissingInput(String),
    #[error("node error: {0}")]
    Other(String),
}

pub(crate) fn get_str(inputs: &HashMap<String, Value>, key: &str) -> Result<String, NodeError> {
    inputs
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| NodeError::MissingInput(key.to_owned()))
}

pub(crate) fn opt_str(inputs: &HashMap<String, Value>, key: &str) -> Option<String> {
    inputs.get(key).and_then(|v| v.as_str()).map(str::to_owned)
}

/// Dispatch a built-in library node by name.
///
/// Name format: `"category.function"`, e.g. `"mail.smtp_send"`, `"list.sort"`.
pub fn dispatch(name: &str, inputs: HashMap<String, Value>) -> NodeResult {
    match name {
        // --- mail ---
        #[cfg(feature = "mail")]
        "mail.smtp_send" => mail::smtp_send(inputs),
        #[cfg(feature = "mail")]
        "mail.imap_receive" => mail::imap_receive(inputs),

        // --- excel ---
        "excel.add_sheet" => excel::add_sheet(inputs),
        "excel.delete_sheet" => excel::delete_sheet(inputs),
        "excel.rename_sheet" => excel::rename_sheet(inputs),
        "excel.copy_sheet" => excel::copy_sheet(inputs),
        "excel.save_as" => excel::save_as(inputs),
        "excel.list_sheets" => excel::list_sheets(inputs),
        "excel.write_range" => excel::write_range(inputs),
        "excel.set_formula" => excel::set_formula(inputs),

        // --- list ---
        "list.sort" => list::sort(inputs),
        "list.unique" => list::unique(inputs),
        "list.reverse" => list::reverse(inputs),
        "list.length" => list::length(inputs),
        "list.get" => list::get(inputs),
        "list.slice" => list::slice(inputs),
        "list.flatten" => list::flatten(inputs),

        // --- number ---
        "number.format" => number::format(inputs),
        "number.round" => number::round(inputs),
        "number.to_percent" => number::to_percent(inputs),

        // --- db (optional feature) ---
        #[cfg(feature = "db")]
        "db.query" => db::query(inputs),
        #[cfg(feature = "db")]
        "db.query_one" => db::query_one(inputs),
        #[cfg(feature = "db")]
        "db.execute" => db::execute(inputs),

        // --- archive (optional feature) ---
        #[cfg(feature = "archive")]
        "archive.compress" => archive::compress(inputs),
        #[cfg(feature = "archive")]
        "archive.extract" => archive::extract(inputs),
        #[cfg(feature = "archive")]
        "archive.list" => archive::list(inputs),

        // --- ftp (optional feature) ---
        #[cfg(feature = "ftp")]
        "ftp.upload" => ftp::upload(inputs),
        #[cfg(feature = "ftp")]
        "ftp.download" => ftp::download(inputs),
        #[cfg(feature = "ftp")]
        "ftp.list" => ftp::list(inputs),
        #[cfg(feature = "ftp")]
        "ftp.delete" => ftp::delete(inputs),
        #[cfg(feature = "ftp")]
        "ftp.mkdir" => ftp::mkdir(inputs),

        // --- pdf (optional feature) ---
        #[cfg(feature = "pdf")]
        "pdf.extract_text" => pdf::extract_text(inputs),
        #[cfg(feature = "pdf")]
        "pdf.page_count" => pdf::page_count(inputs),

        // --- notify (Slack / Teams webhook) ---
        "notify.slack_send" => notify::slack_send(inputs),
        "notify.teams_send" => notify::teams_send(inputs),

        // --- keychain (optional feature) ---
        #[cfg(feature = "keychain")]
        "keychain.get" => keychain::keychain_get(inputs),
        #[cfg(feature = "keychain")]
        "keychain.set" => keychain::keychain_set(inputs),
        #[cfg(feature = "keychain")]
        "keychain.delete" => keychain::keychain_delete(inputs),

        other => Err(NodeError::Other(format!("unknown built-in node: {other}"))),
    }
}
