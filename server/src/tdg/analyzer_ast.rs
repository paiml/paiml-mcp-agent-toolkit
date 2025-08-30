use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tdg::{
    config::TdgConfig, Language, MetricCategory, PenaltyTracker, ProjectScore, TdgScore,
};

/// AST-based TDG analyzer - proper implementation per specification
pub struct TdgAnalyzerAst {
    config: TdgConfig,
}

impl TdgAnalyzerAst {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: TdgConfig::default(),
        })
    }

    pub fn with_config(config: TdgConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        let language = Language::from_extension(path);
        let source = fs::read_to_string(path)?;
        self.analyze_source(&source, language, Some(path.to_path_buf()))
    }

    pub fn analyze_source(
        &self,
        source: &str,
        language: Language,
        file_path: Option<PathBuf>,
    ) -> Result<TdgScore> {
        let mut tracker = PenaltyTracker::new();
        let mut score = TdgScore {
            language,
            confidence: language.confidence(),
            file_path,
            ..Default::default()
        };

        // Use proper AST-based analysis
        match language {
            Language::Rust => self.analyze_rust_ast(source, &mut score, &mut tracker)?,
            Language::Python => self.analyze_python_ast(source, &mut score, &mut tracker)?,
            Language::JavaScript | Language::TypeScript => {
                self.analyze_javascript_ast(source, &mut score, &mut tracker)?
            }
            Language::Go => self.analyze_go_ast(source, &mut score, &mut tracker)?,
            Language::Java => self.analyze_java_ast(source, &mut score, &mut tracker)?,
            Language::C | Language::Cpp => self.analyze_c_ast(source, &mut score, &mut tracker)?,
            _ => {
                // Fallback to heuristics for unsupported languages
                // but with reduced confidence
                score.confidence *= 0.5;
                self.analyze_heuristic(source, &mut score, &mut tracker)?;
            }
        }

        score.penalties_applied = tracker.get_attributions();
        score.calculate_total();

        Ok(score)
    }

    fn analyze_rust_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "rust-ast")]
        {
            use syn::{visit::Visit, File};

            let ast = syn::parse_str::<File>(source)?;

            let mut visitor = RustComplexityVisitor::new();
            visitor.visit_file(&ast);

            // Calculate structural complexity based on AST
            let cyclomatic = visitor.cyclomatic_complexity;
            let cognitive = visitor.cognitive_complexity;

            score.structural_complexity = self.score_structural_complexity(
                cyclomatic,
                cognitive,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            // Calculate semantic complexity
            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.generic_count,
                visitor.abstraction_levels,
                tracker,
            );

            // Calculate duplication (requires AST comparison)
            score.duplication_ratio = self.analyze_duplication_ast(source, Language::Rust, tracker);

            // Calculate coupling
            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                visitor.interface_implementations,
                tracker,
            );

            // Calculate documentation coverage
            score.doc_coverage = self.score_documentation(
                visitor.documented_items,
                visitor.total_public_items,
                visitor.comment_lines,
                visitor.total_lines,
                tracker,
            );

            // Calculate consistency
            score.consistency_score = self.score_consistency_rust(&ast, tracker);
        }
        #[cfg(not(feature = "rust-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_python_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "python-ast")]
        {
            use rustpython_parser::{parse, Mode};

            let ast = parse(source, Mode::Module, "<string>")
                .map_err(|e| anyhow::anyhow!("Python parse error: {:?}", e))?;

            let mut visitor = PythonComplexityVisitor::new();
            visitor.analyze_module(ast);

            score.structural_complexity = self.score_structural_complexity(
                visitor.cyclomatic_complexity,
                visitor.cognitive_complexity,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.decorator_count,
                visitor.metaclass_count,
                tracker,
            );

            score.duplication_ratio =
                self.analyze_duplication_ast(source, Language::Python, tracker);

            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                0, // Python doesn't have explicit interfaces
                tracker,
            );

            score.doc_coverage = self.score_documentation(
                visitor.documented_functions,
                visitor.total_functions,
                visitor.docstring_lines,
                visitor.total_lines,
                tracker,
            );

            score.consistency_score = self.score_consistency_python(source, tracker);
        }
        #[cfg(not(feature = "python-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_javascript_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "typescript-ast")]
        {
            use swc_common::{sync::Lrc, FileName, SourceMap};
            use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
            use swc_ecma_visit::VisitWith;

            let cm: Lrc<SourceMap> = Default::default();
            let fm = cm.new_source_file(FileName::Custom("test.js".into()).into(), source.into());

            let lexer = Lexer::new(
                Syntax::Typescript(TsConfig {
                    tsx: true,
                    decorators: true,
                    ..Default::default()
                }),
                Default::default(),
                StringInput::from(&*fm),
                None,
            );

            let mut parser = Parser::new_from(lexer);

            match parser.parse_module() {
                Ok(module) => {
                    let mut visitor = JavaScriptComplexityVisitor::new();
                    module.visit_with(&mut visitor);

                    score.structural_complexity = self.score_structural_complexity(
                        visitor.cyclomatic_complexity,
                        visitor.cognitive_complexity,
                        visitor.max_nesting_depth,
                        visitor.max_function_length,
                        tracker,
                    );

                    score.semantic_complexity = self.score_semantic_complexity(
                        visitor.max_params,
                        visitor.async_count,
                        visitor.callback_depth,
                        tracker,
                    );

                    score.duplication_ratio =
                        self.analyze_duplication_ast(source, Language::JavaScript, tracker);

                    score.coupling_score = self.score_coupling(
                        visitor.import_count,
                        visitor.external_calls,
                        visitor.class_count,
                        tracker,
                    );

                    score.doc_coverage = self.score_documentation(
                        visitor.jsdoc_count,
                        visitor.function_count,
                        visitor.comment_lines,
                        visitor.total_lines,
                        tracker,
                    );

                    score.consistency_score = self.score_consistency_javascript(source, tracker);
                }
                Err(_) => {
                    self.analyze_heuristic(source, score, tracker)?;
                }
            }
        }
        #[cfg(not(feature = "typescript-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_go_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Go requires tree-sitter-go which we don't have yet
        // Use tree-sitter generic parser as fallback
        self.analyze_tree_sitter_generic(source, Language::Go, score, tracker)
    }

    fn analyze_java_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Java requires tree-sitter-java which we don't have yet
        // Use tree-sitter generic parser as fallback
        self.analyze_tree_sitter_generic(source, Language::Java, score, tracker)
    }

    fn analyze_c_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "c-ast")]
        {
            use tree_sitter::{Parser, Query, QueryCursor};

            let mut parser = Parser::new();
            let language = if score.language == Language::Cpp {
                tree_sitter_cpp::language()
            } else {
                tree_sitter_c::language()
            };

            parser
                .set_language(&language)
                .map_err(|e| anyhow::anyhow!("Failed to set language: {:?}", e))?;

            if let Some(tree) = parser.parse(source, None) {
                let root_node = tree.root_node();

                // Count control flow statements for cyclomatic complexity
                let query_str = r#"
                    (if_statement) @if
                    (while_statement) @while
                    (for_statement) @for
                    (do_statement) @do
                    (switch_statement) @switch
                    (case_statement) @case
                    (conditional_expression) @ternary
                "#;

                let query = Query::new(&language, query_str)
                    .map_err(|e| anyhow::anyhow!("Query error: {:?}", e))?;

                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, root_node, source.as_bytes());
                let cyclomatic = 1 + matches.count() as u32;

                score.structural_complexity = self.score_structural_complexity(
                    cyclomatic,
                    cyclomatic, // Use same value for cognitive for now
                    self.calculate_max_nesting(&root_node),
                    self.calculate_max_function_length(&root_node, source),
                    tracker,
                );

                // Count includes for coupling
                let include_query = Query::new(&language, "(preproc_include) @include")
                    .map_err(|e| anyhow::anyhow!("Query error: {:?}", e))?;

                let mut cursor = QueryCursor::new();
                let include_matches = cursor.matches(&include_query, root_node, source.as_bytes());
                let import_count = include_matches.count() as u32;

                score.coupling_score = self.score_coupling(import_count, 0, 0, tracker);

                // Basic documentation coverage
                let comment_query = Query::new(&language, "(comment) @comment")
                    .map_err(|e| anyhow::anyhow!("Query error: {:?}", e))?;

                let mut cursor = QueryCursor::new();
                let comment_matches = cursor.matches(&comment_query, root_node, source.as_bytes());
                let comment_count = comment_matches.count() as u32;

                score.doc_coverage = (comment_count as f32 / source.lines().count() as f32 * 10.0)
                    .min(self.config.weights.documentation);

                score.semantic_complexity = self.config.weights.semantic_complexity;
                score.duplication_ratio =
                    self.analyze_duplication_ast(source, score.language, tracker);
                score.consistency_score = self.config.weights.consistency;
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "c-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_tree_sitter_generic(
        &self,
        source: &str,
        _language: Language,
        score: &mut TdgScore,
        _tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Generic tree-sitter analysis for languages without specific parsers
        // Falls back to heuristic with reduced confidence
        score.confidence *= 0.7;
        self.analyze_heuristic(source, score, _tracker)
    }

    fn analyze_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Fallback heuristic analysis (mark as low confidence)
        score.confidence *= 0.3;

        // Use the simple analyzer's methods as fallback
        let simple_analyzer = crate::tdg::analyzer_simple::TdgAnalyzer::new()?;
        let simple_score = simple_analyzer.analyze_source(source, score.language, None)?;

        score.structural_complexity = simple_score.structural_complexity;
        score.semantic_complexity = simple_score.semantic_complexity;
        score.duplication_ratio = simple_score.duplication_ratio;
        score.coupling_score = simple_score.coupling_score;
        score.doc_coverage = simple_score.doc_coverage;
        score.consistency_score = simple_score.consistency_score;

        Ok(())
    }

    fn score_structural_complexity(
        &self,
        cyclomatic: u32,
        cognitive: u32,
        nesting_depth: usize,
        method_length: usize,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.structural_complexity;

        // Penalize high cyclomatic complexity
        if cyclomatic > self.config.thresholds.max_cyclomatic_complexity {
            let excess = (cyclomatic - self.config.thresholds.max_cyclomatic_complexity) as f32;
            let penalty = (excess * 0.5).min(15.0);

            if let Some(applied) = tracker.apply(
                format!("high_cyclomatic_{}", cyclomatic),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cyclomatic complexity: {}", cyclomatic),
            ) {
                points -= applied;
            }
        }

        // Penalize high cognitive complexity
        if cognitive > 15 {
            let excess = (cognitive - 15) as f32;
            let penalty = (excess * 0.3).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("high_cognitive_{}", cognitive),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cognitive complexity: {}", cognitive),
            ) {
                points -= applied;
            }
        }

        // Penalize deep nesting
        if nesting_depth > self.config.thresholds.max_nesting_depth as usize {
            let excess = (nesting_depth - self.config.thresholds.max_nesting_depth as usize) as f32;
            let penalty = excess.min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_nesting_{}", nesting_depth),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("Deep nesting: {} levels", nesting_depth),
            ) {
                points -= applied;
            }
        }

        // Penalize long methods
        if method_length > 50 {
            let excess = ((method_length - 50) as f32 / 10.0).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("long_method_{}", method_length),
                MetricCategory::StructuralComplexity,
                excess,
                format!("Long method: {} lines", method_length),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn score_semantic_complexity(
        &self,
        max_params: usize,
        type_complexity: u32,
        abstraction_levels: u32,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.semantic_complexity;

        // Penalize too many parameters
        if max_params > 5 {
            let penalty = ((max_params - 5) as f32 * 0.5).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_params_{}", max_params),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Too many parameters: {}", max_params),
            ) {
                points -= applied;
            }
        }

        // Penalize high type complexity
        if type_complexity > 10 {
            let penalty = ((type_complexity - 10) as f32 * 0.3).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("complex_types_{}", type_complexity),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Complex type usage: {}", type_complexity),
            ) {
                points -= applied;
            }
        }

        // Penalize too many abstraction levels
        if abstraction_levels > 3 {
            let penalty = ((abstraction_levels - 3) as f32).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_abstraction_{}", abstraction_levels),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Deep abstraction: {} levels", abstraction_levels),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_duplication_ast(
        &self,
        source: &str,
        _language: Language,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        // TODO: Implement AST-based duplication detection
        // For now, use a simple hash-based approach
        let mut points = self.config.weights.duplication;

        let lines: Vec<&str> = source
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
            .collect();

        if lines.len() < 3 {
            return points;
        }

        // Count exact duplicates
        let mut duplicates = 0;
        let mut seen = std::collections::HashSet::new();

        for line in &lines {
            if line.len() > 10 && !seen.insert(line) {
                duplicates += 1;
            }
        }

        let duplication_ratio = duplicates as f32 / lines.len() as f32;

        if duplication_ratio > 0.1 {
            let penalty = (duplication_ratio * 20.0).min(20.0);

            if let Some(applied) = tracker.apply(
                format!("duplication_{:.2}", duplication_ratio),
                MetricCategory::Duplication,
                penalty,
                format!("Code duplication: {:.1}%", duplication_ratio * 100.0),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn score_coupling(
        &self,
        import_count: u32,
        external_calls: u32,
        _interface_implementations: u32,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.coupling;

        // Penalize too many imports
        if import_count > 20 {
            let penalty = ((import_count - 20) as f32 * 0.2).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("many_imports_{}", import_count),
                MetricCategory::Coupling,
                penalty,
                format!("Too many imports: {}", import_count),
            ) {
                points -= applied;
            }
        }

        // Penalize too many external calls
        if external_calls > 50 {
            let penalty = ((external_calls - 50) as f32 * 0.1).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_external_calls_{}", external_calls),
                MetricCategory::Coupling,
                penalty,
                format!("Too many external calls: {}", external_calls),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn score_documentation(
        &self,
        documented_items: u32,
        total_public_items: u32,
        comment_lines: u32,
        total_lines: u32,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        if total_public_items == 0 {
            return self.config.weights.documentation;
        }

        let coverage = documented_items as f32 / total_public_items as f32;
        let comment_ratio = comment_lines as f32 / total_lines as f32;

        // Weight: 70% API documentation, 30% inline comments
        let score = coverage * 0.7 + comment_ratio * 0.3;

        (score * self.config.weights.documentation).min(self.config.weights.documentation)
    }

    fn score_consistency_rust(&self, _ast: &syn::File, _tracker: &mut PenaltyTracker) -> f32 {
        // Check naming conventions for Rust
        let points = self.config.weights.consistency;

        // TODO: Implement proper Rust naming convention checks
        // For now, return default score
        points
    }

    fn score_consistency_python(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        // Check PEP 8 compliance
        let mut points = self.config.weights.consistency;

        // Simple indentation consistency check
        let lines: Vec<&str> = source.lines().collect();
        let mut tab_count = 0;
        let mut space_count = 0;

        for line in &lines {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("    ") || line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if total_indented > 0 {
            let consistency = if tab_count > space_count {
                tab_count as f32 / total_indented as f32
            } else {
                space_count as f32 / total_indented as f32
            };

            points = consistency * self.config.weights.consistency;
        }

        points
    }

    fn score_consistency_javascript(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        // Check JavaScript/TypeScript style consistency
        self.score_consistency_python(source, tracker) // Use same logic for now
    }

    fn calculate_max_nesting(&self, node: &tree_sitter::Node) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;

        fn traverse(node: tree_sitter::Node, depth: usize, max: &mut usize) {
            *max = (*max).max(depth);

            for child in node.children(&mut node.walk()) {
                let new_depth = if matches!(
                    child.kind(),
                    "if_statement" | "while_statement" | "for_statement" | "compound_statement"
                ) {
                    depth + 1
                } else {
                    depth
                };
                traverse(child, new_depth, max);
            }
        }

        traverse(*node, 0, &mut max_depth);
        max_depth
    }

    fn calculate_max_function_length(&self, node: &tree_sitter::Node, source: &str) -> usize {
        let mut max_length = 0;

        fn find_functions(node: tree_sitter::Node, source: &str, max: &mut usize) {
            if node.kind() == "function_definition" {
                let start_line = node.start_position().row;
                let end_line = node.end_position().row;
                let length = end_line - start_line + 1;
                *max = (*max).max(length);
            }

            for child in node.children(&mut node.walk()) {
                find_functions(child, source, max);
            }
        }

        find_functions(*node, source, &mut max_length);
        max_length
    }

    pub fn analyze_project(&self, dir: &Path) -> Result<ProjectScore> {
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();

        for file in files {
            match self.analyze_file(&file) {
                Ok(score) => scores.push(score),
                Err(e) => eprintln!("Warning: Failed to analyze {}: {}", file.display(), e),
            }
        }

        Ok(ProjectScore::aggregate(scores))
    }

    pub fn compare(&self, path1: &Path, path2: &Path) -> Result<crate::tdg::Comparison> {
        let score1 = if path1.is_dir() {
            self.analyze_project(path1)?.average()
        } else {
            self.analyze_file(path1)?
        };

        let score2 = if path2.is_dir() {
            self.analyze_project(path2)?.average()
        } else {
            self.analyze_file(path2)?
        };

        Ok(crate::tdg::Comparison::new(score1, score2))
    }

    fn discover_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.discover_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn discover_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if !self.should_skip_directory(&path) {
                    self.discover_files_recursive(&path, files)?;
                }
            } else if self.should_analyze_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(
                name,
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | ".git"
                    | "__pycache__"
                    | ".pytest_cache"
                    | "venv"
                    | ".venv"
                    | "vendor"
                    | ".idea"
                    | ".vscode"
            )
        } else {
            false
        }
    }

    fn should_analyze_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "go"
                    | "java"
                    | "c"
                    | "h"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "hpp"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "kts"
            )
        } else {
            false
        }
    }
}

