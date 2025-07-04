use crate::models::defect_report::{Defect, DefectCategory, Severity};
use crate::models::tdg::{TDGScore, TDGSeverity};
use crate::services::{
    big_o_analyzer::FunctionComplexity,
    dead_code_analyzer::{DeadCodeItem, UnreachableBlock},
    defect_analyzer::{AnalyzerConfig, DefectAnalyzer},
    duplicate_detector::{CloneGroup, CloneType},
    satd_detector::{DebtCategory, TechnicalDebt},
};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Helper function to discover source files
async fn discover_source_files(path: &Path) -> Result<Vec<PathBuf>> {
    use walkdir::WalkDir;

    let mut files = Vec::new();
    let extensions = ["rs", "js", "ts", "py", "java", "cpp", "c", "go"];

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Adapter for complexity analysis (TDG-based)
pub struct ComplexityDefectAnalyzer;

#[async_trait]
impl DefectAnalyzer for ComplexityDefectAnalyzer {
    type Config = ComplexityConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();
        let tdg_calculator = crate::services::tdg_calculator::TDGCalculator::new();

        // Analyze all source files
        let files = discover_source_files(project_path).await?;
        let scores = tdg_calculator.calculate_batch(files.clone()).await?;

        let mut index = 0;
        for (file_path, score) in files.into_iter().zip(scores.into_iter()) {
            if score.value > config.max_tdg_score {
                index += 1;
                defects.push(self.tdg_score_to_defect(file_path, score, index, &config));
            }
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Complexity
    }

    fn supports_incremental(&self) -> bool {
        true
    }
}

impl ComplexityDefectAnalyzer {
    fn tdg_score_to_defect(
        &self,
        file_path: PathBuf,
        score: TDGScore,
        index: usize,
        config: &ComplexityConfig,
    ) -> Defect {
        let severity = match score.severity {
            TDGSeverity::Critical => Severity::Critical,
            TDGSeverity::Warning => Severity::High,
            TDGSeverity::Normal => {
                if score.value > config.high_threshold {
                    Severity::Medium
                } else {
                    Severity::Low
                }
            }
        };

        let mut metrics = HashMap::new();
        metrics.insert("tdg_score".to_string(), score.value);
        metrics.insert("complexity_factor".to_string(), score.components.complexity);
        metrics.insert("churn_factor".to_string(), score.components.churn);
        metrics.insert("coupling_factor".to_string(), score.components.coupling);
        metrics.insert("domain_risk".to_string(), score.components.domain_risk);
        metrics.insert(
            "duplication_factor".to_string(),
            score.components.duplication,
        );
        metrics.insert("confidence".to_string(), score.confidence);

        Defect {
            id: format!("CPLX-{:04}", index),
            severity,
            category: DefectCategory::Complexity,
            file_path: file_path.clone(),
            line_start: 1, // TDG is file-level
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!(
                "File has high complexity with TDG score of {:.2} (threshold: {:.1})",
                score.value, config.max_tdg_score
            ),
            rule_id: "tdg-complexity".to_string(),
            fix_suggestion: Some(self.generate_fix_suggestion(&score)),
            metrics,
        }
    }

    fn generate_fix_suggestion(&self, score: &TDGScore) -> String {
        let mut suggestions = Vec::new();

        if score.components.complexity > 0.7 {
            suggestions.push("reduce cyclomatic complexity by extracting methods");
        }
        if score.components.coupling > 0.7 {
            suggestions.push("reduce coupling by improving module boundaries");
        }
        if score.components.duplication > 0.5 {
            suggestions.push("eliminate code duplication");
        }

        if suggestions.is_empty() {
            "Consider refactoring to reduce overall complexity".to_string()
        } else {
            format!("Consider: {}", suggestions.join(", "))
        }
    }
}

#[derive(Clone)]
pub struct ComplexityConfig {
    pub max_tdg_score: f64,
    pub high_threshold: f64,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_tdg_score: 2.0,
            high_threshold: 1.5,
        }
    }
}

impl AnalyzerConfig for ComplexityConfig {}

#[derive(Clone, Default)]
pub struct SATDConfig {
    pub include_test_files: bool,
}

impl AnalyzerConfig for SATDConfig {}

/// Adapter for SATD (Self-Admitted Technical Debt) detection
pub struct SATDDefectAnalyzer {
    detector: crate::services::satd_detector::SATDDetector,
}

impl SATDDefectAnalyzer {
    pub fn new() -> Self {
        Self {
            detector: crate::services::satd_detector::SATDDetector::new(),
        }
    }
}

impl Default for SATDDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DefectAnalyzer for SATDDefectAnalyzer {
    type Config = SATDConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();

