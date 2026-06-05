// ---- settings ---------------------------------------------------------------

use crate::i18n::Lang;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum AiProvider {
    Anthropic,
    OpenAI,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub(crate) enum Theme {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) provider: AiProvider,
    pub(crate) api_key: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) lang: Lang,
    #[serde(default)]
    pub(crate) canvas_snap: bool,
    #[serde(default)]
    pub(crate) minimap_show: bool,
    #[serde(default)]
    pub(crate) theme: Theme,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            provider: AiProvider::Anthropic,
            api_key: String::new(),
            model: "claude-haiku-4-5-20251001".to_owned(),
            lang: Lang::default(),
            canvas_snap: false,
            minimap_show: false,
            theme: Theme::default(),
        }
    }
}

pub(crate) fn settings_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".config")
        .join("robost")
        .join("settings.toml")
}

pub(crate) const KEYRING_SERVICE: &str = "robost-editor";
pub(crate) const KEYRING_USER: &str = "api_key";

pub(crate) fn load_settings() -> AppSettings {
    let path = settings_path();
    let mut settings: AppSettings = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    // Load the API key from the OS keychain (never stored in the TOML file).
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if let Ok(key) = entry.get_password() {
            settings.api_key = key;
        }
    }
    settings
}

pub(crate) fn save_settings(s: &AppSettings) {
    // Store the API key in the OS keychain; write everything else to TOML.
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if s.api_key.is_empty() {
            let _ = entry.delete_credential();
        } else {
            let _ = entry.set_password(&s.api_key);
        }
    }
    // Persist non-secret settings (provider, model) to TOML without the api_key.
    let mut s_safe = s.clone();
    s_safe.api_key = String::new();
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string(&s_safe) {
        #[cfg(unix)]
        {
            use std::fs::OpenOptions;
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            if let Ok(mut f) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
            {
                let _ = f.write_all(text.as_bytes());
            }
        }
        #[cfg(not(unix))]
        {
            let _ = std::fs::write(&path, text);
        }
    }
}
