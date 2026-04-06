#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use super::super::{Grade, ProjectScore};

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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
