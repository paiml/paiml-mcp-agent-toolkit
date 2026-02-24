pub struct DemoRunner {
    server: Arc<StatelessTemplateServer>,
    execution_log: Vec<DemoStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStep {
    pub name: String,
    pub capability: &'static str,
    pub request: McpRequest,
    pub response: McpResponse,
    pub elapsed_ms: u64,
    pub success: bool,
    pub output: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct DemoReport {
    pub repository: String,
    pub total_time_ms: u64,
    pub steps: Vec<DemoStep>,
    pub system_diagram: Option<String>,
    pub analysis: DemoAnalysisResult,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct DemoAnalysisResult {
    pub files_analyzed: usize,
    pub functions_analyzed: usize,
    pub avg_complexity: f64,
    pub hotspot_functions: usize,
    pub quality_score: f64,
    pub tech_debt_hours: u32,
    pub qa_verification: Option<String>,
    pub language_stats: Option<HashMap<String, Value>>,
    pub complexity_metrics: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Component {
    id: String,
    label: String,
    color: String,
    connections: Vec<(String, String)>,
}
