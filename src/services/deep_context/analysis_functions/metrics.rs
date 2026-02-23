// Shared metrics computation (complexity, churn, dead code, duplicate detection,
// SATD, provability, DAG, Big-O)
// Extracted for file health (CB-040)

use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::services::complexity::{ComplexityReport, FileComplexityMetrics};
use crate::services::satd_detector::SATDAnalysisResult;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{info, warn};

use super::super::DagType;

// Re-export thread-local caches so sibling submodules can access them
// These are defined here because metrics.rs owns the complexity cache logic

thread_local! {
    pub static RUST_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static TYPESCRIPT_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static PYTHON_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static GO_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static WASM_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static BASH_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
}

/// Detect programming language from file extension
pub fn detect_language(path: &std::path::Path) -> String {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            // Core languages with full support
            "rs" => "rust".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "js" | "jsx" | "mjs" | "cjs" => "javascript".to_string(),
            "py" | "pyi" => "python".to_string(),
            "go" => "go".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp".to_string(),

            // JVM languages
            "java" => "java".to_string(),
            "kt" | "kts" => "kotlin".to_string(),

            // .NET languages
            "cs" => "csharp".to_string(),

            // Scripting languages
            "sh" | "bash" => "bash".to_string(),
            "rb" => "ruby".to_string(),
            "lua" => "lua".to_string(),

            // Functional languages
            "ex" | "exs" => "elixir".to_string(),
            "erl" | "hrl" => "erlang".to_string(),
            "hs" | "lhs" => "haskell".to_string(),
            "ml" | "mli" => "ocaml".to_string(),

            // Apple ecosystem
            "swift" => "swift".to_string(),

            // WebAssembly
            "wat" | "wasm" => "wasm".to_string(),

            // Proof assistants
            "lean" => "lean".to_string(),

            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}

// --- Complexity analysis ---

pub async fn analyze_complexity(path: &std::path::Path) -> anyhow::Result<ComplexityReport> {
    use crate::services::complexity::aggregate_results;

    info!("Starting complexity analysis for path: {:?}", path);

    // Extract Method: Discover source files
    let source_files = discover_source_files_for_complexity(path)?;
    info!(
        "Discovered {} source files for complexity analysis",
        source_files.len()
    );

    // Extract Method: Analyze all files
    let file_metrics = analyze_files_complexity(source_files).await;
    info!(
        "Complexity analysis completed. Analyzed {} files",
        file_metrics.len()
    );

    // Aggregate results into final report
    Ok(aggregate_results(file_metrics))
}

fn discover_source_files_for_complexity(
    path: &std::path::Path,
) -> anyhow::Result<Vec<std::path::PathBuf>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(5_000), // Reasonable limit for complexity analysis
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);
    discovery.discover_files()
}

async fn analyze_files_complexity(
    source_files: Vec<std::path::PathBuf>,
) -> Vec<crate::services::complexity::FileComplexityMetrics> {
    // Parallelize complexity analysis using futures for better performance
    use futures::stream::{self, StreamExt};

    stream::iter(source_files)
        .map(|file_path| async move { analyze_single_file_complexity(&file_path).await })
        .buffer_unordered(num_cpus::get()) // Process multiple files concurrently
        .filter_map(|opt| async move { opt })
        .collect::<Vec<_>>()
        .await
}

