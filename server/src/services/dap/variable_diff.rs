// TRACE-009: Variable Diff Highlighting
// Sprint 73 - GREEN Phase
//
// Visual comparison of variables between execution snapshots.
// Provides colored diff output and side-by-side comparisons.

use super::types::ExecutionSnapshot;
use std::collections::HashMap;

/// Represents a change in a variable's value
#[derive(Debug, Clone)]
pub struct VariableChange {
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub type_changed: bool,
}

/// Statistics about variable differences
#[derive(Debug, Clone)]
pub struct DiffStatistics {
    pub changed_count: usize,
    pub added_count: usize,
    pub removed_count: usize,
    pub unchanged_count: usize,
    pub total_variables_before: usize,
    pub total_variables_after: usize,
}

/// Diff between two execution snapshots
#[derive(Debug, Clone)]
pub struct VariableDiff {
    /// Variables that changed values
    pub changed: HashMap<String, VariableChange>,
    /// Variables that were added
    pub added: HashMap<String, serde_json::Value>,
    /// Variables that were removed
    pub removed: HashMap<String, serde_json::Value>,
    /// Variables that stayed the same
    pub unchanged: Vec<String>,
}

impl VariableDiff {
    /// Compute diff between two snapshots
    pub fn compute(before: &ExecutionSnapshot, after: &ExecutionSnapshot) -> Self {
        let mut changed = HashMap::new();
        let mut added = HashMap::new();
        let mut removed = HashMap::new();
        let mut unchanged = Vec::new();

        // Find changed and unchanged variables
        for (name, old_value) in &before.variables {
            if let Some(new_value) = after.variables.get(name) {
                if old_value != new_value {
                    // Variable changed
                    let type_changed =
                        Self::value_type_name(old_value) != Self::value_type_name(new_value);

                    changed.insert(
                        name.clone(),
                        VariableChange {
                            old_value: old_value.clone(),
                            new_value: new_value.clone(),
                            type_changed,
                        },
                    );
                } else {
                    // Variable unchanged
                    unchanged.push(name.clone());
                }
            } else {
                // Variable removed
                removed.insert(name.clone(), old_value.clone());
            }
        }

        // Find added variables
        for (name, new_value) in &after.variables {
            if !before.variables.contains_key(name) {
                added.insert(name.clone(), new_value.clone());
            }
        }

        Self {
            changed,
            added,
            removed,
            unchanged,
        }
    }

    /// Render diff with ANSI colors
    pub fn render_colored(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Variable Diff ===\n\n");

        // Changed variables (yellow)
        if !self.changed.is_empty() {
            output.push_str("\x1b[33mChanged:\x1b[0m\n");
            for (name, change) in &self.changed {
                if change.type_changed {
                    output.push_str(&format!(
                        "  \x1b[33m{}\x1b[0m: {} -> {} \x1b[35m(type changed)\x1b[0m\n",
                        name, change.old_value, change.new_value
                    ));
                } else {
                    output.push_str(&format!(
                        "  \x1b[33m{}\x1b[0m: {} -> {}\n",
                        name, change.old_value, change.new_value
                    ));
                }
            }
            output.push('\n');
        }

        // Added variables (green)
        if !self.added.is_empty() {
            output.push_str("\x1b[32mAdded:\x1b[0m\n");
            for (name, value) in &self.added {
                output.push_str(&format!("  \x1b[32m+{}\x1b[0m: {}\n", name, value));
            }
            output.push('\n');
        }

        // Removed variables (red)
        if !self.removed.is_empty() {
            output.push_str("\x1b[31mRemoved:\x1b[0m\n");
            for (name, value) in &self.removed {
                output.push_str(&format!("  \x1b[31m-{}\x1b[0m: {}\n", name, value));
            }
            output.push('\n');
        }

        // Unchanged variables (gray)
        if !self.unchanged.is_empty() {
            output.push_str("\x1b[90mUnchanged:\x1b[0m ");
            output.push_str(&self.unchanged.join(", "));
            output.push('\n');
        }

        output
    }

    /// Render side-by-side comparison
    pub fn render_side_by_side(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Side-by-Side Comparison ===\n\n");
        output.push_str(&format!(
            "{:<30} | {:<30}\n",
            "Snapshot #0 (Before)", "Snapshot #1 (After)"
        ));
        output.push_str(&"-".repeat(63));
        output.push('\n');

        // Show changed variables
        for (name, change) in &self.changed {
            let old_display = format!("{}: {}", name, change.old_value);
            let new_display = format!("{}: {}", name, change.new_value);
            output.push_str(&format!("{:<30} | {:<30}\n", old_display, new_display));
        }

        // Show removed variables (only in before)
        for (name, value) in &self.removed {
            let old_display = format!("{}: {}", name, value);
            output.push_str(&format!("{:<30} | {:<30}\n", old_display, "(removed)"));
        }

        // Show added variables (only in after)
        for (name, value) in &self.added {
            let new_display = format!("{}: {}", name, value);
            output.push_str(&format!("{:<30} | {:<30}\n", "(new)", new_display));
        }

        output
    }

