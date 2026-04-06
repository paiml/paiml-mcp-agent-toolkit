// Scorer trait impl for DuplicationDetector and CloneSet methods
// Included from duplication.rs — shares parent module scope (no `use` imports here)

impl Scorer for DuplicationDetector {
    fn score(&self, tree: &Tree, source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut points = config.weights.duplication;
        let root = tree.root_node();

        let sequences = self.extract_token_sequences(root, source);
        if sequences.is_empty() {
            return Ok(points);
        }

        let exact_clones = self.find_exact_clones(&sequences);
        let renamed_clones = self.find_renamed_clones(&sequences);
        let modified_clones = self.find_modified_clones(&sequences);

        let total_tokens = source.len();
        let duplicate_tokens =
            exact_clones.total_tokens() +
            (renamed_clones.total_tokens() as f32 * 0.8) as usize +
            (modified_clones.total_tokens() as f32 * 0.5) as usize;

        let duplication_ratio = duplicate_tokens as f32 / total_tokens.max(1) as f32;

        let penalty = (duplication_ratio * 40.0).min(20.0);
        if penalty > 0.0 {
            if let Some(applied) = tracker.apply(
                format!("duplication_{:.2}", duplication_ratio),
                MetricCategory::Duplication,
                penalty,
                format!("Code duplication: {:.1}%", duplication_ratio * 100.0)
            ) {
                points -= applied;
            }
        }

        Ok(points.max(0.0))
    }

    fn category(&self) -> MetricCategory {
        MetricCategory::Duplication
    }
}

impl CloneSet {
    fn new() -> Self {
        Self { clones: Vec::new() }
    }

    fn add_clone(&mut self, clone_type: CloneType, sequences: Vec<TokenSequence>) {
        debug_assert!(!sequences.is_empty(), "sequences must not be empty");
        self.clones.push((clone_type, sequences));
    }

    fn total_tokens(&self) -> usize {
        self.clones.iter()
            .map(|(_, sequences)| {
                sequences.iter()
                    .map(|seq| seq.tokens.len())
                    .sum::<usize>()
            })
            .sum()
    }
}
