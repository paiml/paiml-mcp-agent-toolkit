// analyzer_simple_core.rs — Public API methods for TdgAnalyzer
// Included by analyzer_simple.rs — shares parent module scope

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
                    // CB-1400: Mark files with provable-contract coverage
                    if !contracted_paths.is_empty() {
                        if let Some(ref fp) = score.file_path {
                            let rel = fp.strip_prefix(dir).unwrap_or(fp);
                            let rel_str = rel.to_string_lossy();
                            score.has_contract_coverage = contracted_paths
                                .iter()
                                .any(|cp| rel_str.contains(cp));
                            if score.has_contract_coverage {
                                // Recalculate to apply/lift the A- cap
                                score.calculate_total();
                            }
                        }
                    }
                    scores.push(score);
                }
                Err(e) => {
                    // Suppress warnings for include!() fragment files (PMAT-507)
                    if !crate::cli::language_analyzer::is_include_fragment(&file) {
                        eprintln!("Warning: Failed to analyze {}: {}", file.display(), e);
                    }
                }
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
