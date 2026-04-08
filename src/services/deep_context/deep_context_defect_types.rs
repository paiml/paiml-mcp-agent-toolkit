// Deep context defect types - annotations, dead code, technical debt, complexity violations
// Included from mod.rs - shares parent module scope (no `use` imports)

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defect annotations.
pub struct DefectAnnotations {
    pub dead_code: Option<DeadCodeAnnotation>,
    pub technical_debt: Vec<TechnicalDebtItem>,
    pub complexity_violations: Vec<ComplexityViolation>,
    pub tdg_score: Option<TDGScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Dead code annotation.
pub struct DeadCodeAnnotation {
    pub confidence: ConfidenceLevel,
    pub reason: String,
    pub items: Vec<DeadCodeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Level classification for confidence.
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Dead code item.
pub struct DeadCodeItem {
    pub name: String,
    pub item_type: DeadCodeItemType,
    pub line: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Type classification for dead code item.
pub enum DeadCodeItemType {
    Function,
    Class,
    Module,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Technical debt item.
pub struct TechnicalDebtItem {
    pub category: TechnicalDebtCategory,
    pub severity: TechnicalDebtSeverity,
    pub content: String,
    pub line: u32,
    pub age_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Category classification for technical debt.
pub enum TechnicalDebtCategory {
    Design,
    Requirements,
    Implementation,
    Testing,
    Documentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Severity level classification for technical debt.
pub enum TechnicalDebtSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Violation record for complexity.
pub struct ComplexityViolation {
    pub metric_type: ComplexityMetricType,
    pub actual_value: u32,
    pub threshold: u32,
    pub function_name: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Type classification for complexity metric.
pub enum ComplexityMetricType {
    Cyclomatic,
    Cognitive,
    Halstead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Cross lang reference.
pub struct CrossLangReference {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub reference_type: CrossLangReferenceType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Type classification for cross lang reference.
pub enum CrossLangReferenceType {
    WasmBinding,
    FfiCall,
    PythonBinding,
    TypeDefinition,
}
