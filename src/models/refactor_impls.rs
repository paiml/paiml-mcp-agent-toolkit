impl Default for RefactorConfig {
    fn default() -> Self {
        Self {
            target_complexity: 20,
            remove_satd: true,
            max_function_lines: 50,
            thresholds: Thresholds::default(),
            strategies: RefactorStrategies::default(),
            parallel_workers: 4,
            memory_limit_mb: 512,
            batch_size: 10,
            priority_expression: None,
            auto_commit_template: None,
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cyclomatic_warn: 10,
            cyclomatic_error: 20,
            cognitive_warn: 15,
            cognitive_error: 30,
            tdg_warn: 1.5,
            tdg_error: 2.0,
        }
    }
}

impl Default for RefactorStrategies {
    fn default() -> Self {
        Self {
            prefer_functional: true,
            use_early_returns: true,
            extract_helpers: true,
        }
    }
}

impl RefactorStateMachine {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(targets: Vec<PathBuf>, config: RefactorConfig) -> Self {
        let initial_state = if targets.is_empty() {
            State::Complete {
                summary: Summary::default(),
            }
        } else {
            State::Scan {
                targets: targets.clone(),
            }
        };

        Self {
            current: initial_state,
            history: Vec::new(),
            config,
            targets,
            current_target_index: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Advance.
    pub fn advance(&mut self) -> Result<&State, String> {
        let next = match &self.current {
            State::Scan { targets } => {
                if targets.is_empty() {
                    State::Complete {
                        summary: self.session_summary(),
                    }
                } else {
                    State::Analyze {
                        current: Self::file_id(&targets[0]),
                    }
                }
            }
            State::Analyze { current } => State::Plan {
                violations: self.find_violations(current),
            },
            State::Plan { violations } => {
                if violations.is_empty() {
                    let summary = self.session_summary();
                    self.next_target()
                        .map_or(State::Complete { summary }, |t| State::Analyze { current: t })
                } else {
                    State::Refactor {
                        operation: violations[0].suggested_fix.clone().unwrap_or(
                            RefactorOp::SimplifyExpression {
                                expr: "complex".to_string(),
                                simplified: "simple".to_string(),
                            },
                        ),
                    }
                }
            }
            State::Refactor { .. } => State::Test {
                command: "make test-fast".to_string(),
            },
            State::Test { .. } => State::Lint { strict: true },
            State::Lint { .. } => State::Emit {
                payload: self.compute_payload(),
            },
            State::Emit { .. } => State::Checkpoint {
                reason: "cycle_complete".to_string(),
            },
            State::Checkpoint { .. } => {
                let summary = self.session_summary();
                self.next_target()
                    .map_or(State::Complete { summary }, |t| State::Analyze { current: t })
            }
            State::Complete { .. } => {
                return Ok(&self.current);
            }
        };

        self.transition_to(next)
    }

    fn transition_to(&mut self, new_state: State) -> Result<&State, String> {
        let transition = StateTransition {
            from: self.current.clone(),
            to: new_state.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("internal error")
                .as_secs(),
            metrics_before: MetricSet::default(),
            metrics_after: None,
            applied_refactor: None,
        };

        self.history.push(transition);
        self.current = new_state;
        Ok(&self.current)
    }

    fn find_violations(&self, file_id: &FileId) -> Vec<Violation> {
        // Check thresholds and create violations
        let mut violations = Vec::new();

        // Simulate finding a high complexity violation
        if file_id.path.to_string_lossy().contains("complex") {
            violations.push(Violation {
                violation_type: ViolationType::HighComplexity,
                location: Location {
                    file: file_id.path.clone(),
                    line: 100,
                    column: 1,
                },
                severity: Severity::High,
                description: "Function exceeds complexity threshold".to_string(),
                suggested_fix: Some(RefactorOp::ExtractFunction {
                    name: "extract_helper".to_string(),
                    start: BytePos {
                        byte: 1000,
                        line: 100,
                        column: 1,
                    },
                    end: BytePos {
                        byte: 2000,
                        line: 150,
                        column: 1,
                    },
                    params: vec!["param1".to_string()],
                }),
            });
        }

        violations
    }

    fn next_target(&mut self) -> Option<FileId> {
        self.current_target_index += 1;
        if self.current_target_index < self.targets.len() {
            Some(Self::file_id(&self.targets[self.current_target_index]))
        } else {
            None
        }
    }

    /// Identify a target by its *contents*.
    ///
    /// `FileId.hash` was the literal `0` with the comment "Will be computed
    /// during analysis" — nothing ever computed it, so `refactor.nextIteration`
    /// reported `{"path": "…", "hash": 0}` for every file, including files that
    /// did not exist. An unreadable file keeps hash 0: that is "not measured",
    /// not "empty".
    fn file_id(path: &std::path::Path) -> FileId {
        let hash = std::fs::read(path).map_or(0, |bytes| {
            let digest = blake3::hash(&bytes);
            let mut first8 = [0u8; 8];
            first8.copy_from_slice(&digest.as_bytes()[..8]);
            u64::from_le_bytes(first8)
        });
        FileId {
            path: path.to_path_buf(),
            hash,
        }
    }

    /// Summarise the session from what actually happened.
    ///
    /// Every `State::Complete` used to carry `Summary::default()` — a struct
    /// of hardcoded zeros — so a session that scanned and analysed a real file
    /// still reported `files_processed: 0` and `total_time: 0s`. The counts
    /// below are derived from the recorded transitions. `refactors_applied`,
    /// `complexity_reduction` and `satd_removed` stay zero because this state
    /// machine applies nothing (the tools disclose the simulated analysis
    /// engine in their MCP descriptions); reporting anything else would be
    /// inventing work that never happened.
    fn session_summary(&self) -> Summary {
        let files_processed = self
            .history
            .iter()
            .filter(|t| matches!(t.to, State::Analyze { .. }))
            .count() as u32;

        let total_time = self
            .history
            .first()
            .and_then(|first| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|now| Duration::from_secs(now.as_secs().saturating_sub(first.timestamp)))
            })
            .unwrap_or_default();

