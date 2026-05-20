use crate::{get_str, NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

fn keyring_err(e: keyring::Error) -> NodeError {
    // Don't include keyring::Error's Display (which may expose service/account names)
    NodeError::Other(format!(
        "keychain operation failed: {}",
        e.to_string().split(':').next().unwrap_or("unknown error")
    ))
}

/// Store a password in the OS keychain.
///
/// Required: service, account, password
pub fn keychain_set(inputs: HashMap<String, Value>) -> NodeResult {
    let service = get_str(&inputs, "service")?;
    let account = get_str(&inputs, "account")?;
    let password = get_str(&inputs, "password")?;

    let entry = keyring::Entry::new(&service, &account).map_err(keyring_err)?;
    entry.set_password(&password).map_err(keyring_err)?;

    tracing::info!(service, account, "keychain.set");
    Ok(HashMap::new())
}

/// Retrieve a password from the OS keychain.
///
/// Required: service, account
/// Output: { password: "..." }
pub fn keychain_get(inputs: HashMap<String, Value>) -> NodeResult {
    let service = get_str(&inputs, "service")?;
    let account = get_str(&inputs, "account")?;

    let entry = keyring::Entry::new(&service, &account).map_err(keyring_err)?;
    let password = entry.get_password().map_err(keyring_err)?;

    tracing::info!(service, account, "keychain.get");
    let mut out = HashMap::new();
    out.insert("password".to_owned(), Value::String(password));
    Ok(out)
}

/// Delete a password from the OS keychain.
///
/// Required: service, account
pub fn keychain_delete(inputs: HashMap<String, Value>) -> NodeResult {
    let service = get_str(&inputs, "service")?;
    let account = get_str(&inputs, "account")?;

    let entry = keyring::Entry::new(&service, &account).map_err(keyring_err)?;
    entry.delete_credential().map_err(keyring_err)?;

    tracing::info!(service, account, "keychain.delete");
    Ok(HashMap::new())
}
