use serde::{Deserialize, Serialize};

/// Progress event emitted during scenario execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// Scenario execution started.
    ScenarioStart {
        /// Total number of steps in the scenario.
        total: usize,
    },

    /// Step execution started.
    StepStart {
        /// 0-based step index.
        index: usize,
        /// Step display name.
        name: String,
        /// Total number of steps in the scenario.
        total: usize,
    },

    /// Step execution completed.
    StepDone {
        /// 0-based step index.
        index: usize,
        /// Elapsed time in milliseconds.
        elapsed_ms: u64,
    },

    /// Log message.
    Log {
        /// Log level: "info", "warn", "error", "debug".
        level: String,
        /// Log message.
        message: String,
    },

    /// Scenario execution finished.
    Finished {
        /// True if execution was successful.
        success: bool,
        /// Error message if execution failed.
        error: Option<String>,
    },
}

impl ProgressEvent {
    /// Serialize to SSE (Server-Sent Events) line format.
    pub fn to_sse_line(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => format!("data: {json}\n\n"),
            Err(_) => "data: {\"type\":\"error\"}\n\n".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_event_serialization() {
        let event = ProgressEvent::StepStart {
            index: 0,
            name: "click_button".to_string(),
            total: 5,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"step_start\""));
        assert!(json.contains("\"index\":0"));
        assert!(json.contains("\"name\":\"click_button\""));
    }

    #[test]
    fn test_sse_line_format() {
        let event = ProgressEvent::Finished {
            success: true,
            error: None,
        };

        let line = event.to_sse_line();
        assert!(line.starts_with("data: "));
        assert!(line.ends_with("\n\n"));
    }
}
