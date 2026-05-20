use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// OS-specific config directory: ~/Library/Application Support on macOS,
/// %APPDATA% on Windows, ~/.config on Linux.
pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".config"))
            .unwrap_or_else(|_| PathBuf::from("."))
    }
}

pub fn rust_rpa_dir() -> PathBuf {
    config_dir().join("rust_rpa")
}

pub fn plugins_dir() -> PathBuf {
    rust_rpa_dir().join("plugins")
}

pub fn plugin_registry_path() -> PathBuf {
    rust_rpa_dir().join("plugins.json")
}

// ── Plugin registry ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub name: String,
    pub version: String,
    /// Absolute path to the installed .wasm file.
    pub wasm_path: PathBuf,
}

pub fn load_registry() -> Vec<PluginEntry> {
    let path = plugin_registry_path();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_registry(entries: &[PluginEntry]) -> Result<()> {
    let path = plugin_registry_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(entries)?;
    std::fs::write(&path, json)?;
    Ok(())
}

pub fn install_plugin(wasm_src: &std::path::Path, manifest: &rpa_plugin_api::PluginManifest) -> Result<PathBuf> {
    let name = &manifest.plugin.name;
    let dest_dir = plugins_dir().join(name);
    std::fs::create_dir_all(&dest_dir)?;

    // Copy .wasm file.
    let wasm_filename = wasm_src
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("plugin.wasm"));
    let wasm_dest = dest_dir.join(wasm_filename);
    std::fs::copy(wasm_src, &wasm_dest)?;

    // Copy plugin.toml.
    let toml_src = wasm_src.with_file_name("plugin.toml");
    if toml_src.exists() {
        std::fs::copy(&toml_src, dest_dir.join("plugin.toml"))?;
    }

    // Update registry: remove any old entry with the same name, prepend new one.
    let mut entries = load_registry();
    entries.retain(|e| e.name != *name);
    entries.insert(
        0,
        PluginEntry {
            name: name.clone(),
            version: manifest.plugin.version.clone(),
            wasm_path: wasm_dest.clone(),
        },
    );
    save_registry(&entries)?;

    Ok(wasm_dest)
}
