// SATD false positive detection: filters for string literals, documentation,
// metadata, functional descriptions, bug tracking IDs, and other non-debt patterns.

impl SATDDetector {
    /// Check if line is false positive SATD
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn is_false_positive_line(&self, line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let trimmed = line.trim();

        self.is_string_literal(trimmed)
            || self.is_raw_string_literal(trimmed)
            || self.is_satd_processing_code(trimmed)
            || self.is_assignment_with_satd(trimmed)
            || self.is_format_string(trimmed)
            || self.is_url_or_path(trimmed)
            || self.is_markdown_header(trimmed)
            || self.is_security_documentation(trimmed)
            || self.is_pattern_definition(trimmed)
            || self.is_enum_or_struct_field(trimmed)
            || self.is_functional_description(trimmed)
    }

    fn is_string_literal(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        trimmed.contains(r#""TODO"#)
            || trimmed.contains(r#""FIXME"#)
            || trimmed.contains(r#""HACK"#)
            || trimmed.contains(r#"'TODO'"#)
            || trimmed.contains(r#"'FIXME'"#)
            || trimmed.contains(r#"'HACK'"#)
    }

    fn is_raw_string_literal(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        trimmed.contains("r#\"") || trimmed.contains("r\"")
    }

    fn is_satd_processing_code(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        trimmed.contains(".matches(")
            || trimmed.contains("regex:")
            || trimmed.contains("DebtPattern")
            || trimmed.contains("comment_text:")
            || trimmed.contains("classify_comment")
            || trimmed.contains("debt_classifier")
            || trimmed.contains("SATDAnalysis")
    }

    fn is_assignment_with_satd(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        trimmed.contains(" = ") && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_format_string(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        (trimmed.contains("format!")
            || trimmed.contains("println!")
            || trimmed.contains("write!")
            || trimmed.contains("{}"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_url_or_path(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Check for actual URLs or file paths, not just comment markers
        (trimmed.contains("http://")
            || trimmed.contains("https://")
            || trimmed.contains("file://")
            || trimmed.contains(".com/")
            || (trimmed.contains('/') && !trimmed.starts_with("//"))
            || trimmed.contains('\\'))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_security_documentation(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Security-related documentation/comments (not actual security debt)
        (trimmed.contains("Security") || trimmed.contains("security"))
            && (trimmed.contains("check")
                || trimmed.contains("validation")
                || trimmed.contains("properties")
                || trimmed.contains("vulnerabilities")
                || trimmed.contains("patterns")
                || trimmed.contains("issues")
                || trimmed.contains("concerns")
                || trimmed.starts_with("//")
                || trimmed.starts_with('*')
                || trimmed.starts_with('/'))
    }

    fn is_pattern_definition(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Pattern definitions in SATD detection code
        trimmed.contains("let valid_patterns")
            || trimmed.contains("let patterns")
            || trimmed.contains("vec![\"")
            || (trimmed.contains("\"TODO\"") && trimmed.contains('['))
            || (trimmed.contains("FIXME") && trimmed.contains("regex"))
    }

    fn is_enum_or_struct_field(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Enum variants or struct fields that mention SATD concepts
        (trimmed.contains("Security") || trimmed.contains("Design") || trimmed.contains("Defect"))
            && (trimmed.contains(',') || trimmed.contains('=') || trimmed.contains("::"))
    }

    fn is_markdown_header(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Markdown headers: # Security, ## Security, ### Security, etc.
        // Common in CHANGELOG.md, README.md, and documentation templates
        let starts_with_hash = trimmed.starts_with('#');
        if !starts_with_hash {
            return false;
        }

        // Remove leading # symbols and whitespace to get header content
        let content = trimmed.trim_start_matches('#').trim();

        // Check if it's a common section header (especially CHANGELOG sections)
        // or a version header pattern like [1.0.0]
        content == "Security"
            || content == "Added"
            || content == "Changed"
            || content == "Deprecated"
            || content == "Removed"
            || content == "Fixed"
            || content == "Unreleased"
            || content == "Changelog"
            || content == "CHANGELOG"
            || content.starts_with('[') // [Unreleased], [1.0.0], etc.
    }

    fn is_functional_description(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Comments describing functionality, not admitting technical debt
        if trimmed.starts_with("//") {
            let comment_text = trimmed.trim_start_matches("//").trim().to_lowercase();

            // Section headers with separators (=== or ---)
            if comment_text.contains("===")
                || comment_text.contains("---")
                || comment_text.contains("───")
            {
                return true;
            }

            // Mathematical notation (e.g., "s^T x temp")
            if comment_text.contains("\u{00d7}")
                || comment_text.contains("\u{2211}")
                || comment_text.contains("^t ")
                || comment_text.contains("^t\u{00d7}")
            {
                return true;
            }

            // Section header patterns (capitalized with parenthetical)
            if comment_text.contains("mitigation")
                || comment_text.contains("isolation")
                || comment_text.starts_with("output ")
                || comment_text.starts_with("input ")
                || comment_text.starts_with("all ")
            {
                return true;
            }

            // Phone/format patterns with XXX
            if comment_text.contains("xxx-xxx") || comment_text.contains("xxx.xxx") {
                return true;
            }

            // Check for common functional description patterns
            comment_text.starts_with("check for")
                || comment_text.starts_with("handle ")
                || comment_text.starts_with("phase ")
                || comment_text.starts_with("load ")
                || comment_text.starts_with("create ")
                || comment_text.starts_with("process ")
                || comment_text.starts_with("detect ")
                || comment_text.starts_with("scan ")
                || comment_text.starts_with("parse ")
                || comment_text.starts_with("analyze ")
                || comment_text.starts_with("extract ")
                || comment_text.starts_with("find ")
                || comment_text.starts_with("search ")
                || comment_text.starts_with("identify ")
                || comment_text.starts_with("validate ")
                || comment_text.starts_with("verify ")
                || comment_text.contains("relative links")
                || comment_text.contains("special modes")
                || comment_text.contains("documentation issues")
                || comment_text.contains("single file")
                || (comment_text.contains("broken") && comment_text.contains("links"))
                || (comment_text.contains("bug") && comment_text.contains("report"))
                // False positive fixes: Comments describing bug-related functionality
                || (comment_text.contains("broken") && comment_text.contains("dep"))
                || (comment_text.contains("bug") && comment_text.contains("fix") && (comment_text.contains("pattern") || comment_text.contains("patterns")))
                || (comment_text.contains("bug") && comment_text.contains("fix") && (comment_text.contains("claim") || comment_text.contains("claims")))
                || (comment_text.contains("bug") && comment_text.contains("fix") && comment_text.contains("commit"))
                || (comment_text.contains("describes functionality") && comment_text.contains("bug"))
                || (comment_text.contains("extract") && comment_text.contains("bug"))
                // Bug tracking ID patterns (BUG-XXX, PMAT-BUG-XXX like JIRA tickets)
                || self.is_bug_tracking_id(&comment_text)
                // Fixed bug descriptions ("Bug: Previously...", "BUG-064 FIX:")
                || self.is_fixed_bug_description(&comment_text)
                // Bug estimation/metrics functionality
                || (comment_text.contains("bug") && comment_text.contains("estimate"))
                // Comments about detection/markers (describing functionality, not debt)
                || (comment_text.contains("marker") && !comment_text.contains("add"))
                || (comment_text.contains("detection") && !comment_text.contains("need"))
                || (comment_text.contains("pattern") && comment_text.contains("match"))
        } else {
            false
        }
    }

    /// Check if comment contains bug tracking ID (like JIRA tickets)
    /// Patterns: BUG-123, PMAT-BUG-456, Issue-789
    fn is_bug_tracking_id(&self, text: &str) -> bool {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let text_lower = text.to_lowercase();
        // Pattern 1: BUG-XXX (where XXX is digits)
        if text_lower.contains("bug-") {
            // Check if followed by digits
            if let Some(pos) = text_lower.find("bug-") {
                let after_dash = &text[pos + 4..];
                if after_dash.chars().take(3).all(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
        // Pattern 2: PMAT-BUG-XXX, PROJECT-BUG-XXX
        if text_lower.contains("-bug-") {
            return true;
        }
        // Pattern 3: "BUG-XXX FIX:" or "BUG-XXX:" at start
        if text_lower.contains("bug-") && (text_lower.contains(" fix:") || text_lower.contains(":"))
        {
            return true;
        }
        false
    }

    /// Check if comment describes a FIXED bug (not a current bug)
    /// Patterns: "Bug: Previously...", "CRITICAL FIX:", "Root cause:", etc.
    fn is_fixed_bug_description(&self, text: &str) -> bool {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let text_lower = text.to_lowercase();
        // Pattern 1: "Bug: Previously..." - past tense description
        if text_lower.starts_with("bug:") && text_lower.contains("previous") {
            return true;
        }
        // Pattern 2: "CRITICAL FIX:", "BUG FIX:"
        if text_lower.contains(" fix:") {
            return true;
        }
        // Pattern 3: "This ensures..." after "Bug: ..." (describing fix)
        if text_lower.contains("bug:")
            && (text_lower.contains("ensure") || text_lower.contains("prevent"))
        {
            return true;
        }
        // Pattern 4: "Root cause:" explanations (often follow bug IDs)
        if text_lower.contains("root cause") {
            return true;
        }
        false
    }

    /// Check if line is documentation, test, or metadata about SATD
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn is_documentation_or_metadata(&self, line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let trimmed = line.trim();

        self.is_documentation_comment(trimmed)
            || self.is_test_code(trimmed)
            || self.is_log_message(trimmed)
            || self.is_error_description(trimmed)
    }

    fn is_documentation_comment(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        self.is_module_documentation(trimmed)
            || self.is_technical_debt_documentation(trimmed)
            || self.is_api_documentation(trimmed)
            || self.is_doctest_example(trimmed)
    }

    fn is_module_documentation(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        trimmed.starts_with("//!") || trimmed.starts_with("///")
    }

    fn is_technical_debt_documentation(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        let lower = trimmed.to_lowercase();
        let mentions_td_concepts = lower.contains("technical debt")
            || trimmed.contains("TDG")
            || trimmed.contains("SATD")
            || lower.contains("self-admitted")
            || lower.contains("debt marker")
            || lower.contains("debt detection")
            || lower.contains("debt pattern");
        let is_comment =
            trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with('/');
        mentions_td_concepts && is_comment
    }

    fn is_api_documentation(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        let is_doc_marker = trimmed.starts_with('*')
            || trimmed.contains("@param")
            || trimmed.contains("@return")
            || trimmed.contains("Example:")
            || trimmed.contains("# Examples")
            || trimmed.contains("# Parameters");
        let mentions_markers =
            trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("security");
        is_doc_marker && mentions_markers
    }

    fn is_doctest_example(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        let has_comment_marker = trimmed.contains("// ");
        let has_debt_marker = trimmed.contains("TODO") || trimmed.contains("FIXME");
        let has_code_marker =
            trimmed.contains("let ") || trimmed.contains("assert") || trimmed.contains("unwrap");
        has_comment_marker && has_debt_marker && has_code_marker
    }

    fn is_test_code(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        (trimmed.contains("assert")
            || trimmed.contains("expect")
            || trimmed.contains(".unwrap()")
            || trimmed.contains("panic!"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_log_message(&self, trimmed: &str) -> bool {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        (trimmed.contains("log::")
            || trimmed.contains("debug!")
            || trimmed.contains("info!")
            || trimmed.contains("warn!")
            || trimmed.contains("error!")
            || trimmed.contains("trace!"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_error_description(&self, trimmed: &str) -> bool {
        (trimmed.contains("Error:")
            || trimmed.contains("error:")
            || trimmed.contains("message:")
            || trimmed.contains("description:"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    /// Comprehensive false positive detection for SATD
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_likely_test_data_or_pattern(&self, line: &str, file_path: &Path) -> bool {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        // First check: Should we exclude this entire file?
        if self.should_exclude_file(file_path) {
            return true;
        }

        // Second check: Is this line a false positive?
        if self.is_false_positive_line(line) {
            return true;
        }

        // Third check: Is this documentation or metadata?
        if self.is_documentation_or_metadata(line) {
            return true;
        }

        false
    }
}
