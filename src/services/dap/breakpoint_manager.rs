#![cfg_attr(coverage_nightly, coverage(off))]
// Breakpoint Management System
// Sprint 71 - TRACE-002: Breakpoint Manager Implementation
//
// Advanced breakpoint management with conditional breakpoints,
// hit count tracking, and expression evaluation

use super::types::Breakpoint;
use serde_json::Value;
use std::collections::HashMap;

/// Breakpoint metadata including hit count
#[derive(Debug, Clone)]
struct BreakpointMetadata {
    breakpoint: Breakpoint,
    hit_count: u64,
}

/// Breakpoint Manager for managing debug breakpoints
#[derive(Debug)]
pub struct BreakpointManager {
    /// Map of source file -> line number -> breakpoint metadata
    breakpoints: HashMap<String, HashMap<i64, BreakpointMetadata>>,
}

impl BreakpointManager {
    /// Create a new breakpoint manager
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
        }
    }

    /// Set a breakpoint (or update existing)
    pub fn set_breakpoint(&mut self, breakpoint: Breakpoint) -> Result<(), String> {
        let source = breakpoint.source.clone();
        let line = breakpoint.line;

        let file_breakpoints = self.breakpoints.entry(source).or_default();

        let metadata = BreakpointMetadata {
            breakpoint,
            hit_count: 0,
        };

        file_breakpoints.insert(line, metadata);
        Ok(())
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, source: &str, line: i64) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if let Some(file_breakpoints) = self.breakpoints.get_mut(source) {
            file_breakpoints.remove(&line);
            if file_breakpoints.is_empty() {
                self.breakpoints.remove(source);
            }
            Ok(())
        } else {
            Err(format!("No breakpoints in file: {}", source))
        }
    }

    /// Check if a breakpoint exists at the given location
    pub fn has_breakpoint(&self, source: &str, line: i64) -> bool {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.breakpoints
            .get(source)
            .map(|file_bp| file_bp.contains_key(&line))
            .unwrap_or(false)
    }

    /// Get a breakpoint at the given location
    pub fn get_breakpoint(&self, source: &str, line: i64) -> Option<Breakpoint> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.breakpoints
            .get(source)
            .and_then(|file_bp| file_bp.get(&line))
            .map(|metadata| metadata.breakpoint.clone())
    }

    /// Get total count of breakpoints
    pub fn count(&self) -> usize {
        self.breakpoints.values().map(|file_bp| file_bp.len()).sum()
    }

    /// Get all breakpoints in a specific file
    pub fn breakpoints_in_file(&self, source: &str) -> Vec<Breakpoint> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.breakpoints
            .get(source)
            .map(|file_bp| {
                file_bp
                    .values()
                    .map(|metadata| metadata.breakpoint.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.breakpoints.clear();
    }

    /// Clear all breakpoints in a specific file
    pub fn clear_file(&mut self, source: &str) {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.breakpoints.remove(source);
    }

    /// Get all breakpoints across all files
    pub fn all_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoints
            .values()
            .flat_map(|file_bp| file_bp.values().map(|metadata| metadata.breakpoint.clone()))
            .collect()
    }

    /// Record a hit on a breakpoint
    pub fn record_hit(&mut self, source: &str, line: i64) {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if let Some(file_breakpoints) = self.breakpoints.get_mut(source) {
            if let Some(metadata) = file_breakpoints.get_mut(&line) {
                metadata.hit_count += 1;
            }
        }
    }

    /// Get hit count for a breakpoint
    pub fn get_hit_count(&self, source: &str, line: i64) -> u64 {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.breakpoints
            .get(source)
            .and_then(|file_bp| file_bp.get(&line))
            .map(|metadata| metadata.hit_count)
            .unwrap_or(0)
    }

    /// Check if execution should break at this location
    ///
    /// Evaluates conditional breakpoints if variables are provided
    pub fn should_break(&self, source: &str, line: i64, variables: Option<&Value>) -> bool {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if let Some(breakpoint) = self.get_breakpoint(source, line) {
            if let Some(condition) = &breakpoint.condition {
                // Evaluate condition
                if let Some(vars) = variables {
                    return self.evaluate_condition(condition, vars);
                }
                // No variables provided, treat as false
                return false;
            }
            // Unconditional breakpoint
            return true;
        }
        false
    }

    /// Evaluate a conditional breakpoint expression
    ///
    /// Supports simple expressions like:
    /// - "x > 5"
    /// - "count == 3"
    /// - "name == 'test'"
    fn evaluate_condition(&self, condition: &str, variables: &Value) -> bool {
        // Simple expression parser for common cases
        let condition = condition.trim();

        // Handle equality: variable == value
        if let Some((var, value)) = parse_equality(condition) {
            if let Some(var_value) = variables.get(var.trim()) {
                return check_equality(var_value, value.trim());
            }
            return false;
        }

        // Handle greater than: variable > value
        if let Some((var, value)) = parse_comparison(condition, ">") {
            if let Some(var_value) = variables.get(var.trim()) {
                return check_greater_than(var_value, value.trim());
            }
            return false;
        }

        // Handle less than: variable < value
        if let Some((var, value)) = parse_comparison(condition, "<") {
            if let Some(var_value) = variables.get(var.trim()) {
                return check_less_than(var_value, value.trim());
            }
            return false;
        }

        // Handle greater than or equal: variable >= value
        if let Some((var, value)) = parse_comparison(condition, ">=") {
            if let Some(var_value) = variables.get(var.trim()) {
                return check_greater_than_or_equal(var_value, value.trim());
            }
            return false;
        }

        // Handle less than or equal: variable <= value
        if let Some((var, value)) = parse_comparison(condition, "<=") {
            if let Some(var_value) = variables.get(var.trim()) {
                return check_less_than_or_equal(var_value, value.trim());
            }
            return false;
        }

        // Handle inequality: variable != value
        if let Some((var, value)) = parse_inequality(condition) {
            if let Some(var_value) = variables.get(var.trim()) {
                return !check_equality(var_value, value.trim());
            }
            return false;
        }

        // Unknown condition format, treat as false
        false
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for expression parsing

fn parse_equality(condition: &str) -> Option<(&str, &str)> {
    if let Some(pos) = condition.find("==") {
        let var = &condition[..pos];
        let value = &condition[pos + 2..];
        return Some((var, value));
    }
    None
}

fn parse_inequality(condition: &str) -> Option<(&str, &str)> {
    if let Some(pos) = condition.find("!=") {
        let var = &condition[..pos];
        let value = &condition[pos + 2..];
        return Some((var, value));
    }
    None
}

fn parse_comparison<'a>(condition: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    if let Some(pos) = condition.find(op) {
        let var = &condition[..pos];
        let value = &condition[pos + op.len()..];
        return Some((var, value));
    }
    None
}

