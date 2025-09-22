use super::gate::SatdResult;
use once_cell::sync::Lazy;
use regex::Regex;

static SATD_PATTERNS: Lazy<Vec<(&str, Regex)>> = Lazy::new(|| {
    vec![
        ("TODO", Regex::new(r"\bTODO\b").unwrap()),
        ("FIXME", Regex::new(r"\bFIXME\b").unwrap()),
        ("HACK", Regex::new(r"\bHACK\b").unwrap()),
        ("XXX", Regex::new(r"\bXXX\b").unwrap()),
        ("REFACTOR", Regex::new(r"\bREFACTOR\b").unwrap()),
        ("OPTIMIZE", Regex::new(r"\bOPTIMIZE\b").unwrap()),
        ("REVIEW", Regex::new(r"\bREVIEW\b").unwrap()),
        ("DEPRECATED", Regex::new(r"\bDEPRECATED\b").unwrap()),
        ("TEMPORARY", Regex::new(r"\bTEMPORARY\b").unwrap()),
    ]
});

pub struct SatdDetector {
    patterns: Vec<(&'static str, Regex)>,
}

impl Default for SatdDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SatdDetector {
    pub fn new() -> Self {
        Self {
            patterns: SATD_PATTERNS.clone(),
        }
    }

    pub fn detect(&self, source: &str) -> SatdResult {
        let mut count = 0;
        let mut found_patterns = Vec::new();

        for (pattern_name, regex) in &self.patterns {
            let matches = regex.find_iter(source).count();
            if matches > 0 {
                count += matches;
                if !found_patterns.contains(&pattern_name.to_string()) {
                    found_patterns.push(pattern_name.to_string());
                }
            }
        }

        SatdResult {
            count,
            patterns: found_patterns,
        }
    }

    pub fn detect_in_comments(&self, source: &str) -> SatdResult {
        // Extract only comments from source
        let comments = self.extract_comments(source);
        self.detect(&comments)
    }

    fn extract_comments(&self, source: &str) -> String {
        let mut in_block_comment = false;
        let mut comments = String::new();
        let lines = source.lines();

        for line in lines {
            let trimmed = line.trim();

            // Block comment handling
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                comments.push_str(line);
                comments.push('\n');
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            if in_block_comment {
                comments.push_str(line);
                comments.push('\n');
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            // Line comment handling
            if let Some(comment_start) = line.find("//") {
                comments.push_str(&line[comment_start..]);
                comments.push('\n');
            }
        }

        comments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_todo_patterns() {
        let detector = SatdDetector::new();
        let source = "// TODO: implement this\n// FIXME: broken code";

        let result = detector.detect(source);
        assert_eq!(result.count, 2);
        assert!(result.patterns.contains(&"TODO".to_string()));
        assert!(result.patterns.contains(&"FIXME".to_string()));
    }

    #[test]
    fn test_no_satd_in_clean_code() {
        let detector = SatdDetector::new();
        let source = "fn clean_function() {\n    println!(\"Clean code\");\n}";

        let result = detector.detect(source);
        assert_eq!(result.count, 0);
        assert!(result.patterns.is_empty());
    }
}