#[allow(clippy::manual_map)] // Complex async + feature-gate pattern, not easily simplified
async fn analyze_single_file_complexity(
    file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    let ext = file_path.extension()?.to_str()?;

    match ext {
        "rs" => {
            // OPTIMIZATION: Check unified cache first (from analyze_rust_language)
            // This avoids the second parse that analyze_rust_file_with_complexity would do
            let cached = RUST_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned());

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path if not in cache (shouldn't happen in normal flow)
                use crate::services::ast_rust::analyze_rust_file_with_complexity;
                analyze_rust_file_with_complexity(file_path).await.ok()
            }
        }
        "ts" | "js" | "jsx" | "tsx" => {
            // OPTIMIZATION: Check TypeScript unified cache first (from analyze_typescript_language)
            // This avoids the second parse that analyze_typescript_file_with_complexity would do
            let cached =
                TYPESCRIPT_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned());

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path if not in cache (shouldn't happen in normal flow)
                #[cfg(feature = "typescript-ast")]
                {
                    use crate::services::ast_typescript::analyze_typescript_file_with_complexity;
                    analyze_typescript_file_with_complexity(file_path)
                        .await
                        .ok()
                }
                #[cfg(not(feature = "typescript-ast"))]
                None
            }
        }
        "py" => {
            // OPTIMIZATION: Check Python unified cache first (from analyze_python_language)
            // This avoids the second parse that analyze_python_file_with_complexity would do
            let cached = PYTHON_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned());

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path if not in cache (shouldn't happen in normal flow)
                #[cfg(feature = "python-ast")]
                {
                    use crate::services::ast_python::analyze_python_file_with_complexity;
                    analyze_python_file_with_complexity(file_path, None)
                        .await
                        .ok()
                }
                #[cfg(not(feature = "python-ast"))]
                None
            }
        }
        "go" => {
            // OPTIMIZATION: Check Go unified cache first (from analyze_go_language)
            // This avoids the second parse - TICKET-3004
            GO_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned())
        }
        "wat" | "wasm" => {
            // OPTIMIZATION: Check WASM unified cache first (from analyze_wasm_language)
            // This avoids the second parse - TICKET-3005
            WASM_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned())
        }
        "sh" | "bash" => {
            // OPTIMIZATION: Check Bash unified cache first (from analyze_bash_language)
            // This avoids the second parse - TICKET-3006
            BASH_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned())
        }
        "lua" => analyze_lua_complexity_metrics(file_path).await,
        _ => None,
    }
}

/// Lua complexity metrics: tree-sitter for totals, regex for function names
#[cfg(feature = "lua-ast")]
#[allow(clippy::cast_possible_truncation)]
async fn analyze_lua_complexity_metrics(
    file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    use crate::ast::languages::lua::LuaStrategy;
    use crate::ast::languages::LanguageStrategy;
    use crate::services::complexity::{
        ComplexityMetrics as CMetrics, FileComplexityMetrics as FCMetrics,
    };

    let content = tokio::fs::read_to_string(file_path).await.ok()?;
    let strategy = LuaStrategy::new();
    let ast = strategy.parse_file(file_path, &content).await.ok()?;
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);

    let mut func_complexities = extract_lua_function_complexities(&content);
    let func_count = func_complexities.len().max(1) as u32;
    for fc in &mut func_complexities {
        fc.metrics.cyclomatic = (cyclomatic / func_count) as u16;
        fc.metrics.cognitive = (cognitive / func_count) as u16;
    }

    Some(FCMetrics {
        path: file_path.to_string_lossy().to_string(),
        total_complexity: CMetrics {
            cyclomatic: cyclomatic as u16,
            cognitive: cognitive as u16,
            nesting_max: 0,
            lines: content.lines().count() as u16,
            halstead: None,
        },
        functions: func_complexities,
        classes: vec![],
    })
}

#[cfg(not(feature = "lua-ast"))]
async fn analyze_lua_complexity_metrics(
    _file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    None
}

/// Extract Lua function names and line numbers using regex patterns
#[allow(clippy::cast_possible_truncation)]
fn extract_lua_function_complexities(
    content: &str,
) -> Vec<crate::services::complexity::FunctionComplexity> {
    use crate::services::complexity::{ComplexityMetrics as CMetrics, FunctionComplexity as FComp};

    let patterns = [
        r"(?m)^\s*function\s+(\w+(?:[.:]\w+)*)\s*\(",
        r"(?m)^\s*local\s+function\s+(\w+)\s*\(",
    ];
    let mut funcs = Vec::new();
    for pat in &patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            for cap in re.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let line = content.get(..m.start()).unwrap_or_default().lines().count() as u32;
                    funcs.push(FComp {
                        name: m.as_str().to_string(),
                        line_start: line,
                        line_end: 0,
                        metrics: CMetrics {
                            cyclomatic: 0,
                            cognitive: 0,
                            nesting_max: 0,
                            lines: 0,
                            halstead: None,
                        },
                    });
                }
            }
        }
    }
    funcs.sort_by_key(|f| f.line_start);
    funcs
}

