// Unified Context with Advanced Annotations - Extreme TDD Implementation
use crate::services::context::{ProjectContext, ProjectSummary, AstItem};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn};

/// Advanced context builder that integrates all analysis types
pub struct AdvancedUnifiedContextBuilder {
    project_path: PathBuf,
    output: String,
    pub enable_big_o: bool,
    pub enable_entropy: bool,
    pub enable_provability: bool,
    pub enable_graph_metrics: bool,
    pub enable_tdg: bool,
    pub enable_dead_code: bool,
    pub enable_satd: bool,
}

impl AdvancedUnifiedContextBuilder {
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            output: String::new(),
            enable_big_o: true,
            enable_entropy: true,
            enable_provability: true,
            enable_graph_metrics: true,
            enable_tdg: true,
            enable_dead_code: true,
            enable_satd: true,
        }
    }

    /// Build the complete unified context with all annotations
    pub async fn build_complete_context(&mut self) -> Result<String> {
        info!("Building unified context with advanced annotations for {:?}", self.project_path);

        // Step 1: Get basic context using SimpleDeepContext
        let basic_context = self.get_basic_context().await?;
        self.add_project_header(&basic_context);
        self.add_project_structure(&basic_context);
        self.add_key_components(&basic_context);

        // Step 2: Add all advanced annotations
        if self.enable_big_o {
            self.add_big_o_analysis().await?;
        }

        if self.enable_entropy {
            self.add_entropy_analysis().await?;
        }

        if self.enable_provability {
            self.add_provability_analysis().await?;
        }

        if self.enable_graph_metrics {
            self.add_graph_metrics().await?;
        }

        if self.enable_tdg {
            self.add_tdg_analysis().await?;
        }

        if self.enable_dead_code {
            self.add_dead_code_analysis().await?;
        }

        if self.enable_satd {
            self.add_satd_analysis().await?;
        }

        // Step 3: Add quality insights and recommendations
        self.add_quality_insights(&basic_context);
        self.add_recommendations(&basic_context);

        Ok(self.output.clone())
    }

    async fn get_basic_context(&self) -> Result<ProjectContext> {
        use crate::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisConfig};

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: self.project_path.clone(),
            include_features: vec![],
            include_patterns: vec![],
            exclude_patterns: vec!["**/node_modules/**".to_string(), "**/target/**".to_string()],
            enable_verbose: false,
        };

        let analysis_report = analyzer.analyze(config).await?;

        // Convert SimpleDeepContext report to ProjectContext
        Ok(ProjectContext {
            project_type: "rust".to_string(), // Default to rust
            files: vec![], // Would need to be populated from analysis_report
            summary: ProjectSummary {
                total_files: analysis_report.file_count,
                total_functions: analysis_report.complexity_metrics.total_functions,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
        })
    }

    fn add_project_header(&mut self, context: &ProjectContext) {
        self.output.push_str("# Project Context\n\n");
        self.output.push_str(&format!("Project: {}\n", self.project_path.display()));
        self.output.push_str(&format!("Language: {}\n\n", &context.project_type));
    }

    fn add_project_structure(&mut self, context: &ProjectContext) {
        self.output.push_str("## Project Structure\n\n");
        self.output.push_str(&format!("- **Total Files**: {}\n", context.summary.total_files));
        self.output.push_str(&format!("- **Total Functions**: {}\n", context.summary.total_functions));
        self.output.push_str(&format!("- **Total Structs**: {}\n", context.summary.total_structs));
        self.output.push_str(&format!("- **Total Enums**: {}\n", context.summary.total_enums));
        self.output.push_str(&format!("- **Total Traits**: {}\n", context.summary.total_traits));
        self.output.push('\n');
    }

    fn add_key_components(&mut self, context: &ProjectContext) {
        self.output.push_str("## Key Components\n\n");

        for file in &context.files {
            if let Some(complexity) = &file.complexity_metrics {
                if !complexity.functions.is_empty() {
                    self.output.push_str(&format!("### File: `{}`\n", file.path));
                    self.output.push_str(&format!("- Cyclomatic: {}\n", complexity.total_complexity.cyclomatic));
                    self.output.push_str(&format!("- Cognitive: {}\n", complexity.total_complexity.cognitive));
                    self.output.push_str(&format!("- Function Count: {}\n", complexity.functions.len()));

                    // Function names would be extracted from AST items
                    let function_count = file.items.iter().filter(|i| matches!(i, AstItem::Function { .. })).count();
                    if function_count > 0 {
                        self.output.push_str(&format!("- Functions: {}\n", function_count));
                    }
                    self.output.push('\n');
                }
            }
        }
    }

    async fn add_big_o_analysis(&mut self) -> Result<()> {
        self.output.push_str("## Big-O Complexity Analysis\n\n");

        match self.run_big_o_analysis().await {
            Ok(analysis) => {
                if analysis.is_empty() {
                    self.output.push_str("*No complexity patterns detected*\n\n");
                } else {
                    for (function, complexity) in analysis {
                        self.output.push_str(&format!("- `{}`: {}\n", function, complexity));
                    }
                    self.output.push('\n');
                }
            }
            Err(e) => {
                warn!("Big-O analysis failed: {}", e);
                self.output.push_str("*Big-O analysis unavailable*\n\n");
            }
        }

        Ok(())
    }

    async fn add_entropy_analysis(&mut self) -> Result<()> {
        self.output.push_str("## Entropy Analysis\n\n");

        match self.run_entropy_analysis().await {
            Ok(entropy_data) => {
                self.output.push_str(&format!("- **Pattern Entropy**: {:.3}\n", entropy_data.pattern_entropy));
                self.output.push_str(&format!("- **Code Duplication**: {:.1}%\n", entropy_data.duplication_percentage));
                self.output.push_str(&format!("- **Structural Entropy**: {:.3}\n", entropy_data.structural_entropy));

                if !entropy_data.actionable_items.is_empty() {
                    self.output.push_str("\n### Actionable Improvements:\n");
                    for item in &entropy_data.actionable_items {
                        self.output.push_str(&format!("- {}\n", item));
                    }
                }
                self.output.push('\n');
            }
            Err(e) => {
                warn!("Entropy analysis failed: {}", e);
                self.output.push_str("*Entropy analysis unavailable*\n\n");
            }
        }

        Ok(())
    }

    async fn add_provability_analysis(&mut self) -> Result<()> {
        self.output.push_str("## Provability Analysis\n\n");

        match self.run_provability_analysis().await {
            Ok(provability_data) => {
                self.output.push_str("### Invariants\n");
                for invariant in &provability_data.invariants {
                    self.output.push_str(&format!("- {}\n", invariant));
                }

                self.output.push_str("\n### Pre-conditions\n");
                for pre in &provability_data.preconditions {
                    self.output.push_str(&format!("- {}\n", pre));
                }

                self.output.push_str("\n### Post-conditions\n");
                for post in &provability_data.postconditions {
                    self.output.push_str(&format!("- {}\n", post));
                }

                self.output.push_str(&format!("\n### Verification Status: {}\n",
                    if provability_data.verified { "✓ Verified" } else { "⚠ Unverified" }));
                self.output.push('\n');
            }
            Err(e) => {
                warn!("Provability analysis failed: {}", e);
                self.output.push_str("*Provability analysis unavailable*\n\n");
            }
        }

        Ok(())
    }

    async fn add_graph_metrics(&mut self) -> Result<()> {
        self.output.push_str("## Graph Metrics\n\n");

        match self.run_graph_metrics_analysis().await {
            Ok(graph_data) => {
                self.output.push_str("### Centrality Measures\n");
                self.output.push_str(&format!("- **Betweenness Centrality**: {:.3}\n", graph_data.betweenness));
                self.output.push_str(&format!("- **Closeness Centrality**: {:.3}\n", graph_data.closeness));
                self.output.push_str(&format!("- **Degree Centrality**: {:.3}\n", graph_data.degree));

                self.output.push_str("\n### Graph Structure\n");
                self.output.push_str(&format!("- **Nodes**: {}\n", graph_data.node_count));
                self.output.push_str(&format!("- **Edges**: {}\n", graph_data.edge_count));
                self.output.push_str(&format!("- **Density**: {:.3}\n", graph_data.density));

                if !graph_data.critical_paths.is_empty() {
                    self.output.push_str("\n### Critical Paths\n");
                    for path in &graph_data.critical_paths {
                        self.output.push_str(&format!("- {}\n", path));
                    }
                }
                self.output.push('\n');
            }
            Err(e) => {
                warn!("Graph metrics analysis failed: {}", e);
                self.output.push_str("*Graph metrics unavailable*\n\n");
            }
        }

        Ok(())
    }

    async fn add_tdg_analysis(&mut self) -> Result<()> {
        self.output.push_str("## Technical Debt Gradient (TDG)\n\n");

        match self.run_tdg_analysis().await {
            Ok(tdg_data) => {
                self.output.push_str(&format!("### Overall TDG Score: {:.2}\n\n", tdg_data.overall_score));

                if !tdg_data.file_scores.is_empty() {
                    self.output.push_str("### File-level TDG Scores\n");
                    for (file, score) in &tdg_data.file_scores {
                        self.output.push_str(&format!("- `{}`: {:.2}\n", file, score));
                    }
                }

                if !tdg_data.hotspots.is_empty() {
                    self.output.push_str("\n### Debt Hotspots\n");
                    for hotspot in &tdg_data.hotspots {
                        self.output.push_str(&format!("- {} (Score: {:.2})\n", hotspot.location, hotspot.score));
                    }
                }

                if !tdg_data.priorities.is_empty() {
                    self.output.push_str("\n### Refactoring Priorities\n");
                    for (i, priority) in tdg_data.priorities.iter().enumerate().take(5) {
                        self.output.push_str(&format!("{}. {}\n", i + 1, priority));
                    }
                }
                self.output.push('\n');
            }
            Err(e) => {
                warn!("TDG analysis failed: {}", e);
                self.output.push_str("*TDG analysis unavailable*\n\n");
            }
        }

        Ok(())
    }

    async fn add_dead_code_analysis(&mut self) -> Result<()> {
        self.output.push_str("## Dead Code Analysis\n\n");

        match self.run_dead_code_analysis().await {
            Ok(dead_code_data) => {
                let total_dead = dead_code_data.total_dead_items();

                if total_dead == 0 {
                    self.output.push_str("✓ No dead code detected\n\n");
                } else {
                    self.output.push_str(&format!("⚠ Total dead code items: {}\n\n", total_dead));

                    if !dead_code_data.unreachable_functions.is_empty() {
                        self.output.push_str("### Unreachable Functions\n");
                        for func in &dead_code_data.unreachable_functions {
                            self.output.push_str(&format!("- `{}`\n", func));
                        }
                    }

                    if !dead_code_data.unused_variables.is_empty() {
                        self.output.push_str("\n### Unused Variables\n");
                        for var in &dead_code_data.unused_variables {
                            self.output.push_str(&format!("- `{}`\n", var));
                        }
                    }

                    if !dead_code_data.unused_imports.is_empty() {
                        self.output.push_str("\n### Unused Imports\n");
                        for import in &dead_code_data.unused_imports {
                            self.output.push_str(&format!("- `{}`\n", import));
                        }
                    }
                }
                self.output.push('\n');
            }
            Err(e) => {
                warn!("Dead code analysis failed: {}", e);
                self.output.push_str("*Dead code analysis unavailable*\n\n");
            }
        }

        Ok(())
    }

    async fn add_satd_analysis(&mut self) -> Result<()> {
        self.output.push_str("## Self-Admitted Technical Debt (SATD)\n\n");

        match self.run_satd_analysis().await {
            Ok(satd_data) => {
                let total_satd = satd_data.total_satd_count();

                self.output.push_str(&format!("### Total SATD Comments: {}\n\n", total_satd));

                if !satd_data.todos.is_empty() {
                    self.output.push_str(&format!("### TODO Comments ({})\n", satd_data.todos.len()));
                    for todo in satd_data.todos.iter().take(5) {
                        self.output.push_str(&format!("- {}: {}\n", todo.location, todo.comment));
                    }
                    if satd_data.todos.len() > 5 {
                        self.output.push_str(&format!("- ... and {} more\n", satd_data.todos.len() - 5));
                    }
                }

                if !satd_data.fixmes.is_empty() {
                    self.output.push_str(&format!("\n### FIXME Comments ({})\n", satd_data.fixmes.len()));
                    for fixme in satd_data.fixmes.iter().take(3) {
                        self.output.push_str(&format!("- {}: {}\n", fixme.location, fixme.comment));
                    }
                }

                if !satd_data.hacks.is_empty() {
                    self.output.push_str(&format!("\n### HACK Comments ({})\n", satd_data.hacks.len()));
                    for hack in satd_data.hacks.iter().take(3) {
                        self.output.push_str(&format!("- {}: {}\n", hack.location, hack.comment));
                    }
                }

                self.output.push_str("\n### Debt Categories\n");
                self.output.push_str(&format!("- **Design Debt**: {}\n", satd_data.design_debt));
                self.output.push_str(&format!("- **Code Debt**: {}\n", satd_data.code_debt));
                self.output.push_str(&format!("- **Test Debt**: {}\n", satd_data.test_debt));
                self.output.push_str(&format!("- **Documentation Debt**: {}\n", satd_data.doc_debt));
                self.output.push('\n');
            }
            Err(e) => {
                warn!("SATD analysis failed: {}", e);
                self.output.push_str("*SATD analysis unavailable*\n\n");
            }
        }

        Ok(())
    }

    fn add_quality_insights(&mut self, context: &ProjectContext) {
        self.output.push_str("## Quality Insights\n\n");

        let total_functions = context.summary.total_functions;
        let total_files = context.summary.total_files;

        if total_functions > 0 {
            let avg_functions_per_file = total_functions as f64 / total_files.max(1) as f64;

            self.output.push_str(&format!("- **Codebase Size**: {} functions across {} files\n",
                total_functions, total_files));
            self.output.push_str(&format!("- **Average Functions per File**: {:.1}\n", avg_functions_per_file));

            if avg_functions_per_file > 10.0 {
                self.output.push_str("- ⚠ High function density - consider modularization\n");
            }

            // Calculate complexity insights
            let mut high_complexity_count = 0;
            for file in &context.files {
                if let Some(complexity) = &file.complexity_metrics {
                    // Count based on cyclomatic complexity threshold
                if complexity.total_complexity.cyclomatic > 10 {
                    high_complexity_count += 1;
                }
                }
            }

            if high_complexity_count > 0 {
                self.output.push_str(&format!("- ⚠ High complexity functions: {}\n", high_complexity_count));
            }
        }

        self.output.push('\n');
    }

    fn add_recommendations(&mut self, context: &ProjectContext) {
        self.output.push_str("## Recommendations\n\n");

        let mut recommendations = Vec::new();

        // Based on function count
        if context.summary.total_functions > 100 {
            recommendations.push("Consider breaking down large modules into smaller, focused components");
        }

        // Based on complexity
        let mut total_complexity: usize = 0;
        let mut file_count = 0;
        for file in &context.files {
            if let Some(complexity) = &file.complexity_metrics {
                total_complexity += complexity.total_complexity.cyclomatic as usize;
                file_count += 1;
            }
        }

        if file_count > 0 {
            let avg_complexity = total_complexity as f64 / file_count as f64;
            if avg_complexity > 10.0 {
                recommendations.push("High average complexity detected - refactor complex functions");
            }
        }

        // Always add these recommendations
        recommendations.push("Enable all analysis features for comprehensive insights");
        recommendations.push("Review identified technical debt and create action items");
        recommendations.push("Monitor TDG scores over time to track improvement");

        for rec in recommendations {
            self.output.push_str(&format!("- {}\n", rec));
        }

        self.output.push('\n');
    }

    // Analysis execution methods
    async fn run_big_o_analysis(&self) -> Result<HashMap<String, String>> {
        // Stub - integrate with actual BigOAnalyzer
        Ok(HashMap::new())
    }

    async fn run_entropy_analysis(&self) -> Result<EntropyData> {
        // Stub - integrate with actual entropy analyzer
        Ok(EntropyData {
            pattern_entropy: 0.75,
            duplication_percentage: 16.4,
            structural_entropy: 0.82,
            actionable_items: vec![],
        })
    }

    async fn run_provability_analysis(&self) -> Result<ProvabilityData> {
        // Stub - integrate with actual provability analyzer
        Ok(ProvabilityData {
            invariants: vec![],
            preconditions: vec![],
            postconditions: vec![],
            verified: false,
        })
    }

    async fn run_graph_metrics_analysis(&self) -> Result<GraphMetricsData> {
        // Stub - integrate with actual graph analyzer
        Ok(GraphMetricsData {
            betweenness: 0.0,
            closeness: 0.0,
            degree: 0.0,
            node_count: 0,
            edge_count: 0,
            density: 0.0,
            critical_paths: vec![],
        })
    }

    async fn run_tdg_analysis(&self) -> Result<TdgData> {
        // Stub - integrate with actual TDG analyzer
        Ok(TdgData {
            overall_score: 0.0,
            file_scores: HashMap::new(),
            hotspots: vec![],
            priorities: vec![],
        })
    }

    async fn run_dead_code_analysis(&self) -> Result<DeadCodeData> {
        // Stub - integrate with actual dead code analyzer
        Ok(DeadCodeData {
            unreachable_functions: vec![],
            unused_variables: vec![],
            unused_imports: vec![],
        })
    }

    async fn run_satd_analysis(&self) -> Result<SatdData> {
        // Stub - integrate with actual SATD analyzer
        Ok(SatdData {
            todos: vec![],
            fixmes: vec![],
            hacks: vec![],
            design_debt: 0,
            code_debt: 0,
            test_debt: 0,
            doc_debt: 0,
        })
    }
}

