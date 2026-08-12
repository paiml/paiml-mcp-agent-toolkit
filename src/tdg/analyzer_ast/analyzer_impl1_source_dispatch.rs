impl TdgAnalyzerAst {
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

        // Use proper AST-based analysis
        match language {
            Language::Rust => self.analyze_rust_ast(source, &mut score, &mut tracker)?,
            Language::Python => self.analyze_python_ast(source, &mut score, &mut tracker)?,
            Language::JavaScript | Language::TypeScript => {
                self.analyze_javascript_ast(source, &mut score, &mut tracker)?;
            }
            Language::Go => self.analyze_go_ast(source, &mut score, &mut tracker)?,
            Language::Java => self.analyze_java_ast(source, &mut score, &mut tracker)?,
            Language::C | Language::Cpp => self.analyze_c_ast(source, &mut score, &mut tracker)?,
            Language::Ruchy => self.analyze_ruchy_ast(source, &mut score, &mut tracker)?,
            Language::Lua => self.analyze_lua_ast(source, &mut score, &mut tracker)?,
            Language::Sql => self.analyze_sql_heuristic(source, &mut score, &mut tracker)?,
            Language::Scala => self.analyze_scala_heuristic(source, &mut score, &mut tracker)?,
            Language::Yaml => self.analyze_yaml_heuristic(source, &mut score, &mut tracker)?,
            Language::Lean => {
                self.analyze_lean_heuristic(source, &mut score, &mut tracker)?;
            }
            Language::Markdown => {
                self.analyze_markdown_heuristic(source, &mut score, &mut tracker)?;
            }
            _ => {
                // Fallback to heuristics for unsupported languages
                // but with reduced confidence
                score.confidence *= 0.5;
                self.analyze_heuristic(source, &mut score, &mut tracker)?;
            }
        }

        score.penalties_applied = tracker.get_attributions();

        // Known Defects v2.1: Detect critical defects for auto-fail
        if let Some(ref path) = score.file_path {
            let defects = match language {
                Language::Rust => {
                    let detector = RustDefectDetector::new();
                    detector.detect(source, path)
                }
                Language::Lua => {
                    let detector = LuaDefectDetector::new();
                    detector.detect(source, path)
                }
                _ => Vec::new(),
            };

            let critical_count: usize = defects
                .iter()
                .filter(|d| d.severity == DefectSeverity::Critical)
                .map(|d| d.instances.len())
                .sum();

            // Lean-specific: sorry = critical defect (proof incompleteness)
            let lean_sorry_count = if language == Language::Lean {
                count_lean_sorry_ast(source)
            } else {
                0
            };

            score.critical_defects_count = critical_count + lean_sorry_count;
            score.has_critical_defects = score.critical_defects_count > 0;

            // Issue #279: a file with no git history must not be auto-failed by a
            // gate it cannot pass until it is committed. That exemption is real,
            // but it used to be expressed by clearing `has_critical_defects` while
            // leaving `critical_defects_count` set — so the record said "1 critical
            // defect" and "no critical defects" at the same time, and the same
            // bytes scored 0.0/F inside a repo and 99.5/A+ outside one, because
            // `is_file_git_tracked` is also false when there is no repository at
            // all. Worse, the pair is written into `.pmat/baseline.json`, so a
            // baseline captured before `git add` recorded the clean answer
            // permanently (#919).
            //
            // The exemption now names itself instead of contradicting the count.
            if score.has_critical_defects && is_exempt_as_new_file(path) {
                score.critical_defects_suppressed = Some(
                    "file is not tracked by git; critical-defect auto-fail is not applied \
                     to code with no history (#279)"
                        .to_string(),
                );
            }
        }

        score.calculate_total();

        Ok(score)
    }

    fn analyze_rust_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "rust-ast")]
        {
            use syn::{visit::Visit, File};

            let ast = syn::parse_str::<File>(source)?;

            let mut visitor = RustComplexityVisitor::new();
            visitor.visit_file(&ast);

            // Calculate structural complexity based on AST
            let cyclomatic = visitor.cyclomatic_complexity;
            let cognitive = visitor.cognitive_complexity;

            score.structural_complexity = self.score_structural_complexity(
                cyclomatic,
                cognitive,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            // Calculate semantic complexity
            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.generic_count,
                visitor.abstraction_levels,
                tracker,
            );

            // Calculate duplication (requires AST comparison)
            score.duplication_ratio = self.analyze_duplication_ast(source, Language::Rust, tracker);

            // Calculate coupling
            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                visitor.interface_implementations,
                tracker,
            );

            // Calculate documentation coverage
            score.doc_coverage = self.score_documentation(
                visitor.documented_items,
                visitor.total_public_items,
                visitor.comment_lines,
                visitor.total_lines,
                tracker,
            );

            // Calculate consistency
            score.consistency_score = self.score_consistency_rust(&ast, tracker);

            // Calculate entropy - pattern analysis for code quality
            score.entropy_score = self.score_entropy_analysis(source, Language::Rust, tracker);
        }
        #[cfg(not(feature = "rust-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_python_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "python-ast")]
        {
            // Modern tree-sitter-python parsing (replaces rustpython-parser)
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Python language: {e}"))?;

            let tree = parser
                .parse(source, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse Python code"))?;

            let mut visitor = PythonComplexityVisitor::new(source);
            visitor.analyze_tree(&tree);

            score.structural_complexity = self.score_structural_complexity(
                visitor.cyclomatic_complexity,
                visitor.cognitive_complexity,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.decorator_count,
                visitor.metaclass_count,
                tracker,
            );

            score.duplication_ratio =
                self.analyze_duplication_ast(source, Language::Python, tracker);

            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                0, // Python doesn't have explicit interfaces
                tracker,
            );

            score.doc_coverage = self.score_documentation(
                visitor.documented_functions,
                visitor.total_functions,
                visitor.docstring_lines,
                visitor.total_lines,
                tracker,
            );

            score.consistency_score = self.score_consistency_python(source, tracker);

            score.entropy_score = self.score_entropy_analysis(source, Language::Python, tracker);
        }
        #[cfg(not(feature = "python-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }
}

