// Scorer trait implementation for CouplingAnalyzer: computes coupling score
// using afferent/efferent coupling, instability, abstractness, and distance
// from the main sequence.

impl Scorer for CouplingAnalyzer {
    fn score(&self, tree: &Tree, source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        let mut points = config.weights.coupling;
        let root = tree.root_node();

        let afferent = self.calculate_afferent_coupling(root, source);
        let efferent = self.calculate_efferent_coupling(root, source);

        let instability = if afferent + efferent > 0 {
            efferent as f32 / (afferent + efferent) as f32
        } else {
            0.0
        };

        let abstractness = self.calculate_abstractness(root, source);
        let distance = (instability + abstractness - 1.0).abs();

        if afferent + efferent > config.thresholds.max_coupling {
            let excess = (afferent + efferent - config.thresholds.max_coupling) as f32;
            let penalty = config.penalties.coupling_penalty_curve.apply(excess * 0.3, 1.0).min(7.0);

            if let Some(applied) = tracker.apply(
                format!("high_coupling_{}", afferent + efferent),
                MetricCategory::Coupling,
                penalty,
                format!("High coupling: Ca={}, Ce={}", afferent, efferent)
            ) {
                points -= applied;
            }
        }

        let distance_penalty = (distance * 8.0).min(8.0);
        if distance_penalty > 0.5 {
            if let Some(applied) = tracker.apply(
                format!("main_sequence_distance_{:.2}", distance),
                MetricCategory::Coupling,
                distance_penalty,
                format!("Distance from main sequence: {:.2}", distance)
            ) {
                points -= applied;
            }
        }

        Ok(points.max(0.0))
    }

    fn category(&self) -> MetricCategory {
        MetricCategory::Coupling
    }
}
