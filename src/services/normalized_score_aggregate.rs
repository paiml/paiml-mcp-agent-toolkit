// Aggregate scoring and clone trait support for NormalizedScore.
// Included by normalized_score.rs — no `use` imports or `#!` attributes.

/// Helper trait for cloning boxed NormalizedScore.
pub trait NormalizedScoreClone: NormalizedScore {
    fn clone_box(&self) -> Box<dyn NormalizedScoreClone>;
}

impl<T: NormalizedScore + Clone + 'static> NormalizedScoreClone for T {
    fn clone_box(&self) -> Box<dyn NormalizedScoreClone> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NormalizedScoreClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl fmt::Debug for dyn NormalizedScoreClone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NormalizedScore({:.1})", self.normalized())
    }
}

/// Aggregate multiple scores into a weighted normalized score.
#[derive(Debug, Clone)]
pub struct AggregateScore {
    components: Vec<(Box<dyn NormalizedScoreClone>, f64)>, // (score, weight)
    name: String,
}

impl AggregateScore {
    /// Create a new aggregate score.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            components: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a component score with weight.
    pub fn add<S: NormalizedScoreClone + 'static>(&mut self, score: S, weight: f64) {
        self.components.push((Box::new(score), weight.max(0.0)));
    }

    /// Get the total weight.
    pub fn total_weight(&self) -> f64 {
        self.components.iter().map(|(_, w)| w).sum()
    }
}

impl NormalizedScore for AggregateScore {
    fn raw(&self) -> f64 {
        let total_weight = self.total_weight();
        if total_weight <= 0.0 {
            return 0.0;
        }
        self.components
            .iter()
            .map(|(score, weight)| score.normalized() * weight)
            .sum::<f64>()
            / total_weight
    }

    fn max_raw(&self) -> f64 {
        100.0 // Aggregates are already normalized
    }
}

impl fmt::Display for AggregateScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:.1}/100 ({})",
            self.name,
            self.normalized(),
            self.grade()
        )
    }
}
