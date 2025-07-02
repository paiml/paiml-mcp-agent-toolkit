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

/// Main entry point for duplicate analysis
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
    )
    .await?;

    eprintln!("✅ Found {} duplicate blocks", report.total_duplicates);
    eprintln!(
        "📊 Duplication: {:.1}% ({} / {} lines)",
        report.duplication_percentage, report.duplicate_lines, report.total_lines
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

/// Detect duplicate code blocks
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
                let blocks =
                    extract_blocks(&lines, path, min_lines, max_tokens, detection_type.clone());
                all_blocks.extend(blocks);

                file_stats.insert(
                    path.to_string_lossy().to_string(),
                    FileStats {
                        duplicate_lines: 0,
                        total_lines: lines.len(),
                        duplication_percentage: 0.0,
                    },
                );
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
            stats.duplication_percentage =
                (stats.duplicate_lines as f32 / stats.total_lines as f32) * 100.0;
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

/// Extract code blocks from lines
fn extract_blocks(
    lines: &[&str],
    path: &Path,
    min_lines: usize,
    max_tokens: usize,
    detection_type: crate::cli::DuplicateType,
) -> Vec<(String, String, usize, usize, String)> {
    let mut blocks = Vec::new();
    let file_str = path.to_string_lossy().to_string();

    match detection_type {
        crate::cli::DuplicateType::Exact => {
            extract_exact_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        crate::cli::DuplicateType::Fuzzy => {
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        _ => {} // Structural matching not implemented yet
    }

    blocks
}

/// Extract exact match blocks using sliding window
fn extract_exact_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Sliding window for exact matches
    for i in 0..lines.len().saturating_sub(min_lines) {
        let block_lines = &lines[i..i + min_lines];
        let content = normalize_block(block_lines);

        if count_tokens(&content) <= max_tokens {
            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher);
            let hash = format!("{:x}", hasher.finish());

            blocks.push((hash, file_str.to_string(), i + 1, i + min_lines, content));
        }
    }
}

/// Extract fuzzy match blocks based on code structure
fn extract_fuzzy_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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

                    blocks.push((hash, file_str.to_string(), i + 1, end, content));
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
}

/// Normalize code block (remove whitespace variations)
fn normalize_block(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Count tokens in content
fn count_tokens(content: &str) -> usize {
    content.split_whitespace().count()
}

/// Check if line starts a code block - refactored to reduce complexity
fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim();

    // Check for function/method declarations
    if is_function_declaration(trimmed) {
        return true;
    }

    // Check for class/type declarations
    if is_type_declaration(trimmed) {
        return true;
    }

    // Check for block opening
    if is_block_opening(trimmed) {
        return true;
    }

    false
}

/// Check if line is a function declaration
fn is_function_declaration(line: &str) -> bool {
    line.contains("fn ") || line.contains("function") || line.contains("def ")
}

/// Check if line is a type declaration
fn is_type_declaration(line: &str) -> bool {
    line.contains("class ") || line.contains("struct ") || line.contains("impl ")
}

/// Check if line is a block opening
fn is_block_opening(line: &str) -> bool {
    line.ends_with('{') && !line.starts_with('{')
}

/// Find end of code block
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

/// Find duplicate blocks from all blocks
fn find_duplicate_blocks(
    all_blocks: Vec<(String, String, usize, usize, String)>,
    _threshold: f32,
) -> Vec<DuplicateBlock> {
    let mut hash_groups: HashMap<String, Vec<(String, usize, usize, String)>> = HashMap::new();

    // Group by hash
    for (hash, file, start, end, content) in all_blocks {
        hash_groups
            .entry(hash)
            .or_default()
            .push((file, start, end, content));
    }

    // Find duplicates
    let mut duplicates = Vec::new();
    for (hash, locations) in hash_groups {
        if locations.len() > 1 {
            let lines = locations[0].2 - locations[0].1 + 1;
            let tokens = count_tokens(&locations[0].3);

            let duplicate_locations: Vec<DuplicateLocation> = locations
                .into_iter()
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

/// Check if file should be processed
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

/// Check if file is source code
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "kt" | "kts")
    )
}

/// Format output based on format type
fn format_output(
    report: &DuplicateReport,
    format: crate::cli::DuplicateOutputFormat,
) -> Result<String> {
    match format {
        crate::cli::DuplicateOutputFormat::Json => format_json_output(report),
        crate::cli::DuplicateOutputFormat::Human => format_human_output(report),
        crate::cli::DuplicateOutputFormat::Sarif => format_sarif_output(report),
        _ => Ok("Duplicate analysis completed.".to_string()),
    }
}