// --- Churn analysis ---

pub async fn analyze_churn(path: &std::path::Path, days: u32) -> anyhow::Result<CodeChurnAnalysis> {
    use crate::services::git_analysis::GitAnalysisService;
    use std::time::{Duration, Instant};

    info!("Starting churn analysis for path: {:?}", path);
    let start = Instant::now();

    // Smart bounds: timeout after 3 seconds for churn analysis
    let timeout = Duration::from_secs(3);

    match tokio::time::timeout(timeout, async {
        GitAnalysisService::analyze_code_churn(path, days)
            .map_err(|e| anyhow::anyhow!("Failed to analyze code churn: {e}"))
    })
    .await
    {
        Ok(result) => {
            info!("Churn analysis completed in {:?}", start.elapsed());
            result
        }
        Err(_) => {
            warn!("Churn analysis timed out after {:?}", timeout);
            // Return empty churn analysis instead of failing
            use crate::models::churn::ChurnSummary;
            use chrono::Utc;

            Ok(CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days: days,
                repository_root: path.to_path_buf(),
                files: Vec::new(),
                summary: ChurnSummary {
                    total_commits: 0,
                    total_files_changed: 0,
                    hotspot_files: Vec::new(),
                    stable_files: Vec::new(),
                    author_contributions: std::collections::HashMap::new(),
                    mean_churn_score: 0.0,
                    variance_churn_score: 0.0,
                    stddev_churn_score: 0.0,
                },
            })
        }
    }
}

// --- Dead code analysis ---

#[allow(clippy::cast_possible_truncation)]
pub async fn analyze_dead_code(
    path: &std::path::Path,
) -> anyhow::Result<crate::models::dead_code::DeadCodeRankingResult> {
    use crate::models::dead_code::{
        DeadCodeAnalysisConfig, DeadCodeRankingResult, DeadCodeSummary,
    };
    use crate::services::file_discovery::ProjectFileDiscovery;

    // Phase 1: Discover files for analysis without async AST parsing
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    let all_files = discovery_service.discover_files()?;

    // Filter for source code files
    let files: Vec<_> = all_files
        .into_iter()
        .filter(|file| {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                matches!(ext, "rs" | "ts" | "js" | "py")
            } else {
                false
            }
        })
        .collect();

    // Phase 2: Perform lightweight static analysis for dead code detection
    // Use parallel processing for file I/O and analysis
    let mut file_metrics: Vec<crate::models::dead_code::FileDeadCodeMetrics> = files
        .par_iter()
        .filter_map(|file_path| {
            std::fs::read_to_string(file_path)
                .ok()
                .map(|content| analyze_file_for_dead_code(file_path, &content))
        })
        .collect();

    // Aggregate metrics
    let total_dead_functions: usize = file_metrics.par_iter().map(|m| m.dead_functions).sum();
    let total_dead_classes: usize = file_metrics.par_iter().map(|m| m.dead_classes).sum();
    let total_dead_lines: usize = file_metrics.par_iter().map(|m| m.dead_lines).sum();

    // Phase 3: Calculate summary statistics
    let files_with_dead_code = file_metrics
        .par_iter()
        .filter(|f| f.dead_score > 0.0)
        .count();
    let total_lines_estimate: usize = file_metrics.par_iter().map(|f| f.total_lines).sum();
    let dead_percentage = if total_lines_estimate > 0 {
        (total_dead_lines as f32 / total_lines_estimate as f32) * 100.0
    } else {
        0.0
    };

    // Phase 4: Sort files by dead code score
    file_metrics.sort_unstable_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(DeadCodeRankingResult {
        summary: DeadCodeSummary {
            total_files_analyzed: files.len(),
            files_with_dead_code,
            total_dead_lines,
            dead_percentage,
            dead_functions: total_dead_functions,
            dead_classes: total_dead_classes,
            dead_modules: 0,
            unreachable_blocks: 0,
        },
        ranked_files: file_metrics,
        analysis_timestamp: chrono::Utc::now(),
        config: DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: false,
            min_dead_lines: 5,
        },
    })
}

