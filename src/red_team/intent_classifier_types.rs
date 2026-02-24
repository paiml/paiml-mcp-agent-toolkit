#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommitIntent {
    HallucinationFix, // Fixing a false claim from previous commit
    PlannedIteration, // Expected follow-up work
    Uncertain,        // Cannot determine with confidence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub message: String,
    pub timestamp_seconds: i64,
    pub modified_files: Vec<String>,
    pub issue_number: Option<u32>,
    pub issue_created_timestamp: Option<i64>,
    pub branch: String,
    pub test_changes: TestChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestChanges {
    pub added_tests: usize,
    pub fixed_tests: usize,
    pub modified_test_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub intent: CommitIntent,
    pub confidence: f64, // 0.0 to 1.0
    pub signals: Vec<SignalResult>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    pub signal_name: String,
    pub vote: CommitIntent,
    pub confidence: f64,
    pub evidence: String,
}

pub struct IntentClassifier {
    hallucination_keywords: Vec<String>,
    iteration_keywords: Vec<String>,
    grace_period_hours: i64,
    code_overlap_threshold: f64,
}