        // Analyze the project directory
        let result = self
            .detector
            .analyze_project(project_path, config.include_test_files)
            .await?;

        for (index, debt) in result.items.iter().enumerate() {
            defects.push(self.technical_debt_to_defect(debt, index + 1));
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::TechnicalDebt
    }

    fn supports_incremental(&self) -> bool {
        true
    }
}

impl SATDDefectAnalyzer {
    fn technical_debt_to_defect(&self, debt: &TechnicalDebt, index: usize) -> Defect {
        let severity = match debt.severity {
            crate::services::satd_detector::Severity::Critical => Severity::Critical,
            crate::services::satd_detector::Severity::High => Severity::High,
            crate::services::satd_detector::Severity::Medium => Severity::Medium,
            crate::services::satd_detector::Severity::Low => Severity::Low,
        };

        let mut metrics = HashMap::new();
        metrics.insert(
            "debt_category".to_string(),
            debt.category.to_string().parse::<f64>().unwrap_or(0.0),
        );

        Defect {
            id: format!("SATD-{:04}", index),
            severity,
            category: DefectCategory::TechnicalDebt,
            file_path: debt.file.clone(),
            line_start: debt.line,
            line_end: None,
            column_start: Some(debt.column),
            column_end: None,
            message: format!("{}: {}", debt.category, debt.text),
            rule_id: format!("satd-{}", debt.category.to_string().to_lowercase()),
            fix_suggestion: Some(self.get_debt_fix_suggestion(&debt.category)),
            metrics,
        }
    }

    fn get_debt_fix_suggestion(&self, category: &DebtCategory) -> String {
        match category {
            DebtCategory::Design => "Refactor to improve design and architecture",
            DebtCategory::Defect => "Fix the known defect or bug",
            DebtCategory::Requirement => "Complete the missing requirement implementation",
            DebtCategory::Test => "Add proper test coverage",
            DebtCategory::Performance => "Optimize the performance issue",
            DebtCategory::Security => "Address the security vulnerability",
        }
        .to_string()
    }
}

/// Adapter for dead code detection
pub struct DeadCodeDefectAnalyzer;

impl DeadCodeDefectAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DeadCodeDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct DeadCodeConfig {
    pub min_confidence: f64,
}

impl Default for DeadCodeConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
        }
    }
}

impl AnalyzerConfig for DeadCodeConfig {}

#[async_trait]
impl DefectAnalyzer for DeadCodeDefectAnalyzer {
    type Config = DeadCodeConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();

        // Build dependency graph
        let project = crate::services::project_analyzer::Project::new(project_path)?;
        let dag = project.build_dependency_graph().await?;
        let mut analyzer = crate::services::dead_code_analyzer::DeadCodeAnalyzer::new(1000);
        let report = analyzer.analyze_dependency_graph(&dag);

        // Convert dead functions
        for (index, item) in report.dead_functions.iter().enumerate() {
            if item.confidence as f64 >= config.min_confidence {
                defects.push(self.dead_code_item_to_defect(item, "DEAD-FN", index + 1));
            }
        }

        // Convert dead classes
        for (index, item) in report.dead_classes.iter().enumerate() {
            if item.confidence as f64 >= config.min_confidence {
                defects.push(self.dead_code_item_to_defect(item, "DEAD-CLS", index + 1));
            }
        }

        // Convert unreachable code
        for (index, block) in report.unreachable_code.iter().enumerate() {
            defects.push(self.unreachable_block_to_defect(block, index + 1));
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::DeadCode
    }

    fn supports_incremental(&self) -> bool {
        false // Requires full graph analysis
    }
}

impl DeadCodeDefectAnalyzer {
    fn dead_code_item_to_defect(&self, item: &DeadCodeItem, prefix: &str, index: usize) -> Defect {
        let severity = if item.confidence > 0.9 {
            Severity::High
        } else if item.confidence > 0.7 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let mut metrics = HashMap::new();
        metrics.insert("confidence".to_string(), item.confidence as f64);

        Defect {
            id: format!("{}-{:04}", prefix, index),
            severity,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from(&item.file_path),
            line_start: item.line_number,
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!(
                "Dead {}: '{}' is never used (confidence: {:.0}%)",
                format!("{:?}", item.dead_type).to_lowercase(),
                item.name,
                item.confidence * 100.0
            ),
            rule_id: format!("dead-{}", format!("{:?}", item.dead_type).to_lowercase()),
            fix_suggestion: Some(format!(
                "Remove unused {} '{}'",
                format!("{:?}", item.dead_type).to_lowercase(),
                item.name
            )),
            metrics,
        }
    }

