/// Oracle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub max_iterations: usize,
    pub min_progress_per_iteration: f32,
    pub stagnation_threshold: usize,
    pub andon_enabled: bool,
    pub require_human_approval_above: Option<usize>,
    pub auto_apply_threshold: f32,
    pub review_threshold: f32,
    pub batch_size: usize,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            min_progress_per_iteration: 0.001,
            stagnation_threshold: 5,
            andon_enabled: true,
            require_human_approval_above: Some(10),
            auto_apply_threshold: 0.9,
            review_threshold: 0.7,
            batch_size: 10,
        }
    }
}
