// Parameter structures for each operation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityParams {
    pub file_path: Option<String>,
    pub max_cyclomatic: Option<u32>,
    pub max_cognitive: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdParams {
    pub file_path: Option<String>,
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeParams {
    pub file_path: Option<String>,
    pub include_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextParams {
    pub file_path: Option<String>,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateParams {
    pub file_path: Option<String>,
    pub fail_on_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProxyParams {
    pub file_path: String,
    pub content: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStartParams {
    pub file_path: String,
    pub target_complexity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorNextParams {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStopParams {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParams {
    pub name: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentParams {
    pub name: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtParams {
    pub requirement: String,
    pub granularity: String,
    pub seed: Option<u64>,
}