/// Whether the #279 auto-fail exemption applies to this file.
///
/// #279 exempts a file that has no git history yet, because a gate that blocks
/// the commit is a gate the file cannot pass. That reasoning presupposes a
/// repository — the file is *about to* gain history. It says nothing about code
/// that is simply not under version control at all (an unpacked tarball, a
/// vendored tree, a scratch directory), where there is no commit to be blocked
/// and so nothing to exempt.
///
/// The old predicate collapsed those two cases into one `false`, so analysing
/// any code outside a repository silently waived the Known-Defects gate — the
/// same bytes scoring 0.0/F inside a repo and 100.0/A+ outside it (#919).
///
/// The git query MUST be rooted at the ANALYSED FILE, never at the process
/// working directory. Before this was fixed the command was plain
/// `git log --oneline -1 -- <path>`, which git resolves against the CWD: run
/// from anywhere outside the analysed repository git exits 128 ("not a git
/// repository"), the committed file was read as untracked, and
/// `has_critical_defects` was silently cleared. Observed on one unchanged
/// fixture (round 3): `cd <repo> && pmat analyze tdg -p .` scored 0.0 / grade F
/// while `cd /tmp && pmat analyze tdg -p <repo>` scored 100.0 / grade A+ — a
/// 5-band grade swing produced by nothing but the caller's CWD, which also made
/// the Known-Defects auto-fail unreachable from the normal CI invocation form.
pub(crate) fn is_exempt_as_new_file(path: &Path) -> bool {
    matches!(git_tracking_status(path), GitTracking::UntrackedInRepo)
}

/// Where a file stands with git, keeping "no repository" distinct from
/// "in a repository but not yet committed" — see [`is_exempt_as_new_file`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitTracking {
    /// Has at least one commit.
    Tracked,
    /// Inside a work tree, but with no history yet — the #279 case.
    UntrackedInRepo,
    /// Not under version control, or git is unavailable. Not a #279 case: the
    /// gate applies exactly as it would to committed code.
    NotVersioned,
}

pub(crate) fn git_tracking_status(path: &Path) -> GitTracking {
    let Some(repo_anchor) = git_anchor_for(path) else {
        return GitTracking::NotVersioned;
    };
    let absolute = absolute_path(path);

    let run = |args: &[&str], file: Option<&Path>| {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("-C").arg(&repo_anchor).args(args);
        if let Some(f) = file {
            cmd.arg("--").arg(f);
        }
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
    };

    // Is there a work tree at all? If git is missing or errors, treat the code
    // as unversioned rather than exempt — an exemption must be established, not
    // assumed, or a broken git install silently disables the gate.
    let inside_work_tree = run(&["rev-parse", "--is-inside-work-tree"], None)
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);
    if !inside_work_tree {
        return GitTracking::NotVersioned;
    }

    let has_history = run(&["log", "--oneline", "-1"], Some(&absolute))
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    if has_history {
        GitTracking::Tracked
    } else {
        GitTracking::UntrackedInRepo
    }
}

fn absolute_path(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
    })
}

/// The directory to run git from: the file's own directory, so the query finds
/// the file's repository rather than the process working directory's.
fn git_anchor_for(path: &Path) -> Option<std::path::PathBuf> {
    let absolute = absolute_path(path);
    if absolute.is_dir() {
        return Some(absolute);
    }
    match absolute.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => Some(parent.to_path_buf()),
        _ => Some(absolute),
    }
}

/// Retained for the original call shape; see [`git_tracking_status`].
#[cfg(test)]
fn is_file_git_tracked(path: &Path) -> bool {
    matches!(git_tracking_status(path), GitTracking::Tracked)
}

