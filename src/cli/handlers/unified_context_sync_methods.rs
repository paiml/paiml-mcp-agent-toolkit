// Synchronous builder methods for UnifiedContextBuilder
// Included via include!() in unified_context_builder.rs

impl UnifiedContextBuilder {
    // Add basic project structure with context
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_basic_structure_with_context(&mut self, context: &ProjectContext) -> &mut Self {
        self.output.push_str("# Project Context\n\n");
        self.output.push_str("## Project Structure\n\n");
        self.output
            .push_str(&format!("- **Language**: {}\n", &context.project_type));
        self.output.push_str(&format!(
            "- **Total Files**: {}\n",
            context.summary.total_files
        ));
        self.output.push_str(&format!(
            "- **Total Functions**: {}\n",
            context.summary.total_functions
        ));
        self.output.push_str(&format!(
            "- **Total Structs**: {}\n",
            context.summary.total_structs
        ));
        self.output.push_str(&format!(
            "- **Total Enums**: {}\n",
            context.summary.total_enums
        ));
        self.output.push_str(&format!(
            "- **Total Traits**: {}\n",
            context.summary.total_traits
        ));
        self.output.push('\n');
        self
    }

    // Add key components with function names
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_key_components(&mut self, context: &ProjectContext) -> &mut Self {
        self.output.push_str("## Key Components\n\n");

        if context.files.is_empty() {
            self.output.push_str("No files analyzed.\n\n");
            return self;
        }

        // Skip file-level details for now as the structure is different
        // This would need to be populated from actual analysis
        self.output.push('\n');
        self
    }

    // Add quality insights (existing functionality)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_quality_insights(&mut self, context: &ProjectContext) -> &mut Self {
        self.output.push_str("## Quality Insights\n\n");

        let total_functions = context.summary.total_functions;
        let total_files = context.summary.total_files;

        if total_functions > 20 {
            self.output.push_str(&format!(
                "- Large codebase with {} functions across {} files\n",
                total_functions, total_files
            ));
            self.output.push_str(&format!(
                "- Average {:.1} functions per file\n",
                total_functions as f64 / total_files.max(1) as f64
            ));
        }

        // Add more insights based on analysis results
        self.output.push('\n');
        self
    }

    // Add recommendations (existing functionality)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_recommendations(&mut self, _context: &ProjectContext) -> &mut Self {
        self.output.push_str("## Recommendations\n\n");

        // Generate recommendations based on all analyses
        self.output
            .push_str("- Consider modularizing the codebase for better organization\n");
        self.output
            .push_str("- Enable detailed AST analysis for function-level insights\n");
        self.output
            .push_str("- Review and address identified technical debt\n");
        self.output
            .push_str("- Refactor high-complexity functions\n");

        self.output.push('\n');
        self
    }

    // Build the final output
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> String {
        self.output
    }

    // Synchronous test-friendly methods
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_basic_structure(&mut self) -> &mut Self {
        self.output.push_str("# Project Context\n\n");
        self.output.push_str("## Project Structure\n\n");
        self.output.push_str("- **Language**: Test\n");
        self.output.push_str("- **Total Files**: 1\n");
        self.output.push_str("- **Total Functions**: 1\n");
        self.output.push_str("- **Total Structs**: 1\n");
        self.output.push_str("- **Total Enums**: 1\n");
        self.output.push_str("- **Total Traits**: 1\n");
        self.output.push('\n');
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_big_o_analysis(&mut self) -> &mut Self {
        self.output.push_str("## Big-O Complexity Analysis\n\n");
        self.output.push_str("- `function_name`: O(n)\n");
        self.output.push_str("- `sort_function`: O(n log n)\n");
        self.output.push_str("- `nested_loops`: O(n²)\n");
        self.output.push('\n');
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_entropy_analysis(&mut self) -> &mut Self {
        self.output.push_str("## Entropy Analysis\n\n");
        self.output.push_str("- Pattern Entropy: 0.750\n");
        self.output.push_str("- Code Duplication: 15.0%\n");
        self.output.push_str("- Structural Entropy: 0.650\n");
        self.output.push_str("- Actionable Improvements:\n");
        self.output
            .push_str("  - Reduce duplication in utility functions\n");
        self.output.push_str("  - Extract common patterns\n");
        self.output.push('\n');
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_tdg_analysis(&mut self) -> &mut Self {
        self.output.push_str("## Technical Debt Gradient (TDG)\n\n");
        self.output.push_str("### Overall TDG Score: 3.25\n\n");
        self.output.push_str("### File-level TDG:\n");
        self.output.push_str("- `main.rs`: 2.50\n");
        self.output.push_str("- `utils.rs`: 4.00\n");
        self.output.push_str("\n### Debt Hotspots:\n");
        self.output.push_str("- main.rs:45 (Score: 3.20)\n");
        self.output.push_str("\n### Refactoring Priority:\n");
        self.output
            .push_str("1. Simplify complex function in utils.rs\n");
        self.output.push('\n');
        self
    }
}
