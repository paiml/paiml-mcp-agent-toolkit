//! AST Pattern Extraction
//! 
//! Extracts patterns from AST using pmat context system

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::EntropyConfig;

/// Types of patterns we detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    ErrorHandling,      // try/catch, Result handling patterns
    DataValidation,     // Input validation patterns
    ResourceManagement, // open/close, lock/unlock patterns
    ControlFlow,        // if/else chains, match statements
    DataTransformation, // map/filter/reduce patterns
    ApiCall,           // HTTP/RPC call patterns
}

/// Location of a pattern in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

/// Represents an AST pattern found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstPattern {
    pub pattern_type: PatternType,
    pub pattern_hash: String,
    pub frequency: usize,
    pub locations: Vec<Location>,
    pub variation_score: f64,  // How much patterns vary (0=identical, 1=very different)
    pub example_code: String,
    pub estimated_loc: usize,
}

/// Collection of patterns found in project
#[derive(Debug, Clone)]
pub struct PatternCollection {
    pub patterns: HashMap<String, AstPattern>,
    pub file_patterns: HashMap<PathBuf, Vec<String>>,
    pub total_files: usize,
}

impl PatternCollection {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            file_patterns: HashMap::new(),
            total_files: 0,
        }
    }

    pub fn file_count(&self) -> usize {
        self.total_files
    }

    pub fn summary(&self) -> super::violation_detector::PatternSummary {
        // For now, return a summary based on the most common pattern
        let most_common = self.patterns.values()
            .max_by_key(|p| p.frequency)
            .cloned()
            .unwrap_or_else(|| AstPattern {
                pattern_type: PatternType::ControlFlow,
                pattern_hash: String::new(),
                frequency: 0,
                locations: vec![],
                variation_score: 0.0,
                example_code: String::new(),
                estimated_loc: 0,
            });
        
        super::violation_detector::PatternSummary {
            pattern_type: most_common.pattern_type,
            repetitions: most_common.frequency,
            variation_score: most_common.variation_score,
            example_code: most_common.example_code,
        }
    }

    pub fn add_pattern(&mut self, pattern: AstPattern) {
        let hash = pattern.pattern_hash.clone();
        self.patterns.insert(hash.clone(), pattern);
    }

    pub fn get_patterns_for_file(&self, file: &Path) -> Vec<&AstPattern> {
        self.file_patterns
            .get(file)
            .map(|hashes| {
                hashes.iter()
                    .filter_map(|h| self.patterns.get(h))
                    .collect()
            })
            .unwrap_or_default()
    }
}


/// Extracts patterns from AST
pub struct PatternExtractor {
    config: EntropyConfig,
}

impl PatternExtractor {
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Extract patterns from project using pmat context
    pub async fn extract_patterns(&self, project_path: &Path) -> Result<PatternCollection> {
        // Get project context with AST
        let context = self.get_project_context(project_path).await?;
        
        let mut collection = PatternCollection::new();
        
        // Process each file's AST
        for (file_path, ast_data) in context.files {
            if self.should_process_file(&file_path) {
                self.extract_file_patterns(&file_path, &ast_data, &mut collection)?;
                collection.total_files += 1;
            }
        }
        
        // Post-process to calculate variations
        self.calculate_pattern_variations(&mut collection);
        
        Ok(collection)
    }

    /// Get project context using pmat context command
    async fn get_project_context(&self, project_path: &Path) -> Result<ProjectContext> {
        use tokio::process::Command;
        use std::collections::HashMap;
        
        // Execute pmat context command to get actual project context
        let output = Command::new("pmat")
            .arg("context")
            .arg(project_path)
            .arg("--format")
            .arg("json")
            .arg("--skip-expensive-metrics")
            .output()
            .await?;
            
        if !output.status.success() {
            // Fall back to directory scanning if pmat context fails
            return self.scan_directory_fallback(project_path).await;
        }
        
        let context_json = String::from_utf8(output.stdout)?;
        
        // Parse the context JSON and extract file information
        let context_value: serde_json::Value = serde_json::from_str(&context_json)?;
        let mut files = HashMap::new();
        
        // Extract file contents from context
        if let Some(file_tree) = context_value.get("files") {
            if let Some(file_array) = file_tree.as_array() {
                for file_info in file_array {
                    if let (Some(path), Some(content)) = (
                        file_info.get("path").and_then(|p| p.as_str()),
                        file_info.get("content").and_then(|c| c.as_str())
                    ) {
                        let path_buf = PathBuf::from(path);
                        files.insert(path_buf, content.to_string());
                    }
                }
            }
        }
        
        Ok(ProjectContext { files })
    }
    