    fn unreachable_block_to_defect(&self, block: &UnreachableBlock, index: usize) -> Defect {
        let mut metrics = HashMap::new();
        metrics.insert(
            "lines".to_string(),
            (block.end_line - block.start_line + 1) as f64,
        );

        Defect {
            id: format!("UNREACH-{:04}", index),
            severity: Severity::High,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from(&block.file_path),
            line_start: block.start_line,
            line_end: Some(block.end_line),
            column_start: None,
            column_end: None,
            message: format!(
                "Unreachable code block ({} lines) - {}",
                block.end_line - block.start_line + 1,
                block.reason
            ),
            rule_id: "unreachable-code".to_string(),
            fix_suggestion: Some("Remove unreachable code".to_string()),
            metrics,
        }
    }
}

/// Adapter for code duplication detection
pub struct DuplicationDefectAnalyzer {
    detector: crate::services::duplicate_detector::DuplicateDetectionEngine,
}

impl DuplicationDefectAnalyzer {
    pub fn new() -> Self {
        let config = crate::services::duplicate_detector::DuplicateDetectionConfig::default();
        Self {
            detector: crate::services::duplicate_detector::DuplicateDetectionEngine::new(config),
        }
    }
}

impl Default for DuplicationDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct DuplicationConfig {
    pub min_similarity: f64,
}

impl Default for DuplicationConfig {
    fn default() -> Self {
        Self {
            min_similarity: 0.8,
        }
    }
}

impl AnalyzerConfig for DuplicationConfig {}

#[async_trait]
impl DefectAnalyzer for DuplicationDefectAnalyzer {
    type Config = DuplicationConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();
        let files = discover_source_files(project_path).await?;

        // Prepare file contents
        let mut file_contents = Vec::new();
        for file in files {
            if let Ok(content) = tokio::fs::read_to_string(&file).await {
                let language = self.detect_language(&file);
                file_contents.push((file.clone(), content, language));
            }
        }

        let report = self.detector.detect_duplicates(&file_contents)?;

        // Convert clone groups to defects
        for (group_index, group) in report.groups.iter().enumerate() {
            if group.average_similarity >= config.min_similarity {
                for (instance_index, instance) in group.fragments.iter().enumerate() {
                    defects.push(self.clone_to_defect(
                        group,
                        instance,
                        group_index * 100 + instance_index + 1,
                    ));
                }
            }
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Duplication
    }

    fn supports_incremental(&self) -> bool {
        false // Requires full codebase analysis
    }
}

impl DuplicationDefectAnalyzer {
    fn detect_language(&self, path: &Path) -> crate::services::duplicate_detector::Language {
        use crate::services::duplicate_detector::Language;

        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") => Language::TypeScript,
            Some("js") => Language::JavaScript,
            Some("py") => Language::Python,
            Some("c") => Language::C,
            Some("cpp") | Some("cc") => Language::Cpp,
            Some("kt") => Language::Kotlin,
            _ => Language::Rust, // Default
        }
    }

    fn clone_to_defect(
        &self,
        group: &CloneGroup,
        instance: &crate::services::duplicate_detector::CloneInstance,
        index: usize,
    ) -> Defect {
        let severity = match &group.clone_type {
            CloneType::Type1 { .. } if group.total_lines > 50 => Severity::High,
            CloneType::Type1 { .. } => Severity::Medium,
            CloneType::Type2 { .. } if group.total_lines > 30 => Severity::Medium,
            CloneType::Type2 { .. } => Severity::Low,
            CloneType::Type3 { .. } => Severity::Low,
        };

        let mut metrics = HashMap::new();
        metrics.insert(
            "similarity".to_string(),
            instance.similarity_to_representative,
        );
        metrics.insert("total_lines".to_string(), group.total_lines as f64);
        metrics.insert("group_size".to_string(), group.fragments.len() as f64);

        Defect {
            id: format!("DUP-{:04}", index),
            severity,
            category: DefectCategory::Duplication,
            file_path: instance.file.clone(),
            line_start: instance.start_line as u32,
            line_end: Some(instance.end_line as u32),
            column_start: None,
            column_end: None,
            message: format!(
                "{:?} code clone ({} lines, {:.0}% similarity) in group {}",
                group.clone_type,
                instance.end_line - instance.start_line + 1,
                instance.similarity_to_representative * 100.0,
                group.id
            ),
            rule_id: format!("duplication-{:?}", group.clone_type).to_lowercase(),
            fix_suggestion: Some(
                "Extract duplicated code into a shared function or module".to_string(),
            ),
            metrics,
        }
    }
}

/// Adapter for performance analysis (Big-O)
pub struct PerformanceDefectAnalyzer {
    analyzer: crate::services::big_o_analyzer::BigOAnalyzer,
}

