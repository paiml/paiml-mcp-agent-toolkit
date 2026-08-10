// analyzer_simple_core.rs — Public API methods for TdgAnalyzer
// Included by analyzer_simple.rs — shares parent module scope

impl TdgAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: TdgConfig::default(),
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With config.
    pub fn with_config(config: TdgConfig) -> Result<Self> {
        Ok(Self { config })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Analyze file.
    ///
    /// Refuses anything it cannot read as source. The metrics below are
    /// regex heuristics that only ever *subtract* from each component's cap, so
    /// input they recognise nothing in came back with the untouched
    /// 25/20/20/15/10 vector — 90.0 and an A, byte-identical for `README.md` and
    /// for `fn main( { let x = ;;;`. A file the analyzer cannot parse must not
    /// be graded at all, let alone graded perfect.
    pub fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        let language = Language::from_extension(path);
        if matches!(
            language,
            Language::Unknown | Language::Yaml | Language::Markdown
        ) {
            anyhow::bail!(
                "{}: TDG grades source files; {language} is not one",
                path.display()
            );
        }
        let source = fs::read_to_string(path)?;
        // Only Rust has a parser here; the other languages stay heuristic, so
        // their scores remain unverified by a parse.
        if language == Language::Rust {
            syn::parse_file(&source).map_err(|e| {
                anyhow::anyhow!(
                    "{}: not parseable as Rust ({e}); refusing to grade a file that did not parse",
                    path.display()
                )
            })?;
        }
        self.analyze_source(&source, language, Some(path.to_path_buf()))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Analyze source.
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

        // Lean-specific: detect `sorry` (proof incompleteness = critical defect)
        if language == Language::Lean {
            let sorry_count = count_lean_sorry(source);
            if sorry_count > 0 {
                score.has_critical_defects = true;
                score.critical_defects_count = sorry_count;
            }
        }

        score.penalties_applied = tracker.get_attributions();
        score.calculate_total();

        Ok(score)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Analyze project.
    pub fn analyze_project(&self, dir: &Path) -> Result<ProjectScore> {
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();

        // CB-1400: Resolve contract coverage for A-tier gating
        let contracted_paths = collect_contracted_file_paths(dir);

        for file in &files {
            // Skip include!() fragment files — they aren't standalone Rust modules
            // and tree-sitter can't parse them, resulting in false 0.0 (F-grade) scores
            if crate::cli::language_analyzer::is_include_fragment(file) {
                continue;
            }
            match self.analyze_file(file) {
                Ok(mut score) => {
                    apply_contract_coverage(&mut score, dir, &contracted_paths);
                    scores.push(score);
                }
                Err(e) => {
                    // Suppress warnings for include!() fragment files (PMAT-507)
                    if !crate::cli::language_analyzer::is_include_fragment(file) {
                        eprintln!("Warning: Failed to analyze {}: {}", file.display(), e);
                    }
                }
            }
        }

        Ok(ProjectScore::aggregate(scores))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Compare.
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
}

#[cfg(test)]
mod unparseable_input_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(suffix: &str, body: &str) -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(suffix).expect("temp file");
        write!(f, "{body}").expect("write");
        f.flush().expect("flush");
        f
    }

    /// Rust that does not parse used to score 90.0/A — the untouched component
    /// caps — because the heuristics simply matched nothing in it.
    #[test]
    fn test_unparseable_rust_is_not_graded() {
        let f = write_temp(".rs", "fn main( { let x = ;;;\n");
        let analyzer = TdgAnalyzer::new().expect("analyzer");
        let err = analyzer
            .analyze_file(f.path())
            .expect_err("a file that does not parse must not get a score");
        assert!(err.to_string().contains("not parseable as Rust"), "{err}");
    }

    /// Prose scored identically to source for the same reason.
    #[test]
    fn test_non_source_file_is_not_graded() {
        let f = write_temp(".md", "# Readme\n\nSome prose about the project.\n");
        let analyzer = TdgAnalyzer::new().expect("analyzer");
        let err = analyzer
            .analyze_file(f.path())
            .expect_err("markdown is not source and must not get a score");
        assert!(err.to_string().contains("not one"), "{err}");
    }

    #[test]
    fn test_valid_rust_still_scores() {
        let f = write_temp(".rs", "/// Doc\npub fn add(a: i32, b: i32) -> i32 { a + b }\n");
        let analyzer = TdgAnalyzer::new().expect("analyzer");
        let score = analyzer.analyze_file(f.path()).expect("valid Rust grades");
        assert_eq!(score.language, Language::Rust);
        assert!(score.total > 0.0);
    }
}

