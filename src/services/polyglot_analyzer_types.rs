#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub file_count: usize,
    pub line_count: usize,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotAnalysis {
    pub languages: Vec<LanguageStats>,
    pub cross_language_dependencies: Vec<CrossLanguageDependency>,
    pub architecture_pattern: Option<ArchitecturePattern>,
    pub integration_points: Vec<IntegrationPoint>,
    pub recommendation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub file_count: usize,
    pub line_count: usize,
    pub complexity_score: f64,
    pub test_coverage: f64,
    pub primary_frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLanguageDependency {
    pub from_language: String,
    pub to_language: String,
    pub dependency_type: DependencyType,
    pub coupling_strength: f64,
    pub files_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    FFI,
    ProcessCommunication,
    SharedDataStructure,
    ConfigurationFile,
    BuildSystem,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturePattern {
    Microservices,
    Monolithic,
    LayeredArchitecture,
    EventDriven,
    PluginArchitecture,
    ClientServer,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPoint {
    pub name: String,
    pub languages: Vec<String>,
    pub integration_type: IntegrationType,
    pub risk_level: RiskLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    API,
    Database,
    FileSystem,
    Memory,
    Network,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct PolyglotAnalyzer {
    language_patterns: HashMap<String, LanguagePattern>,
    architecture_signatures: Vec<ArchitectureSignature>,
}

#[derive(Debug, Clone)]
struct LanguagePattern {
    file_extensions: Vec<String>,
    _build_files: Vec<String>,
    _config_files: Vec<String>,
    _dependency_files: Vec<String>,
}

#[derive(Debug, Clone)]
struct ArchitectureSignature {
    pattern: ArchitecturePattern,
    _indicators: Vec<String>,
    required_languages: usize,
    confidence_threshold: f64,
}

#[derive(Debug, Clone)]
struct ArchitectureIndicators {
    has_microservice_indicators: bool,
    has_layered_indicators: bool,
    has_event_indicators: bool,
    has_plugin_indicators: bool,
    directory_structure: Vec<String>,
    config_files: Vec<String>,
}
