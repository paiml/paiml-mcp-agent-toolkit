// Deep context configuration, SATD filtering, analysis type parsing,
// and DAG/cache strategy conversion helpers.

#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn apply_satd_filters(
    items: Vec<crate::models::tdg::SatdItem>,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> Vec<crate::models::tdg::SatdItem> {
    items
        .into_iter()
        .filter(|item| {
            if critical_only {
                matches!(item.severity, crate::models::tdg::SatdSeverity::Critical)
            } else if let Some(min_severity) = &severity {
                // Convert to comparable severity levels
                let item_level = match item.severity {
                    crate::models::tdg::SatdSeverity::Low => 1,
                    crate::models::tdg::SatdSeverity::Medium => 2,
                    crate::models::tdg::SatdSeverity::High => 3,
                    crate::models::tdg::SatdSeverity::Critical => 4,
                };
                let min_level = match min_severity {
                    SatdSeverity::Low => 1,
                    SatdSeverity::Medium => 2,
                    SatdSeverity::High => 3,
                    SatdSeverity::Critical => 4,
                };
                item_level >= min_level
            } else {
                true
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct DeepContextConfigParams {
    pub project_path: PathBuf,
    pub output: Option<PathBuf>,
    pub format: DeepContextOutputFormat,
    pub full: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub period_days: u32,
    pub dag_type: DeepContextDagType,
    pub max_depth: Option<usize>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub cache_strategy: DeepContextCacheStrategy,
    pub parallel: Option<usize>,
    pub verbose: bool,
}

#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn build_deep_context_config(
    _project_path: PathBuf,
    _output: Option<PathBuf>,
    _format: DeepContextOutputFormat,
    _full: bool,
    _include: Vec<String>,
    _exclude: Vec<String>,
    _period_days: u32,
    _dag_type: DeepContextDagType,
    _max_depth: Option<usize>,
    _include_patterns: Vec<String>,
    _exclude_patterns: Vec<String>,
    _cache_strategy: DeepContextCacheStrategy,
    _parallel: Option<usize>,
    _verbose: bool,
) -> anyhow::Result<crate::models::deep_context_config::DeepContextConfig> {
    // Create DeepContextConfig with default values
    Ok(crate::models::deep_context_config::DeepContextConfig {
        entry_points: vec![],
        dead_code_threshold: 0.15,
        complexity_thresholds: Default::default(),
        include_tests: false,
        include_benches: false,
        cross_language_detection: true,
    })
}

/// Converts CLI DAG type to internal model DAG type
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::{convert_dag_type, DeepContextDagType};
/// use pmat::models::dag::DagType;
///
/// let cli_type = DeepContextDagType::CallGraph;
/// let model_type = convert_dag_type(cli_type);
/// assert!(matches!(model_type, DagType::CallGraph));
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn convert_dag_type(dag_type: DeepContextDagType) -> crate::models::dag::DagType {
    match dag_type {
        DeepContextDagType::CallGraph => crate::models::dag::DagType::CallGraph,
        DeepContextDagType::ImportGraph => crate::models::dag::DagType::ImportGraph,
        DeepContextDagType::Inheritance => crate::models::dag::DagType::Inheritance,
        DeepContextDagType::FullDependency => crate::models::dag::DagType::FullDependency,
    }
}

/// Converts cache strategy (currently a pass-through)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::{convert_cache_strategy, DeepContextCacheStrategy};
///
/// let strategy = DeepContextCacheStrategy::Normal;
/// let converted = convert_cache_strategy(strategy);
///
/// // Currently returns the same strategy
/// assert_eq!(converted, DeepContextCacheStrategy::Normal);
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn convert_cache_strategy(strategy: DeepContextCacheStrategy) -> DeepContextCacheStrategy {
    // Return the strategy unchanged as it's already in the correct format
    strategy
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn parse_analysis_filters(
    include: Vec<String>,
    exclude: Vec<String>,
) -> anyhow::Result<(Vec<AnalysisType>, Vec<AnalysisType>)> {
    let include_analysis = include
        .into_iter()
        .map(|s| parse_analysis_type(&s))
        .collect::<Result<Vec<_>, _>>()?;

    let exclude_analysis = exclude
        .into_iter()
        .map(|s| parse_analysis_type(&s))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((include_analysis, exclude_analysis))
}

/// Parses a string into an `AnalysisType` enum
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::{parse_analysis_type, AnalysisType};
///
/// assert_eq!(parse_analysis_type("complexity").unwrap(), AnalysisType::Complexity);
/// assert_eq!(parse_analysis_type("tdg").unwrap(), AnalysisType::TechnicalDebt);
/// assert_eq!(parse_analysis_type("big-o").unwrap(), AnalysisType::BigO);
/// assert!(parse_analysis_type("invalid").is_err());
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn parse_analysis_type(s: &str) -> anyhow::Result<AnalysisType> {
    match s.to_lowercase().as_str() {
        "complexity" => Ok(AnalysisType::Complexity),
        "dead-code" | "deadcode" => Ok(AnalysisType::DeadCode),
        "duplication" | "duplicates" => Ok(AnalysisType::Duplication),
        "technical-debt" | "tdg" => Ok(AnalysisType::TechnicalDebt),
        "big-o" | "bigo" => Ok(AnalysisType::BigO),
        "all" => Ok(AnalysisType::All),
        _ => anyhow::bail!("Unknown analysis type: {s}"),
    }
}
