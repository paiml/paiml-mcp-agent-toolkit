use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// File-level dead code metrics with ranking support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeadCodeMetrics {
    pub path: String,
    pub dead_lines: usize,
    pub total_lines: usize,
    pub dead_percentage: f32,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub dead_modules: usize,
    pub unreachable_blocks: usize,
    pub dead_score: f32,
    pub confidence: ConfidenceLevel,
    pub items: Vec<DeadCodeItem>,
}

/// Confidence level for dead code detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,   // Definitely dead (no references)
    Medium, // Possibly dead (only internal references)
    Low,    // Might be used (dynamic calls possible)
}

/// Individual dead code item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeItem {
    pub item_type: DeadCodeType,
    pub name: String,
    pub line: u32,
    pub reason: String,
}

/// Types of dead code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadCodeType {
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "unreachable")]
    UnreachableCode,
}

/// Complete dead code ranking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeRankingResult {
    pub summary: DeadCodeSummary,
    pub ranked_files: Vec<FileDeadCodeMetrics>,
    pub analysis_timestamp: DateTime<Utc>,
    pub config: DeadCodeAnalysisConfig,
}

/// Dead code analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    pub total_files_analyzed: usize,
    pub files_with_dead_code: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f32,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub dead_modules: usize,
    pub unreachable_blocks: usize,
}

/// Configuration for dead code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeAnalysisConfig {
    pub include_unreachable: bool,
    pub include_tests: bool,
    pub min_dead_lines: usize,
}

impl FileDeadCodeMetrics {
    /// Calculate dead code score using weighted algorithm
    pub fn calculate_score(&mut self) {
        // Weighted scoring similar to complexity ranker
        let percentage_weight = 0.35;
        let absolute_weight = 0.30;
        let function_weight = 0.20;
        let confidence_weight = 0.15;

        let confidence_multiplier = match self.confidence {
            ConfidenceLevel::High => 1.0,
            ConfidenceLevel::Medium => 0.7,
            ConfidenceLevel::Low => 0.4,
        };

        self.dead_score = (self.dead_percentage * percentage_weight)
            + (self.dead_lines.min(1000) as f32 / 10.0 * absolute_weight)
            + (self.dead_functions.min(50) as f32 * 2.0 * function_weight)
            + (100.0 * confidence_multiplier * confidence_weight);
    }

    /// Create a new FileDeadCodeMetrics instance
    pub fn new(path: String) -> Self {
        Self {
            path,
            dead_lines: 0,
            total_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 0.0,
            confidence: ConfidenceLevel::Medium,
            items: Vec::new(),
        }
    }

    /// Add a dead code item
    pub fn add_item(&mut self, item: DeadCodeItem) {
        match item.item_type {
            DeadCodeType::Function => self.dead_functions += 1,
            DeadCodeType::Class => self.dead_classes += 1,
            DeadCodeType::Variable => self.dead_modules += 1, // Using modules for variables for now
            DeadCodeType::UnreachableCode => self.unreachable_blocks += 1,
        }
        self.items.push(item);
    }

    /// Update dead code percentage based on current counts
    pub fn update_percentage(&mut self) {
        if self.total_lines > 0 {
            self.dead_percentage = (self.dead_lines as f32 / self.total_lines as f32) * 100.0;
        }
    }
}

impl DeadCodeSummary {
    /// Create a new summary from file metrics
    pub fn from_files(files: &[FileDeadCodeMetrics]) -> Self {
        let total_files_analyzed = files.len();
        let files_with_dead_code = files.iter().filter(|f| f.dead_lines > 0).count();
        let total_dead_lines = files.iter().map(|f| f.dead_lines).sum();
        let dead_functions = files.iter().map(|f| f.dead_functions).sum();
        let dead_classes = files.iter().map(|f| f.dead_classes).sum();
        let dead_modules = files.iter().map(|f| f.dead_modules).sum();
        let unreachable_blocks = files.iter().map(|f| f.unreachable_blocks).sum();

        let total_lines: usize = files.iter().map(|f| f.total_lines).sum();
        let dead_percentage = if total_lines > 0 {
            (total_dead_lines as f32 / total_lines as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_files_analyzed,
            files_with_dead_code,
            total_dead_lines,
            dead_percentage,
            dead_functions,
            dead_classes,
            dead_modules,
            unreachable_blocks,
        }
    }
}

impl Default for DeadCodeAnalysisConfig {
    fn default() -> Self {
        Self {
            include_unreachable: false,
            include_tests: false,
            min_dead_lines: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_file_dead_code_metrics_creation() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());

        assert_eq!(metrics.path, "test.rs");
        assert_eq!(metrics.dead_lines, 0);
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.dead_percentage, 0.0);
        assert_eq!(metrics.dead_functions, 0);
        assert_eq!(metrics.dead_classes, 0);
        assert_eq!(metrics.dead_modules, 0);
        assert_eq!(metrics.unreachable_blocks, 0);
        assert_eq!(metrics.dead_score, 0.0);
        assert!(matches!(metrics.confidence, ConfidenceLevel::Medium));
        assert!(metrics.items.is_empty());

        // Test adding an item
        let item = DeadCodeItem {
            item_type: DeadCodeType::Function,
            name: "unused_function".to_string(),
            line: 42,
            reason: "Never called".to_string(),
        };

        metrics.add_item(item);
        assert_eq!(metrics.dead_functions, 1);
        assert_eq!(metrics.items.len(), 1);

        // Test score calculation
        metrics.total_lines = 100;
        metrics.dead_lines = 20;
        metrics.update_percentage();
        assert_eq!(metrics.dead_percentage, 20.0);