#[allow(clippy::cast_possible_truncation)]
fn analyze_file_for_dead_code(
    file_path: &std::path::Path,
    content: &str,
) -> crate::models::dead_code::FileDeadCodeMetrics {
    use crate::models::dead_code::{ConfidenceLevel, FileDeadCodeMetrics};

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let file_ext = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let mut dead_functions = 0;
    let mut dead_classes = 0;
    let mut dead_items = Vec::new();

    // Analyze based on file type
    match file_ext {
        "rs" => analyze_rust_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        "ts" | "js" => analyze_typescript_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        "py" => analyze_python_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        _ => {}
    }

    let dead_lines = dead_items.len() * 5; // Conservative estimate
    let dead_percentage = if total_lines > 0 {
        (dead_lines as f32 / total_lines as f32) * 100.0
    } else {
        0.0
    };

    let confidence = if dead_items.is_empty() {
        ConfidenceLevel::High // High confidence in no dead code
    } else if dead_percentage > 20.0 {
        ConfidenceLevel::Medium
    } else {
        ConfidenceLevel::Low
    };

    let mut metrics = FileDeadCodeMetrics {
        path: file_path.to_string_lossy().to_string(),
        dead_lines,
        total_lines,
        dead_percentage,
        dead_functions,
        dead_classes,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence,
        items: dead_items,
    };

    metrics.calculate_score();
    metrics
}

fn analyze_rust_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_rust_dead_functions(lines, dead_functions, dead_items);
    analyze_rust_dead_structs(lines, dead_classes, dead_items);
}

