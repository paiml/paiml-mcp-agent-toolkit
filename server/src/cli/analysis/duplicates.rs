//! Duplicate detection analysis - finds duplicate code blocks

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateBlock {
    pub hash: String,
    pub locations: Vec<DuplicateLocation>,
    pub lines: usize,
    pub tokens: usize,
    pub similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateLocation {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content_preview: String,
}

#[derive(Debug, Serialize)]
pub struct DuplicateReport {
    pub total_duplicates: usize,
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_percentage: f32,
    pub duplicate_blocks: Vec<DuplicateBlock>,
    pub file_statistics: HashMap<String, FileStats>,
}

#[derive(Debug, Serialize)]
pub struct FileStats {
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_percentage: f32,
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_duplicates(
    project_path: PathBuf,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    format: crate::cli::DuplicateOutputFormat,
    _perf: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    eprintln!("🔍 Detecting duplicate code blocks...");
    
    // Find duplicate blocks
    let report = detect_duplicates(
        &project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        &include,
        &exclude,
    ).await?;
    
    eprintln!("✅ Found {} duplicate blocks", report.total_duplicates);
    eprintln!("📊 Duplication: {:.1}% ({} / {} lines)", 
        report.duplication_percentage,
        report.duplicate_lines,
        report.total_lines
    );
    
    // Format output
    let content = format_output(&report, format)?;
    
    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }
    
    Ok(())
}

// Detect duplicate code blocks
async fn detect_duplicates(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<DuplicateReport> {
    use walkdir::WalkDir;
    
    let mut all_blocks = Vec::new();
    let mut total_lines = 0usize;
    let mut file_stats = HashMap::new();
    
    // Collect code blocks from all files
    for entry in WalkDir::new(project_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && is_source_file(path) && should_process_file(path, include, exclude) {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                let lines: Vec<&str> = content.lines().collect();
                total_lines += lines.len();
                
                // Extract blocks based on detection type
                let blocks = extract_blocks(&lines, path, min_lines, max_tokens, detection_type.clone());
                all_blocks.extend(blocks);
                
                file_stats.insert(path.to_string_lossy().to_string(), FileStats {
                    duplicate_lines: 0,
                    total_lines: lines.len(),
                    duplication_percentage: 0.0,
                });
            }
        }
    }
    
    // Find duplicates
    let duplicate_blocks = find_duplicate_blocks(all_blocks, threshold);
    
    // Calculate statistics
    let mut duplicate_lines = 0;
    for block in &duplicate_blocks {
        duplicate_lines += block.lines * block.locations.len();
        
        // Update file statistics
        for loc in &block.locations {
            if let Some(stats) = file_stats.get_mut(&loc.file) {
                stats.duplicate_lines += block.lines;
            }
        }
    }
    
    // Update duplication percentages
    for stats in file_stats.values_mut() {
        if stats.total_lines > 0 {
            stats.duplication_percentage = (stats.duplicate_lines as f32 / stats.total_lines as f32) * 100.0;
        }
    }
    
    let duplication_percentage = if total_lines > 0 {
        (duplicate_lines as f32 / total_lines as f32) * 100.0
    } else {
        0.0
    };
    
    Ok(DuplicateReport {
        total_duplicates: duplicate_blocks.len(),
        duplicate_lines,
        total_lines,
        duplication_percentage,
        duplicate_blocks,
        file_statistics: file_stats,
    })
}

// Extract code blocks from lines
fn extract_blocks(
    lines: &[&str],
    path: &Path,
    min_lines: usize,
    max_tokens: usize,
    detection_type: crate::cli::DuplicateType,
) -> Vec<(String, String, usize, usize, String)> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut blocks = Vec::new();
    let file_str = path.to_string_lossy().to_string();
    
    match detection_type {
        crate::cli::DuplicateType::Exact => {
            // Sliding window for exact matches
            for i in 0..lines.len().saturating_sub(min_lines) {
                let block_lines = &lines[i..i + min_lines];
                let content = normalize_block(block_lines);
                
                if count_tokens(&content) <= max_tokens {
                    let mut hasher = DefaultHasher::new();
                    content.hash(&mut hasher);
                    let hash = format!("{:x}", hasher.finish());
                    
                    blocks.push((
                        hash,
                        file_str.clone(),
                        i + 1,
                        i + min_lines,
                        content,
                    ));
                }
            }
        }
        crate::cli::DuplicateType::Fuzzy => {
            // Extract function/method blocks for fuzzy matching
            let mut i = 0;
            while i < lines.len() {
                if is_block_start(lines[i]) {
                    let end = find_block_end(&lines[i..]).unwrap_or(min_lines) + i;
                    if end - i >= min_lines {
                        let block_lines = &lines[i..end];
                        let content = normalize_block(block_lines);
                        
                        if count_tokens(&content) <= max_tokens {
                            let mut hasher = DefaultHasher::new();
                            content.hash(&mut hasher);
                            let hash = format!("{:x}", hasher.finish());
                            
                            blocks.push((
                                hash,
                                file_str.clone(),
                                i + 1,
                                end,
                                content,
                            ));
                        }
                    }
                    i = end;
                } else {
                    i += 1;
                }
            }
        }
        _ => {} // Structural matching not implemented yet
    }
    
    blocks
}

