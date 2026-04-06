#![cfg_attr(coverage_nightly, coverage(off))]
//! Heuristic-based complexity and function-name analysis for non-AST languages.
use std::path::Path;
use tracing::info;

use super::types::SimpleDeepContext;

impl SimpleDeepContext {
    /// Heuristic fallback for complexity analysis
    pub(super) async fn complexity_heuristic_fallback(
        &self,
        file_path: &Path,
        extension: &str,
    ) -> (usize, usize, f64) {
        match self
            .analyze_file_complexity_heuristic(file_path, extension)
            .await
        {
            Ok((count, high, avg)) => (count, high, avg),
            Err(_) => (0, 0, 0.0),
        }
    }

    /// Extract function names using regex patterns (shared implementation)
    pub(super) fn extract_names_by_regex(content: &str, patterns: &[&str]) -> Vec<String> {
        let mut names = Vec::new();
        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(content) {
                    if let Some(name) = cap.get(1) {
                        names.push(name.as_str().to_string());
                    }
                }
            }
        }
        names
    }

    /// JS/TS function name extraction with multi-pattern dedup
    pub(super) fn extract_js_ts_function_names(content: &str, file_path: &Path) -> Vec<String> {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        let patterns = [
            r"function\s+(\w+)\s*\(",
            r"(?m)^\s*(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>",
            r"(?m)^\s*(?:async\s+)?(\w+)\s*\([^)]*\)\s*\{",
            r"(?m)^\s*(?:static\s+)?(\w+)\s*\([^)]*\)\s*\{",
            r"(\w+)\s*:\s*function\s*\([^)]*\)",
            r"(\w+)\s*\([^)]*\)\s*\{",
            r"(?m)^\s*(?:async\s+)?(\w+)\s*\([^)]*\)\s*:",
        ];
        info!(
            "Using comprehensive TypeScript/JavaScript regex patterns for {}",
            file_path.display()
        );
        let mut function_names = Vec::new();
        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(content) {
                    if let Some(name) = cap.get(1) {
                        let name_str = name.as_str().to_string();
                        if !function_names.contains(&name_str) {
                            function_names.push(name_str);
                        }
                    }
                }
            }
        }
        function_names
    }

    /// Extract function names using heuristic regex patterns
    pub(super) async fn extract_function_names_heuristic(
        &self,
        file_path: &Path,
        extension: &str,
    ) -> anyhow::Result<Vec<String>> {
        use tokio::fs;
        let content = fs::read_to_string(file_path).await?;

        // JS/TS has special multi-pattern dedup logic
        if matches!(extension, "js" | "ts") {
            return Ok(Self::extract_js_ts_function_names(&content, file_path));
        }

        // All other languages use simple regex patterns
        let patterns: &[&str] = match extension {
            "py" => &[r"(?m)^\s*(?:async\s+)?def\s+(\w+)\s*\("],
            "java" => &[
                r"(?:public|private|protected)\s+(?:static\s+)?(?:\w+(?:<[^>]*>)?\s+)+(\w+)\s*\([^)]*\)\s*\{",
            ],
            "go" => &[r"(?m)^func\s+(?:\([^)]*\)\s+)?(\w+)\s*\("],
            "c" | "cpp" | "cc" | "cxx" | "cu" | "cuh" => {
                &[r"(?m)^\s*\w+(?:\s*\**)?\s+(\w+)\s*\([^)]*\)\s*\{"]
            }
            "rb" | "ruchy" => &[r"(?m)^\s*def\s+(\w+)"],
            "kt" => &[r"(?m)^\s*(?:suspend\s+)?fun\s+(\w+)\s*\("],
            "cs" => &[
                r"(?:public|private|protected|internal)?\s*(?:static|async)?\s*\w+\s+(\w+)\s*\([^)]*\)",
            ],
            "lua" => &[
                r"(?m)^\s*function\s+(\w+(?:[.:]\w+)*)\s*\(",
                r"(?m)^\s*local\s+function\s+(\w+)\s*\(",
            ],
            "lean" => &[
                r"(?m)^\s*(?:noncomputable\s+|partial\s+|private\s+|protected\s+)?def\s+(\w+)",
                r"(?m)^\s*(?:private\s+)?theorem\s+(\w+)",
                r"(?m)^\s*(?:private\s+)?lemma\s+(\w+)",
                r"(?m)^\s*(?:structure|class|inductive)\s+(\w+)",
            ],
            _ => return Ok(vec![]),
        };

        let mut names = Self::extract_names_by_regex(&content, patterns);

        // Filter language-specific keywords
        let keywords: &[&str] = match extension {
            "c" | "cpp" | "cc" | "cxx" | "cu" | "cuh" => &["if", "for", "while", "switch", "catch"],
            "cs" => &["if", "while", "for", "foreach", "switch"],
            _ => &[],
        };
        if !keywords.is_empty() {
            names.retain(|n| !keywords.contains(&n.as_str()));
        }
        Ok(names)
    }

    /// Analyze file complexity using heuristics for non-Rust languages
    pub(super) async fn analyze_file_complexity_heuristic(
        &self,
        file_path: &Path,
        extension: &str,
    ) -> anyhow::Result<(usize, usize, f64)> {
        use tokio::fs;
        let content = fs::read_to_string(file_path).await?;

        let function_patterns = match extension {
            "py" => vec![r"(?m)^\s*def\s+\w+", r"(?m)^\s*async\s+def\s+\w+"],
            "js" | "ts" => vec![
                r"function\s+\w+",
                r"(?m)^\s*const\s+\w+\s*=.*=>",
                r"(?m)^\s*\w+\s*\([^)]*\)\s*\{",
            ],
            "java" => vec![r"(public|private|protected)\s+\w+\s+\w+\s*\("],
            "go" => vec![r"(?m)^func\s+(\(\w+\s+\*?\w+\)\s+)?\w+\s*\("],
            "c" | "cpp" | "cc" | "cxx" | "cu" | "cuh" => vec![r"(?m)^\w+\s+\w+\s*\([^)]*\)\s*\{"],
            "cs" => {
                vec![r"(public|private|protected|internal)?\s*(static|async)?\s*\w+\s+\w+\s*\("]
            }
            "kt" => vec![r"(?m)^\s*(?:suspend\s+)?fun\s+\w+\s*\("],
            "lua" => vec![r"(?m)^\s*function\s+\w+", r"(?m)^\s*local\s+function\s+\w+"],
            "lean" => vec![
                r"(?m)^\s*(?:noncomputable\s+|partial\s+|private\s+|protected\s+)?def\s+\w+",
                r"(?m)^\s*(?:private\s+)?theorem\s+\w+",
                r"(?m)^\s*(?:private\s+)?lemma\s+\w+",
            ],
            _ => vec![],
        };

        if function_patterns.is_empty() {
            return Ok((0, 0, 0.0));
        }

        let mut function_count = 0;
        let mut complexity_sum = 0;
        let mut high_complexity_count = 0;

        for pattern in function_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(&content) {
                    function_count += 1;
                    if let Some(func_match) = cap.get(0) {
                        let start = func_match.start();
                        let func_end = self
                            .find_function_end(content.get(start..).unwrap_or_default(), extension);
                        if let Some(end) = func_end {
                            let func_body = content.get(start..start + end).unwrap_or_default();
                            let complexity = self.estimate_complexity(func_body, extension);
                            complexity_sum += complexity;
                            if complexity > 10 {
                                high_complexity_count += 1;
                            }
                        }
                    }
                }
            }
        }

        let avg_complexity = if function_count > 0 {
            complexity_sum as f64 / function_count as f64
        } else {
            0.0
        };

        Ok((function_count, high_complexity_count, avg_complexity))
    }

    /// Find the end of a function body (dispatches to per-language helpers)
    pub(super) fn find_function_end(&self, content: &str, extension: &str) -> Option<usize> {
        match extension {
            "py" => Self::find_function_end_python(content),
            "lua" => Self::find_function_end_lua(content),
            _ => Self::find_function_end_brace(content),
        }
    }

    /// Python: indentation-based function end detection
    pub(super) fn find_function_end_python(content: &str) -> Option<usize> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return None;
        }
        let first_indent = lines[0].len() - lines[0].trim_start().len();
        for (i, line) in lines.iter().enumerate().skip(1) {
            if !line.trim().is_empty() {
                let indent = line.len() - line.trim_start().len();
                if indent <= first_indent {
                    return Some(lines[..i].join("\n").len());
                }
            }
        }
        Some(content.len())
    }

    /// Lua: end-keyword depth tracking
    pub(super) fn find_function_end_lua(content: &str) -> Option<usize> {
        let mut depth = 0;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("function ")
                || trimmed.starts_with("local function ")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed == "do"
                || trimmed.starts_with("do ")
            {
                depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end,") {
                depth -= 1;
                if depth <= 0 {
                    let byte_offset: usize = content.lines().take(i + 1).map(|l| l.len() + 1).sum();
                    return Some(byte_offset);
                }
            }
        }
        Some(content.len())
    }

    /// C-like languages: string-aware brace counting
    pub(super) fn find_function_end_brace(content: &str) -> Option<usize> {
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;
        for (i, ch) in content.chars().enumerate() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && in_string {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            if ch == '{' {
                depth += 1;
            }
            if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
        }
        None
    }

    /// Estimate complexity based on control flow keywords
    pub(super) fn estimate_complexity(&self, func_body: &str, extension: &str) -> usize {
        let control_flow_keywords = match extension {
            "py" => vec![
                "if ", "elif ", "else:", "for ", "while ", "try:", "except:", "finally:",
            ],
            "js" | "ts" => vec![
                "if ", "else ", "for ", "while ", "do ", "switch ", "case ", "catch ", "finally ",
            ],
            "java" | "c" | "cpp" | "cu" | "go" => vec![
                "if ", "else ", "for ", "while ", "do ", "switch ", "case ", "catch ", "finally ",
            ],
            "lua" => vec![
                "if ", "elseif ", "else", "for ", "while ", "repeat", "until ",
            ],
            _ => vec![],
        };

        let mut complexity = 1;
        for keyword in control_flow_keywords {
            complexity += func_body.matches(keyword).count();
        }
        complexity += func_body.matches("&&").count();
        complexity += func_body.matches("||").count();
        // Lua uses "and"/"or" instead of &&/||
        if extension == "lua" {
            complexity += func_body.matches(" and ").count();
            complexity += func_body.matches(" or ").count();
        }
        complexity
    }
}
