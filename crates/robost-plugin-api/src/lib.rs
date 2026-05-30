use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Dynamically-typed value passed between the host and a plugin function.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Errors a plugin function may return to the host.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum PluginError {
    #[error("invalid argument '{name}': {reason}")]
    InvalidArgument { name: String, reason: String },
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("plugin error: {0}")]
    Other(String),
}

/// Type alias for plugin function results.
pub type PluginResult = Result<HashMap<String, Value>, PluginError>;

/// Manifest read from `plugin.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,
    #[serde(default)]
    pub function: Vec<FunctionDef>,
    #[serde(default)]
    pub permissions: Permissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<ParamDef>,
    #[serde(default)]
    pub outputs: Vec<ParamDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub filesystem: Vec<String>,
    #[serde(default = "bool_false")]
    pub network: bool,
    #[serde(default = "bool_false")]
    pub screen: bool,
}

fn bool_false() -> bool {
    false
}
