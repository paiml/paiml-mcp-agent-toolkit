#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use crate::tdg::{Grade, TdgScore};

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

pub(super) fn format_metric_name(metric: &crate::tdg::MetricCategory) -> &'static str {
    match metric {
        crate::tdg::MetricCategory::StructuralComplexity => "Structural Complexity",
        crate::tdg::MetricCategory::SemanticComplexity => "Semantic Complexity",
        crate::tdg::MetricCategory::Duplication => "Code Duplication",
        crate::tdg::MetricCategory::Coupling => "Coupling",
        crate::tdg::MetricCategory::Documentation => "Documentation",
        crate::tdg::MetricCategory::Consistency => "Consistency",
    }
}

pub(super) fn grade_description(grade: Grade) -> &'static str {
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
