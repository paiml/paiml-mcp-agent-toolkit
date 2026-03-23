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

        for file in files {
            match self.analyze_file(&file) {
                Ok(score) => scores.push(score),
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
