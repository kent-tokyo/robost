use std::collections::HashMap;

use rhai::{Dynamic, Engine, EvalAltResult, Scope};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script error: {0}")]
    Eval(Box<EvalAltResult>),
    #[error("type error: {0}")]
    Type(String),
}

pub type Result<T> = std::result::Result<T, ScriptError>;

/// Wraps a Rhai engine. Reuse one instance per scenario to share the compiled
/// function cache across steps.
pub struct ScriptEngine {
    engine: Engine,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(64, 64);
        Self { engine }
    }

    /// Evaluate `expr` as a Rhai boolean expression using current scenario variables.
    pub fn eval_bool(&self, expr: &str, vars: &HashMap<String, serde_json::Value>) -> Result<bool> {
        let (result, _) = self.run(expr, vars)?;
        result
            .as_bool()
            .map_err(|_| ScriptError::Type(format!("condition '{expr}' did not return bool")))
    }

    /// Run `source` (Rhai script) inside a sandboxed scope seeded from scenario variables.
    /// Rhai provides no file system or OS access — the engine itself is the sandbox boundary.
    #[instrument(name = "script_run", skip(self, source, vars))]
    pub fn run(
        &self,
        source: &str,
        vars: &HashMap<String, serde_json::Value>,
    ) -> Result<(Dynamic, HashMap<String, Dynamic>)> {
        let mut scope = Scope::new();
        for (k, v) in vars {
            scope.push_dynamic(k.clone(), json_to_dynamic(v));
        }

        let result = self
            .engine
            .eval_with_scope::<Dynamic>(&mut scope, source)
            .map_err(ScriptError::Eval)?;

        let exports: HashMap<String, Dynamic> = scope
            .iter_raw()
            .map(|(name, _, val)| (name.to_owned(), val.clone()))
            .collect();

        Ok((result, exports))
    }
}

fn json_to_dynamic(v: &serde_json::Value) -> Dynamic {
    match v {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else {
                Dynamic::from(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            Dynamic::from(arr.iter().map(json_to_dynamic).collect::<Vec<_>>())
        }
        serde_json::Value::Object(map) => {
            let m: rhai::Map = map
                .iter()
                .map(|(k, v)| (k.clone().into(), json_to_dynamic(v)))
                .collect();
            Dynamic::from(m)
        }
    }
}