        Summary {
            files_processed,
            refactors_applied: 0,
            complexity_reduction: 0.0,
            satd_removed: 0,
            total_time,
        }
    }

    fn compute_payload(&self) -> DefectPayload {
        DefectPayload {
            file_hash: 0,
            tdg_score: 1.0,
            complexity: (10, 15),
            dead_symbols: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("internal error")
                .as_secs(),
            severity_flags: 0,
            refactor_available: true,
            refactor_type: RefactorType::None,
            estimated_improvement: 0.5,
            _padding: [0; 2],
        }
    }
}

#[cfg(test)]
mod session_summary_tests {
    //! The Complete summary must describe the session that ran, not a
    //! `Summary::default()` of hardcoded zeros.
    //!
    //! #975: `completing_a_one_file_session_reports_that_file` used
    //! `let State::X { .. } = .. else { panic!(..) }`. Those `else` arms only
    //! execute when the test *fails*, so four of its lines were permanently
    //! uncovered and no passing run could ever close the gap. The state is
    //! projected through the two helpers below instead, and a dedicated test
    //! exercises their rejecting arm — so every line here is reachable by a
    //! green run.
    use super::*;

    /// `Some` iff the machine is currently analysing a file.
    fn analysed_file(state: &State) -> Option<&FileId> {
        match state {
            State::Analyze { current } => Some(current),
            _ => None,
        }
    }

    /// `Some` iff the session has finished.
    fn finished_summary(state: &State) -> Option<&Summary> {
        match state {
            State::Complete { summary } => Some(summary),
            _ => None,
        }
    }

    /// Advance until the session completes, or give up after `budget` steps.
    fn run_to_completion(sm: &mut RefactorStateMachine, budget: usize) -> Summary {
        for _ in 0..budget {
            if let Some(summary) = finished_summary(&sm.current) {
                return summary.clone();
            }
            sm.advance().expect("advance must not fail");
        }
        panic!("session did not complete within {budget} transitions");
    }

    #[test]
    fn the_state_projections_reject_states_they_do_not_describe() {
        // A fresh multi-target machine sits in Scan, which is neither Analyze
        // nor Complete. This covers the rejecting arm of both helpers.
        let sm =
            RefactorStateMachine::new(vec![PathBuf::from("src/lib.rs")], RefactorConfig::default());
        assert!(matches!(sm.current, State::Scan { .. }));
        assert!(analysed_file(&sm.current).is_none());
        assert!(finished_summary(&sm.current).is_none());
    }