    /// Fallback method to scan directory when pmat context fails
    async fn scan_directory_fallback(&self, project_path: &Path) -> Result<ProjectContext> {
        use std::fs;
        use walkdir::WalkDir;
        
        let mut files = HashMap::new();
        
        // Walk directory and read Rust files
        for entry in WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Only process Rust files
            if let Some(extension) = path.extension() {
                if extension == "rs" && self.should_process_file(path) {
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            files.insert(path.to_path_buf(), content);
                        }
                        Err(_) => continue, // Skip files we can't read
                    }
                }
            }
        }
        
        Ok(ProjectContext { files })
    }

    /// Check if file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        !self.config.exclude_paths.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
        })
    }

    /// Extract patterns from a single file's AST
    fn extract_file_patterns(
        &self,
        file_path: &Path,
        ast_data: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        // Extract patterns using regex-based AST pattern matching
        // This could be enhanced with full syn parsing in the future
        
        self.extract_error_handling_patterns(file_path, ast_data, collection)?;
        self.extract_data_validation_patterns(file_path, ast_data, collection)?;
        self.extract_resource_management_patterns(file_path, ast_data, collection)?;
        self.extract_control_flow_patterns(file_path, ast_data, collection)?;
        self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
        self.extract_api_call_patterns(file_path, ast_data, collection)?;
        
        Ok(())
    }
    
    /// Extract error handling patterns
    fn extract_error_handling_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;
        
        // Pattern: Result<T, E> handling
        let result_pattern = Regex::new(r"(?m)^\s*(match|if let)\s+.*Result\s*<.*>\s*\{").unwrap();
        let matches: Vec<_> = result_pattern.find_iter(content).collect();
        
        if matches.len() > 1 {
            let pattern_hash = self.hash_pattern(&format!("result_handling_{}", file_path.display()));
            let mut locations = Vec::new();
            
            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });
                
                // Limit to prevent excessive processing
                if i >= 10 { break; }
            }
            
            let pattern = AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches.first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 100)].to_string())
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 5, // Estimate 5 lines per match
            };
            
            collection.add_pattern(pattern);
        }
        
        Ok(())
    }
    
    /// Extract data validation patterns  
    fn extract_data_validation_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;
        
        // Pattern: Input validation (is_empty, len, contains checks)
        let validation_pattern = Regex::new(r"(?m)if\s+.*\.(is_empty|len|contains|starts_with|ends_with)\(\)").unwrap();
        let matches: Vec<_> = validation_pattern.find_iter(content).collect();
        
        if matches.len() > 2 {
            let pattern_hash = self.hash_pattern(&format!("validation_{}", file_path.display()));
            let mut locations = Vec::new();
            
            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });
                
                if i >= 10 { break; }
            }
            
            let pattern = AstPattern {
                pattern_type: PatternType::DataValidation,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches.first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 80)].to_string())
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 3,
            };
            
            collection.add_pattern(pattern);
        }
        
        Ok(())
    }
    
    /// Extract resource management patterns
    fn extract_resource_management_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;
        
        // Pattern: File/resource management (open/close, lock/unlock)
        let resource_pattern = Regex::new(r"(?m)\.(open|close|lock|unlock|acquire|release)\(\)").unwrap();
        let matches: Vec<_> = resource_pattern.find_iter(content).collect();
        
        if matches.len() > 1 {
            let pattern_hash = self.hash_pattern(&format!("resource_{}", file_path.display()));
            let mut locations = Vec::new();
            
            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });
                
                if i >= 10 { break; }
            }
            
            let pattern = AstPattern {
                pattern_type: PatternType::ResourceManagement,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches.first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 60)].to_string())
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 4,
            };
            
            collection.add_pattern(pattern);
        }
        
        Ok(())
    }
    
    /// Extract control flow patterns
    fn extract_control_flow_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;
        
        // Pattern: Complex if-else chains
        let if_else_pattern = Regex::new(r"(?m)^\s*}\s*else\s+if\s+").unwrap();
        let matches: Vec<_> = if_else_pattern.find_iter(content).collect();
        
        if matches.len() > 2 {
            let pattern_hash = self.hash_pattern(&format!("control_flow_{}", file_path.display()));
            let mut locations = Vec::new();
            
            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });
                
                if i >= 8 { break; }
            }
            
            let pattern = AstPattern {
                pattern_type: PatternType::ControlFlow,
                pattern_hash,
                frequency: matches.len().min(8),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: "if-else-if chains".to_string(),
                estimated_loc: matches.len() * 6,
            };
            
            collection.add_pattern(pattern);
        }
        
        Ok(())
    }
    
    /// Extract data transformation patterns
    fn extract_data_transformation_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;
        
        // Pattern: Iterator chains (map, filter, collect)
        let iter_pattern = Regex::new(r"\.(map|filter|collect|fold|reduce)\(").unwrap();
        let matches: Vec<_> = iter_pattern.find_iter(content).collect();
        
        if matches.len() > 3 {
            let pattern_hash = self.hash_pattern(&format!("transform_{}", file_path.display()));
            let mut locations = Vec::new();
            
            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });
                
                if i >= 10 { break; }
            }
            
            let pattern = AstPattern {
                pattern_type: PatternType::DataTransformation,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: "iterator transformations".to_string(),
                estimated_loc: matches.len() * 2,
            };
            
            collection.add_pattern(pattern);
        }
        
        Ok(())
    }
    
    /// Extract API call patterns
    fn extract_api_call_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;
        
        // Pattern: HTTP/API calls (reqwest, fetch, etc.)
        let api_pattern = Regex::new(r"(?m)(client\.|http\.|fetch\(|\.get\(|\.post\(|\.put\(|\.delete\()").unwrap();
        let matches: Vec<_> = api_pattern.find_iter(content).collect();
        
        if matches.len() > 1 {
            let pattern_hash = self.hash_pattern(&format!("api_call_{}", file_path.display()));
            let mut locations = Vec::new();
            
            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });
                
                if i >= 10 { break; }
            }
            
            let pattern = AstPattern {
                pattern_type: PatternType::ApiCall,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches.first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 50)].to_string())
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 3,
            };
            
            collection.add_pattern(pattern);
        }
        
        Ok(())
    }
    
    /// Calculate variation score for pattern matches
    fn calculate_variation_score(&self, matches: &[regex::Match], content: &str) -> f64 {
        if matches.len() <= 1 {
            return 0.0;
        }
        
        // Simple variation calculation based on context differences
        let contexts: Vec<String> = matches.iter().take(5).map(|m| {
            let start = m.start().saturating_sub(20);
            let end = (m.end() + 20).min(content.len());
            content[start..end].to_string()
        }).collect();
        
        // Calculate similarity between contexts
        let mut total_similarity = 0.0;
        let mut comparisons = 0;
        
        for i in 0..contexts.len() {
            for j in (i + 1)..contexts.len() {
                let similarity = self.calculate_string_similarity(&contexts[i], &contexts[j]);
                total_similarity += similarity;
                comparisons += 1;
            }
        }
        
        if comparisons > 0 {
            1.0 - (total_similarity / comparisons as f64) // Higher variation = less similarity
        } else {
            0.0
        }
    }
    
    /// Calculate string similarity (simplified Jaccard similarity)
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = s1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = s2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Calculate how much patterns vary from each other
    fn calculate_pattern_variations(&self, collection: &mut PatternCollection) {
        // Compare similar patterns and calculate variation scores
        for pattern in collection.patterns.values_mut() {
            if pattern.locations.len() > 1 {
                // Simplified: more locations = more variation
                pattern.variation_score = (pattern.locations.len() as f64 / 10.0).min(1.0);
            }
        }
    }

    /// Create a hash for a pattern to identify similar ones
    fn hash_pattern(&self, ast_data: &str) -> String {
        // Simplified hashing - real implementation would normalize AST first
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        ast_data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Temporary struct - will be replaced with actual context from pmat
#[derive(Debug)]
struct ProjectContext {
    files: HashMap<PathBuf, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_type_equality() {
        assert_eq!(PatternType::ErrorHandling, PatternType::ErrorHandling);
        assert_ne!(PatternType::ErrorHandling, PatternType::DataValidation);
    }

    #[test]
    fn test_pattern_collection() {
        let mut collection = PatternCollection::new();
        assert_eq!(collection.file_count(), 0);
        
        let pattern = AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "test123".to_string(),
            frequency: 3,
            locations: vec![],
            variation_score: 0.0,
            example_code: "test".to_string(),
            estimated_loc: 10,
        };
        
        collection.add_pattern(pattern);
        let summary = collection.summary();
        assert_eq!(summary.repetitions, 3);
        assert_eq!(summary.pattern_type, PatternType::ErrorHandling);
    }
}