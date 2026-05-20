use crate::{get_str, NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Compress files/directories into a ZIP archive.
///
/// Required: dest (output .zip path), files (JSON array of source paths)
pub fn compress(inputs: HashMap<String, Value>) -> NodeResult {
    let dest = get_str(&inputs, "dest")?;
    let files: Vec<String> = inputs
        .get("files")
        .and_then(|v| v.as_array())
        .ok_or_else(|| NodeError::MissingInput("files".to_owned()))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();

    let dest_path = Path::new(&dest);
    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NodeError::Other(format!("create dir failed: {e}")))?;
        }
    }

    let file = std::fs::File::create(dest_path)
        .map_err(|e| NodeError::Other(format!("create zip failed: {e}")))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for src in &files {
        let src_path = Path::new(src);
        if src_path.is_file() {
            let name = src_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(src.as_str());
            zip.start_file(name, options)
                .map_err(|e| NodeError::Other(format!("zip start_file failed: {e}")))?;
            let content = std::fs::read(src_path)
                .map_err(|e| NodeError::Other(format!("read {src}: {e}")))?;
            use std::io::Write;
            zip.write_all(&content)
                .map_err(|e| NodeError::Other(format!("zip write failed: {e}")))?;
        } else if src_path.is_dir() {
            add_dir_to_zip(&mut zip, src_path, src_path, options)?;
        } else {
            return Err(NodeError::Other(format!("not found: {src}")));
        }
    }

    zip.finish()
        .map_err(|e| NodeError::Other(format!("zip finish failed: {e}")))?;

    tracing::info!(dest, count = files.len(), "archive.compress");
    Ok(HashMap::new())
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), NodeError> {
    for entry in std::fs::read_dir(dir)
        .map_err(|e| NodeError::Other(format!("read_dir failed: {e}")))?
    {
        let entry = entry.map_err(|e| NodeError::Other(format!("dir entry: {e}")))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .map_err(|e| NodeError::Other(format!("strip_prefix: {e}")))?;
        let name = rel.to_string_lossy().replace('\\', "/");

        if path.is_file() {
            zip.start_file(&name, options)
                .map_err(|e| NodeError::Other(format!("zip start_file: {e}")))?;
            let content = std::fs::read(&path)
                .map_err(|e| NodeError::Other(format!("read {}: {e}", path.display())))?;
            use std::io::Write;
            zip.write_all(&content)
                .map_err(|e| NodeError::Other(format!("zip write: {e}")))?;
        } else if path.is_dir() {
            zip.add_directory(&name, options)
                .map_err(|e| NodeError::Other(format!("zip add_directory: {e}")))?;
            add_dir_to_zip(zip, base, &path, options)?;
        }
    }
    Ok(())
}

/// Extract a ZIP archive to a directory.
///
/// Required: src (.zip path), dest (output directory)
pub fn extract(inputs: HashMap<String, Value>) -> NodeResult {
    let src = get_str(&inputs, "src")?;
    let dest = get_str(&inputs, "dest")?;

    let file = std::fs::File::open(&src)
        .map_err(|e| NodeError::Other(format!("open zip failed: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| NodeError::Other(format!("read zip failed: {e}")))?;

    let dest_path = Path::new(&dest);
    std::fs::create_dir_all(dest_path)
        .map_err(|e| NodeError::Other(format!("create dest failed: {e}")))?;

    let count = archive.len();
    for i in 0..count {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| NodeError::Other(format!("zip entry {i}: {e}")))?;

        let out_path = dest_path.join(
            entry
                .enclosed_name()
                .ok_or_else(|| NodeError::Other(format!("unsafe path in zip entry {i}")))?,
        );

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| NodeError::Other(format!("mkdir: {e}")))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| NodeError::Other(format!("mkdir: {e}")))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| NodeError::Other(format!("create file: {e}")))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| NodeError::Other(format!("extract write: {e}")))?;
        }
    }

    tracing::info!(src, dest, files = count, "archive.extract");
    let mut out = HashMap::new();
    out.insert("files".to_owned(), Value::Number(count.into()));
    Ok(out)
}

/// List files inside a ZIP archive.
///
/// Required: src (.zip path)
/// Output: { files: ["name", ...] }
pub fn list(inputs: HashMap<String, Value>) -> NodeResult {
    let src = get_str(&inputs, "src")?;

    let file = std::fs::File::open(&src)
        .map_err(|e| NodeError::Other(format!("open zip failed: {e}")))?;
    let archive = zip::ZipArchive::new(file)
        .map_err(|e| NodeError::Other(format!("read zip failed: {e}")))?;

    let names: Vec<Value> = (0..archive.len())
        .filter_map(|i| archive.name_for_index(i).map(|n| Value::String(n.to_owned())))
        .collect();

    tracing::info!(src, count = names.len(), "archive.list");
    let mut out = HashMap::new();
    out.insert("files".to_owned(), Value::Array(names));
    Ok(out)
}
