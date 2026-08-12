#![cfg_attr(coverage_nightly, coverage(off))]
//! WebAssembly security validation
//!
//! This module provides security validation for WebAssembly modules.

use super::types::Severity;
use anyhow::Result;

/// Security validation result
#[derive(Debug, Clone)]
pub struct SecurityValidation {
    /// Whether validation passed
    pub passed: bool,
    /// Security issues found
    pub issues: Vec<SecurityIssue>,
}

/// Security issue found during validation
#[derive(Debug, Clone)]
pub struct SecurityIssue {
    /// Issue severity
    pub severity: Severity,
    /// Issue description
    pub description: String,
    /// Category of security issue
    pub category: SecurityCategory,
}

/// Security issue categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityCategory {
    /// Invalid file format
    InvalidFormat,
    /// Memory safety issue
    MemorySafety,
    /// Resource exhaustion risk
    ResourceExhaustion,
    /// Potential code injection
    CodeInjection,
    /// Other security concerns
    Other,
}

/// WebAssembly security validator
pub struct WasmSecurityValidator;

impl WasmSecurityValidator {
    /// Create a new security validator
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }

    /// Validate WebAssembly binary
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate(&self, data: &[u8]) -> Result<SecurityValidation> {
        let mut issues = Vec::new();

        // Check magic number
        if data.len() < 8 {
            issues.push(SecurityIssue {
                severity: Severity::Critical,
                description: "File too small to be valid WASM".to_string(),
                category: SecurityCategory::InvalidFormat,
            });
        } else if &data[0..4] != b"\0asm" {
            issues.push(SecurityIssue {
                severity: Severity::Critical,
                description: "Invalid WASM magic number".to_string(),
                category: SecurityCategory::InvalidFormat,
            });
        }

        // Check file size for potential DoS
        if data.len() > 100 * 1024 * 1024 {
            issues.push(SecurityIssue {
                severity: Severity::High,
                description: "File size exceeds safe limit (100MB)".to_string(),
                category: SecurityCategory::ResourceExhaustion,
            });
        }

        Ok(SecurityValidation {
            passed: issues.is_empty(),
            issues,
        })
    }

    /// Validate WebAssembly *text* (`.wat`) or `AssemblyScript` source.
    ///
    /// # Why this replaced `validate_ast`
    ///
    /// `--security` used to reach exactly one function,
    /// `validate_ast(&self, _ast: &AstDag) -> Result<()> { Ok(()) }`: a
    /// constant that ignored its argument and could never return `Err`, so the
    /// `if let Err(e) = …` in both WASM handlers was dead for every possible
    /// input and `--security` was byte-identical to no flag. The argument was
    /// empty anyway — `WatParser::parse` and `AssemblyScriptParser::parse_file`
    /// both `Ok(AstDag::new())` and add no nodes — so no implementation over
    /// that DAG could ever have seen anything. The check now runs over the text
    /// the parsers were handed.
    ///
    /// # What it reports
    ///
    /// Every issue names a construct present in `content`, with the 1-based
    /// line it appears on. Nothing is inferred or estimated:
    ///
    /// * `(memory N)` with no maximum — linear memory that can grow without
    ///   bound (`ResourceExhaustion`).
    /// * `(memory N …)` with `N` above [`MAX_SAFE_INITIAL_PAGES`] — a module
    ///   reserving more than 64 MiB before it runs (`ResourceExhaustion`).
    /// * `changetype<` — `AssemblyScript`'s unchecked pointer reinterpretation
    ///   (`MemorySafety`).
    /// * `load<`/`store<` — raw linear-memory access that bypasses
    ///   `AssemblyScript`'s bounds checks (`MemorySafety`).
    /// * `memory.grow` — runtime growth of linear memory
    ///   (`ResourceExhaustion`).
    ///
    /// `passed == true` means these rules found nothing, which is not the same
    /// claim as "this module is safe"; callers must not report it as one.
    ///
    /// # Errors
    ///
    /// Never returns `Err`; the `Result` matches [`Self::validate`] so callers
    /// treat both inputs the same way.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_text(&self, content: &str) -> Result<SecurityValidation> {
        let mut issues = Vec::new();

        for (index, line) in content.lines().enumerate() {
            let line_no = index + 1;
            Self::check_memory_declaration(line, line_no, &mut issues);
            Self::check_unsafe_constructs(line, line_no, &mut issues);
        }

        Ok(SecurityValidation {
            passed: issues.is_empty(),
            issues,
        })
    }

    /// `(memory …)` limits on one line: unbounded growth and oversized reservations.
    fn check_memory_declaration(line: &str, line_no: usize, issues: &mut Vec<SecurityIssue>) {
        let Some(start) = line.find("(memory") else {
            return;
        };

        // Numeric tokens inside the declaration are its limits: `min [max]`.
        let rest = &line[start + "(memory".len()..];
        let decl = rest.split(')').next().unwrap_or(rest);
        let limits: Vec<u64> = decl
            .split_whitespace()
            .filter_map(|token| {
                token
                    .trim_matches(|c: char| !c.is_ascii_digit())
                    .parse()
                    .ok()
            })
            .collect();

        let Some(&min) = limits.first() else {
            return;
        };

        if limits.len() == 1 {
            issues.push(SecurityIssue {
                severity: Severity::Medium,
                description: format!(
                    "line {line_no}: linear memory declared with initial {min} page(s) and no \
                     maximum — the module can grow memory without bound"
                ),
                category: SecurityCategory::ResourceExhaustion,
            });
        }

        if min > MAX_SAFE_INITIAL_PAGES {
            issues.push(SecurityIssue {
                severity: Severity::High,
                description: format!(
                    "line {line_no}: linear memory reserves {min} pages \
                     ({} MiB) before the module runs",
                    min / 16
                ),
                category: SecurityCategory::ResourceExhaustion,
            });
        }
    }

    /// `AssemblyScript` escape hatches out of the checked memory model.
    fn check_unsafe_constructs(line: &str, line_no: usize, issues: &mut Vec<SecurityIssue>) {
        const MEMORY_SAFETY: &[(&str, &str)] = &[
            (
                "changetype<",
                "changetype<> reinterprets a value as another type without a check",
            ),
            (
                "load<",
                "load<T>() reads linear memory directly, bypassing bounds checks",
            ),
            (
                "store<",
                "store<T>() writes linear memory directly, bypassing bounds checks",
            ),
        ];

        for (needle, why) in MEMORY_SAFETY {
            if line.contains(needle) {
                issues.push(SecurityIssue {
                    severity: Severity::Medium,
                    description: format!("line {line_no}: {why}"),
                    category: SecurityCategory::MemorySafety,
                });
            }
        }

        if line.contains("memory.grow") {
            issues.push(SecurityIssue {
                severity: Severity::Low,
                description: format!("line {line_no}: memory.grow grows linear memory at runtime"),
                category: SecurityCategory::ResourceExhaustion,
            });
        }
    }
}