/// Analyze dead functions in Rust code
#[allow(clippy::cast_possible_truncation)]
fn analyze_rust_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("fn ") && !trimmed.contains("pub ") {
            if let Some(function_name) = extract_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead structs in Rust code
#[allow(clippy::cast_possible_truncation)]
fn analyze_rust_dead_structs(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("struct ") && !trimmed.contains("pub ") {
            if let Some(struct_name) = extract_struct_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: struct_name,
                    line: (line_num + 1) as u32,
                    reason: "Private struct with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract function name if unused
fn extract_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract struct name if unused
fn extract_struct_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let struct_name = extract_struct_name(trimmed);
    if !struct_name.is_empty() && !is_type_used_in_file(lines, &struct_name) {
        Some(struct_name)
    } else {
        None
    }
}

fn analyze_typescript_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_typescript_dead_functions(lines, dead_functions, dead_items);
    analyze_typescript_dead_classes(lines, dead_classes, dead_items);
}

/// Analyze dead functions in TypeScript code
#[allow(clippy::cast_possible_truncation)]
fn analyze_typescript_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("function ") && !trimmed.contains("export") {
            if let Some(function_name) = extract_js_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead classes in TypeScript code
#[allow(clippy::cast_possible_truncation)]
fn analyze_typescript_dead_classes(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("class ") && !trimmed.contains("export") {
            if let Some(class_name) = extract_class_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: class_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported class with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract JS function name if unused
fn extract_js_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_js_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract class name if unused
fn extract_class_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let class_name = extract_class_name(trimmed);
    if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
        Some(class_name)
    } else {
        None
    }
}

fn analyze_python_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_python_dead_functions(lines, dead_functions, dead_items);
    analyze_python_dead_classes(lines, dead_classes, dead_items);
}

/// Analyze dead functions in Python code
#[allow(clippy::cast_possible_truncation)]
fn analyze_python_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("def _") {
            if let Some(function_name) = extract_python_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead classes in Python code
#[allow(clippy::cast_possible_truncation)]
fn analyze_python_dead_classes(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("class _") {
            if let Some(class_name) = extract_python_class_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: class_name,
                    line: (line_num + 1) as u32,
                    reason: "Private class with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract Python function name if unused
fn extract_python_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_python_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract Python class name if unused
fn extract_python_class_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let class_name = extract_python_class_name(trimmed);
    if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
        Some(class_name)
    } else {
        None
    }
}

fn extract_function_name(line: &str) -> String {
    if let Some(start) = line.find("fn ") {
        let after_fn = line.get(start + 3..).unwrap_or_default();
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn
                .get(..paren_pos)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_struct_name(line: &str) -> String {
    if let Some(start) = line.find("struct ") {
        let after_struct = line.get(start + 7..).unwrap_or_default();
        after_struct
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::with_capacity(1024)
    }
}

fn extract_js_function_name(line: &str) -> String {
    if let Some(start) = line.find("function ") {
        let after_fn = line.get(start + 9..).unwrap_or_default();
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn
                .get(..paren_pos)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_class_name(line: &str) -> String {
    if let Some(start) = line.find("class ") {
        let after_class = line.get(start + 6..).unwrap_or_default();
        after_class
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::with_capacity(1024)
    }
}

fn extract_python_function_name(line: &str) -> String {
    if let Some(start) = line.find("def ") {
        let after_def = line.get(start + 4..).unwrap_or_default();
        if let Some(paren_pos) = after_def.find('(') {
            after_def
                .get(..paren_pos)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_python_class_name(line: &str) -> String {
    if let Some(start) = line.find("class ") {
        let after_class = line.get(start + 6..).unwrap_or_default();
        if let Some(colon_pos) = after_class.find(':') {
            after_class
                .get(..colon_pos)
                .unwrap_or_default()
                .trim()
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        } else {
            after_class
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        }
    } else {
        String::with_capacity(1024)
    }
}

fn is_function_called_in_file(lines: &[&str], function_name: &str) -> bool {
    let call_pattern = format!("{function_name}(");
    lines.iter().any(|line| line.contains(&call_pattern))
}

fn is_type_used_in_file(lines: &[&str], type_name: &str) -> bool {
    lines.iter().any(|line| {
        line.contains(type_name)
            && (line.contains(&format!("new {type_name}"))
                || line.contains(&format!(": {type_name}"))
                || line.contains(&format!("<{type_name}>")))
    })
}

// --- Duplicate code analysis ---

pub async fn analyze_duplicate_code(
    path: &std::path::Path,
) -> anyhow::Result<crate::services::duplicate_detector::CloneReport> {
    use crate::services::duplicate_detector::DuplicateDetectionEngine;

    let all_files = discover_project_files(path)?;
    let files_for_analysis = filter_and_categorize_files_for_duplicates(all_files)?;
    let engine = DuplicateDetectionEngine::default();
    engine.detect_duplicates(&files_for_analysis)
}

fn discover_project_files(path: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    use crate::services::file_discovery::ProjectFileDiscovery;
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    discovery_service.discover_files()
}

fn filter_and_categorize_files_for_duplicates(
    all_files: Vec<std::path::PathBuf>,
) -> anyhow::Result<
    Vec<(
        std::path::PathBuf,
        String,
        crate::services::duplicate_detector::Language,
    )>,
> {
    let mut files_for_analysis = Vec::new();
    for file_path in all_files {
        if let Some((file, content, lang)) = process_file_for_duplicate_detection(&file_path)? {
            files_for_analysis.push((file, content, lang));
        }
    }
    Ok(files_for_analysis)
}

fn process_file_for_duplicate_detection(
    file_path: &std::path::Path,
) -> anyhow::Result<
    Option<(
        std::path::PathBuf,
        String,
        crate::services::duplicate_detector::Language,
    )>,
> {
    let ext = match file_path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => return Ok(None),
    };

    let language = match_extension_to_language(ext)?;
    if language.is_none() {
        return Ok(None);
    }

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) if c.lines().count() >= 10 => c,
        _ => return Ok(None),
    };

    Ok(Some((
        file_path.to_path_buf(),
        content,
        language.expect("internal error"),
    )))
}

fn match_extension_to_language(
    ext: &str,
) -> anyhow::Result<Option<crate::services::duplicate_detector::Language>> {
    use crate::services::duplicate_detector::Language;

    Ok(match ext {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "js" | "jsx" => Some(Language::JavaScript),
        "py" => Some(Language::Python),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
        "kt" | "kts" => Some(Language::Kotlin),
        _ => None,
    })
}

// --- SATD analysis ---

pub async fn analyze_satd(path: &std::path::Path) -> anyhow::Result<SATDAnalysisResult> {
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();
    let result = detector.analyze_project(path, false).await?;

    Ok(result)
}

// --- Provability analysis ---

pub async fn analyze_provability(
    path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    use crate::services::context::{analyze_project, AstItem};
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };
    use std::time::Instant;

    info!("Starting provability analysis for path: {:?}", path);

    let analyzer = LightweightProvabilityAnalyzer::new();

    // No timeouts - use proper concurrency instead
    let start = Instant::now();

    // Detect the primary language of the project
    let language = detect_project_language(path);

    // Discover functions from the project using AST analysis
    let project_context = match analyze_project(path, language).await {
        Ok(context) => context,
        Err(e) => {
            warn!("AST analysis failed for provability: {:?}", e);
            return Ok(vec![]);
        }
    };

    let mut function_ids = Vec::new();

    // Smart bounds: limit to 50 functions to prevent timeouts
    let mut function_count = 0;
    const MAX_FUNCTIONS: usize = 50;

    for file in &project_context.files {
        for item in &file.items {
            if let AstItem::Function { name, line, .. } = item {
                if function_count < MAX_FUNCTIONS {
                    function_ids.push(FunctionId {
                        file_path: file.path.clone(),
                        function_name: name.clone(),
                        line_number: *line,
                    });
                    function_count += 1;
                } else {
                    break;
                }
            }
        }
        if function_count >= MAX_FUNCTIONS {
            break;
        }
    }

    // If no functions found, add a mock one
    if function_ids.is_empty() {
        function_ids.push(FunctionId {
            file_path: format!("{}/src/main.rs", path.display()),
            function_name: "main".to_string(),
            line_number: 1,
        });
    }

    // Analyze all functions with proper parallel processing
    let summaries = analyzer.analyze_incrementally(&function_ids).await;

    info!(
        "Provability analysis completed for {} functions in {:?}",
        summaries.len(),
        start.elapsed()
    );
    Ok(summaries)
}

pub fn detect_project_language(path: &std::path::Path) -> &'static str {
    use crate::services::file_discovery::ProjectFileDiscovery;
    let discovery = ProjectFileDiscovery::new(path.to_path_buf());
    let files = discovery.discover_files().unwrap_or_default();

    let mut counts = [0; 5]; // rust, python, ruby, ts, js
    for file in &files {
        if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => counts[0] += 1,
                "py" => counts[1] += 1,
                "rb" => counts[2] += 1,
                "ts" | "tsx" => counts[3] += 1,
                "js" | "jsx" => counts[4] += 1,
                _ => {}
            }
        }
    }

    let (max_idx, _) = counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .unwrap_or((0, &0));
    match max_idx {
        0 => "rust",
        1 => "python",
        2 => "ruby",
        3 => "typescript",
        _ => "javascript",
    }
}

// --- DAG analysis ---

pub async fn analyze_dag(
    path: &std::path::Path,
    dag_type: DagType,
) -> anyhow::Result<DependencyGraph> {
    use crate::services::{
        context::analyze_project,
        dag_builder::{
            filter_call_edges, filter_import_edges, filter_inheritance_edges, DagBuilder,
        },
    };
    use std::time::Instant;

    info!("Starting DAG analysis for path: {:?}", path);
    let _start = Instant::now();

    // No timeout - efficient DAG analysis

    // Detect the primary language of the project
    let language = detect_project_language(path);

    // Analyze the project to get AST information - NO TIMEOUT!
    let project_context = analyze_project(path, language).await.map_err(|e| {
        warn!("AST analysis failed for DAG: {:?}", e);
        anyhow::anyhow!("AST analysis failed: {}", e)
    })?;

    // Smart bounds: limit graph size to 200 nodes (was 400)
    let graph = DagBuilder::build_from_project_with_limit(&project_context, 200);

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };

    Ok(filtered_graph)
}

// --- Big-O analysis ---

pub async fn analyze_big_o(
    path: &std::path::Path,
) -> anyhow::Result<crate::services::big_o_analyzer::BigOAnalysisReport> {
    use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};

    let analyzer = BigOAnalyzer::new();
    let config = BigOAnalysisConfig {
        project_path: path.to_path_buf(),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        confidence_threshold: 50,
        analyze_space_complexity: false,
    };

    analyzer.analyze(config).await
}
