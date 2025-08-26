use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tdg::{
    config::TdgConfig, language_simple::Language, Comparison, MetricCategory, PenaltyTracker,
    ProjectScore, TdgScore,
};

pub struct TdgAnalyzer {
    config: TdgConfig,
}

impl TdgAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: TdgConfig::default(),
        })
    }

    pub fn with_config(config: TdgConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        let language = Language::from_extension(path);
        let source = fs::read_to_string(path)?;
        self.analyze_source(&source, language, Some(path.to_path_buf()))
    }

    pub fn analyze_source(
        &self,
        source: &str,
        language: Language,
        file_path: Option<PathBuf>,
    ) -> Result<TdgScore> {
        let mut tracker = PenaltyTracker::new();
        let mut score = TdgScore {
            language,
            confidence: language.confidence(),
            file_path,
            ..Default::default()
        };

        // Simple heuristic-based analysis for now
        score.structural_complexity = self.analyze_structural_complexity(source, &mut tracker);
        score.semantic_complexity = self.analyze_semantic_complexity(source, &mut tracker);
        score.duplication_ratio = self.analyze_duplication(source, &mut tracker);
        score.coupling_score = self.analyze_coupling(source, &mut tracker);
        score.doc_coverage = self.analyze_documentation(source, language, &mut tracker);
        score.consistency_score = self.analyze_consistency(source, language, &mut tracker);

        score.penalties_applied = tracker.get_attributions();
        score.calculate_total();

        Ok(score)
    }

    pub fn analyze_project(&self, dir: &Path) -> Result<ProjectScore> {
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();

        for file in files {
            match self.analyze_file(&file) {
                Ok(score) => scores.push(score),
                Err(e) => eprintln!("Warning: Failed to analyze {}: {}", file.display(), e),
            }
        }

        Ok(ProjectScore::aggregate(scores))
    }

    pub fn compare(&self, path1: &Path, path2: &Path) -> Result<Comparison> {
        let score1 = if path1.is_dir() {
            self.analyze_project(path1)?.average()
        } else {
            self.analyze_file(path1)?
        };

        let score2 = if path2.is_dir() {
            self.analyze_project(path2)?.average()
        } else {
            self.analyze_file(path2)?
        };

        Ok(Comparison::new(score1, score2))
    }

    fn analyze_structural_complexity(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.structural_complexity;

        let lines: Vec<&str> = source.lines().collect();
        let cyclomatic = self.estimate_cyclomatic_complexity(&lines);

        if cyclomatic > self.config.thresholds.max_cyclomatic_complexity {
            let excess = (cyclomatic - self.config.thresholds.max_cyclomatic_complexity) as f32;
            let penalty = (excess * 0.5).min(15.0);

            if let Some(applied) = tracker.apply(
                format!("high_cyclomatic_{}", cyclomatic),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cyclomatic complexity: {}", cyclomatic),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_semantic_complexity(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.semantic_complexity;

        let nesting_depth = self.estimate_nesting_depth(source);
        if nesting_depth > self.config.thresholds.max_nesting_depth as usize {
            let penalty = ((nesting_depth - self.config.thresholds.max_nesting_depth as usize)
                as f32)
                .min(10.0);

            if let Some(applied) = tracker.apply(
                format!("deep_nesting_{}", nesting_depth),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Deep nesting: {} levels", nesting_depth),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_duplication(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.duplication;

        let duplication_ratio = self.estimate_duplication_ratio(source);
        if duplication_ratio > 0.1 {
            let penalty = (duplication_ratio * 20.0).min(20.0);

            if let Some(applied) = tracker.apply(
                format!("duplication_{:.2}", duplication_ratio),
                MetricCategory::Duplication,
                penalty,
                format!("Code duplication: {:.1}%", duplication_ratio * 100.0),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_coupling(&self, source: &str, _tracker: &mut PenaltyTracker) -> f32 {
        let import_count = source
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("use ")
                    || trimmed.starts_with("import ")
                    || trimmed.starts_with("from ")
                    || trimmed.starts_with("#include ")
            })
            .count();

        let base_score = self.config.weights.coupling;
        if import_count > 20 {
            base_score - ((import_count - 20) as f32 * 0.2).min(10.0)
        } else {
            base_score
        }
        .max(0.0)
    }

    fn analyze_documentation(
        &self,
        source: &str,
        language: Language,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        let total_lines = source.lines().count();
        if total_lines == 0 {
            return self.config.weights.documentation;
        }

        let doc_lines = source
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                match language {
                    Language::Rust => trimmed.starts_with("///") || trimmed.starts_with("//!"),
                    Language::Python => trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''"),
                    Language::JavaScript | Language::TypeScript => {
                        trimmed.starts_with("/**") || trimmed.starts_with("*")
                    }
                    _ => trimmed.starts_with("//") || trimmed.starts_with("/*"),
                }
            })
            .count();

        let coverage = doc_lines as f32 / total_lines as f32;
        (coverage * self.config.weights.documentation).min(self.config.weights.documentation)
    }

    fn analyze_consistency(
        &self,
        source: &str,
        _language: Language,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        // Simple consistency check based on indentation style
        let lines: Vec<&str> = source.lines().collect();
        if lines.is_empty() {
            return self.config.weights.consistency;
        }

        let mut tab_count = 0;
        let mut space_count = 0;

        for line in &lines {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("    ") || line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if total_indented == 0 {
            return self.config.weights.consistency;
        }

        let consistency = if tab_count > space_count {
            tab_count as f32 / total_indented as f32
        } else {
            space_count as f32 / total_indented as f32
        };

        consistency * self.config.weights.consistency
    }

    fn estimate_cyclomatic_complexity(&self, lines: &[&str]) -> u32 {
        let mut complexity = 1;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                complexity += 1;
            }
            if trimmed.starts_with("for ") || trimmed.contains(" for ") {
                complexity += 1;
            }
            if trimmed.starts_with("while ") || trimmed.contains(" while ") {
                complexity += 1;
            }
            if trimmed.starts_with("match ") || trimmed.contains(" match ") {
                complexity += 1;
            }
            if trimmed.contains(" && ") || trimmed.contains(" || ") {
                complexity += trimmed.matches(" && ").count() as u32;
                complexity += trimmed.matches(" || ").count() as u32;
            }
        }

        complexity
    }

    fn estimate_nesting_depth(&self, source: &str) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.contains('{') {
                current_depth += trimmed.matches('{').count();
                max_depth = max_depth.max(current_depth);
            }
            if trimmed.contains('}') {
                current_depth = current_depth.saturating_sub(trimmed.matches('}').count());
            }
        }

        max_depth
    }

    fn estimate_duplication_ratio(&self, source: &str) -> f32 {
        let lines: Vec<&str> = source
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
            .collect();

        if lines.len() < 3 {
            return 0.0;
        }

        let mut duplicates = 0;
        for i in 0..lines.len() {
            for j in i + 1..lines.len() {
                if lines[i] == lines[j] && lines[i].len() > 10 {
                    duplicates += 1;
                }
            }
        }

        duplicates as f32 / lines.len() as f32
    }

    fn discover_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.discover_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn discover_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if !self.should_skip_directory(&path) {
                    self.discover_files_recursive(&path, files)?;
                }
            } else if self.should_analyze_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(
                name,
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | ".git"
                    | "__pycache__"
                    | ".pytest_cache"
                    | "venv"
                    | ".venv"
                    | "vendor"
                    | ".idea"
                    | ".vscode"
            )
        } else {
            false
        }
    }

    fn should_analyze_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "go"
                    | "java"
                    | "c"
                    | "h"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "hpp"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "kts"
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_simple_rust_code() -> Result<()> {
        let mut temp_file = NamedTempFile::with_suffix(".rs")?;
        writeln!(
            temp_file,
            r#"
            /// A simple function
            pub fn simple_function() -> i32 {{
                42
            }}
            "#
        )?;

        let analyzer = TdgAnalyzer::new()?;
        let score = analyzer.analyze_file(temp_file.path())?;

        assert_eq!(score.language, Language::Rust);
        assert!(score.total > 0.0);
        assert!(score.total <= 100.0);
        assert!(score.confidence > 0.0);

        Ok(())
    }

    #[test]
    fn test_analyze_complex_code() -> Result<()> {
        let source = r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        if x > 20 {
                            if x > 30 {
                                return x * 2;
                            }
                        }
                    }
                }
                x
            }
        "#;

        let analyzer = TdgAnalyzer::new()?;
        let score = analyzer.analyze_source(source, Language::Rust, None)?;

        assert!(score.structural_complexity < 25.0);
        assert!(!score.penalties_applied.is_empty());

        Ok(())
    }
}
