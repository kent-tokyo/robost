use robost_plugin_api::{PluginError, PluginManifest, PluginResult, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "wasm")]
use wasmtime::{Engine, Module};

#[derive(Debug, Error)]
pub enum HostError {
    #[error("manifest not found at {0}")]
    ManifestNotFound(PathBuf),
    #[error("manifest parse error: {0}")]
    ManifestParse(String),
    #[error("wasm load error: {0}")]
    WasmLoad(String),
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("permission denied: plugin requested '{0}' but it was not granted")]
    PermissionDenied(String),
    #[error("execution timeout")]
    Timeout,
    #[error("wasm trap: {0}")]
    Trap(String),
}

pub type Result<T> = std::result::Result<T, HostError>;

/// A loaded plugin ready to call functions on.
pub struct PluginInstance {
    pub manifest: PluginManifest,
    /// Directory containing the plugin's `.wasm` and `plugin.toml` — the root
    /// a plugin's sandboxed filesystem access (if any) is scoped under.
    #[cfg_attr(not(feature = "wasm"), allow(dead_code))]
    plugin_dir: PathBuf,
    #[cfg(feature = "wasm")]
    engine: Arc<Engine>,
    #[cfg(feature = "wasm")]
    module: Module,
}

impl PluginInstance {
    /// Load a plugin from a `.wasm` file and accompanying `plugin.toml`.
    /// Pre-compiles the wasm module so subsequent `call()` invocations are fast.
    pub fn load(wasm_path: impl AsRef<Path>) -> Result<Self> {
        let wasm_path = wasm_path.as_ref().to_path_buf();
        let manifest_path = wasm_path.with_file_name("plugin.toml");
        let plugin_dir = wasm_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let raw = std::fs::read_to_string(&manifest_path)
            .map_err(|_| HostError::ManifestNotFound(manifest_path.clone()))?;
        let manifest: PluginManifest =
            toml::from_str(&raw).map_err(|e| HostError::ManifestParse(e.to_string()))?;

        tracing::info!(
            plugin = %manifest.plugin.name,
            version = %manifest.plugin.version,
            "loaded plugin manifest"
        );

        #[cfg(feature = "wasm")]
        let (engine, module) = Self::compile(&wasm_path)?;

        Ok(Self {
            manifest,
            plugin_dir,
            #[cfg(feature = "wasm")]
            engine,
            #[cfg(feature = "wasm")]
            module,
        })
    }

    /// Maps the manifest's `permissions.filesystem` flags to a single sandboxed
    /// directory (`<plugin_dir>/data`) plugins may be granted read and/or write
    /// access to. Returns `None` if the plugin declared no filesystem permission.
    #[cfg_attr(not(feature = "wasm"), allow(dead_code))]
    fn resolve_fs_preopen(
        filesystem_perms: &[String],
        plugin_dir: &Path,
    ) -> Option<(PathBuf, bool)> {
        let readable = filesystem_perms.iter().any(|p| p == "read");
        let writable = filesystem_perms.iter().any(|p| p == "write");
        if !readable && !writable {
            return None;
        }
        Some((plugin_dir.join("data"), writable))
    }

    /// Compile a wasm module with epoch-interruption enabled.
    #[cfg(feature = "wasm")]
    fn compile(path: &Path) -> Result<(Arc<Engine>, Module)> {
        let mut config = wasmtime::Config::new();
        config.epoch_interruption(true);
        let engine =
            Arc::new(Engine::new(&config).map_err(|e| HostError::WasmLoad(e.to_string()))?);
        let module =
            Module::from_file(&engine, path).map_err(|e| HostError::WasmLoad(e.to_string()))?;
        Ok((engine, module))
    }

    /// Call a named function with the given inputs.
    ///
    /// Protocol (JSON over WASI stdio):
    /// - stdin  → `{"function":"<name>","inputs":{...}}\n`
    /// - stdout → `{"ok":{...}}\n` or `{"err":"<message>"}\n`
    ///
    /// Permissions are enforced by restricting WASI capabilities; the manifest
    /// declaration is a secondary human-readable annotation.
    pub fn call(&self, function: &str, inputs: HashMap<String, Value>) -> PluginResult {
        // Verify the function is declared in the manifest.
        self.manifest
            .function
            .iter()
            .find(|f| f.name == function)
            .ok_or_else(|| PluginError::Other(format!("function not found: {function}")))?;

        #[cfg(feature = "wasm")]
        return self.run_wasm(function, inputs);

        #[cfg(not(feature = "wasm"))]
        {
            let _ = inputs;
            tracing::warn!(function, "plugin call stubbed — wasm feature not enabled");
            Err(PluginError::Other("wasm feature not enabled".into()))
        }
    }

    #[cfg(feature = "wasm")]
    fn run_wasm(&self, function: &str, inputs: HashMap<String, Value>) -> PluginResult {
        use serde_json::Value as JValue;
        use std::sync::atomic::{AtomicBool, Ordering};
        use wasmtime::Linker;
        use wasmtime::Store;
        use wasmtime_wasi::p1::{self, WasiP1Ctx};
        use wasmtime_wasi::p2::pipe::{MemoryInputPipe, MemoryOutputPipe};
        use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

        // --- Build JSON request ---
        let request = serde_json::json!({
            "function": function,
            "inputs": inputs,
        });
        let mut stdin_bytes =
            serde_json::to_vec(&request).map_err(|e| PluginError::Other(e.to_string()))?;
        stdin_bytes.push(b'\n');

        // --- Set up in-memory stdio ---
        let stdout = MemoryOutputPipe::new(4 * 1024 * 1024); // 4 MiB cap
        let stderr = MemoryOutputPipe::new(64 * 1024);

        let mut builder = WasiCtxBuilder::new();
        builder
            .stdin(MemoryInputPipe::new(stdin_bytes))
            .stdout(stdout.clone())
            .stderr(stderr.clone());

        // Gate filesystem access: default = none. Plugins that declare "read"/"write"
        // in their manifest get a single sandboxed directory scoped to their own plugin dir.
        if let Some((data_dir, writable)) =
            Self::resolve_fs_preopen(&self.manifest.permissions.filesystem, &self.plugin_dir)
        {
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| PluginError::Other(format!("plugin data dir: {e}")))?;
            let (dir_perms, file_perms) = if writable {
                (DirPerms::all(), FilePerms::all())
            } else {
                (DirPerms::READ, FilePerms::READ)
            };
            builder
                .preopened_dir(&data_dir, "/data", dir_perms, file_perms)
                .map_err(|e| PluginError::Other(format!("preopen plugin data dir: {e}")))?;
        }

