//! Mutation engine for generating and executing mutants
use super::language::LanguageAdapter;
use super::types::*;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use syn::visit::Visit;
use syn::{Expr, File};
/// Mutation engine configuration
#[derive(Debug, Clone)]
pub struct MutationConfig {
    /// Mutation strategy
    pub strategy: MutationStrategy,
    /// Maximum mutants to generate (0 = unlimited)
    pub max_mutants: usize,
    /// Parallel execution threads
    pub parallel_threads: usize,
}
impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            strategy: MutationStrategy::Selective,
            max_mutants: 0,
            parallel_threads: num_cpus::get(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MutationStrategy {
    /// Select only high-kill-probability mutations
    Selective,
    /// Random mutation selection
    Random,
    /// Hybrid: mix of selective and random
    Hybrid { selective: f64, random: f64 },
}
/// Mutation engine
#[derive(Clone)]
pub struct MutationEngine {
    adapter: Arc<dyn LanguageAdapter>,
    config: MutationConfig,
}
impl MutationEngine {
    /// Create new mutation engine
    pub fn new(adapter: Arc<dyn LanguageAdapter>, config: MutationConfig) -> Self {
        Self { adapter, config }
    }
    /// Create default mutation engine with Rust adapter
    pub fn default_rust() -> Self {
        use super::RustAdapter;
        Self::new(Arc::new(RustAdapter::new()), MutationConfig::default())
    }
    /// Generate mutants from source file
    pub async fn generate_mutants_from_file(
        &self,
        source_file: &Path,
    ) -> Result<Vec<Mutant>> {
        let source = tokio::fs::read_to_string(source_file)
            .await
            .context("Failed to read source file")?;
        self.generate_mutants_from_source(source_file, &source).await
    }
    /// Generate mutants from source code
    pub async fn generate_mutants_from_source(
        &self,
        file_path: &Path,
        source: &str,
    ) -> Result<Vec<Mutant>> {
        let syntax_tree: File = syn::parse_file(source)
            .context("Failed to parse source")?;
        let mut visitor = MutationVisitor {
            mutants: Vec::new(),
            operators: self.adapter.mutation_operators(),
            file_path: file_path.to_path_buf(),
            syntax_tree: &syntax_tree,
        };
        visitor.visit_file(&syntax_tree);
        let mut mutants = visitor.mutants;
        self.apply_strategy(&mut mutants);
        Ok(mutants)
    }
    /// Apply mutation strategy to filter/limit mutants
    fn apply_strategy(&self, mutants: &mut Vec<Mutant>) {
        match &self.config.strategy {
            MutationStrategy::Selective => {
                mutants
                    .retain(|m| {
                        matches!(
                            m.operator, MutationOperatorType::ArithmeticReplacement |
                            MutationOperatorType::RelationalReplacement |
                            MutationOperatorType::ConditionalReplacement |
                            MutationOperatorType::UnaryReplacement |
                            MutationOperatorType::ConstantReplacement |
                            MutationOperatorType::StatementDeletion
                        )
                    });
            }
            MutationStrategy::Random => {
                if self.config.max_mutants > 0 && mutants.len() > self.config.max_mutants
                {
                    use rand::seq::SliceRandom;
                    let mut rng = rand::rng();
                    mutants.shuffle(&mut rng);
                    mutants.truncate(self.config.max_mutants);
                }
            }
            MutationStrategy::Hybrid { selective, random: _ } => {
                let selective_count = (mutants.len() as f64 * selective) as usize;
                mutants.sort_by(|a, b| b.operator.cmp(&a.operator));
                mutants.truncate(selective_count);
            }
        }
        if self.config.max_mutants > 0 && mutants.len() > self.config.max_mutants {
            mutants.truncate(self.config.max_mutants);
        }
    }
    /// Execute mutants and return results (sequential)
    pub async fn execute_mutants(
        &self,
        mutants: Vec<Mutant>,
    ) -> Result<Vec<MutationResult>> {
        let mut results = Vec::new();
        for mutant in mutants {
            let result = self.execute_mutant(&mutant).await?;
            results.push(result);
        }
        Ok(results)
    }
    /// Execute mutants in parallel using distributed executor
    pub async fn execute_mutants_parallel(
        &self,
        mutants: Vec<Mutant>,
    ) -> Result<Vec<MutationResult>> {
        let config = super::distributed::DistributedConfig {
            worker_count: self.config.parallel_threads,
            max_concurrent: self.config.parallel_threads * 2,
            queue_size: 1000,
            track_progress: true,
        };
        let executor = super::distributed::DistributedExecutor::new(
            self.adapter.clone(),
            config,
        );
        executor.execute_parallel(mutants).await
    }
    /// Execute a single mutant
    async fn execute_mutant(&self, mutant: &Mutant) -> Result<MutationResult> {
        let temp_file = self.write_temp_mutant(mutant).await?;
        let start = std::time::Instant::now();
        let test_result = self.adapter.run_tests(&temp_file).await?;
        let execution_time_ms = start.elapsed().as_millis() as u64;
        let status = if test_result.passed {
            MutantStatus::Survived
        } else {
            MutantStatus::Killed
        };
        let _ = tokio::fs::remove_file(&temp_file).await;
        Ok(MutationResult {
            mutant: mutant.clone(),
            status,
            test_failures: test_result.failures,
            execution_time_ms,
            error_message: None,
        })
    }
    /// Write mutant to temporary file
    async fn write_temp_mutant(&self, mutant: &Mutant) -> Result<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("mutant_{}.rs", mutant.id));
        tokio::fs::write(&temp_file, &mutant.mutated_source)
            .await
            .context("Failed to write temp mutant")?;
        Ok(temp_file)
    }
}
impl Default for MutationEngine {
    fn default() -> Self {
        Self::default_rust()
    }
}
/// AST visitor for finding mutation points
struct MutationVisitor<'a> {
    mutants: Vec<Mutant>,
    operators: Vec<Box<dyn super::operators::MutationOperator>>,
    file_path: std::path::PathBuf,
    syntax_tree: &'a File,
}
impl<'a> MutationVisitor<'a> {
    /// Delete a statement from the entire file and return the modified source
    fn delete_statement_in_file(&self, stmt_to_delete: &syn::Stmt) -> String {
        let mut modified_tree = self.syntax_tree.clone();
        let mut deleter = StatementDeletion {
            target_stmt: quote::quote!(# stmt_to_delete).to_string(),
            deleted: false,
        };
        use syn::visit_mut::VisitMut;
        deleter.visit_file_mut(&mut modified_tree);
        Self::format_syn_file(&modified_tree)
    }
    /// Replace an expression in the entire file and return the modified source
    fn replace_expression_in_file(
        &self,
        original_expr: &Expr,
        mutated_expr: &Expr,
    ) -> String {
        let mut modified_tree = self.syntax_tree.clone();
        let mut replacer = ExpressionReplacer {
            original: quote::quote!(# original_expr).to_string(),
            replacement: mutated_expr.clone(),
            replaced: false,
        };
        use syn::visit_mut::VisitMut;
        replacer.visit_file_mut(&mut modified_tree);
        Self::format_syn_file(&modified_tree)
    }
    /// Format a syn::File back to readable Rust code using prettyplease
    fn format_syn_file(file: &File) -> String {
        prettyplease::unparse(file)
    }
}
/// Visitor that deletes a specific statement from the AST
struct StatementDeletion {
    target_stmt: String,
    deleted: bool,
}
impl syn::visit_mut::VisitMut for StatementDeletion {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        if !self.deleted {
            block
                .stmts
                .retain(|stmt| {
                    let stmt_str = quote::quote!(# stmt).to_string();
                    if stmt_str == self.target_stmt && !self.deleted {
                        self.deleted = true;
                        false
                    } else {
                        true
                    }
                });
        }
        syn::visit_mut::visit_block_mut(self, block);
    }
}
/// Visitor that replaces a specific expression in the AST
struct ExpressionReplacer {
    original: String,
    replacement: Expr,
    replaced: bool,
}
impl syn::visit_mut::VisitMut for ExpressionReplacer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if !self.replaced {
            let current = quote::quote!(# expr).to_string();
            if current == self.original {
                *expr = self.replacement.clone();
                self.replaced = true;
                return;
            }
        }
        syn::visit_mut::visit_expr_mut(self, expr);
    }
}
impl<'a> Visit<'_> for MutationVisitor<'a> {
    fn visit_stmt(&mut self, stmt: &syn::Stmt) {
        let can_delete = match stmt {
            syn::Stmt::Expr(expr, semi) => {
                let is_deletable_type = matches!(
                    expr, Expr::Call(_) | Expr::MethodCall(_) | Expr::Assign(_) |
                    Expr::Macro(_)
                );
                is_deletable_type && semi.is_some()
            }
            syn::Stmt::Macro(_) => true,
            _ => false,
        };
        if can_delete {
            for operator in &self.operators {
                if operator.name() == "SDL" {
                    let mutated_source = self.delete_statement_in_file(stmt);
                    let mut hasher = Sha256::new();
                    let stmt_str = quote::quote!(# stmt).to_string();
                    hasher.update(&stmt_str);
                    let hash = format!("{:x}", hasher.finalize());
                    let location = SourceLocation {
                        line: 0,
                        column: 0,
                        end_line: 0,
                        end_column: 0,
                    };
                    let mutant = Mutant {
                        id: format!("{}_{}", operator.name(), & hash[..8]),
                        original_file: self.file_path.clone(),
                        mutated_source,
                        location,
                        operator: operator.operator_type(),
                        hash,
                        status: MutantStatus::Pending,
                    };
                    self.mutants.push(mutant);
                    break;
                }
            }
        }
        syn::visit::visit_stmt(self, stmt);
    }
    fn visit_expr(&mut self, expr: &Expr) {
        for operator in &self.operators {
            if operator.name() == "SDL" {
                continue;
            }
            if operator.can_mutate(expr) {
                let location = SourceLocation {
                    line: 0,
                    column: 0,
                    end_line: 0,
                    end_column: 0,
                };
                if let Ok(mutated_exprs) = operator.mutate(expr, location.clone()) {
                    for mutated_expr in mutated_exprs {
                        let mutated_source = self
                            .replace_expression_in_file(expr, &mutated_expr);
                        let mut hasher = Sha256::new();
                        let expr_str = quote::quote!(# mutated_expr).to_string();
                        hasher.update(&expr_str);
                        let hash = format!("{:x}", hasher.finalize());
                        let mutant = Mutant {
                            id: format!("{}_{}", operator.name(), & hash[..8]),
                            original_file: self.file_path.clone(),
                            mutated_source,
                            location: location.clone(),
                            operator: operator.operator_type(),
                            hash,
                            status: MutantStatus::Pending,
                        };
                        self.mutants.push(mutant);
                    }
                }
            }
        }
        syn::visit::visit_expr(self, expr);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::rust_adapter::RustAdapter;
    #[tokio::test]
    async fn test_mutation_engine_generate_mutants() {
        let adapter = Arc::new(RustAdapter::new());
        let config = MutationConfig::default();
        let engine = MutationEngine::new(adapter, config);
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let mutants = engine
            .generate_mutants_from_source(Path::new("test.rs"), source)
            .await
            .unwrap();
        assert!(! mutants.is_empty());
    }
    #[tokio::test]
    async fn test_mutation_engine_selective_strategy() {
        let adapter = Arc::new(RustAdapter::new());
        let config = MutationConfig {
            strategy: MutationStrategy::Selective,
            max_mutants: 2,
            ..Default::default()
        };
        let engine = MutationEngine::new(adapter, config);
        let source = "fn test(a: i32, b: i32) -> bool { a + b > 10 && a < b }";
        let mutants = engine
            .generate_mutants_from_source(Path::new("test.rs"), source)
            .await
            .unwrap();
        assert!(mutants.len() <= 2);
    }
}