// Visitor implementations for AST analysis
#[cfg(feature = "rust-ast")]
struct RustComplexityVisitor {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_method_length: usize,
    max_params: usize,
    generic_count: u32,
    abstraction_levels: u32,
    import_count: u32,
    external_calls: u32,
    interface_implementations: u32,
    documented_items: u32,
    total_public_items: u32,
    comment_lines: u32,
    total_lines: u32,
    current_depth: usize,
}

#[cfg(feature = "rust-ast")]
impl RustComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_method_length: 0,
            max_params: 0,
            generic_count: 0,
            abstraction_levels: 0,
            import_count: 0,
            external_calls: 0,
            interface_implementations: 0,
            documented_items: 0,
            total_public_items: 0,
            comment_lines: 0,
            total_lines: 0,
            current_depth: 0,
        }
    }
}

#[cfg(feature = "rust-ast")]
impl<'ast> syn::visit::Visit<'ast> for RustComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_if(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_while(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_for_loop(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.cyclomatic_complexity += node.arms.len() as u32;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_match(self, node);
        self.current_depth -= 1;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Count parameters
        let param_count = node.sig.inputs.len();
        self.max_params = self.max_params.max(param_count);

        // Count generics
        self.generic_count += node.sig.generics.params.len() as u32;

        // Check if documented
        if !node.attrs.is_empty() {
            for attr in &node.attrs {
                if attr.path().is_ident("doc") {
                    self.documented_items += 1;
                    break;
                }
            }
        }

        // Count as public item if applicable
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.total_public_items += 1;
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_use_tree(&mut self, _node: &'ast syn::UseTree) {
        self.import_count += 1;
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.trait_.is_some() {
            self.interface_implementations += 1;
        }
        syn::visit::visit_item_impl(self, node);
    }
}

#[cfg(feature = "python-ast")]
struct PythonComplexityVisitor {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_method_length: usize,
    max_params: usize,
    decorator_count: u32,
    metaclass_count: u32,
    import_count: u32,
    external_calls: u32,
    documented_functions: u32,
    total_functions: u32,
    docstring_lines: u32,
    total_lines: u32,
}

#[cfg(feature = "python-ast")]
impl PythonComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_method_length: 0,
            max_params: 0,
            decorator_count: 0,
            metaclass_count: 0,
            import_count: 0,
            external_calls: 0,
            documented_functions: 0,
            total_functions: 0,
            docstring_lines: 0,
            total_lines: 0,
        }
    }

    fn analyze_module(&mut self, _module: rustpython_parser::ast::Mod) {
        // TODO: Implement Python AST analysis
        // This requires walking the Python AST and counting various metrics
    }
}

