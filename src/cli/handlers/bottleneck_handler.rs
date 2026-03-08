//! Architectural bottleneck detection handler
//!
//! Analyzes git history to find files with disproportionate churn
//! that indicate registry/dispatch architectural bottlenecks.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// A detected bottleneck file
#[derive(Debug, serde::Serialize)]
struct BottleneckFile {
    path: String,
    touches: usize,
    authors: usize,
    lines: usize,
    churn_ratio: f64,
    pattern: String,
    recommendation: String,
}

/// Co-change coupling between files
#[derive(Debug, serde::Serialize)]
struct CouplingPair {
    file_a: String,
    file_b: String,
    co_changes: usize,
}

/// Full bottleneck analysis result
#[derive(Debug, serde::Serialize)]
struct BottleneckAnalysis {
    period_days: u32,
    total_commits: usize,
    total_files_changed: usize,
    bottlenecks: Vec<BottleneckFile>,
    couplings: Vec<CouplingPair>,
}

/// Handle the bottleneck analysis command
pub async fn handle_bottleneck(
    path: &Path,
    format: &crate::cli::enums::OutputFormat,
    period: u32,
    threshold: usize,
    output: Option<&Path>,
) -> Result<()> {
    use crate::cli::colors as c;

    eprintln!(
        "{}",
        c::dim(&format!("Analyzing git churn for last {} days...", period))
    );

    let analysis = analyze_bottlenecks(path, period, threshold)?;

    let formatted = match format {
        crate::cli::enums::OutputFormat::Json => serde_json::to_string_pretty(&analysis)?,
        _ => format_text(&analysis),
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        eprintln!(
            "{} Written to: {}",
            c::pass(""),
            c::path(&output_path.display().to_string())
        );
    } else {
        println!("{formatted}");
    }

    Ok(())
}

/// Main analysis function
fn analyze_bottlenecks(path: &Path, period: u32, threshold: usize) -> Result<BottleneckAnalysis> {
    // Get per-file touch counts from git log
    let (file_touches, commit_files, total_commits) = get_git_churn(path, period)?;

    // Get file sizes
    let file_sizes = get_file_sizes(path, &file_touches)?;

    // Get author counts per file
    let file_authors = get_file_authors(path, period, &file_touches)?;

    // Build bottleneck list
    let mut bottlenecks: Vec<BottleneckFile> = file_touches
        .iter()
        .filter(|(_, &count)| count >= threshold)
        .filter(|(path, _)| !is_generated_file(path))
        .map(|(file_path, &touches)| {
            let lines = file_sizes.get(file_path.as_str()).copied().unwrap_or(0);
            let authors = file_authors.get(file_path.as_str()).copied().unwrap_or(1);
            let churn_ratio = if lines > 0 {
                touches as f64 / (lines as f64 / 100.0)
            } else {
                touches as f64
            };
            let pattern = classify_pattern(file_path, touches, lines);
            let recommendation = get_recommendation(&pattern);

            BottleneckFile {
                path: file_path.clone(),
                touches,
                authors,
                lines,
                churn_ratio,
                pattern,
                recommendation,
            }
        })
        .collect();

    // Sort by touches descending
    bottlenecks.sort_by(|a, b| b.touches.cmp(&a.touches));
    bottlenecks.truncate(20);

    // Detect co-change coupling
    let couplings = detect_coupling(&commit_files, threshold);

    Ok(BottleneckAnalysis {
        period_days: period,
        total_commits,
        total_files_changed: file_touches.len(),
        bottlenecks,
        couplings,
    })
}

/// Get file touch counts from git log
fn get_git_churn(
    path: &Path,
    period: u32,
) -> Result<(HashMap<String, usize>, Vec<Vec<String>>, usize)> {
    let output = std::process::Command::new("git")
        .args([
            "log",
            &format!("--since={} days ago", period),
            "--name-only",
            "--pretty=format:COMMIT_SEPARATOR",
        ])
        .current_dir(path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut file_touches: HashMap<String, usize> = HashMap::new();
    let mut commit_files: Vec<Vec<String>> = Vec::new();
    let mut current_files: Vec<String> = Vec::new();
    let mut total_commits = 0;

    for line in stdout.lines() {
        let line = line.trim();
        if line == "COMMIT_SEPARATOR" {
            if !current_files.is_empty() {
                commit_files.push(current_files.clone());
                current_files.clear();
            }
            total_commits += 1;
        } else if !line.is_empty() {
            *file_touches.entry(line.to_string()).or_default() += 1;
            current_files.push(line.to_string());
        }
    }
    if !current_files.is_empty() {
        commit_files.push(current_files);
    }

    Ok((file_touches, commit_files, total_commits))
}

/// Get file line counts
fn get_file_sizes(path: &Path, files: &HashMap<String, usize>) -> Result<HashMap<String, usize>> {
    let mut sizes = HashMap::new();
    for file_path in files.keys() {
        let full_path = path.join(file_path);
        if full_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                sizes.insert(file_path.clone(), content.lines().count());
            }
        }
    }
    Ok(sizes)
}

