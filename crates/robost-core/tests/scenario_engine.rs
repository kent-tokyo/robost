//! Integration tests for `ScenarioEngine::run` exercising the real backend/engine wiring
//! (as opposed to the unit tests in `engine.rs`, which call individual step handlers directly).
//! Runs against `LocalBackend` with `.with_dry_run(true)` so no real OS input is sent.

use robost_backend::LocalBackend;
use robost_core::{Scenario, ScenarioEngine, Variables};
use std::sync::Arc;

fn dry_run_engine(base_dir: &std::path::Path) -> ScenarioEngine {
    ScenarioEngine::new(
        Arc::new(LocalBackend::new().unwrap()),
        base_dir.to_path_buf(),
    )
    .with_silent(true)
    .with_dry_run(true)
}

#[tokio::test]
async fn runs_a_simple_scenario_end_to_end() {
    let yaml = r#"
name: integration_smoke
steps:
  - wait_ms: 1
  - mouse_move:
      x: "10"
      y: "10"
  - mouse_click_xy:
      x: "20"
      y: "20"
"#;
    let scenario = Scenario::from_yaml(yaml).expect("scenario should parse");
    let dir = tempfile::tempdir().unwrap();
    let engine = dry_run_engine(dir.path());
    let mut vars = Variables::new();

    engine
        .run(&scenario, &mut vars)
        .await
        .expect("dry-run scenario should complete without touching the real OS");
}

#[tokio::test]
async fn scenario_variables_are_available_to_later_steps() {
    let yaml = r#"
name: variables_smoke
variables:
  greeting: "hello"
steps:
  - wait_ms: 1
"#;
    let scenario = Scenario::from_yaml(yaml).expect("scenario should parse");
    let dir = tempfile::tempdir().unwrap();
    let engine = dry_run_engine(dir.path());
    let mut vars = Variables::new();

    engine
        .run(&scenario, &mut vars)
        .await
        .expect("scenario should run");
    assert_eq!(vars.get("greeting").unwrap(), &serde_json::json!("hello"));
}
