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
        if !grades_source(path) {
            anyhow::bail!(
                "{}: TDG grades source files; {language} is not one",
                path.display()
            );
        }
        let source = fs::read_to_string(path)?;
        // The parse gate used to be Rust-only, so `def f(:` in a .py file still
        // collected the untouched component caps from the line heuristics — the
        // same fabricated grade the Rust gate was added to stop. Every language
        // with a grammar in this build is now gated the same way; languages
        // without one still fall through to the heuristics, unverified.
        if let Some(parse_error) = source_parse_error(&source, language) {
            anyhow::bail!(
                "{}: {parse_error}; refusing to grade a file that did not parse",
                path.display()
            );
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
        Ok(self.analyze_project_reporting_ungraded(dir)?.0)
    }

    /// Analyze project, returning the files that could NOT be graded alongside
    /// the score, each with the reason it was refused.
    ///
    /// The skipped files used to exist only as an `eprintln!`, and stderr is not
    /// part of an MCP response: the `quality_gate` tool answered
    /// `{"passed":true,"grade":"A","not_measured":[],"files_analyzed":1}` for a
    /// 9-file tree whose other 8 files this same build refuses one at a time
    /// (`quality-gate --file` exits 4 on each). `total_files` counts the files
    /// that SUCCEEDED, so a caller holding only a `ProjectScore` cannot tell
    /// "9 files, all graded" from "9 files, 1 graded" — the skip has to come back
    /// through the return value for a gate to be able to disclose it.
    pub fn analyze_project_reporting_ungraded(
        &self,
        dir: &Path,
    ) -> Result<(ProjectScore, Vec<(PathBuf, String)>)> {
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();
        let mut ungraded = Vec::new();

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
                        ungraded.push((file.clone(), e.to_string()));
                    }
                }
            }
        }

        // `discover_files` walks in `read_dir` order, which is the filesystem's.
        // This list is reported verbatim in a JSON payload, so it is sorted for
        // the same reason `grade_distribution` is a `BTreeMap`: identical input
        // must serialise identically.
        ungraded.sort();

        // The skip list also travels ON the score, not only beside it: every
        // consumer that holds a bare `ProjectScore` (`analyze tdg --format
        // json`, the table renderer, SARIF) used to have no way to learn that
        // `average_score` was computed over a subset. See `UngradedFile`.
        let mut project = ProjectScore::aggregate(scores);
        project.ungraded_files = ungraded
            .iter()
            .map(|(path, reason)| crate::tdg::UngradedFile {
                path: path.display().to_string(),
                reason: reason.clone(),
            })
            .collect();

        Ok((project, ungraded))
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

/// Refuse a file this build can *prove* does not parse.
///
/// The same gate `TdgAnalyzer::analyze_file` applies, minus its "TDG only
/// grades source files" rule, so callers that legitimately scan Markdown or
/// YAML (the SATD checks, for instance) are unaffected. Exposed because the
/// guard was reachable only through the MCP `quality_gate` tool: the CLI's
/// `quality-gate --file` ran its own checks and reported
/// "✅ Quality Gate: PASSED / Total Violations: 0" for `def f(:` in a .py file —
/// the two surfaces disagreeing about the same file, which is the contradiction
/// class this sweep exists to remove.
///
/// `Ok(())` means "it parsed" OR "this build has no parser for that language",
/// which are deliberately not distinguished: neither is grounds for refusal.
pub fn ensure_parseable(path: &Path) -> Result<()> {
    let language = Language::from_extension(path);
    let Ok(source) = fs::read_to_string(path) else {
        // Unreadable / non-UTF-8 is the caller's problem to report, not ours.
        return Ok(());
    };
    if let Some(parse_error) = source_parse_error(&source, language) {
        anyhow::bail!(
            "{}: {parse_error}; refusing to report a quality verdict on a file that did not parse",
            path.display()
        );
    }
    Ok(())
}

