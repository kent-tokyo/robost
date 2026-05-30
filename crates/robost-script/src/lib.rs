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
        engine.set_max_operations(1_000_000);
        register_date_fns(&mut engine);
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

// ── Date helper functions registered into the Rhai engine ──────────────────

fn register_date_fns(engine: &mut Engine) {
    use chrono::{Datelike, Local, NaiveDate};

    engine.register_fn("now_year", || Local::now().year() as i64);
    engine.register_fn("now_month", || Local::now().month() as i64);
    engine.register_fn("now_day", || Local::now().day() as i64);
    engine.register_fn("today", || Local::now().format("%Y-%m-%d").to_string());

    /// Returns `Some((next_year, next_month))` for the first day of the following month,
    /// or `None` if any value is out of range.
    fn next_month(year: i64, month: i64) -> Option<(i32, u32)> {
        if !(1..=12).contains(&month) { return None; }
        if year < i32::MIN as i64 || year > i32::MAX as i64 { return None; }
        if month == 12 {
            let y2 = year.checked_add(1)?;
            if y2 > i32::MAX as i64 { return None; }
            Some((y2 as i32, 1))
        } else {
            Some((year as i32, (month + 1) as u32))
        }
    }

    fn valid_ym(year: i64, month: i64) -> Option<(i32, u32)> {
        if !(1..=12).contains(&month) { return None; }
        if year < i32::MIN as i64 || year > i32::MAX as i64 { return None; }
        Some((year as i32, month as u32))
    }

    engine.register_fn("end_of_month", |year: i64, month: i64| -> i64 {
        next_month(year, month)
            .and_then(|(y2, m2)| NaiveDate::from_ymd_opt(y2, m2, 1))
            .and_then(|d| d.pred_opt())
            .map(|d| d.day() as i64)
            .unwrap_or(0)
    });

    engine.register_fn(
        "end_of_month_str",
        |year: i64, month: i64, fmt: &str| -> String {
            next_month(year, month)
                .and_then(|(y2, m2)| NaiveDate::from_ymd_opt(y2, m2, 1))
                .and_then(|d| d.pred_opt())
                .map(|d| d.format(fmt).to_string())
                .unwrap_or_default()
        },
    );

    engine.register_fn("start_of_month", |year: i64, month: i64| -> i64 {
        valid_ym(year, month)
            .and_then(|(y, m)| NaiveDate::from_ymd_opt(y, m, 1))
            .map(|d| d.day() as i64)
            .unwrap_or(0)
    });

    engine.register_fn(
        "start_of_month_str",
        |year: i64, month: i64, fmt: &str| -> String {
            valid_ym(year, month)
                .and_then(|(y, m)| NaiveDate::from_ymd_opt(y, m, 1))
                .map(|d| d.format(fmt).to_string())
                .unwrap_or_default()
        },
    );
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