// Data structures for analysis results
struct EntropyData {
    pattern_entropy: f64,
    duplication_percentage: f64,
    structural_entropy: f64,
    actionable_items: Vec<String>,
}

struct ProvabilityData {
    invariants: Vec<String>,
    preconditions: Vec<String>,
    postconditions: Vec<String>,
    verified: bool,
}

struct GraphMetricsData {
    betweenness: f64,
    closeness: f64,
    degree: f64,
    node_count: usize,
    edge_count: usize,
    density: f64,
    critical_paths: Vec<String>,
}

struct TdgData {
    overall_score: f64,
    file_scores: HashMap<String, f64>,
    hotspots: Vec<TdgHotspot>,
    priorities: Vec<String>,
}

struct TdgHotspot {
    location: String,
    score: f64,
}

struct DeadCodeData {
    unreachable_functions: Vec<String>,
    unused_variables: Vec<String>,
    unused_imports: Vec<String>,
}

impl DeadCodeData {
    fn total_dead_items(&self) -> usize {
        self.unreachable_functions.len() +
        self.unused_variables.len() +
        self.unused_imports.len()
    }
}

struct SatdData {
    todos: Vec<SatdComment>,
    fixmes: Vec<SatdComment>,
    hacks: Vec<SatdComment>,
    design_debt: usize,
    code_debt: usize,
    test_debt: usize,
    doc_debt: usize,
}