/// CB-1400: Apply provable-contract coverage to a TDG score.
///
/// If the score's file path matches any contracted path fragment, marks
/// `has_contract_coverage = true` and recalculates to apply/lift the A- cap.
fn apply_contract_coverage(score: &mut TdgScore, dir: &Path, contracted_paths: &[String]) {
    if contracted_paths.is_empty() {
        return;
    }
    let fp = match score.file_path {
        Some(ref fp) => fp,
        None => return,
    };
    let rel = fp.strip_prefix(dir).unwrap_or(fp);
    let rel_str = rel.to_string_lossy();
    score.has_contract_coverage = contracted_paths.iter().any(|cp| rel_str.contains(cp));
    if score.has_contract_coverage {
        score.calculate_total();
    }
}

/// CB-1400: Collect file-path fragments from provable-contracts binding.yaml.
///
/// Resolves the project's contract directory (local or sibling provable-contracts),
/// parses binding.yaml for `module_path` entries with `status: implemented`,
/// and converts them to file-path fragments (e.g., `services/tdg_calculator_core`).
///
/// Returns an empty vec if no contracts found (no A-tier gating applied).
fn collect_contracted_file_paths(project_path: &Path) -> Vec<String> {
    let binding_yaml = match resolve_binding_yaml(project_path) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let content = match fs::read_to_string(&binding_yaml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();
    let mut in_implemented = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track status field
        if trimmed.starts_with("status:") {
            in_implemented = trimmed.contains("implemented");
            continue;
        }

        // Extract module_path from implemented bindings
        if trimmed.starts_with("module_path:") && in_implemented {
            if let Some(val) = trimmed.strip_prefix("module_path:") {
                let module_path = val.trim().trim_matches('"').trim_matches('\'');
                // Convert module path to file-path fragment:
                // pmat::services::tdg_calculator_core → services/tdg_calculator_core
                // aprender::nn::functional::softmax → nn/functional/softmax
                let parts: Vec<&str> = module_path.split("::").collect();
                if parts.len() >= 2 {
                    // Skip crate name (first segment), join rest with /
                    let file_frag = parts[1..].join("/");
                    if !file_frag.is_empty() {
                        paths.push(file_frag);
                    }
                }
            }
        }

        // Reset on new binding entry
        if trimmed.starts_with("- contract:") {
            in_implemented = false;
        }
    }

    paths
}

/// Resolve binding.yaml for a project — local or sibling provable-contracts.
fn resolve_binding_yaml(project_path: &Path) -> Option<PathBuf> {
    // Local contracts/binding.yaml
    let local = project_path.join("contracts").join("binding.yaml");
    if local.exists() {
        return Some(local);
    }

    // Sibling provable-contracts repo
    let abs = std::fs::canonicalize(project_path).ok()?;
    let parent = abs.parent()?;
    let pv_contracts = parent.join("provable-contracts").join("contracts");
    if !pv_contracts.exists() {
        return None;
    }

    // Try directory name
    let dir_name = abs.file_name()?.to_str()?;
    let by_dir = pv_contracts.join(dir_name).join("binding.yaml");
    if by_dir.exists() {
        return Some(by_dir);
    }

    // Try Cargo.toml package name
    let cargo_toml = project_path.join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name") && trimmed.contains('=') {
                if let Some(name) = trimmed.split('=').nth(1) {
                    let pkg = name.trim().trim_matches('"');
                    let by_pkg = pv_contracts.join(pkg).join("binding.yaml");
                    if by_pkg.exists() {
                        return Some(by_pkg);
                    }
                }
                break;
            }
        }
    }

    None
}