// Normalize code block (remove whitespace variations)
fn normalize_block(lines: &[&str]) -> String {
    lines.iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

// Count tokens in content
fn count_tokens(content: &str) -> usize {
    content.split_whitespace().count()
}

// Check if line starts a code block
fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains("fn ") || 
    trimmed.contains("function") || 
    trimmed.contains("def ") ||
    trimmed.contains("class ") ||
    trimmed.contains("impl ") ||
    (trimmed.ends_with('{') && !trimmed.starts_with('{'))
}

// Find end of code block
fn find_block_end(lines: &[&str]) -> Option<usize> {
    let mut brace_count = 0;
    let mut in_block = false;
    
    for (i, line) in lines.iter().enumerate() {
        for ch in line.chars() {
            match ch {
                '{' => {
                    brace_count += 1;
                    in_block = true;
                }
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 && in_block {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
        }
    }
    
    None
}

// Find duplicate blocks from all blocks
fn find_duplicate_blocks(
    all_blocks: Vec<(String, String, usize, usize, String)>,
    _threshold: f32,
) -> Vec<DuplicateBlock> {
    let mut hash_groups: HashMap<String, Vec<(String, usize, usize, String)>> = HashMap::new();
    
    // Group by hash
    for (hash, file, start, end, content) in all_blocks {
        hash_groups.entry(hash).or_default().push((file, start, end, content));
    }
    
    // Find duplicates
    let mut duplicates = Vec::new();
    for (hash, locations) in hash_groups {
        if locations.len() > 1 {
            let lines = locations[0].2 - locations[0].1 + 1;
            let tokens = count_tokens(&locations[0].3);
            
            let duplicate_locations: Vec<DuplicateLocation> = locations.into_iter()
                .map(|(file, start, end, content)| {
                    let preview = content.lines().take(3).collect::<Vec<_>>().join("\n");
                    DuplicateLocation {
                        file,
                        start_line: start,
                        end_line: end,
                        content_preview: if content.lines().count() > 3 {
                            format!("{}...", preview)
                        } else {
                            preview
                        },
                    }
                })
                .collect();
            
            duplicates.push(DuplicateBlock {
                hash,
                locations: duplicate_locations,
                lines,
                tokens,
                similarity: 1.0, // Exact match for now
            });
        }
    }
    
    // Sort by lines descending
    duplicates.sort_by(|a, b| b.lines.cmp(&a.lines));
    
    duplicates
}

// Check if file should be processed
fn should_process_file(path: &Path, include: &Option<String>, exclude: &Option<String>) -> bool {
    let path_str = path.to_string_lossy();
    
    if let Some(excl) = exclude {
        if path_str.contains(excl) {
            return false;
        }
    }
    
    if let Some(incl) = include {
        return path_str.contains(incl);
    }
    
    true
}

// Check if file is source code
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs") | Some("js") | Some("ts") | Some("py") | Some("java") | Some("cpp") | Some("c")
    )
}

