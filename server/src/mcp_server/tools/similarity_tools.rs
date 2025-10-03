//! MCP tools for advanced code similarity detection

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::services::similarity::{
    ComprehensiveReport, SimilarityConfig, SimilarityDetector,
};

/// MCP tool for detecting code duplicates and similarities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityAnalysisTool {
    #[serde(default = "default_project_path")]
    pub project_path: String,
    #[serde(default = "default_detection_type")]
    pub detection_type: String,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
    #[serde(default = "default_min_lines")]
    pub min_lines: usize,
    #[serde(default = "default_min_tokens")]
    pub min_tokens: usize,
    #[serde(default = "default_enable_entropy")]
    pub enable_entropy: bool,
}

fn default_project_path() -> String {
    ".".to_string()
}

fn default_detection_type() -> String {
    "all".to_string()
}

fn default_threshold() -> f64 {
    0.7
}

fn default_min_lines() -> usize {
    6
}

fn default_min_tokens() -> usize {
    50
}

fn default_enable_entropy() -> bool {
    true
}

/// Execute similarity analysis via MCP
pub async fn analyze_similarity(params: Value) -> Result<Value> {
    let tool: SimilarityAnalysisTool = serde_json::from_value(params)?;
    
    // Configure detector
    let config = SimilarityConfig {
        min_lines: tool.min_lines,
        min_tokens: tool.min_tokens,
        similarity_threshold: tool.threshold,
        enable_entropy: tool.enable_entropy,
        enable_ast: matches!(tool.detection_type.as_str(), "fuzzy" | "all"),
        enable_semantic: matches!(tool.detection_type.as_str(), "semantic" | "all"),
        window_size: 40,
        k_gram_size: 15,
    };
    
    let detector = SimilarityDetector::new(config);
    
    // Collect files from project
    let project_path = PathBuf::from(&tool.project_path);
    let files = collect_project_files(&project_path).await?;
    
    // Perform analysis
    let report = detector.comprehensive_analysis(&files);
    
    // Convert to JSON response
    Ok(json!({
        "success": true,
        "project_path": tool.project_path,
        "files_analyzed": files.len(),
        "metrics": {
            "duplication_percentage": report.metrics.duplication_percentage,
            "average_entropy": report.metrics.average_entropy,
            "total_clones": report.metrics.total_clones,
        },
        "exact_duplicates": report.exact_duplicates.len(),
        "structural_similarities": report.structural_similarities.len(),
        "semantic_similarities": report.semantic_similarities.len(),
        "refactoring_opportunities": report.refactoring_opportunities.len(),
        "report": report,
    }))
}

/// MCP tool for entropy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyAnalysisTool {
    #[serde(default = "default_project_path")]
    pub project_path: String,
    #[serde(default = "default_min_lines")]
    pub min_lines: usize,
}

/// Execute entropy analysis via MCP
pub async fn analyze_entropy(params: Value) -> Result<Value> {
    let tool: EntropyAnalysisTool = serde_json::from_value(params)?;
    
    let config = SimilarityConfig {
        min_lines: tool.min_lines,
        ..Default::default()
    };
    
    let detector = SimilarityDetector::new(config);
    
    // Collect files
    let project_path = PathBuf::from(&tool.project_path);
    let files = collect_project_files(&project_path).await?;
    
    // Analyze entropy
    let report = detector.analyze_entropy(&files);
    
    Ok(json!({
        "success": true,
        "project_path": tool.project_path,
        "files_analyzed": files.len(),
        "average_entropy": report.average_entropy,
        "high_entropy_blocks": report.high_entropy_blocks.len(),
        "low_entropy_patterns": report.low_entropy_patterns.len(),
        "recommendations": report.recommendations,
        "details": report,
    }))
}

/// MCP tool for finding refactoring opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOpportunitiesTool {
    #[serde(default = "default_project_path")]
    pub project_path: String,
}

