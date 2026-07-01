//! YAML-driven scenario engine for Rust RPA: branching, loops, retry, and sub-scenarios.
#![warn(missing_docs)]

/// Loading data sources (CSV/XLSX) for data-driven scenario runs.
pub mod data_source;
/// The scenario execution engine ([`engine::ScenarioEngine`]).
pub mod engine;
/// Progress events emitted while a scenario runs.
pub mod progress;
/// Execution reports (CSV/HTML) summarizing a completed scenario run.
pub mod report;
// ponytail: scenario.rs is a ~150-struct step catalog (605 pub fields, 176 enum
// variants) — documenting all of it is a separate, much larger effort than this
// trial. Exempt it here; lift the allow once that catalog gets doc'd.
#[allow(missing_docs)]
pub mod scenario;
/// The variable store shared across scenario steps.
pub mod variables;

pub use engine::{EngineError, Flow, ScenarioEngine, ScreenshotsMode};
pub use progress::ProgressEvent;
pub use report::{ExecutionReport, Outcome, StepOutcome, StepRecord};
pub use scenario::{DataSource, Scenario, ScenarioStep};
pub use variables::Variables;