// Format output based on format type
fn format_output(report: &DuplicateReport, format: crate::cli::DuplicateOutputFormat) -> Result<String> {
    use std::fmt::Write;
    
    match format {
        crate::cli::DuplicateOutputFormat::Json => {
            Ok(serde_json::to_string_pretty(report)?)
        }
        crate::cli::DuplicateOutputFormat::Human => {
            let mut output = String::new();
            writeln!(&mut output, "# Duplicate Code Analysis\n")?;
            
            writeln!(&mut output, "## Summary")?;
            writeln!(&mut output, "- Total duplicate blocks: {}", report.total_duplicates)?;
            writeln!(&mut output, "- Duplicate lines: {} / {}", report.duplicate_lines, report.total_lines)?;
            writeln!(&mut output, "- Duplication percentage: {:.1}%\n", report.duplication_percentage)?;
            
            if !report.duplicate_blocks.is_empty() {
                writeln!(&mut output, "## Duplicate Blocks\n")?;
                for (i, block) in report.duplicate_blocks.iter().enumerate().take(20) {
                    writeln!(&mut output, "### Block {} ({} lines, {} locations)", i + 1, block.lines, block.locations.len())?;
                    for loc in &block.locations {
                        writeln!(&mut output, "- {}:{}-{}", loc.file, loc.start_line, loc.end_line)?;
                    }
                    writeln!(&mut output, "\nPreview:")?;
                    writeln!(&mut output, "```")?;
                    writeln!(&mut output, "{}", block.locations[0].content_preview)?;
                    writeln!(&mut output, "```\n")?;
                }
                
                if report.duplicate_blocks.len() > 20 {
                    writeln!(&mut output, "... and {} more blocks", report.duplicate_blocks.len() - 20)?;
                }
            }
            
            Ok(output)
        }
        crate::cli::DuplicateOutputFormat::Sarif => {
            // SARIF format for CI/CD integration
            let sarif = serde_json::json!({
                "version": "2.1.0",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "pmat-duplicates",
                            "version": "1.0.0"
                        }
                    },
                    "results": report.duplicate_blocks.iter().map(|block| {
                        serde_json::json!({
                            "ruleId": "duplicate-code",
                            "level": "warning",
                            "message": {
                                "text": format!("Duplicate code block found ({} lines)", block.lines)
                            },
                            "locations": block.locations.iter().map(|loc| {
                                serde_json::json!({
                                    "physicalLocation": {
                                        "artifactLocation": {
                                            "uri": loc.file
                                        },
                                        "region": {
                                            "startLine": loc.start_line,
                                            "endLine": loc.end_line
                                        }
                                    }
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>()
                }]
            });
            
            Ok(serde_json::to_string_pretty(&sarif)?)
        }
        _ => Ok("Duplicate analysis completed.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_block() {
        let lines = vec![
            "  fn test() {",
            "    // comment",
            "    let x = 1;",
            "  }",
        ];
        let normalized = normalize_block(&lines);
        assert!(!normalized.contains("// comment"));
        assert!(normalized.contains("fn test()"));
    }
    
    #[test]
    fn test_count_tokens() {
        assert_eq!(count_tokens("fn test() { }"), 4);
        assert_eq!(count_tokens("let x = 1;"), 4);
    }
    
    #[test]
    fn test_is_block_start() {
        assert!(is_block_start("fn main() {"));
        assert!(is_block_start("function test() {"));
        assert!(is_block_start("class Foo {"));
        assert!(!is_block_start("let x = 1;"));
    }
    
    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.rs")));
        assert!(is_source_file(Path::new("test.js")));
        assert!(!is_source_file(Path::new("test.txt")));
    }
}
