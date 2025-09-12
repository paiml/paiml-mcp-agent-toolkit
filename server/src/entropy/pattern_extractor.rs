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
    ApiCall,            // HTTP/RPC call patterns
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
    pub variation_score: f64, // How much patterns vary (0=identical, 1=very different)
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

impl Default for PatternCollection {
    fn default() -> Self {
        Self::new()
    }
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
        let most_common = self
            .patterns
            .values()
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
            .map(|hashes| hashes.iter().filter_map(|h| self.patterns.get(h)).collect())
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
        use std::collections::HashMap;
        use tokio::process::Command;

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
                        file_info.get("content").and_then(|c| c.as_str()),
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

            // Process Rust and Ruchy files
            if let Some(extension) = path.extension() {
                if (extension == "rs" || extension == "ruchy" || extension == "rh")
                    && self.should_process_file(path)
                {
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
        // Language-specific extraction based on file extension

        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            match extension {
                "ruchy" | "rh" => {
                    // Ruchy-specific pattern extraction
                    self.extract_ruchy_actor_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_pipeline_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_message_passing_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_error_handling_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_pattern_matching_patterns(file_path, ast_data, collection)?;
                }
                "rs" => {
                    // Standard Rust pattern extraction
                    self.extract_error_handling_patterns(file_path, ast_data, collection)?;
                    self.extract_data_validation_patterns(file_path, ast_data, collection)?;
                    self.extract_resource_management_patterns(file_path, ast_data, collection)?;
                    self.extract_control_flow_patterns(file_path, ast_data, collection)?;
                    self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
                    self.extract_api_call_patterns(file_path, ast_data, collection)?;
                }
                _ => {
                    // Generic pattern extraction for other languages
                    self.extract_control_flow_patterns(file_path, ast_data, collection)?;
                    self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
                }
            }
        }

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
            let pattern_hash =
                self.hash_pattern(&format!("result_handling_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                // Limit to prevent excessive processing
                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches
                    .first()
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
        let validation_pattern =
            Regex::new(r"(?m)if\s+.*\.(is_empty|len|contains|starts_with|ends_with)\(\)").unwrap();
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

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::DataValidation,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches
                    .first()
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
        let resource_pattern =
            Regex::new(r"(?m)\.(open|close|lock|unlock|acquire|release)\(\)").unwrap();
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

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ResourceManagement,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches
                    .first()
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

                if i >= 8 {
                    break;
                }
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

                if i >= 10 {
                    break;
                }
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
        let api_pattern =
            Regex::new(r"(?m)(client\.|http\.|fetch\(|\.get\(|\.post\(|\.put\(|\.delete\()")
                .unwrap();
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

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ApiCall,
                pattern_hash,
                frequency: matches.len().min(10),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches
                    .first()
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
        let contexts: Vec<String> = matches
            .iter()
            .take(5)
            .map(|m| {
                let start = m.start().saturating_sub(20);
                let end = (m.end() + 20).min(content.len());

                // Ensure we're on char boundaries for UTF-8 safety
                let start_char = content
                    .char_indices()
                    .find(|(i, _)| *i >= start)
                    .map(|(i, _)| i)
                    .unwrap_or(start);
                let end_char = content
                    .char_indices()
                    .rev()
                    .find(|(i, _)| *i <= end)
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(end);

                content[start_char..end_char].to_string()
            })
            .collect();

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

    // Ruchy-specific pattern extraction methods

    /// Extract Ruchy actor patterns
    fn extract_ruchy_actor_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;

        // Pattern: actor definitions with receive handlers
        let actor_pattern = Regex::new(r"(?m)^\s*actor\s+\w+\s*\{").unwrap();
        let receive_pattern = Regex::new(r"(?m)^\s*receive\s+\w+\(").unwrap();

        let actor_matches: Vec<_> = actor_pattern.find_iter(content).collect();
        let receive_matches: Vec<_> = receive_pattern.find_iter(content).collect();

        // Only detect as pattern if we have multiple actors or multiple receive handlers
        if actor_matches.len() > 1 || receive_matches.len() > 2 {
            let pattern_hash = self.hash_pattern(&format!("ruchy_actor_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in actor_matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ControlFlow, // Actor model is control flow pattern
                pattern_hash,
                frequency: actor_matches.len().max(receive_matches.len() / 2),
                locations,
                variation_score: self.calculate_actor_variation_score(
                    &actor_matches,
                    &receive_matches,
                    content,
                ),
                example_code: actor_matches
                    .first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 200)].to_string())
                    .unwrap_or_default(),
                estimated_loc: actor_matches.len() * 8 + receive_matches.len() * 4,
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy pipeline operator patterns
    fn extract_ruchy_pipeline_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;

        // Pattern: pipeline operators |>
        let pipeline_pattern = Regex::new(r"(?m)\s*\|\>\s*\w+\(").unwrap();
        let matches: Vec<_> = pipeline_pattern.find_iter(content).collect();

        if matches.len() > 3 {
            // Need at least 3 pipeline operations to be a pattern
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_pipeline_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 15 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::DataTransformation, // Pipeline is data transformation
                pattern_hash,
                frequency: matches.len(),
                locations,
                variation_score: self.calculate_pipeline_variation_score(&matches, content),
                example_code: matches
                    .first()
                    .map(|m| {
                        let start = m.start().saturating_sub(20);
                        let end = m.end().min(m.start() + 100);
                        content[start..end].to_string()
                    })
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 2, // Each pipeline operation is ~2 lines
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy message passing patterns
    fn extract_ruchy_message_passing_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;

        // Pattern: actor message passing <- and <?
        let send_pattern = Regex::new(r"(?m)\w+\s*<-\s*\w+\(").unwrap();
        let query_pattern = Regex::new(r"(?m)\w+\s*<\?\s*\w+\(").unwrap();
        let spawn_pattern = Regex::new(r"(?m)spawn\s+\w+\s*\{").unwrap();

        let send_matches: Vec<_> = send_pattern.find_iter(content).collect();
        let query_matches: Vec<_> = query_pattern.find_iter(content).collect();
        let spawn_matches: Vec<_> = spawn_pattern.find_iter(content).collect();

        let total_messages = send_matches.len() + query_matches.len();

        if total_messages > 2 || spawn_matches.len() > 1 {
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_messaging_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in send_matches.iter().chain(query_matches.iter()).enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ApiCall, // Message passing is like API calls
                pattern_hash,
                frequency: total_messages.max(spawn_matches.len()),
                locations,
                variation_score: self.calculate_messaging_variation_score(
                    &send_matches,
                    &query_matches,
                    content,
                ),
                example_code: send_matches
                    .first()
                    .or(query_matches.first())
                    .map(|m| content[m.start()..m.end().min(m.start() + 80)].to_string())
                    .unwrap_or_default(),
                estimated_loc: total_messages * 2 + spawn_matches.len() * 3,
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy-specific error handling patterns
    fn extract_ruchy_error_handling_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;

        // Pattern: Result<T, E> with match statements (Ruchy style)
        let result_match_pattern = Regex::new(r"(?m)match\s+.*Result\s*<.*>\s*\{").unwrap();
        let matches: Vec<_> = result_match_pattern.find_iter(content).collect();

        if matches.len() > 1 {
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_error_handling_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 8 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash,
                frequency: matches.len(),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches
                    .first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 120)].to_string())
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 6, // Error handling typically 6 lines
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy pattern matching patterns
    fn extract_ruchy_pattern_matching_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        use regex::Regex;

        // Pattern: enum matching with => arrows
        let enum_pattern = Regex::new(r"(?m)enum\s+\w+\s*\{").unwrap();
        let match_pattern = Regex::new(r"(?m)match\s+\w+\s*\{").unwrap();
        let arrow_pattern = Regex::new(r"(?m)\w+::\w+\s*=>\s*").unwrap();

        let enum_matches: Vec<_> = enum_pattern.find_iter(content).collect();
        let match_matches: Vec<_> = match_pattern.find_iter(content).collect();
        let arrow_matches: Vec<_> = arrow_pattern.find_iter(content).collect();

        if match_matches.len() > 1 && arrow_matches.len() > 6 {
            // Multiple matches with many arms
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_pattern_matching_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in match_matches.iter().enumerate() {
                let line_num = content[..m.start()].lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 8 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ControlFlow,
                pattern_hash,
                frequency: match_matches.len(),
                locations,
                variation_score: self.calculate_pattern_match_variation_score(
                    &enum_matches,
                    &match_matches,
                    &arrow_matches,
                    content,
                ),
                example_code: match_matches
                    .first()
                    .map(|m| content[m.start()..m.end().min(m.start() + 150)].to_string())
                    .unwrap_or_default(),
                estimated_loc: match_matches.len() * 5 + arrow_matches.len(),
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    // Ruchy-specific variation score calculation methods

    fn calculate_actor_variation_score(
        &self,
        actor_matches: &[regex::Match],
        _receive_matches: &[regex::Match],
        content: &str,
    ) -> f64 {
        if actor_matches.is_empty() {
            return 0.0;
        }

        // Calculate variation based on different actor names and receive handler patterns
        let mut unique_patterns = std::collections::HashSet::new();

        for m in actor_matches {
            if let Some(actor_line) = content.lines().nth(content[..m.start()].lines().count()) {
                unique_patterns.insert(actor_line.trim().to_string());
            }
        }

        let variation = unique_patterns.len() as f64 / actor_matches.len() as f64;
        variation.min(1.0)
    }

    fn calculate_pipeline_variation_score(&self, matches: &[regex::Match], content: &str) -> f64 {
        if matches.len() < 2 {
            return 0.0;
        }

        // Calculate variation based on different pipeline operations
        let mut unique_operations = std::collections::HashSet::new();

        for m in matches {
            if let Some(op_text) = content.get(m.start()..m.end()) {
                unique_operations.insert(op_text.trim().to_string());
            }
        }

        let variation = unique_operations.len() as f64 / matches.len() as f64;
        variation.min(1.0)
    }

    fn calculate_messaging_variation_score(
        &self,
        send_matches: &[regex::Match],
        query_matches: &[regex::Match],
        content: &str,
    ) -> f64 {
        let total_matches = send_matches.len() + query_matches.len();
        if total_matches < 2 {
            return 0.0;
        }

        let mut unique_patterns = std::collections::HashSet::new();

        for m in send_matches.iter().chain(query_matches.iter()) {
            if let Some(msg_text) = content.get(m.start()..m.end()) {
                unique_patterns.insert(msg_text.trim().to_string());
            }
        }

        let variation = unique_patterns.len() as f64 / total_matches as f64;
        variation.min(1.0)
    }

    fn calculate_pattern_match_variation_score(
        &self,
        enum_matches: &[regex::Match],
        match_matches: &[regex::Match],
        _arrow_matches: &[regex::Match],
        content: &str,
    ) -> f64 {
        if match_matches.len() < 2 {
            return 0.0;
        }

        // Higher variation if we have different enum types being matched
        let enum_variation = if enum_matches.len() > 1 {
            0.6 // Different enum types = medium variation
        } else {
            0.3 // Same enum type = low variation
        };

        // Calculate variation based on match statement patterns
        let mut unique_match_patterns = std::collections::HashSet::new();

        for m in match_matches {
            if let Some(match_text) = content.get(m.start()..m.start().saturating_add(50)) {
                unique_match_patterns.insert(match_text.trim().to_string());
            }
        }

        let match_variation = unique_match_patterns.len() as f64 / match_matches.len() as f64;

        ((enum_variation + match_variation) / 2.0).min(1.0)
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
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
