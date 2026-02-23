#![cfg_attr(coverage_nightly, coverage(off))]

// JSON-based DSL example
pub const WORKFLOW_DSL_EXAMPLE: &str = r#"{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "quality_check_workflow",
  "description": "Comprehensive quality check workflow",
  "version": "1.0.0",
  "steps": [
    {
      "id": "analyze",
      "name": "Code Analysis",
      "step_type": {
        "type": "action",
        "agent": "analyzer",
        "operation": "analyze",
        "params": {
          "language": "rust",
          "metrics": ["complexity", "satd", "entropy"]
        }
      },
      "condition": null,
      "retry": {
        "max_attempts": 3,
        "backoff": {
          "Exponential": {
            "initial": {"secs": 1, "nanos": 0},
            "multiplier": 2.0,
            "max": {"secs": 10, "nanos": 0}
          }
        },
        "retry_on": []
      },
      "timeout": {"secs": 60, "nanos": 0},
      "on_error": null,
      "metadata": {}
    }
  ],
  "error_strategy": "FailFast",
  "timeout": {"secs": 300, "nanos": 0},
  "metadata": {}
}"#;