    /// Get statistics about the diff
    pub fn get_statistics(&self) -> DiffStatistics {
        let total_before = self.changed.len() + self.removed.len() + self.unchanged.len();
        let total_after = self.changed.len() + self.added.len() + self.unchanged.len();

        DiffStatistics {
            changed_count: self.changed.len(),
            added_count: self.added.len(),
            removed_count: self.removed.len(),
            unchanged_count: self.unchanged.len(),
            total_variables_before: total_before,
            total_variables_after: total_after,
        }
    }

    /// Export diff to JSON
    pub fn to_json(&self) -> String {
        let mut json = serde_json::Map::new();

        // Changed variables
        let changed_obj: serde_json::Map<String, serde_json::Value> = self
            .changed
            .iter()
            .map(|(name, change)| {
                let mut change_obj = serde_json::Map::new();
                change_obj.insert("old".to_string(), change.old_value.clone());
                change_obj.insert("new".to_string(), change.new_value.clone());
                change_obj.insert(
                    "type_changed".to_string(),
                    serde_json::Value::Bool(change.type_changed),
                );
                (name.clone(), serde_json::Value::Object(change_obj))
            })
            .collect();
        json.insert(
            "changed".to_string(),
            serde_json::Value::Object(changed_obj),
        );

        // Added variables
        let added_obj: serde_json::Map<String, serde_json::Value> = self
            .added
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        json.insert("added".to_string(), serde_json::Value::Object(added_obj));

        // Removed variables
        let removed_obj: serde_json::Map<String, serde_json::Value> = self
            .removed
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        json.insert(
            "removed".to_string(),
            serde_json::Value::Object(removed_obj),
        );

        // Unchanged variables
        let unchanged_arr: Vec<serde_json::Value> = self
            .unchanged
            .iter()
            .map(|name| serde_json::Value::String(name.clone()))
            .collect();
        json.insert(
            "unchanged".to_string(),
            serde_json::Value::Array(unchanged_arr),
        );

        // Statistics
        let stats = self.get_statistics();
        let mut stats_obj = serde_json::Map::new();
        stats_obj.insert(
            "changed".to_string(),
            serde_json::Value::Number(stats.changed_count.into()),
        );
        stats_obj.insert(
            "added".to_string(),
            serde_json::Value::Number(stats.added_count.into()),
        );
        stats_obj.insert(
            "removed".to_string(),
            serde_json::Value::Number(stats.removed_count.into()),
        );
        stats_obj.insert(
            "unchanged".to_string(),
            serde_json::Value::Number(stats.unchanged_count.into()),
        );
        stats_obj.insert(
            "total_before".to_string(),
            serde_json::Value::Number(stats.total_variables_before.into()),
        );
        stats_obj.insert(
            "total_after".to_string(),
            serde_json::Value::Number(stats.total_variables_after.into()),
        );
        json.insert(
            "statistics".to_string(),
            serde_json::Value::Object(stats_obj),
        );

        serde_json::to_string_pretty(&json).expect("internal error")
    }

    /// Get type name of a JSON value
    fn value_type_name(value: &serde_json::Value) -> &'static str {
        match value {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dap::types::{SourceLocation, StackFrame};

    fn create_test_snapshot(sequence: usize) -> ExecutionSnapshot {
        ExecutionSnapshot {
            timestamp: 1000000 + (sequence as u64 * 1000),
            sequence,
            variables: std::collections::HashMap::new(),
            call_stack: vec![StackFrame {
                id: 1,
                name: "main".to_string(),
                source: None,
                line: 10,
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: Some(0),
            },
            delta: None,
        }
    }

    #[test]
    fn test_basic_diff_computation() {
        let mut before = create_test_snapshot(0);
        before
            .variables
            .insert("x".to_string(), serde_json::json!(10));

        let mut after = create_test_snapshot(1);
        after
            .variables
            .insert("x".to_string(), serde_json::json!(15));

        let diff = VariableDiff::compute(&before, &after);

        assert_eq!(diff.changed.len(), 1);
        assert!(diff.changed.contains_key("x"));
    }

    #[test]
    fn test_type_detection() {
        assert_eq!(
            VariableDiff::value_type_name(&serde_json::json!(42)),
            "number"
        );
        assert_eq!(
            VariableDiff::value_type_name(&serde_json::json!("hello")),
            "string"
        );
        assert_eq!(
            VariableDiff::value_type_name(&serde_json::json!(true)),
            "boolean"
        );
        assert_eq!(
            VariableDiff::value_type_name(&serde_json::json!([1, 2, 3])),
            "array"
        );
        assert_eq!(
            VariableDiff::value_type_name(&serde_json::json!({"key": "value"})),
            "object"
        );
    }
}
