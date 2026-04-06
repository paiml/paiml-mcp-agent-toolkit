//! CB-303: Equation-Driven Development (EDD) Compliance
//!
//! Validates that simulation projects using `simular` or `trueno-sim`
//! reference LaTeX equations in public API doc comments.
//!
//! Per COMPLY-043: Every core simulation function must reference a
//! mathematical model (LaTeX or markdown math) in its documentation.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// EDD compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EddReport {
    /// Whether this is a simulation project
    pub is_simulation_project: bool,
    /// Total public functions in simulation modules
    pub total_simulation_fns: usize,
    /// Functions with math documentation
    pub documented_fns: usize,
    /// Functions missing math documentation
    pub undocumented_fns: Vec<EddViolation>,
    /// Compliance percentage (0-100)
    pub compliance_pct: f64,
}

/// A single EDD violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EddViolation {
    pub file: String,
    pub function_name: String,
    pub line: usize,
}

/// Check if a project is a simulation project (has simular or trueno-sim deps).
pub fn is_simulation_project(project_path: &Path) -> bool {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return false;
    }

    match std::fs::read_to_string(&cargo_toml) {
        Ok(content) => {
            content.contains("simular")
                || content.contains("trueno-sim")
                || content.contains("trueno_sim")
        }
        Err(_) => false,
    }
}

/// Check EDD compliance for a simulation project (CB-303).
///
/// Scans `pub fn` declarations in .rs files and checks if their doc comments
/// contain LaTeX math notation (`$$...$$`, `$...$`, or ` ```math` blocks).
pub fn check_edd_compliance(project_path: &Path) -> EddReport {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if !is_simulation_project(project_path) {
        return EddReport {
            is_simulation_project: false,
            total_simulation_fns: 0,
            documented_fns: 0,
            undocumented_fns: vec![],
            compliance_pct: 100.0,
        };
    }

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return EddReport {
            is_simulation_project: true,
            total_simulation_fns: 0,
            documented_fns: 0,
            undocumented_fns: vec![],
            compliance_pct: 100.0,
        };
    }

    let mut total_fns = 0;
    let mut documented = 0;
    let mut violations = Vec::new();

    scan_dir_for_edd(
        &src_dir,
        project_path,
        &mut total_fns,
        &mut documented,
        &mut violations,
    );

    let compliance_pct = if total_fns == 0 {
        100.0
    } else {
        (documented as f64 / total_fns as f64) * 100.0
    };

    EddReport {
        is_simulation_project: true,
        total_simulation_fns: total_fns,
        documented_fns: documented,
        undocumented_fns: violations,
        compliance_pct,
    }
}

/// Recursively scan a directory for Rust files and check EDD compliance.
fn scan_dir_for_edd(
    dir: &Path,
    project_root: &Path,
    total_fns: &mut usize,
    documented: &mut usize,
    violations: &mut Vec<EddViolation>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_edd(&path, project_root, total_fns, documented, violations);
        } else if path.extension().is_some_and(|e| e == "rs") {
            check_file_edd(&path, project_root, total_fns, documented, violations);
        }
    }
}

/// Check a single Rust file for EDD compliance.
///
/// Looks for `pub fn` declarations and checks if the preceding doc comments
/// contain mathematical notation.
fn check_file_edd(
    file_path: &Path,
    project_root: &Path,
    total_fns: &mut usize,
    documented: &mut usize,
    violations: &mut Vec<EddViolation>,
) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    let relative_path = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for public function declarations
        if !trimmed.starts_with("pub fn ") && !trimmed.starts_with("pub async fn ") {
            continue;
        }

        // Extract function name
        let fn_name = extract_fn_name(trimmed);

        *total_fns += 1;

        // Collect doc comments above this function
        let doc_block = collect_doc_comments(&lines, i);

        if has_math_notation(&doc_block) {
            *documented += 1;
        } else {
            violations.push(EddViolation {
                file: relative_path.clone(),
                function_name: fn_name,
                line: i + 1,
            });
        }
    }
}

/// Extract function name from a `pub fn` line.
fn extract_fn_name(line: &str) -> String {
    let after_fn = if line.contains("async fn ") {
        line.split("async fn ").nth(1)
    } else {
        line.split("fn ").nth(1)
    };

    match after_fn {
        Some(rest) => rest
            .split(|c: char| c == '(' || c == '<' || c.is_whitespace())
            .next()
            .unwrap_or("unknown")
            .to_string(),
        None => "unknown".to_string(),
    }
}

