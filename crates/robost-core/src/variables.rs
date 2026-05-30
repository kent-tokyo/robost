use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Scenario-level variable store. Typed values passed between steps, scripts, and plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Variables(HashMap<String, Value>);

impl Variables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn as_map(&self) -> &HashMap<String, Value> {
        &self.0
    }

    /// Compact key=value string for debug display.
    pub fn debug_dump(&self) -> String {
        let mut parts: Vec<String> = self
            .0
            .iter()
            .map(|(k, v)| {
                let vs = match v {
                    Value::String(s) if s.len() > 40 => format!("{:?}…", &s[..40]),
                    other => format!("{other}"),
                };
                format!("{k}={vs}")
            })
            .collect();
        parts.sort();
        format!("{{{}}}", parts.join(", "))
    }

    /// Expand `{{ var_name }}` placeholders in a string.
    ///
    /// Two passes resolve one level of indirection (e.g. a variable whose value
    /// contains another `{{ placeholder }}`). The cap prevents infinite loops on
    /// circular references and keeps the expansion deterministic regardless of
    /// the internal HashMap iteration order.
    pub fn expand(&self, template: &str) -> String {
        let mut result = template.to_owned();
        for _ in 0..2 {
            let prev = result.clone();
            for (k, v) in &self.0 {
                let placeholder = format!("{{{{ {k} }}}}");
                let val = match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                result = result.replace(&placeholder, &val);
            }
            if result == prev {
                break;
            }
        }
        result
    }
}
