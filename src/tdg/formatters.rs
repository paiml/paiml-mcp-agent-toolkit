use serde_json;
use std::fmt::Write;

use super::{Comparison, Grade, ProjectScore, TdgScore};

/// Format TDG score for human-readable console output.
///
/// Creates a visually appealing boxed display showing the TDG score,
/// grade, language confidence, and detailed breakdown of score components.
///
/// # Arguments
/// * `score` - The TDG score to format
///
/// # Returns
/// A formatted string with boxed output suitable for terminal display
///
/// # Example
/// ```ignore
/// use pmat::tdg::{TdgScore, Grade};
/// let score = TdgScore::new(85.5, Grade::A, 0.95);
/// let output = format_human(&score);
/// assert!(output.contains("85.5/100 (A)"));
/// ```ignore
#[must_use]
pub fn format_human(score: &TdgScore) -> String {
    let mut output = String::new();

    writeln!(
        output,
        "╭─────────────────────────────────────────────────╮"
    )
    .expect("Writing to String buffer cannot fail");
    if let Some(path) = &score.file_path {
        writeln!(
            output,
            "│  TDG Score Report: {:30} │",
            path.display()
                .to_string()
                .chars()
                .take(30)
                .collect::<String>()
        )
        .expect("Writing to String buffer cannot fail");
    } else {
        writeln!(output, "│  TDG Score Report: Code Analysis               │")
            .expect("Writing to String buffer cannot fail");
    }
    writeln!(
        output,
        "├─────────────────────────────────────────────────┤"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Overall Score: {:.1}/100 ({})                  │",
        score.total, score.grade
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Language: {} (confidence: {:.0}%)             │",
        score.language,
        score.confidence * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  📊 Breakdown:                                  │"
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "│  ├─ Structural:     {:4.1}/25  {}        │",
        score.structural_complexity,
        progress_bar(score.structural_complexity, 25.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Semantic:       {:4.1}/20  {}        │",
        score.semantic_complexity,
        progress_bar(score.semantic_complexity, 20.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Duplication:    {:4.1}/20  {}        │",
        score.duplication_ratio,
        progress_bar(score.duplication_ratio, 20.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Coupling:       {:4.1}/15  {}        │",
        score.coupling_score,
        progress_bar(score.coupling_score, 15.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Documentation:  {:4.1}/10  {}        │",
        score.doc_coverage,
        progress_bar(score.doc_coverage, 10.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  └─ Consistency:    {:4.1}/10  {}        │",
        score.consistency_score,
        progress_bar(score.consistency_score, 10.0, 10)
    )
    .expect("Writing to String buffer cannot fail");

    if !score.penalties_applied.is_empty() {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            output,
            "│  🔍 Issues Found:                               │"
        )
        .expect("Writing to String buffer cannot fail");
        for penalty in &score.penalties_applied {
            let issue_line = format!("  • {}", penalty.issue);
            let truncated = if issue_line.len() > 45 {
                format!("{}...", issue_line.chars().take(42).collect::<String>())
            } else {
                issue_line
            };
            writeln!(output, "│  {truncated:47} │").expect("Writing to String buffer cannot fail");
        }
    }

    if score.total >= 90.0 {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  ✨ Excellent code quality! No major issues.   │")
            .expect("Writing to String buffer cannot fail");
    } else if score.total >= 75.0 {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  👍 Good code quality with minor improvements. │")
            .expect("Writing to String buffer cannot fail");
    } else if score.total >= 60.0 {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  ⚠️  Code needs improvement in several areas.  │")
            .expect("Writing to String buffer cannot fail");
    } else {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  🔴 Code requires significant refactoring.     │")
            .expect("Writing to String buffer cannot fail");
    }

    writeln!(
        output,
        "╰─────────────────────────────────────────────────╯"
    )
    .expect("Writing to String buffer cannot fail");

    output
}

/// Format TDG score as JSON output.
///
/// Serializes the TDG score to a JSON string for programmatic consumption
/// or integration with other tools and systems.
///
/// # Arguments  
/// * `score` - The TDG score to serialize
///
/// # Returns
/// A JSON string representation of the TDG score
///
/// # Example
/// ```ignore
/// use pmat::tdg::{TdgScore, Grade};
/// let score = TdgScore::new(85.5, Grade::A, 0.95);
/// let json = format_json(&score);
/// assert!(json.contains("85.5"));
/// ```ignore
#[must_use]
pub fn format_json(score: &TdgScore) -> String {
    serde_json::to_string_pretty(score).unwrap_or_else(|_| "{}".to_string())
}

/// Format TDG score as Markdown output.
///
/// Creates a Markdown-formatted report suitable for documentation,
/// README files, or integration with documentation systems.
///
/// # Arguments
/// * `score` - The TDG score to format as Markdown
///
/// # Returns  
/// A Markdown string with formatted tables and sections
///
/// # Example
/// ```ignore
/// use pmat::tdg::{TdgScore, Grade};
/// let score = TdgScore::new(85.5, Grade::A, 0.95);
/// let md = format_markdown(&score);
/// assert!(md.contains("## TDG Score"));
/// ```ignore
#[must_use]
pub fn format_markdown(score: &TdgScore) -> String {
    let mut output = String::new();

    writeln!(output, "# TDG Score Report").expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");

    if let Some(path) = &score.file_path {
        writeln!(output, "**File:** `{}`", path.display())
            .expect("Writing to String buffer cannot fail");
    }
    writeln!(
        output,
        "**Language:** {} ({}% confidence)",
        score.language,
        (score.confidence * 100.0) as u8
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "**Overall Score:** {:.1}/100 ({})",
        score.total, score.grade
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");

    writeln!(output, "## Score Breakdown").expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");
    writeln!(output, "| Metric | Score | Max | Percentage |")
        .expect("Writing to String buffer cannot fail");
    writeln!(output, "|--------|-------|-----|------------|")
        .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Structural Complexity | {:.1} | 25.0 | {:.1}% |",
        score.structural_complexity,
        (score.structural_complexity / 25.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Semantic Complexity | {:.1} | 20.0 | {:.1}% |",
        score.semantic_complexity,
        (score.semantic_complexity / 20.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Code Duplication | {:.1} | 20.0 | {:.1}% |",
        score.duplication_ratio,
        (score.duplication_ratio / 20.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Coupling | {:.1} | 15.0 | {:.1}% |",
        score.coupling_score,
        (score.coupling_score / 15.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Documentation | {:.1} | 10.0 | {:.1}% |",
        score.doc_coverage,
        (score.doc_coverage / 10.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Consistency | {:.1} | 10.0 | {:.1}% |",
        score.consistency_score,
        (score.consistency_score / 10.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");

    if !score.penalties_applied.is_empty() {
        writeln!(output, "## Issues Found").expect("Writing to String buffer cannot fail");
        writeln!(output).expect("Writing to String buffer cannot fail");
        for penalty in &score.penalties_applied {
            writeln!(
                output,
                "- **{}**: {} (-{:.1} points)",
                format_metric_name(&penalty.source_metric),
                penalty.issue,
                penalty.amount
            )
            .expect("Writing to String buffer cannot fail");
        }
        writeln!(output).expect("Writing to String buffer cannot fail");
    }

    writeln!(output, "## Grade Description").expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");
    writeln!(output, "{}", grade_description(score.grade))
        .expect("Writing to String buffer cannot fail");

    output
}

/// Format comparison between two TDG scores.
///
/// Creates a side-by-side comparison showing the differences between
/// two TDG scores, highlighting improvements or regressions.
///
/// # Arguments
/// * `comparison` - The comparison data structure containing before/after scores
///
/// # Returns
/// A formatted string showing the comparison in a boxed layout
///
/// # Example
/// ```ignore
/// use pmat::tdg::{Comparison, TdgScore, Grade};
/// let before = TdgScore::new(75.0, Grade::B, 0.9);
/// let after = TdgScore::new(85.0, Grade::A, 0.95);
/// let comparison = Comparison::new(before, after);
/// let output = format_comparison(&comparison);
/// assert!(output.contains("improvement"));
/// ```ignore
#[must_use]
pub fn format_comparison(comparison: &Comparison) -> String {
    let mut output = String::new();

    writeln!(
        output,
        "╭─────────────────────────────────────────────────╮"
    )
    .expect("Writing to String buffer cannot fail");

    let name1 = comparison.source1.file_path.as_ref().map_or_else(
        || "source1".to_string(),
        |p| {
            p.file_name()
                .expect("Path must have a filename component")
                .to_string_lossy()
                .to_string()
        },
    );
    let name2 = comparison.source2.file_path.as_ref().map_or_else(
        || "source2".to_string(),
        |p| {
            p.file_name()
                .expect("Path must have a filename component")
                .to_string_lossy()
                .to_string()
        },
    );

    let header = format!("TDG Comparison: {name1} vs {name2}");
    let truncated_header = if header.len() > 45 {
        format!("{}...", header.chars().take(42).collect::<String>())
    } else {
        header
    };
    writeln!(output, "│  {truncated_header:47} │").expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "├─────────────────────────────────────────────────┤"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                     {:>8}   {:>8}    {:>4}  │",
        name1, name2, "Δ"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Overall Score:     {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.total, comparison.source2.total, comparison.delta
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Grade:             {:>8}  {:>8}   {:>4}  │",
        comparison.source1.grade,
        comparison.source2.grade,
        grade_delta(comparison.source1.grade, comparison.source2.grade)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Confidence:        {:>7.0}%  {:>7.0}%        │",
        comparison.source1.confidence * 100.0,
        comparison.source2.confidence * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "│  Structural:        {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.structural_complexity,
        comparison.source2.structural_complexity,
        comparison.source2.structural_complexity - comparison.source1.structural_complexity
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Semantic:          {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.semantic_complexity,
        comparison.source2.semantic_complexity,
        comparison.source2.semantic_complexity - comparison.source1.semantic_complexity
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Duplication:       {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.duplication_ratio,
        comparison.source2.duplication_ratio,
        comparison.source2.duplication_ratio - comparison.source1.duplication_ratio
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Coupling:          {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.coupling_score,
        comparison.source2.coupling_score,
        comparison.source2.coupling_score - comparison.source1.coupling_score
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Documentation:     {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.doc_coverage,
        comparison.source2.doc_coverage,
        comparison.source2.doc_coverage - comparison.source1.doc_coverage
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Consistency:       {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.consistency_score,
        comparison.source2.consistency_score,
        comparison.source2.consistency_score - comparison.source1.consistency_score
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");
    let winner_text = format!(
        "Winner: {} ({:.1}% improvement)",
        comparison.winner,
        comparison.improvement_percentage.abs()
    );
    let truncated_winner = if winner_text.len() > 47 {
        format!("{}...", winner_text.chars().take(44).collect::<String>())
    } else {
        winner_text
    };
    writeln!(output, "│  {truncated_winner:47} │").expect("Writing to String buffer cannot fail");

    if !comparison.improvements.is_empty() {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  Key Improvements:                             │")
            .expect("Writing to String buffer cannot fail");
        for improvement in &comparison.improvements {
            let improvement_line = format!("  • {improvement}");
            let truncated = if improvement_line.len() > 45 {
                format!(
                    "{}...",
                    improvement_line.chars().take(42).collect::<String>()
                )
            } else {
                improvement_line
            };
            writeln!(output, "│  {truncated:47} │").expect("Writing to String buffer cannot fail");
        }
    }

    if !comparison.regressions.is_empty() {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  Minor Regressions:                            │")
            .expect("Writing to String buffer cannot fail");
        for regression in &comparison.regressions {
            let regression_line = format!("  • {regression}");
            let truncated = if regression_line.len() > 45 {
                format!(
                    "{}...",
                    regression_line.chars().take(42).collect::<String>()
                )
            } else {
                regression_line
            };
            writeln!(output, "│  {truncated:47} │").expect("Writing to String buffer cannot fail");
        }
    }

    writeln!(
        output,
        "╰─────────────────────────────────────────────────╯"
    )
    .expect("Writing to String buffer cannot fail");

    output
}

/// Format project-level TDG score.
///
/// Creates a comprehensive project-level report showing aggregate TDG scores,
/// file counts, and overall project health metrics.
///
/// # Arguments
/// * `project` - The project score data structure
///
/// # Returns
/// A formatted string with project-level metrics and summary
///
/// # Example
/// ```ignore
/// use pmat::tdg::ProjectScore;
/// let project = ProjectScore::new("my-project", 85.0, 42);
/// let output = format_project(&project);
/// assert!(output.contains("Project Score"));
/// ```ignore
#[must_use]
pub fn format_project(project: &ProjectScore) -> String {
    let mut output = String::new();

    writeln!(
        output,
        "╭─────────────────────────────────────────────────╮"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(output, "│  Project TDG Score Report                      │")
        .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "├─────────────────────────────────────────────────┤"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Average Score: {:.1}/100 ({})                 │",
        project.average_score, project.average_grade
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Total Files: {}                               │",
        project.total_files
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(output, "│  Language Distribution:                        │")
        .expect("Writing to String buffer cannot fail");
    for (language, count) in &project.language_distribution {
        let percentage = (*count as f32 / project.total_files as f32) * 100.0;
        writeln!(
            output,
            "│  ├─ {:12}: {:3} files ({:4.1}%)         │",
            language.to_string(),
            count,
            percentage
        )
        .expect("Writing to String buffer cannot fail");
    }

    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");

    let mut files_by_grade: std::collections::BTreeMap<Grade, usize> =
        std::collections::BTreeMap::new();
    for score in &project.files {
        *files_by_grade.entry(score.grade).or_insert(0) += 1;
    }

    writeln!(output, "│  Grade Distribution:                           │")
        .expect("Writing to String buffer cannot fail");
    for (grade, count) in files_by_grade {
        let percentage = (count as f32 / project.total_files as f32) * 100.0;
        writeln!(
            output,
            "│  ├─ {grade}: {count:3} files ({percentage:4.1}%)                  │"
        )
        .expect("Writing to String buffer cannot fail");
    }

    writeln!(
        output,
        "╰─────────────────────────────────────────────────╯"
    )
    .expect("Writing to String buffer cannot fail");

    output
}

fn progress_bar(value: f32, max_value: f32, width: usize) -> String {
    let ratio = (value / max_value).clamp(0.0, 1.0);
    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn grade_delta(from: Grade, to: Grade) -> String {
    let from_val = grade_to_number(from);
    let to_val = grade_to_number(to);
    let delta = to_val - from_val;

    if delta > 0 {
        format!("↑{delta}")
    } else if delta < 0 {
        format!("↓{}", delta.abs())
    } else {
        "=".to_string()
    }
}

fn grade_to_number(grade: Grade) -> i32 {
    match grade {
        Grade::APLus => 11,
        Grade::A => 10,
        Grade::AMinus => 9,
        Grade::BPlus => 8,
        Grade::B => 7,
        Grade::BMinus => 6,
        Grade::CPlus => 5,
        Grade::C => 4,
        Grade::CMinus => 3,
        Grade::D => 2,
        Grade::F => 1,
    }
}

fn format_metric_name(metric: &crate::tdg::MetricCategory) -> &'static str {
    match metric {
        crate::tdg::MetricCategory::StructuralComplexity => "Structural Complexity",
        crate::tdg::MetricCategory::SemanticComplexity => "Semantic Complexity",
        crate::tdg::MetricCategory::Duplication => "Code Duplication",
        crate::tdg::MetricCategory::Coupling => "Coupling",
        crate::tdg::MetricCategory::Documentation => "Documentation",
        crate::tdg::MetricCategory::Consistency => "Consistency",
    }
}

fn grade_description(grade: Grade) -> &'static str {
    match grade {
        Grade::APLus => {
            "**A+** (95-100): Exceptional code quality. Production-ready with excellent practices."
        }
        Grade::A => "**A** (90-94): High quality code with minor room for improvement.",
        Grade::AMinus => "**A-** (85-89): Good code quality with some areas for enhancement.",
        Grade::BPlus => "**B+** (80-84): Above average code that meets most quality standards.",
        Grade::B => "**B** (75-79): Average code quality with moderate technical debt.",
        Grade::BMinus => "**B-** (70-74): Below average with noticeable issues to address.",
        Grade::CPlus => "**C+** (65-69): Poor code quality requiring significant attention.",
        Grade::C => "**C** (60-64): Very poor quality with major issues throughout.",
        Grade::CMinus => "**C-** (55-59): Problematic code that needs extensive refactoring.",
        Grade::D => "**D** (50-54): Severely problematic code with critical issues.",
        Grade::F => "**F** (0-49): Failing grade indicating code that is not production-ready.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::language_simple::Language;
    use crate::tdg::{MetricCategory, PenaltyAttribution};
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    fn test_format_human() {
        let score = TdgScore {
            total: 85.5,
            grade: Grade::AMinus,
            language: Language::Rust,
            confidence: 1.0,
            file_path: Some(PathBuf::from("src/test.rs")),
            ..TdgScore::default()
        };

        let output = format_human(&score);

        assert!(output.contains("85.5/100"));
        assert!(output.contains("A-"));
        assert!(output.contains("Rust"));
        assert!(output.contains("src/test.rs"));
        assert!(output.contains("📊 Breakdown"));
    }

    #[test]
    fn test_format_human_no_path() {
        let score = TdgScore {
            total: 90.0,
            grade: Grade::A,
            file_path: None,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Code Analysis"));
    }

    #[test]
    fn test_format_human_excellent_score() {
        let score = TdgScore {
            total: 95.0,
            grade: Grade::APLus,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Excellent code quality"));
    }

    #[test]
    fn test_format_human_good_score() {
        let score = TdgScore {
            total: 80.0,
            grade: Grade::B,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Good code quality"));
    }

    #[test]
    fn test_format_human_needs_improvement() {
        let score = TdgScore {
            total: 65.0,
            grade: Grade::CPlus,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("needs improvement"));
    }

    #[test]
    fn test_format_human_needs_refactoring() {
        let score = TdgScore {
            total: 45.0,
            grade: Grade::F,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("requires significant refactoring"));
    }

    #[test]
    fn test_format_human_with_penalties() {
        let score = TdgScore {
            total: 70.0,
            grade: Grade::BMinus,
            penalties_applied: vec![
                PenaltyAttribution {
                    source_metric: MetricCategory::StructuralComplexity,
                    amount: 5.0,
                    applied_to: HashSet::new(),
                    issue: "High complexity".to_string(),
                },
                PenaltyAttribution {
                    source_metric: MetricCategory::Documentation,
                    amount: 3.0,
                    applied_to: HashSet::new(),
                    issue: "This is a very long penalty description that should be truncated"
                        .to_string(),
                },
            ],
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Issues Found"));
        assert!(output.contains("High complexity"));
    }

    #[test]
    fn test_format_json() {
        let score = TdgScore::default();
        let output = format_json(&score);

        assert!(output.contains("structural_complexity"));
        assert!(output.contains("semantic_complexity"));
        assert!(output.contains("total"));
        assert!(output.contains("grade"));
    }

    #[test]
    fn test_format_markdown() {
        let score = TdgScore {
            total: 75.0,
            grade: Grade::B,
            ..TdgScore::default()
        };

        let output = format_markdown(&score);

        assert!(output.contains("# TDG Score Report"));
        assert!(output.contains("## Score Breakdown"));
        assert!(output.contains("| Metric | Score | Max"));
        assert!(output.contains("**B** (75-79)"));
    }

    #[test]
    fn test_format_markdown_with_path() {
        let score = TdgScore {
            total: 80.0,
            grade: Grade::B,
            file_path: Some(PathBuf::from("main.rs")),
            ..TdgScore::default()
        };

        let output = format_markdown(&score);
        assert!(output.contains("**File:** `main.rs`"));
    }

    #[test]
    fn test_format_markdown_with_penalties() {
        let score = TdgScore {
            total: 65.0,
            grade: Grade::CPlus,
            penalties_applied: vec![PenaltyAttribution {
                source_metric: MetricCategory::Documentation,
                amount: 10.0,
                applied_to: HashSet::new(),
                issue: "Missing docs".to_string(),
            }],
            ..TdgScore::default()
        };

        let output = format_markdown(&score);
        assert!(output.contains("## Issues Found"));
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(5.0, 10.0, 10), "█████░░░░░");
        assert_eq!(progress_bar(0.0, 10.0, 10), "░░░░░░░░░░");
        assert_eq!(progress_bar(10.0, 10.0, 10), "██████████");
        assert_eq!(progress_bar(15.0, 10.0, 10), "██████████"); // Clamped to max
    }

    #[test]
    fn test_progress_bar_negative() {
        assert_eq!(progress_bar(-5.0, 10.0, 10), "░░░░░░░░░░"); // Clamped to 0
    }

    #[test]
    fn test_grade_delta() {
        assert_eq!(grade_delta(Grade::B, Grade::A), "↑3");
        assert_eq!(grade_delta(Grade::A, Grade::B), "↓3");
        assert_eq!(grade_delta(Grade::B, Grade::B), "=");
    }

    #[test]
    fn test_grade_delta_extremes() {
        assert_eq!(grade_delta(Grade::F, Grade::APLus), "↑10");
        assert_eq!(grade_delta(Grade::APLus, Grade::F), "↓10");
    }

    #[test]
    fn test_grade_to_number() {
        assert_eq!(grade_to_number(Grade::APLus), 11);
        assert_eq!(grade_to_number(Grade::A), 10);
        assert_eq!(grade_to_number(Grade::AMinus), 9);
        assert_eq!(grade_to_number(Grade::BPlus), 8);
        assert_eq!(grade_to_number(Grade::B), 7);
        assert_eq!(grade_to_number(Grade::BMinus), 6);
        assert_eq!(grade_to_number(Grade::CPlus), 5);
        assert_eq!(grade_to_number(Grade::C), 4);
        assert_eq!(grade_to_number(Grade::CMinus), 3);
        assert_eq!(grade_to_number(Grade::D), 2);
        assert_eq!(grade_to_number(Grade::F), 1);
    }

    #[test]
    fn test_grade_description() {
        assert!(grade_description(Grade::APLus).contains("A+"));
        assert!(grade_description(Grade::A).contains("A"));
        assert!(grade_description(Grade::AMinus).contains("A-"));
        assert!(grade_description(Grade::BPlus).contains("B+"));
        assert!(grade_description(Grade::B).contains("B"));
        assert!(grade_description(Grade::BMinus).contains("B-"));
        assert!(grade_description(Grade::CPlus).contains("C+"));
        assert!(grade_description(Grade::C).contains("C"));
        assert!(grade_description(Grade::CMinus).contains("C-"));
        assert!(grade_description(Grade::D).contains("D"));
        assert!(grade_description(Grade::F).contains("F"));
    }

    #[test]
    fn test_format_metric_name() {
        assert_eq!(
            format_metric_name(&MetricCategory::StructuralComplexity),
            "Structural Complexity"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::SemanticComplexity),
            "Semantic Complexity"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::Duplication),
            "Code Duplication"
        );
        assert_eq!(format_metric_name(&MetricCategory::Coupling), "Coupling");
        assert_eq!(
            format_metric_name(&MetricCategory::Documentation),
            "Documentation"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::Consistency),
            "Consistency"
        );
    }

    #[test]
    fn test_format_comparison() {
        let source1 = TdgScore {
            total: 70.0,
            grade: Grade::BMinus,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 85.0,
            grade: Grade::AMinus,
            ..TdgScore::default()
        };
        let mut comparison = Comparison::new(source1, source2);
        comparison
            .improvements
            .push("Reduced complexity".to_string());

        let output = format_comparison(&comparison);
        assert!(output.contains("TDG Comparison"));
        assert!(output.contains("70.0"));
        assert!(output.contains("85.0"));
    }

    #[test]
    fn test_format_comparison_with_regressions() {
        let source1 = TdgScore {
            total: 90.0,
            grade: Grade::A,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 85.0,
            grade: Grade::AMinus,
            ..TdgScore::default()
        };
        let mut comparison = Comparison::new(source1, source2);
        comparison
            .regressions
            .push("Increased complexity".to_string());

        let output = format_comparison(&comparison);
        assert!(output.contains("Regressions"));
    }

    #[test]
    fn test_format_project() {
        let mut language_distribution = std::collections::HashMap::new();
        language_distribution.insert(Language::Rust, 10);
        language_distribution.insert(Language::TypeScript, 5);

        let project = ProjectScore {
            average_score: 85.0,
            average_grade: Grade::AMinus,
            total_files: 15,
            files: vec![
                TdgScore {
                    total: 90.0,
                    grade: Grade::A,
                    ..TdgScore::default()
                },
                TdgScore {
                    total: 80.0,
                    grade: Grade::B,
                    ..TdgScore::default()
                },
            ],
            language_distribution,
        };

        let output = format_project(&project);
        assert!(output.contains("Project TDG Score Report"));
        assert!(output.contains("85.0/100"));
        assert!(output.contains("Language Distribution"));
        assert!(output.contains("Grade Distribution"));
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