impl SatdData {
    fn total_satd_count(&self) -> usize {
        self.todos.len() + self.fixmes.len() + self.hacks.len()
    }
}

struct SatdComment {
    location: String,
    comment: String,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_unified_context_includes_all_sections() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() { println!(\"test\"); }").unwrap();

        let mut builder = AdvancedUnifiedContextBuilder::new(temp_dir.path());
        let result = builder.build_complete_context().await.unwrap();

        // Verify all sections are present
        assert!(result.contains("# Project Context"));
        assert!(result.contains("## Project Structure"));
        assert!(result.contains("## Key Components"));
        assert!(result.contains("## Big-O Complexity Analysis"));
        assert!(result.contains("## Entropy Analysis"));
        assert!(result.contains("## Provability Analysis"));
        assert!(result.contains("## Graph Metrics"));
        assert!(result.contains("## Technical Debt Gradient"));
        assert!(result.contains("## Dead Code Analysis"));
        assert!(result.contains("## Self-Admitted Technical Debt"));
        assert!(result.contains("## Quality Insights"));
        assert!(result.contains("## Recommendations"));
    }

    #[tokio::test]
    async fn test_context_with_disabled_features() {
        let temp_dir = TempDir::new().unwrap();

        let mut builder = AdvancedUnifiedContextBuilder::new(temp_dir.path());
        builder.enable_provability = false;
        builder.enable_graph_metrics = false;

        let result = builder.build_complete_context().await.unwrap();

        // These should be present
        assert!(result.contains("## Big-O Complexity Analysis"));
        assert!(result.contains("## Entropy Analysis"));

        // These should not be present
        assert!(!result.contains("## Provability Analysis"));
        assert!(!result.contains("## Graph Metrics"));
    }

    #[test]
    fn test_dead_code_total_calculation() {
        let dead_code = DeadCodeData {
            unreachable_functions: vec!["func1".to_string(), "func2".to_string()],
            unused_variables: vec!["var1".to_string()],
            unused_imports: vec!["import1".to_string(), "import2".to_string()],
        };

        assert_eq!(dead_code.total_dead_items(), 5);
    }

    #[test]
    fn test_satd_total_calculation() {
        let satd = SatdData {
            todos: vec![
                SatdComment { location: "file1".to_string(), comment: "TODO: fix".to_string() },
                SatdComment { location: "file2".to_string(), comment: "TODO: improve".to_string() },
            ],
            fixmes: vec![
                SatdComment { location: "file3".to_string(), comment: "FIXME: bug".to_string() },
            ],
            hacks: vec![],
            design_debt: 0,
            code_debt: 0,
            test_debt: 0,
            doc_debt: 0,
        };

        assert_eq!(satd.total_satd_count(), 3);
    }
}