/// Initial linear-memory pages a module may reserve before it is called
/// oversized: 1024 pages x 64 KiB = 64 MiB.
pub const MAX_SAFE_INITIAL_PAGES: u64 = 1024;

impl Default for WasmSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_validator_new() {
        let validator = WasmSecurityValidator::new();
        // Validator is a unit struct, just verify it can be created
        let _ = validator;
    }

    #[test]
    fn test_security_validator_default() {
        let validator = WasmSecurityValidator;
        let _ = validator;
    }

    #[test]
    fn test_validate_valid_wasm_header() {
        let validator = WasmSecurityValidator::new();
        // Valid WASM magic number + version
        let data = b"\0asm\x01\x00\x00\x00";
        let result = validator.validate(data).unwrap();
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_too_small() {
        let validator = WasmSecurityValidator::new();
        let data = b"\0asm"; // Only 4 bytes
        let result = validator.validate(data).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues.len(), 1);
        assert!(matches!(
            result.issues[0].category,
            SecurityCategory::InvalidFormat
        ));
    }

    #[test]
    fn test_validate_invalid_magic() {
        let validator = WasmSecurityValidator::new();
        let data = b"invalid\x00"; // Wrong magic number
        let result = validator.validate(data).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].description.contains("magic number"));
    }

    /// A module with no memory and no unsafe construct has nothing to report.
    #[test]
    fn test_validate_text_clean_module() {
        let validator = WasmSecurityValidator::new();
        let result = validator.validate_text("(module)").unwrap();
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    /// The old `validate_ast`/`validate_text` pair returned `Ok(())` for every
    /// input, so `--security` could not report anything at all. These inputs
    /// are exactly the ones a WASM security check exists for.
    #[test]
    fn test_validate_text_flags_unbounded_memory() {
        let validator = WasmSecurityValidator::new();
        let result = validator.validate_text("(module\n  (memory 1))").unwrap();

        assert!(!result.passed, "unbounded memory must be reported");
        assert_eq!(result.issues.len(), 1);
        assert_eq!(
            result.issues[0].category,
            SecurityCategory::ResourceExhaustion
        );
        assert!(
            result.issues[0].description.contains("line 2"),
            "issues name the line they were found on: {}",
            result.issues[0].description
        );
    }

    #[test]
    fn test_validate_text_bounded_memory_is_clean() {
        let validator = WasmSecurityValidator::new();
        let result = validator.validate_text("(module (memory 1 4))").unwrap();
        assert!(
            result.passed,
            "a maximum bounds the growth: {:?}",
            result.issues
        );
    }

    #[test]
    fn test_validate_text_flags_oversized_initial_memory() {
        let validator = WasmSecurityValidator::new();
        let result = validator
            .validate_text("(module (memory 4096 8192))")
            .unwrap();

        assert!(!result.passed);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.severity == Severity::High
                && issue.category == SecurityCategory::ResourceExhaustion));
    }

    #[test]
    fn test_validate_text_flags_assemblyscript_escape_hatches() {
        let validator = WasmSecurityValidator::new();
        let source = "export function f(p: usize): i32 {\n  return load<i32>(p);\n}\n";
        let result = validator.validate_text(source).unwrap();

        assert!(!result.passed, "raw load<T>() must be reported");
        assert_eq!(result.issues[0].category, SecurityCategory::MemorySafety);
    }

    #[test]
    fn test_security_category_eq() {
        assert_eq!(
            SecurityCategory::InvalidFormat,
            SecurityCategory::InvalidFormat
        );
        assert_ne!(
            SecurityCategory::InvalidFormat,
            SecurityCategory::MemorySafety
        );
    }

    #[test]
    fn test_security_category_debug() {
        let category = SecurityCategory::MemorySafety;
        let debug_str = format!("{:?}", category);
        assert!(debug_str.contains("MemorySafety"));
    }

    #[test]
    fn test_security_issue_clone() {
        let issue = SecurityIssue {
            severity: Severity::High,
            description: "test issue".to_string(),
            category: SecurityCategory::Other,
        };
        let cloned = issue.clone();
        assert_eq!(issue.description, cloned.description);
    }

    #[test]
    fn test_security_validation_clone() {
        let validation = SecurityValidation {
            passed: true,
            issues: vec![],
        };
        let cloned = validation.clone();
        assert_eq!(validation.passed, cloned.passed);
    }

    #[test]
    fn test_all_security_categories() {
        // Test all category variants exist
        let _ = SecurityCategory::InvalidFormat;
        let _ = SecurityCategory::MemorySafety;
        let _ = SecurityCategory::ResourceExhaustion;
        let _ = SecurityCategory::CodeInjection;
        let _ = SecurityCategory::Other;
    }
}
