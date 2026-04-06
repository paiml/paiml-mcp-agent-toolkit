// Factor calculations: coupling, duplication, domain risk, provability
// Split from tdg_calculator_factors.rs for complexity budget

impl TDGCalculator {
    /// Analyze coupling metrics for a file
    #[allow(dead_code)]
    fn analyze_coupling(&self, _file: &Path, ast: &UnifiedAstNode) -> CouplingMetrics {
        debug_assert!(_file.exists(), "_file must exist: {}", _file.display());
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        self.extract_dependencies(ast, &mut imports, &mut exports);

        let efferent = imports.len();
        let afferent = exports.len();

        let instability = if afferent + efferent == 0 {
            0.0
        } else {
            efferent as f64 / (afferent + efferent) as f64
        };

        CouplingMetrics {
            afferent,
            efferent,
            instability,
        }
    }

    #[allow(dead_code)]
    fn extract_dependencies(
        &self,
        node: &UnifiedAstNode,
        imports: &mut Vec<String>,
        exports: &mut Vec<String>,
    ) {
        match &node.kind {
            AstKind::Import(_) => imports.push("import".to_string()),
            AstKind::Function(_) => exports.push("function".to_string()),
            AstKind::Class(_) => exports.push("class".to_string()),
            _ => {}
        }
    }

    async fn calculate_coupling_factor(&self, path: &Path) -> Result<f64> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = tokio::fs::read_to_string(path).await?;
        let import_count = self.count_imports(&content);

        let export_count = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("pub fn")
                    || trimmed.starts_with("pub struct")
                    || trimmed.starts_with("pub enum")
                    || trimmed.starts_with("export ")
                    || trimmed.contains("module.exports")
            })
            .count();

        let total = import_count + export_count;
        let instability = if total == 0 {
            0.0
        } else {
            import_count as f64 / total as f64
        };

        let import_factor = (import_count as f64 / 15.0).min(2.0);
        let instability_factor = instability * 2.0;
        let complexity_penalty = if import_count > 20 { 1.0 } else { 0.0 };

        let score = import_factor + instability_factor + complexity_penalty;
        Ok(score.min(5.0))
    }

    async fn calculate_duplication_factor(&self, path: &Path) -> Result<f64> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with('#'))
            .collect();

        if lines.len() < 10 {
            return Ok(0.0);
        }

        let mut line_counts = HashMap::new();
        for line in &lines {
            if line.len() > 10 {
                *line_counts.entry(*line).or_insert(0) += 1;
            }
        }

        let duplicate_lines: usize = line_counts
            .values()
            .filter(|&&count| count > 1)
            .map(|&count| count - 1)
            .sum();

        let duplication_percentage = (duplicate_lines as f64 / lines.len() as f64) * 100.0;
        Ok((duplication_percentage / 30.0).min(5.0))
    }

    async fn calculate_domain_risk(&self, path: &Path) -> Result<f64> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let path_str = path.to_string_lossy();
        let mut risk: f64 = 0.0;

        if path_str.contains("auth") || path_str.contains("crypto") || path_str.contains("security") {
            risk += 2.0;
        }
        if path_str.contains("database") || path_str.contains("migration") {
            risk += 1.5;
        }
        if path_str.contains("api") || path_str.contains("integration") {
            risk += 1.0;
        }

        Ok(risk.min(5.0))
    }

    async fn calculate_provability_factor(&self, path: &Path) -> Result<f64> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let func_id = FunctionId {
            file_path: path.to_string_lossy().to_string(),
            function_name: file_name.to_string(),
            line_number: 1,
        };

        let summaries = self
            .provability_analyzer
            .analyze_incrementally(&[func_id])
            .await;

        if let Some(summary) = summaries.first() {
            Ok(summary.provability_score)
        } else {
            Ok(0.0)
        }
    }

    fn count_imports(&self, content: &str) -> usize {
        use std::sync::OnceLock;

        static IMPORT_PATTERNS: OnceLock<[regex::Regex; 4]> = OnceLock::new();
        let patterns = IMPORT_PATTERNS.get_or_init(|| {
            [
                regex::Regex::new(r"^use\s+").expect("internal error"),
                regex::Regex::new(r"^import\s+").expect("internal error"),
                regex::Regex::new(r"^from\s+.*\s+import").expect("internal error"),
                regex::Regex::new(r"^require\(").expect("internal error"),
            ]
        });

        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                patterns.iter().any(|p| p.is_match(trimmed))
            })
            .count()
    }
}