        metrics.calculate_score();
        assert!(metrics.dead_score > 0.0);
    }

    #[test]
    fn test_dead_code_item_creation() {
        let item = DeadCodeItem {
            item_type: DeadCodeType::Class,
            name: "UnusedClass".to_string(),
            line: 15,
            reason: "Never instantiated".to_string(),
        };

        assert_eq!(item.item_type, DeadCodeType::Class);
        assert_eq!(item.name, "UnusedClass");
        assert_eq!(item.line, 15);
        assert_eq!(item.reason, "Never instantiated");
    }

    #[test]
    fn test_dead_code_type_variants() {
        let types = [
            DeadCodeType::Function,
            DeadCodeType::Class,
            DeadCodeType::Variable,
            DeadCodeType::UnreachableCode,
        ];

        for dead_type in types {
            // Test that the types can be created and compared
            let item = DeadCodeItem {
                item_type: dead_type,
                name: "test".to_string(),
                line: 1,
                reason: "test".to_string(),
            };
            assert_eq!(item.item_type, dead_type);
        }
    }

    #[test]
    fn test_confidence_levels() {
        let levels = [
            ConfidenceLevel::High,
            ConfidenceLevel::Medium,
            ConfidenceLevel::Low,
        ];

        for level in levels {
            let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());
            metrics.confidence = level;
            assert_eq!(metrics.confidence, level);
        }
    }

    #[test]
    fn test_dead_code_ranking_result() {
        let config = DeadCodeAnalysisConfig::default();
        let summary = DeadCodeSummary::from_files(&[]);
        let timestamp = Utc::now();

        let result = DeadCodeRankingResult {
            summary: summary.clone(),
            ranked_files: vec![],
            analysis_timestamp: timestamp,
            config: config.clone(),
        };

        assert_eq!(result.summary.total_files_analyzed, 0);
        assert_eq!(result.ranked_files.len(), 0);
        assert_eq!(result.config.min_dead_lines, config.min_dead_lines);
    }

    #[test]
    fn test_dead_code_summary_from_files() {
        let mut file1 = FileDeadCodeMetrics::new("file1.rs".to_string());
        file1.dead_lines = 10;
        file1.total_lines = 100;
        file1.dead_functions = 2;
        file1.dead_classes = 1;
        file1.dead_modules = 0;
        file1.unreachable_blocks = 1;

        let mut file2 = FileDeadCodeMetrics::new("file2.rs".to_string());
        file2.dead_lines = 5;
        file2.total_lines = 50;
        file2.dead_functions = 1;
        file2.dead_classes = 0;
        file2.dead_modules = 1;
        file2.unreachable_blocks = 0;

        let files = vec![file1, file2];
        let summary = DeadCodeSummary::from_files(&files);

        assert_eq!(summary.total_files_analyzed, 2);
        assert_eq!(summary.files_with_dead_code, 2);
        assert_eq!(summary.total_dead_lines, 15);
        assert_eq!(summary.dead_functions, 3);
        assert_eq!(summary.dead_classes, 1);
        assert_eq!(summary.dead_modules, 1);
        assert_eq!(summary.unreachable_blocks, 1);
        assert_eq!(summary.dead_percentage, 10.0); // 15 dead lines out of 150 total
    }

    #[test]
    fn test_dead_code_analysis_config_default() {
        let config = DeadCodeAnalysisConfig::default();

        assert!(!config.include_unreachable);
        assert!(!config.include_tests);
        assert_eq!(config.min_dead_lines, 10);
    }

    #[test]
    fn test_file_metrics_add_different_item_types() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());

        // Add function
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Function,
            name: "fn1".to_string(),
            line: 10,
            reason: "unused".to_string(),
        });

        // Add class
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Class,
            name: "Class1".to_string(),
            line: 20,
            reason: "unused".to_string(),
        });

        // Add variable (which increments modules counter due to current implementation)
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Variable,
            name: "var1".to_string(),
            line: 30,
            reason: "unused".to_string(),
        });

        // Add unreachable code
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::UnreachableCode,
            name: "block".to_string(),
            line: 40,
            reason: "unreachable".to_string(),
        });

        assert_eq!(metrics.dead_functions, 1);
        assert_eq!(metrics.dead_classes, 1);
        assert_eq!(metrics.dead_modules, 1); // Variable increments modules
        assert_eq!(metrics.unreachable_blocks, 1);
        assert_eq!(metrics.items.len(), 4);
    }

    #[test]
    fn test_score_calculation_with_different_confidence_levels() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());
        metrics.dead_lines = 50;
        metrics.total_lines = 100;
        metrics.dead_functions = 5;
        metrics.update_percentage();

        // Test with high confidence
        metrics.confidence = ConfidenceLevel::High;
        metrics.calculate_score();
        let high_score = metrics.dead_score;

        // Test with medium confidence
        metrics.confidence = ConfidenceLevel::Medium;
        metrics.calculate_score();
        let medium_score = metrics.dead_score;

        // Test with low confidence
        metrics.confidence = ConfidenceLevel::Low;
        metrics.calculate_score();
        let low_score = metrics.dead_score;

        // High confidence should result in higher score than medium, which should be higher than low
        assert!(high_score > medium_score);
        assert!(medium_score > low_score);
    }
}

// Additional type for handler compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeResult {
    pub summary: DeadCodeSummary,
    pub files: Vec<FileDeadCodeMetrics>,
    pub total_files: usize,
    pub analyzed_files: usize,
}