/// Get unique author counts per file
fn get_file_authors(
    path: &Path,
    period: u32,
    files: &HashMap<String, usize>,
) -> Result<HashMap<String, usize>> {
    let mut author_map: HashMap<String, std::collections::HashSet<String>> = HashMap::new();

    let output = std::process::Command::new("git")
        .args([
            "log",
            &format!("--since={} days ago", period),
            "--format=%H %an",
            "--name-only",
        ])
        .current_dir(path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_author = String::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Lines with commit hash + author name are 40+ chars with a space
        if line.len() > 41 && line.chars().nth(40) == Some(' ') {
            current_author = line[41..].to_string();
        } else if !current_author.is_empty() && files.contains_key(line) {
            author_map
                .entry(line.to_string())
                .or_default()
                .insert(current_author.clone());
        }
    }

    Ok(author_map.into_iter().map(|(k, v)| (k, v.len())).collect())
}

/// Check if a file is auto-generated
fn is_generated_file(path: &str) -> bool {
    path.contains(".pmat/")
        || path.ends_with("Cargo.lock")
        || path.ends_with(".pmat/baseline.json")
        || path.contains("target/")
        || path.ends_with(".json") && path.contains("cache")
}

/// Classify the churn pattern
fn classify_pattern(path: &str, touches: usize, lines: usize) -> String {
    let filename = path.rsplit('/').next().unwrap_or(path);

    if filename == "mod.rs" || filename.contains("registry") || filename.contains("dispatch") {
        return "Registry/Dispatch".to_string();
    }
    if filename == "Cargo.toml" || filename == "Cargo.lock" {
        return "Dependency Config".to_string();
    }
    if path.contains("workflows/") || path.contains(".github/") {
        return "CI/CD Config".to_string();
    }
    if filename.contains("test") {
        return "Test Churn".to_string();
    }
    if path.contains("roadmap") || path.contains("docs/") {
        return "Documentation".to_string();
    }
    if lines > 500 && touches > 10 {
        return "Monolith".to_string();
    }
    if touches as f64 / lines.max(1) as f64 * 100.0 > 5.0 {
        return "High Churn Ratio".to_string();
    }

    "Feature Development".to_string()
}

/// Get recommendation based on pattern
fn get_recommendation(pattern: &str) -> String {
    match pattern {
        "Registry/Dispatch" => {
            "Consider proc-macro auto-discovery (inventory/linkme) to avoid touching this file for every new feature".to_string()
        }
        "Dependency Config" => {
            "Use workspace inheritance or cargo-edit for batch dependency updates".to_string()
        }
        "CI/CD Config" => {
            "Use reusable workflows and test CI changes locally with `pmat ci-local`".to_string()
        }
        "Monolith" => {
            "Split this file into focused submodules with `pmat split --auto`".to_string()
        }
        "High Churn Ratio" => {
            "This file changes too often relative to its size — consider architectural refactoring"
                .to_string()
        }
        _ => String::new(),
    }
}

/// Detect file co-change coupling
fn detect_coupling(commit_files: &[Vec<String>], min_co_changes: usize) -> Vec<CouplingPair> {
    let mut co_changes: HashMap<(String, String), usize> = HashMap::new();

    for files in commit_files {
        // Only consider commits with 2-10 files (larger commits are usually bulk changes)
        if files.len() < 2 || files.len() > 10 {
            continue;
        }
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let a = &files[i];
                let b = &files[j];
                if a == b {
                    continue;
                }
                let key = if a < b {
                    (a.clone(), b.clone())
                } else {
                    (b.clone(), a.clone())
                };
                *co_changes.entry(key).or_default() += 1;
            }
        }
    }

    let mut pairs: Vec<CouplingPair> = co_changes
        .into_iter()
        .filter(|(_, count)| *count >= min_co_changes)
        .filter(|((a, b), _)| !is_generated_file(a) && !is_generated_file(b))
        .map(|((a, b), count)| CouplingPair {
            file_a: a,
            file_b: b,
            co_changes: count,
        })
        .collect();

    pairs.sort_by(|a, b| b.co_changes.cmp(&a.co_changes));
    pairs.truncate(15);
    pairs
}