/// Find refactoring opportunities via MCP
pub async fn find_refactoring_opportunities(params: Value) -> Result<Value> {
    let tool: RefactoringOpportunitiesTool = serde_json::from_value(params)?;
    
    let detector = SimilarityDetector::new(SimilarityConfig::default());
    
    // Collect files
    let project_path = PathBuf::from(&tool.project_path);
    let files = collect_project_files(&project_path).await?;
    
    // Find opportunities
    let opportunities = detector.find_refactoring_opportunities(&files);
    
    Ok(json!({
        "success": true,
        "project_path": tool.project_path,
        "files_analyzed": files.len(),
        "total_opportunities": opportunities.len(),
        "by_priority": {
            "high": opportunities.iter()
                .filter(|o| matches!(o.priority, crate::services::similarity::Priority::High))
                .count(),
            "medium": opportunities.iter()
                .filter(|o| matches!(o.priority, crate::services::similarity::Priority::Medium))
                .count(),
            "low": opportunities.iter()
                .filter(|o| matches!(o.priority, crate::services::similarity::Priority::Low))
                .count(),
        },
        "opportunities": opportunities,
    }))
}

/// Collect files from project directory
async fn collect_project_files(project_path: &PathBuf) -> Result<Vec<(PathBuf, String)>> {
    use walkdir::WalkDir;
    
    let mut files = Vec::new();
    
    for entry in WalkDir::new(project_path)
        .follow_links(true)
        .max_depth(10)
    {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && is_source_file(path) {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                files.push((path.to_path_buf(), content));
            }
        }
    }
    
    Ok(files)
}

fn is_source_file(path: &std::path::Path) -> bool {
    !has_excluded_directory(path) && has_source_extension(path)
}

fn has_excluded_directory(path: &std::path::Path) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if is_excluded_directory_name(name) {
                return true;
            }
        }
    }
    false
}

fn is_excluded_directory_name(name: &std::ffi::OsStr) -> bool {
    if let Some(name_str) = name.to_str() {
        name_str.starts_with('.') ||
        name_str == "target" ||
        name_str == "node_modules" ||
        name_str == "dist" ||
        name_str == "build"
    } else {
        false
    }
}

fn has_source_extension(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str(),
            Some("rs") | Some("ts") | Some("tsx") | Some("js") | Some("jsx") |
            Some("py") | Some("c") | Some("cpp") | Some("cc") | Some("h") |
            Some("hpp") | Some("kt") | Some("java") | Some("go")
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_mcp_analyze_similarity() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let file1 = temp_dir.path().join("test1.rs");
        fs::write(&file1, "fn dup() { println!(\"test\"); }\n").unwrap();
        
        let file2 = temp_dir.path().join("test2.rs");
        fs::write(&file2, "fn dup() { println!(\"test\"); }\n").unwrap();
        
        let params = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "detection_type": "exact",
            "threshold": 1.0,
            "min_lines": 1,
        });
        
        let result = analyze_similarity(params).await.unwrap();
        
        assert!(result["success"].as_bool().unwrap());
        assert!(result["files_analyzed"].as_u64().unwrap() >= 2);
    }

    #[tokio::test]
    async fn test_mcp_analyze_entropy() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test file with varying entropy
        let file = temp_dir.path().join("entropy_test.rs");
        fs::write(&file, r#"
fn repetitive() {
    if x { y }
    if x { y }
    if x { y }
}

fn complex() {
    match x {
        A(a) => process(a)?,
        B { f1, f2 } => handle(f1, f2),
        _ => default(),
    }
}
"#).unwrap();
        
        let params = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "min_lines": 3,
        });
        
        let result = analyze_entropy(params).await.unwrap();
        
        assert!(result["success"].as_bool().unwrap());
        assert!(result["average_entropy"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_mcp_find_refactoring_opportunities() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with similar patterns
        let file1 = temp_dir.path().join("pattern1.rs");
        fs::write(&file1, r#"
fn validate_email(email: &str) -> bool {
    if email.is_empty() { return false; }
    if !email.contains('@') { return false; }
    true
}
"#).unwrap();
        
        let file2 = temp_dir.path().join("pattern2.rs");
        fs::write(&file2, r#"
fn validate_phone(phone: &str) -> bool {
    if phone.is_empty() { return false; }
    if phone.len() < 10 { return false; }
    true
}
"#).unwrap();
        
        let params = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
        });
        
        let result = find_refactoring_opportunities(params).await.unwrap();
        
        assert!(result["success"].as_bool().unwrap());
        assert!(result["files_analyzed"].as_u64().unwrap() >= 2);
    }
}