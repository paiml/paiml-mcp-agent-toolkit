#![cfg_attr(coverage_nightly, coverage(off))]
use super::{AgentError, AgentResponse, Priority};
use actix::prelude::*;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<AgentResponse, AgentError>")]
pub struct AnalyzeMessage {
    pub code: String,
    pub priority: Priority,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<AgentResponse, AgentError>")]
pub struct TransformMessage {
    pub code: String,
    pub rules: Vec<String>,
    pub priority: Priority,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<AgentResponse, AgentError>")]
pub struct ValidateMessage {
    pub metrics: crate::modules::analyzer::Metrics,
    pub thresholds: crate::modules::validator::Thresholds,
    pub priority: Priority,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_message_creation() {
        let msg = AnalyzeMessage {
            code: "fn main() {}".to_string(),
            priority: Priority::Normal,
        };

        assert_eq!(msg.code, "fn main() {}");
    }

    #[test]
    fn test_transform_message_creation() {
        let msg = TransformMessage {
            code: "fn main() {}".to_string(),
            rules: vec!["rule1".to_string()],
            priority: Priority::Normal,
        };

        assert_eq!(msg.code, "fn main() {}");
        assert_eq!(msg.rules.len(), 1);
    }

    #[test]
    fn test_analyze_message_clone() {
        let msg = AnalyzeMessage {
            code: "test".to_string(),
            priority: Priority::Normal,
        };

        let cloned = msg.clone();
        assert_eq!(cloned.code, msg.code);
    }

    #[test]
    fn test_transform_message_clone() {
        let msg = TransformMessage {
            code: "test".to_string(),
            rules: vec!["r1".to_string(), "r2".to_string()],
            priority: Priority::Normal,
        };

        let cloned = msg.clone();
        assert_eq!(cloned.code, msg.code);
        assert_eq!(cloned.rules, msg.rules);
    }

    #[test]
    fn test_analyze_message_debug() {
        let msg = AnalyzeMessage {
            code: "test".to_string(),
            priority: Priority::Normal,
        };

        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("AnalyzeMessage"));
    }

    #[test]
    fn test_transform_message_debug() {
        let msg = TransformMessage {
            code: "test".to_string(),
            rules: vec![],
            priority: Priority::Normal,
        };

        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("TransformMessage"));
    }
}