/// Format results as colorized text
fn format_text(analysis: &BottleneckAnalysis) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut out = String::new();

    let _ = writeln!(out, "{}\n", c::header("Architectural Bottleneck Analysis"));
    let _ = writeln!(
        out,
        "  {}Period:{} {} days",
        c::BOLD,
        c::RESET,
        c::number(&analysis.period_days.to_string())
    );
    let _ = writeln!(
        out,
        "  {}Total commits:{} {}",
        c::BOLD,
        c::RESET,
        c::number(&analysis.total_commits.to_string())
    );
    let _ = writeln!(
        out,
        "  {}Files changed:{} {}\n",
        c::BOLD,
        c::RESET,
        c::number(&analysis.total_files_changed.to_string())
    );

    if analysis.bottlenecks.is_empty() {
        let _ = writeln!(out, "  {}", c::pass("No bottleneck files detected"));
        return out;
    }

    let _ = writeln!(out, "{}\n", c::subheader("Bottleneck Files"));

    for (i, b) in analysis.bottlenecks.iter().enumerate() {
        let pattern_color = match b.pattern.as_str() {
            "Registry/Dispatch" | "Monolith" => c::RED,
            "CI/CD Config" | "High Churn Ratio" => c::YELLOW,
            _ => c::DIM,
        };
        let _ = writeln!(
            out,
            "  {}. {} {}({})",
            c::number(&(i + 1).to_string()),
            c::path(&b.path),
            pattern_color,
            b.pattern,
        );
        let _ = writeln!(
            out,
            "{}     {}Touches:{} {}  {}Authors:{} {}  {}Lines:{} {}  {}Churn ratio:{} {:.1}",
            c::RESET,
            c::BOLD,
            c::RESET,
            c::number(&b.touches.to_string()),
            c::BOLD,
            c::RESET,
            c::number(&b.authors.to_string()),
            c::BOLD,
            c::RESET,
            c::number(&b.lines.to_string()),
            c::BOLD,
            c::RESET,
            b.churn_ratio,
        );
        if !b.recommendation.is_empty() {
            let _ = writeln!(
                out,
                "     {}Recommendation:{} {}",
                c::BOLD,
                c::RESET,
                b.recommendation
            );
        }
        let _ = writeln!(out);
    }

    if !analysis.couplings.is_empty() {
        let _ = writeln!(out, "{}\n", c::subheader("Co-Change Coupling"));
        for pair in &analysis.couplings {
            let _ = writeln!(
                out,
                "  {} <-> {} ({} co-changes)",
                c::path(&pair.file_a),
                c::path(&pair.file_b),
                c::number(&pair.co_changes.to_string()),
            );
        }
    }

    out
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_pattern_registry() {
        assert_eq!(
            classify_pattern("src/commands/mod.rs", 10, 50),
            "Registry/Dispatch"
        );
        assert_eq!(
            classify_pattern("src/registry.rs", 5, 100),
            "Registry/Dispatch"
        );
        assert_eq!(
            classify_pattern("src/dispatch.rs", 5, 100),
            "Registry/Dispatch"
        );
    }

    #[test]
    fn test_classify_pattern_cargo() {
        assert_eq!(classify_pattern("Cargo.toml", 10, 50), "Dependency Config");
    }

    #[test]
    fn test_classify_pattern_ci() {
        assert_eq!(
            classify_pattern(".github/workflows/ci.yml", 7, 100),
            "CI/CD Config"
        );
    }

    #[test]
    fn test_classify_pattern_monolith() {
        assert_eq!(classify_pattern("src/big_file.rs", 12, 800), "Monolith");
    }

    #[test]
    fn test_is_generated_file() {
        assert!(is_generated_file(".pmat/baseline.json"));
        assert!(is_generated_file("Cargo.lock"));
        assert!(!is_generated_file("src/main.rs"));
    }

    #[test]
    fn test_get_recommendation() {
        let rec = get_recommendation("Registry/Dispatch");
        assert!(rec.contains("proc-macro"));

        let rec = get_recommendation("Monolith");
        assert!(rec.contains("split"));
    }

    #[test]
    fn test_detect_coupling_empty() {
        let pairs = detect_coupling(&[], 3);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_detect_coupling_below_threshold() {
        let commits = vec![vec!["a.rs".to_string(), "b.rs".to_string()]];
        let pairs = detect_coupling(&commits, 3);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_detect_coupling_above_threshold() {
        let commits = vec![
            vec!["a.rs".to_string(), "b.rs".to_string()],
            vec!["a.rs".to_string(), "b.rs".to_string()],
            vec!["a.rs".to_string(), "b.rs".to_string()],
        ];
        let pairs = detect_coupling(&commits, 3);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].co_changes, 3);
    }

    #[test]
    fn test_format_text_empty() {
        let analysis = BottleneckAnalysis {
            period_days: 14,
            total_commits: 0,
            total_files_changed: 0,
            bottlenecks: vec![],
            couplings: vec![],
        };
        let text = format_text(&analysis);
        assert!(text.contains("No bottleneck files detected"));
    }

    #[tokio::test]
    async fn test_handle_bottleneck_runs() {
        // Just verify it doesn't panic on the actual repo
        let result = handle_bottleneck(
            Path::new("."),
            &crate::cli::enums::OutputFormat::Json,
            14,
            5,
            None,
        )
        .await;
        assert!(result.is_ok());
    }
}