/// Collect doc comments (`///` or `//!`) above a given line index.
fn collect_doc_comments(lines: &[&str], fn_line: usize) -> String {
    let mut doc_lines = Vec::new();
    let mut i = fn_line.saturating_sub(1);

    loop {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            doc_lines.push(trimmed);
        } else if trimmed.starts_with("#[") || trimmed.is_empty() {
            // Skip attributes and blank lines between docs and fn
        } else {
            break;
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }

    doc_lines.reverse();
    doc_lines.join("\n")
}

/// Check if a doc comment block contains mathematical notation.
///
/// Looks for:
/// - `$$...$$` (display math)
/// - `$...$` (inline math)
/// - ` ```math` code blocks
/// - `\frac`, `\sum`, `\int`, `\partial` etc. (LaTeX commands)
fn has_math_notation(doc: &str) -> bool {
    // Display math: $$ ... $$
    if doc.contains("$$") {
        return true;
    }

    // Inline math: $...$ (at least one non-space char between)
    let bytes = doc.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && (i + 2 < bytes.len()) {
            // Find closing $
            if let Some(end) = doc.get(i + 1..).unwrap_or_default().find('$') {
                let inner = doc.get(i + 1..i + 1 + end).unwrap_or_default();
                if !inner.trim().is_empty() {
                    return true;
                }
            }
        }
        i += 1;
    }

    // Math code blocks
    if doc.contains("```math") {
        return true;
    }

    // Common LaTeX commands
    let latex_commands = [
        "\\frac",
        "\\sum",
        "\\int",
        "\\partial",
        "\\nabla",
        "\\sqrt",
        "\\alpha",
        "\\beta",
        "\\gamma",
        "\\delta",
        "\\epsilon",
        "\\theta",
        "\\lambda",
        "\\sigma",
        "\\omega",
        "\\mathbf",
        "\\mathrm",
        "\\vec",
        "\\hat",
        "\\dot",
        "\\cdot",
        "\\times",
        "\\approx",
        "\\equiv",
        "\\leq",
        "\\geq",
    ];

    for cmd in &latex_commands {
        if doc.contains(cmd) {
            return true;
        }
    }

    false
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_not_simulation_project() {
        // This project (pmat) is not a simulation project
        let report = check_edd_compliance(&PathBuf::from("."));
        assert!(!report.is_simulation_project);
        assert_eq!(report.compliance_pct, 100.0);
    }

    #[test]
    fn test_nonexistent_path() {
        let report = check_edd_compliance(&PathBuf::from("/nonexistent/path"));
        assert!(!report.is_simulation_project);
    }

    #[test]
    fn test_has_math_notation_display() {
        assert!(has_math_notation("/// Formula: $$ E = mc^2 $$"));
    }

    #[test]
    fn test_has_math_notation_inline() {
        assert!(has_math_notation("/// Uses $\\alpha$ decay"));
    }

    #[test]
    fn test_has_math_notation_code_block() {
        assert!(has_math_notation("/// ```math\n/// x = y + z\n/// ```"));
    }

    #[test]
    fn test_has_math_notation_latex_command() {
        assert!(has_math_notation("/// Computes \\frac{a}{b}"));
    }

    #[test]
    fn test_no_math_notation() {
        assert!(!has_math_notation("/// This function does stuff"));
    }

    #[test]
    fn test_extract_fn_name_simple() {
        assert_eq!(
            extract_fn_name("pub fn calculate(x: f64) -> f64"),
            "calculate"
        );
    }

    #[test]
    fn test_extract_fn_name_generic() {
        assert_eq!(extract_fn_name("pub fn process<T>(val: T)"), "process");
    }

    #[test]
    fn test_extract_fn_name_async() {
        assert_eq!(extract_fn_name("pub async fn fetch_data()"), "fetch_data");
    }

    #[test]
    fn test_collect_doc_comments() {
        let lines = vec![
            "/// First line",
            "/// Second line",
            "#[inline]",
            "pub fn foo() {}",
        ];
        let doc = collect_doc_comments(&lines, 3);
        assert!(doc.contains("First line"));
        assert!(doc.contains("Second line"));
    }

    #[test]
    fn test_is_simulation_project_false() {
        // pmat itself is not a simulation project
        assert!(!is_simulation_project(&PathBuf::from(".")));
    }
}