    #[test]
    fn completing_a_one_file_session_reports_that_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "pub fn f() -> i32 { 1 }\n").expect("write fixture");

        let mut sm = RefactorStateMachine::new(vec![file.clone()], RefactorConfig::default());
        // Scan -> Analyze
        sm.advance().expect("scan");
        let current = analysed_file(&sm.current).expect("Scan must advance to Analyze");
        assert_eq!(
            current.path, file,
            "the analysed target must be the file we handed in"
        );
        assert_ne!(
            current.hash, 0,
            "the analysed file's id must be a real content hash"
        );

        sm.advance().expect("analyze"); // -> Plan (no violations)
        sm.advance().expect("plan"); // -> Complete
        let summary = finished_summary(&sm.current).expect("an empty plan must complete");
        assert_eq!(
            summary.files_processed, 1,
            "one file was scanned and analysed"
        );
    }

    #[test]
    fn a_two_file_session_reports_both_files() {
        // Guards against `files_processed` being pinned to 0 or 1: the count
        // has to track how many files the run actually reached Analyze on.
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "pub fn a() {}\n").expect("write a");
        std::fs::write(&b, "pub fn b() {}\n").expect("write b");

        let mut sm = RefactorStateMachine::new(vec![a, b], RefactorConfig::default());
        let summary = run_to_completion(&mut sm, 16);
        assert_eq!(
            summary.files_processed, 2,
            "both targets were analysed, so both must be counted"
        );
    }

    #[test]
    fn a_session_with_no_targets_reports_nothing_processed() {
        let mut sm = RefactorStateMachine::new(vec![], RefactorConfig::default());
        let summary = finished_summary(&sm.current)
            .expect("an empty target list completes immediately")
            .clone();
        assert_eq!(summary.files_processed, 0);
        assert_eq!(summary.total_time, Duration::from_secs(0));
        // And Complete is terminal: no further transition is recorded.
        sm.advance().expect("advance on Complete");
        assert!(sm.history.is_empty());
    }

    #[test]
    fn a_files_contents_decide_its_hash() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "pub fn a() {}\n").expect("write a");
        std::fs::write(&b, "pub fn b() {}\n").expect("write b");
        assert_ne!(
            RefactorStateMachine::file_id(&a).hash,
            RefactorStateMachine::file_id(&b).hash
        );
        // A path that cannot be read is "not measured", i.e. still 0.
        assert_eq!(
            RefactorStateMachine::file_id(&dir.path().join("missing.rs")).hash,
            0
        );
    }
}

impl Default for MetricSet {
    fn default() -> Self {
        Self {
            complexity: (0, 0),
            tdg_score: 0.0,
            dead_code: Vec::new(),
            satd_count: 0,
            provability: 0.0,
        }
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            files_processed: 0,
            refactors_applied: 0,
            complexity_reduction: 0.0,
            satd_removed: 0,
            total_time: Duration::from_secs(0),
        }
    }
}

impl Violation {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// To op.
    pub fn to_op(&self) -> RefactorOp {
        self.suggested_fix
            .clone()
            .unwrap_or_else(|| match self.violation_type {
                ViolationType::HighComplexity => RefactorOp::ExtractFunction {
                    name: "extracted_function".to_string(),
                    start: BytePos {
                        byte: 0,
                        line: self.location.line,
                        column: self.location.column,
                    },
                    end: BytePos {
                        byte: 100,
                        line: self.location.line + 10,
                        column: 0,
                    },
                    params: vec![],
                },
                ViolationType::DeepNesting => RefactorOp::FlattenNesting {
                    function: "function_name".to_string(),
                    strategy: NestingStrategy::EarlyReturn,
                },
                ViolationType::SelfAdmittedTechDebt => RefactorOp::RemoveSatd {
                    location: self.location.clone(),
                    fix: SatdFix::Remove,
                },
                _ => RefactorOp::SimplifyExpression {
                    expr: "complex".to_string(),
                    simplified: "simple".to_string(),
                },
            })
    }
}