        let wasi: WasiP1Ctx = builder.build_p1();

        let mut store = Store::new(&self.engine, wasi);
        // 30-epoch budget at 100 ms/tick → 3-second timeout.
        store.set_epoch_deadline(30);

        // Ticker thread: increments epoch every 100 ms so the deadline can fire.
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let engine_for_ticker = Arc::clone(&self.engine);
        let ticker = std::thread::spawn(move || {
            while !stop_clone.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
                engine_for_ticker.increment_epoch();
            }
        });
        // Guard ensures the ticker is always stopped, even if instantiate/get_typed_func fails.
        struct StopGuard(Arc<AtomicBool>, Option<std::thread::JoinHandle<()>>);
        impl Drop for StopGuard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
                if let Some(h) = self.1.take() {
                    let _ = h.join();
                }
            }
        }
        let _stop_guard = StopGuard(Arc::clone(&stop), Some(ticker));

        let mut linker: Linker<WasiP1Ctx> = Linker::new(&self.engine);
        p1::add_to_linker_sync(&mut linker, |s| s)
            .map_err(|e| PluginError::Other(format!("linker setup: {e}")))?;

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| PluginError::Other(format!("instantiate: {e}")))?;

        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|_| PluginError::Other("plugin has no _start export".into()))?;

        let call_result = start.call(&mut store, ());
        drop(_stop_guard); // stop ticker thread before processing result

        match call_result {
            Ok(()) => {}
            Err(e) => {
                // proc_exit(0) is normal termination in WASI; non-zero is a failure.
                if let Some(exit) = e.downcast_ref::<wasmtime_wasi::I32Exit>() {
                    let code = exit.0;
                    Self::log_stderr(&stderr);
                    if code != 0 {
                        return Err(PluginError::Other(format!(
                            "plugin exited with non-zero code: {code}"
                        )));
                    }
                    // code == 0 → fall through to parse stdout
                } else {
                    return Err(PluginError::Other(format!("wasm trap: {e:#}")));
                }
            }
        }

        Self::log_stderr(&stderr);

        // --- Parse stdout as JSON response ---
        let out_bytes = stdout.contents();
        if out_bytes.is_empty() {
            return Err(PluginError::Other("plugin produced no output".into()));
        }

        let response: JValue = serde_json::from_slice(&out_bytes).map_err(|e| {
            // Do not include raw output in the error — it may contain secrets from plugin inputs.
            tracing::trace!(raw = %String::from_utf8_lossy(&out_bytes), "plugin output parse failed");
            PluginError::Other(format!("parse plugin output: {e}"))
        })?;

        if let Some(ok) = response.get("ok") {
            let outputs: HashMap<String, Value> = serde_json::from_value(ok.clone())
                .map_err(|e| PluginError::Other(format!("deserialize outputs: {e}")))?;
            Ok(outputs)
        } else if let Some(err) = response.get("err") {
            Err(PluginError::Other(
                err.as_str().unwrap_or("unknown plugin error").to_string(),
            ))
        } else {
            Err(PluginError::Other(
                "unexpected plugin response (expected {\"ok\":...} or {\"err\":...})".into(),
            ))
        }
    }

    #[cfg(feature = "wasm")]
    fn log_stderr(pipe: &wasmtime_wasi::p2::pipe::MemoryOutputPipe) {
        let bytes = pipe.contents();
        if !bytes.is_empty() {
            tracing::debug!(stderr = %String::from_utf8_lossy(&bytes), "plugin stderr");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_fs_preopen_none_when_no_permission_declared() {
        let dir = Path::new("/plugins/example");
        assert!(PluginInstance::resolve_fs_preopen(&[], dir).is_none());
        assert!(PluginInstance::resolve_fs_preopen(&["network".to_string()], dir).is_none());
    }

    #[test]
    fn resolve_fs_preopen_read_only() {
        let dir = Path::new("/plugins/example");
        let (data_dir, writable) =
            PluginInstance::resolve_fs_preopen(&["read".to_string()], dir).unwrap();
        assert_eq!(data_dir, dir.join("data"));
        assert!(!writable);
    }

    #[test]
    fn resolve_fs_preopen_write_implies_writable() {
        let dir = Path::new("/plugins/example");
        let (data_dir, writable) =
            PluginInstance::resolve_fs_preopen(&["write".to_string()], dir).unwrap();
        assert_eq!(data_dir, dir.join("data"));
        assert!(writable);
    }

    #[test]
    fn resolve_fs_preopen_read_and_write() {
        let dir = Path::new("/plugins/example");
        let perms = vec!["read".to_string(), "write".to_string()];
        let (_, writable) = PluginInstance::resolve_fs_preopen(&perms, dir).unwrap();
        assert!(writable);
    }
}