impl PerformanceDefectAnalyzer {
    pub fn new() -> Self {
        Self {
            analyzer: crate::services::big_o_analyzer::BigOAnalyzer::new(),
        }
    }
}

impl Default for PerformanceDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct PerformanceConfig {
    pub include_nlogn: bool,
}

impl AnalyzerConfig for PerformanceConfig {}

#[async_trait]
impl DefectAnalyzer for PerformanceDefectAnalyzer {
    type Config = PerformanceConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();
        let analysis_config = crate::services::big_o_analyzer::BigOAnalysisConfig {
            project_path: project_path.to_path_buf(),
            include_patterns: vec![],
            exclude_patterns: vec![],
            confidence_threshold: 50,
            analyze_space_complexity: true,
        };
        let report = self.analyzer.analyze(analysis_config).await?;

        for (index, func) in report.high_complexity_functions.iter().enumerate() {
            if self.is_problematic_complexity(&func.time_complexity, &config) {
                defects.push(self.function_complexity_to_defect(func, index + 1));
            }
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Performance
    }

    fn supports_incremental(&self) -> bool {
        true
    }
}

impl PerformanceDefectAnalyzer {
    fn is_problematic_complexity(
        &self,
        complexity: &crate::models::complexity_bound::ComplexityBound,
        config: &PerformanceConfig,
    ) -> bool {
        use crate::models::complexity_bound::BigOClass;

        matches!(
            complexity.class,
            BigOClass::Quadratic | BigOClass::Cubic | BigOClass::Exponential | BigOClass::Factorial
        ) || (config.include_nlogn && matches!(complexity.class, BigOClass::Linearithmic))
    }

    fn function_complexity_to_defect(&self, func: &FunctionComplexity, index: usize) -> Defect {
        use crate::models::complexity_bound::BigOClass;

        let severity = match func.time_complexity.class {
            BigOClass::Exponential | BigOClass::Factorial => Severity::Critical,
            BigOClass::Cubic => Severity::High,
            BigOClass::Quadratic => Severity::Medium,
            _ => Severity::Low,
        };

        let mut metrics = HashMap::new();
        metrics.insert(
            "time_complexity_class".to_string(),
            func.time_complexity.class as u8 as f64,
        );
        metrics.insert(
            "space_complexity_class".to_string(),
            func.space_complexity.class as u8 as f64,
        );
        metrics.insert("confidence".to_string(), func.confidence as f64);

        Defect {
            id: format!("PERF-{:04}", index),
            severity,
            category: DefectCategory::Performance,
            file_path: func.file_path.clone(),
            line_start: func.line_number as u32,
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!(
                "Function '{}' has high time complexity: {}",
                func.function_name,
                func.time_complexity.notation()
            ),
            rule_id: "high-complexity".to_string(),
            fix_suggestion: Some(self.generate_performance_suggestion(&func.time_complexity)),
            metrics,
        }
    }

    fn generate_performance_suggestion(
        &self,
        complexity: &crate::models::complexity_bound::ComplexityBound,
    ) -> String {
        use crate::models::complexity_bound::BigOClass;

        match complexity.class {
            BigOClass::Quadratic => {
                "Consider using a more efficient algorithm or data structure to reduce quadratic complexity"
            }
            BigOClass::Cubic => {
                "Cubic complexity is rarely acceptable; consider algorithmic improvements"
            }
            BigOClass::Exponential => {
                "Exponential complexity should be avoided; consider dynamic programming or approximation"
            }
            BigOClass::Factorial => {
                "Factorial complexity is unacceptable for most use cases; fundamental algorithm redesign needed"
            }
            _ => {
                "Review algorithm efficiency and consider optimization"
            }
        }
        .to_string()
    }
}

/// Adapter for architecture issues detection
pub struct ArchitectureDefectAnalyzer;

impl ArchitectureDefectAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ArchitectureDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ArchitectureConfig {
    pub max_coupling: usize,
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        Self { max_coupling: 10 }
    }
}

impl AnalyzerConfig for ArchitectureConfig {}

#[async_trait]
impl DefectAnalyzer for ArchitectureDefectAnalyzer {
    type Config = ArchitectureConfig;

    async fn analyze(&self, project_path: &Path, _config: Self::Config) -> Result<Vec<Defect>> {
        let defects = Vec::new();

        // Build dependency graph
        let project = crate::services::project_analyzer::Project::new(project_path)?;
        let _dag = project.build_dependency_graph().await?;

        // Analyze architecture issues
        // This is a placeholder for actual architecture analysis

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Architecture
    }

    fn supports_incremental(&self) -> bool {
        false
    }
}