#[cfg(feature = "typescript-ast")]
struct JavaScriptComplexityVisitor {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_function_length: usize,
    max_params: usize,
    async_count: u32,
    callback_depth: u32,
    import_count: u32,
    external_calls: u32,
    class_count: u32,
    jsdoc_count: u32,
    function_count: u32,
    comment_lines: u32,
    total_lines: u32,
}

#[cfg(feature = "typescript-ast")]
impl JavaScriptComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_function_length: 0,
            max_params: 0,
            async_count: 0,
            callback_depth: 0,
            import_count: 0,
            external_calls: 0,
            class_count: 0,
            jsdoc_count: 0,
            function_count: 0,
            comment_lines: 0,
            total_lines: 0,
        }
    }
}

#[cfg(feature = "typescript-ast")]
impl swc_ecma_visit::Visit for JavaScriptComplexityVisitor {
    fn visit_if_stmt(&mut self, node: &swc_ecma_ast::IfStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
        swc_ecma_visit::visit_if_stmt(self, node);
    }

    fn visit_while_stmt(&mut self, node: &swc_ecma_ast::WhileStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
        swc_ecma_visit::visit_while_stmt(self, node);
    }

    fn visit_for_stmt(&mut self, node: &swc_ecma_ast::ForStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
        swc_ecma_visit::visit_for_stmt(self, node);
    }

    fn visit_switch_stmt(&mut self, node: &swc_ecma_ast::SwitchStmt) {
        self.cyclomatic_complexity += node.cases.len() as u32;
        self.cognitive_complexity += 1;
        swc_ecma_visit::visit_switch_stmt(self, node);
    }

    fn visit_function(&mut self, node: &swc_ecma_ast::Function) {
        self.function_count += 1;
        self.max_params = self.max_params.max(node.params.len());

        if node.is_async {
            self.async_count += 1;
        }

        swc_ecma_visit::visit_function(self, node);
    }

    fn visit_import_decl(&mut self, _node: &swc_ecma_ast::ImportDecl) {
        self.import_count += 1;
    }

    fn visit_class_decl(&mut self, _node: &swc_ecma_ast::ClassDecl) {
        self.class_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_analyzer_creation() {
        let analyzer = TdgAnalyzerAst::new().unwrap();
        assert!(analyzer.config.weights.structural_complexity > 0.0);
    }

    #[test]
    fn test_language_detection() {
        use crate::tdg::Language;

        assert_eq!(
            Language::from_extension(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            Language::from_extension(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            Language::from_extension(Path::new("test.js")),
            Language::JavaScript
        );
    }
}