/// Is this file's language one TDG grades at all?
///
/// `analyze_file`'s refusal is about TDG's own scope, not about the file being
/// bad input, so a caller that reports a *gate verdict* rather than a grade must
/// not turn it into an error: `pmat quality-gate --file a.md` reports the
/// `TODO:` on line 3 as a violation, while the MCP `quality_gate` tool answered
/// `-32603 Internal error: ... Markdown is not one` for the same file in the
/// same build — the surface contradiction `ensure_parseable` above was added to
/// remove, reintroduced from the other side. Such callers leave score/grade
/// unmeasured and let their language-agnostic checks give the verdict.
pub fn grades_source(path: &Path) -> bool {
    !matches!(
        Language::from_extension(path),
        Language::Unknown | Language::Yaml | Language::Markdown
    )
}

/// Does `source` parse as `language`, in THIS build?
///
/// `None` means either "it parsed" or "this build has no parser for that
/// language, so there is no verdict to give" — the two are deliberately not
/// distinguished here, because both leave the heuristics as the only signal and
/// neither is grounds for refusing the file. `Some(msg)` is a real syntax error.
fn source_parse_error(source: &str, language: Language) -> Option<String> {
    if language == Language::Rust {
        return syn::parse_file(source)
            .err()
            .map(|e| format!("not parseable as Rust ({e})"));
    }

    #[cfg(feature = "tree-sitter")]
    {
        let grammar: Option<tree_sitter::Language> = match language {
            #[cfg(feature = "python-ast")]
            Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
            #[cfg(feature = "typescript-ast")]
            Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            #[cfg(feature = "javascript-ast")]
            Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            #[cfg(feature = "c-ast")]
            Language::C => Some(tree_sitter_c::LANGUAGE.into()),
            #[cfg(feature = "cpp-ast")]
            Language::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            #[cfg(feature = "lua-ast")]
            Language::Lua => Some(tree_sitter_lua::LANGUAGE.into()),
            #[cfg(feature = "go-ast")]
            Language::Go => Some(tree_sitter_go::LANGUAGE.into()),
            _ => None,
        };
        if let Some(grammar) = grammar {
            let mut parser = tree_sitter::Parser::new();
            if parser.set_language(&grammar).is_err() {
                // No usable grammar ⇒ no verdict, not a failed one.
                return None;
            }
            let Some(tree) = parser.parse(source, None) else {
                return Some(format!("not parseable as {language}"));
            };
            if tree.root_node().has_error() {
                return Some(format!("not parseable as {language} (syntax error)"));
            }
        }
    }

    None
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

    /// The parse gate used to be Rust-only, so unparseable Python collected the
    /// untouched component caps from the line heuristics — a graded verdict for
    /// a file nothing had parsed.
    #[cfg(feature = "python-ast")]
    #[test]
    fn test_unparseable_python_is_not_graded() {
        let f = write_temp(".py", "def f(:\n  ???\n");
        let analyzer = TdgAnalyzer::new().expect("analyzer");
        let err = analyzer
            .analyze_file(f.path())
            .expect_err("Python that does not parse must not get a score");
        assert!(err.to_string().contains("not parseable as Python"), "{err}");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_valid_python_still_scores() {
        let f = write_temp(".py", "def add(a, b):\n    \"\"\"Adds.\"\"\"\n    return a + b\n");
        let analyzer = TdgAnalyzer::new().expect("analyzer");
        let score = analyzer.analyze_file(f.path()).expect("valid Python grades");
        assert_eq!(score.language, Language::Python);
        assert!(score.total > 0.0);
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_unparseable_typescript_is_not_graded() {
        let f = write_temp(".ts", "function f( { return ;;; }\n");
        let analyzer = TdgAnalyzer::new().expect("analyzer");
        let err = analyzer
            .analyze_file(f.path())
            .expect_err("TypeScript that does not parse must not get a score");
        assert!(
            err.to_string().contains("not parseable as TypeScript"),
            "{err}"
        );
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