fn check_equality(var_value: &Value, expected: &str) -> bool {
    match var_value {
        Value::Number(n) => {
            if let Ok(expected_num) = expected.parse::<i64>() {
                return n.as_i64() == Some(expected_num);
            }
            if let Ok(expected_num) = expected.parse::<f64>() {
                return n.as_f64() == Some(expected_num);
            }
        }
        Value::String(s) => {
            let expected_clean = expected.trim_matches(|c| c == '\'' || c == '"');
            return s == expected_clean;
        }
        Value::Bool(b) => {
            if let Ok(expected_bool) = expected.parse::<bool>() {
                return *b == expected_bool;
            }
        }
        _ => {}
    }
    false
}

fn check_greater_than(var_value: &Value, expected: &str) -> bool {
    if let Value::Number(n) = var_value {
        if let Ok(expected_num) = expected.parse::<i64>() {
            if let Some(actual) = n.as_i64() {
                return actual > expected_num;
            }
        }
        if let Ok(expected_num) = expected.parse::<f64>() {
            if let Some(actual) = n.as_f64() {
                return actual > expected_num;
            }
        }
    }
    false
}

fn check_less_than(var_value: &Value, expected: &str) -> bool {
    if let Value::Number(n) = var_value {
        if let Ok(expected_num) = expected.parse::<i64>() {
            if let Some(actual) = n.as_i64() {
                return actual < expected_num;
            }
        }
        if let Ok(expected_num) = expected.parse::<f64>() {
            if let Some(actual) = n.as_f64() {
                return actual < expected_num;
            }
        }
    }
    false
}

fn check_greater_than_or_equal(var_value: &Value, expected: &str) -> bool {
    if let Value::Number(n) = var_value {
        if let Ok(expected_num) = expected.parse::<i64>() {
            if let Some(actual) = n.as_i64() {
                return actual >= expected_num;
            }
        }
        if let Ok(expected_num) = expected.parse::<f64>() {
            if let Some(actual) = n.as_f64() {
                return actual >= expected_num;
            }
        }
    }
    false
}

fn check_less_than_or_equal(var_value: &Value, expected: &str) -> bool {
    if let Value::Number(n) = var_value {
        if let Ok(expected_num) = expected.parse::<i64>() {
            if let Some(actual) = n.as_i64() {
                return actual <= expected_num;
            }
        }
        if let Ok(expected_num) = expected.parse::<f64>() {
            if let Some(actual) = n.as_f64() {
                return actual <= expected_num;
            }
        }
    }
    false
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_manager_creation() {
        let mgr = BreakpointManager::new();
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_parse_equality() {
        let (var, value) = parse_equality("x == 5").unwrap();
        assert_eq!(var, "x ");
        assert_eq!(value, " 5");
    }

    #[test]
    fn test_parse_comparison() {
        let (var, value) = parse_comparison("x > 5", ">").unwrap();
        assert_eq!(var, "x ");
        assert_eq!(value, " 5");
    }

    #[test]
    fn test_check_equality_number() {
        let var_value = json!(5);
        assert!(check_equality(&var_value, "5"));
        assert!(!check_equality(&var_value, "3"));
    }

    #[test]
    fn test_check_greater_than() {
        let var_value = json!(10);
        assert!(check_greater_than(&var_value, "5"));
        assert!(!check_greater_than(&var_value, "15"));
    }
}