/// Format output as JSON
fn format_json_output(report: &DuplicateReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

/// Format output for human reading
fn format_human_output(report: &DuplicateReport) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(&mut output, "# Duplicate Code Analysis\n")?;

    writeln!(&mut output, "## Summary")?;
    writeln!(
        &mut output,
        "- Total duplicate blocks: {}",
        report.total_duplicates
    )?;
    writeln!(
        &mut output,
        "- Duplicate lines: {} / {}",
        report.duplicate_lines, report.total_lines
    )?;
    writeln!(
        &mut output,
        "- Duplication percentage: {:.1}%\n",
        report.duplication_percentage
    )?;

    if !report.duplicate_blocks.is_empty() {
        writeln!(&mut output, "## Duplicate Blocks\n")?;
        for (i, block) in report.duplicate_blocks.iter().enumerate().take(20) {
            writeln!(
                &mut output,
                "### Block {} ({} lines, {} locations)",
                i + 1,
                block.lines,
                block.locations.len()
            )?;
            for loc in &block.locations {
                writeln!(
                    &mut output,
                    "- {}:{}-{}",
                    loc.file, loc.start_line, loc.end_line
                )?;
            }
            writeln!(&mut output, "\nPreview:")?;
            writeln!(&mut output, "```")?;
            writeln!(&mut output, "{}", block.locations[0].content_preview)?;
            writeln!(&mut output, "```\n")?;
        }

        if report.duplicate_blocks.len() > 20 {
            writeln!(
                &mut output,
                "... and {} more blocks",
                report.duplicate_blocks.len() - 20
            )?;
        }
    }

    Ok(output)
}

/// Format output as SARIF
fn format_sarif_output(report: &DuplicateReport) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_normalize_block() {
        let lines = vec!["  fn test() {", "    // comment", "    let x = 1;", "  }"];
        let normalized = normalize_block(&lines);
        assert!(!normalized.contains("// comment"));
        assert!(normalized.contains("fn test()"));
        assert_eq!(normalized, "fn test() {\nlet x = 1;\n}");
    }

    #[test]
    fn test_count_tokens() {
        assert_eq!(count_tokens("fn test() { }"), 4);
        assert_eq!(count_tokens("let x = 1;"), 4);
        assert_eq!(count_tokens(""), 0);
        assert_eq!(count_tokens("  \n  \t  "), 0);
    }

    #[test]
    fn test_is_function_declaration() {
        assert!(is_function_declaration("fn main() {"));
        assert!(is_function_declaration("function test() {"));
        assert!(is_function_declaration("def calculate():"));
        assert!(!is_function_declaration("let x = 1;"));
    }

    #[test]
    fn test_is_type_declaration() {
        assert!(is_type_declaration("class Foo {"));
        assert!(is_type_declaration("struct Bar {"));
        assert!(is_type_declaration("impl Display for Foo {"));
        assert!(!is_type_declaration("let x = 1;"));
    }

    #[test]
    fn test_is_block_opening() {
        assert!(is_block_opening("fn main() {"));
        assert!(is_block_opening("if true {"));
        assert!(!is_block_opening("{ x: 1 }"));
        assert!(!is_block_opening("let x = 1;"));
    }

    #[test]
    fn test_is_block_start() {
        // Function declarations
        assert!(is_block_start("fn main() {"));
        assert!(is_block_start("function test() {"));
        assert!(is_block_start("def calculate():"));

        // Type declarations
        assert!(is_block_start("class Foo {"));
        assert!(is_block_start("struct Bar {"));
        assert!(is_block_start("impl Display for Foo {"));

        // Block openings
        assert!(is_block_start("if condition {"));

        // Not block starts
        assert!(!is_block_start("let x = 1;"));
        assert!(!is_block_start("{ x: 1 }"));
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.rs")));
        assert!(is_source_file(Path::new("test.js")));
        assert!(is_source_file(Path::new("test.ts")));
        assert!(is_source_file(Path::new("test.py")));
        assert!(is_source_file(Path::new("test.java")));
        assert!(is_source_file(Path::new("test.cpp")));
        assert!(is_source_file(Path::new("test.c")));
        assert!(is_source_file(Path::new("test.kt")));
        assert!(is_source_file(Path::new("test.kts")));
        assert!(!is_source_file(Path::new("test.txt")));
        assert!(!is_source_file(Path::new("README.md")));
    }

    #[test]
    fn test_should_process_file() {
        let path = Path::new("src/main.rs");

        // No filters
        assert!(should_process_file(path, &None, &None));

        // Include filter
        assert!(should_process_file(path, &Some("src".to_string()), &None));
        assert!(!should_process_file(
            path,
            &Some("tests".to_string()),
            &None
        ));

        // Exclude filter
        assert!(!should_process_file(path, &None, &Some("src".to_string())));
        assert!(should_process_file(path, &None, &Some("tests".to_string())));

        // Both filters (exclude takes precedence)
        assert!(!should_process_file(
            path,
            &Some("src".to_string()),
            &Some("src".to_string())
        ));
    }

    #[test]
    fn test_find_block_end() {
        let lines = vec![
            "fn test() {",
            "    let x = 1;",
            "    if true {",
            "        println!(\"hello\");",
            "    }",
            "}",
        ];

        assert_eq!(find_block_end(&lines), Some(6));

        let lines2 = vec!["fn test() {", "    let x = 1;"];
        assert_eq!(find_block_end(&lines2), None);
    }

    #[test]
    fn test_extract_exact_blocks() {
        let lines = vec![
            "fn test1() {",
            "    let x = 1;",
            "    println!(\"x = {}\", x);",
            "}",
            "",
            "fn test2() {",
            "    let y = 2;",
            "    println!(\"y = {}\", y);",
            "}",
        ];

        let mut blocks = Vec::new();
        extract_exact_blocks(&mut blocks, &lines, "test.rs", 3, 100);

        // Should find multiple sliding windows
        assert!(!blocks.is_empty());
        assert!(blocks.iter().all(|(_, file, _, _, _)| file == "test.rs"));
    }

    #[test]
    fn test_find_duplicate_blocks_no_duplicates() {
        let blocks = vec![
            (
                "hash1".to_string(),
                "file1.rs".to_string(),
                1,
                10,
                "content1".to_string(),
            ),
            (
                "hash2".to_string(),
                "file2.rs".to_string(),
                1,
                10,
                "content2".to_string(),
            ),
        ];

        let duplicates = find_duplicate_blocks(blocks, 0.8);
        assert!(duplicates.is_empty());
    }

    #[test]
    fn test_find_duplicate_blocks_with_duplicates() {
        let blocks = vec![
            (
                "hash1".to_string(),
                "file1.rs".to_string(),
                1,
                10,
                "content1".to_string(),
            ),
            (
                "hash1".to_string(),
                "file2.rs".to_string(),
                20,
                29,
                "content1".to_string(),
            ),
            (
                "hash2".to_string(),
                "file3.rs".to_string(),
                1,
                5,
                "content2".to_string(),
            ),
        ];

        let duplicates = find_duplicate_blocks(blocks, 0.8);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].hash, "hash1");
        assert_eq!(duplicates[0].locations.len(), 2);
        assert_eq!(duplicates[0].lines, 10);
    }

    #[test]
    fn test_file_stats_calculation() {
        let mut stats = FileStats {
            duplicate_lines: 20,
            total_lines: 100,
            duplication_percentage: 0.0,
        };

        // Calculate percentage
        stats.duplication_percentage =
            (stats.duplicate_lines as f32 / stats.total_lines as f32) * 100.0;
        assert_eq!(stats.duplication_percentage, 20.0);
    }

    #[tokio::test]
    async fn test_detect_duplicates_empty_project() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = detect_duplicates(
            temp_dir.path(),
            crate::cli::DuplicateType::Exact,
            0.8,
            5,
            100,
            &None,
            &None,
        )
        .await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.total_duplicates, 0);
        assert_eq!(report.duplicate_lines, 0);
        assert_eq!(report.total_lines, 0);
        assert_eq!(report.duplication_percentage, 0.0);
    }

    #[test]
    fn test_format_json_output() {
        let report = DuplicateReport {
            total_duplicates: 1,
            duplicate_lines: 10,
            total_lines: 100,
            duplication_percentage: 10.0,
            duplicate_blocks: vec![],
            file_statistics: HashMap::new(),
        };

        let result = format_json_output(&report);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"total_duplicates\": 1"));
        assert!(json.contains("\"duplication_percentage\": 10.0"));
    }

    #[test]
    fn test_format_human_output() {
        let report = DuplicateReport {
            total_duplicates: 2,
            duplicate_lines: 20,
            total_lines: 100,
            duplication_percentage: 20.0,
            duplicate_blocks: vec![DuplicateBlock {
                hash: "hash1".to_string(),
                locations: vec![
                    DuplicateLocation {
                        file: "file1.rs".to_string(),
                        start_line: 10,
                        end_line: 20,
                        content_preview: "fn test() {".to_string(),
                    },
                    DuplicateLocation {
                        file: "file2.rs".to_string(),
                        start_line: 30,
                        end_line: 40,
                        content_preview: "fn test() {".to_string(),
                    },
                ],
                lines: 10,
                tokens: 20,
                similarity: 1.0,
            }],
            file_statistics: HashMap::new(),
        };

        let result = format_human_output(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("# Duplicate Code Analysis"));
        assert!(output.contains("Total duplicate blocks: 2"));
        assert!(output.contains("Block 1 (10 lines, 2 locations)"));
    }
}
