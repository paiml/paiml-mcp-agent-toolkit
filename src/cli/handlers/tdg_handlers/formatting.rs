#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG output formatting: score, comparison, and output helpers
//!
//! Handles formatting of TDG scores and comparisons for various output
//! formats (Table, JSON, Markdown, SARIF).

use super::{format_grade, TdgCommandConfig};
use crate::cli::colors as c;
use crate::cli::TdgOutputFormat;
use anyhow::Result;

/// What the project aggregate knows that a bare `TdgScore` cannot say.
///
/// Two facts travel with the score to every renderer, because without them the
/// rendered output contradicts itself:
///
/// * the F-GRADE CAP. `ProjectScore::aggregate` caps the project grade at B
///   whenever any file grades F, so this repo printed
///   `Overall Score: 99.8/100 (B)` while a 96.7 sub-tree printed `(A+)` — a
///   strictly higher score with a strictly worse grade, from the same command,
///   with no explanation anywhere in the six-line box or in the JSON.
/// * how many files were analysed. Zero means the breakdown is not a
///   measurement and must not be printed as one.
type ProjectContext<'a> = Option<&'a crate::tdg::ProjectScore>;

/// The disclosure of the F-grade cap, when it fired.
fn cap_note(project: ProjectContext<'_>) -> Option<String> {
    // GH #704: `uncapped_grade()` is an Option (nothing analysed ⇒ no grade).
    // A cap can only fire on a measured project, so the None arm simply has no
    // note to make rather than inventing one.
    project
        .filter(|p| p.grade_capped)
        .and_then(|p| p.uncapped_grade().map(|g| (p, g)))
        .map(|(p, uncapped)| {
            format!(
                "capped from {} by {} F-grade file{}",
                format_grade(uncapped),
                p.f_grade_count,
                if p.f_grade_count == 1 { "" } else { "s" }
            )
        })
}

/// Grade text for the headline, disclosing the cap when it fired. Used where
/// the line has no width budget; the box renderer puts the note on its own row
/// because `box_row` clips at 47 columns.
fn grade_headline(score: &crate::tdg::TdgScore, project: ProjectContext<'_>) -> String {
    let grade = format_grade(score.grade);
    match cap_note(project) {
        Some(note) => format!("{grade} — {note}"),
        None => grade,
    }
}

/// True when the run analysed no file at all, so there is no breakdown to show.
fn nothing_was_measured(project: ProjectContext<'_>) -> bool {
    project.is_some_and(|p| p.total_files == 0)
}

/// Format TDG output based on config (cognitive complexity ≤3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_tdg_output(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    config: &TdgCommandConfig,
) -> Result<String> {
    format_tdg_output_with_project(score, git_context, config, None)
}

/// As `format_tdg_output`, but carrying the project aggregate so the renderers
/// can disclose the F-grade cap and an empty analysis.
pub(crate) fn format_tdg_output_with_project(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    config: &TdgCommandConfig,
    project: ProjectContext<'_>,
) -> Result<String> {
    if config.quiet {
        Ok(format!("{:.1}", score.total))
    } else {
        format_tdg_score_with_project(
            score.clone(),
            git_context,
            config.format.clone(),
            config.include_components,
            project,
        )
    }
}

/// Write TDG output to file or stdout (cognitive complexity ≤3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_tdg_output(output_str: &str, config: &TdgCommandConfig) -> Result<()> {
    if let Some(output_path) = &config.output {
        std::fs::write(output_path, output_str)?;
    } else {
        println!("{output_str}");
    }
    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn format_tdg_score(
    score: crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    format: TdgOutputFormat,
    include_components: bool,
) -> Result<String> {
    format_tdg_score_with_project(score, git_context, format, include_components, None)
}

pub(crate) fn format_tdg_score_with_project(
    score: crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    format: TdgOutputFormat,
    include_components: bool,
    project: ProjectContext<'_>,
) -> Result<String> {
    match format {
        TdgOutputFormat::Table => {
            format_tdg_score_table(&score, git_context, include_components, project)
        }
        TdgOutputFormat::Json => {
            format_tdg_score_json(&score, git_context, include_components, project)
        }
        TdgOutputFormat::Markdown => {
            format_tdg_score_markdown(&score, git_context, include_components, project)
        }
        // Issue #669: this arm used to `return format!("{:.1}", score.total)`,
        // so `tdg --format sarif` emitted exactly `84.0\n` — the same 5 bytes
        // as `tdg -q`. A SARIF uploader fed that gets a scalar. Reuse the
        // SARIF 2.1.0 emitter that `analyze tdg --format sarif` already uses.
        TdgOutputFormat::Sarif => Ok(serde_json::to_string_pretty(
            &crate::cli::handlers::new_tdg_handler::create_file_sarif_output(&score),
        )?),
    }
}

/// Render the one analysis in the declared format (issue #669, second round).
///
/// Every format reads the SAME `TdgAnalysis`, so no two renderers of `pmat tdg`
/// can report different numbers for the same run.
pub(crate) fn format_tdg_analysis(
    analysis: &super::quality_gates::TdgAnalysis,
    git_context: Option<&crate::models::git_context::GitContext>,
    config: &TdgCommandConfig,
) -> Result<String> {
    if matches!(config.format, TdgOutputFormat::Sarif) {
        // `--quiet` does NOT downgrade SARIF to a bare number. That is issue
        // #669 verbatim: `tdg -q --format sarif` wrote `84.0`, which any SARIF
        // uploader rejects. A declared machine format must produce that format.
        return Ok(serde_json::to_string_pretty(&sarif_for(analysis))?);
    }
    format_tdg_output_with_project(
        &analysis.score,
        git_context,
        config,
        analysis.project.as_ref(),
    )
}

/// SARIF for the analysis: project document for a directory, file document for
/// a single file. Both are built from the already-computed scores.
fn sarif_for(analysis: &super::quality_gates::TdgAnalysis) -> serde_json::Value {
    use crate::cli::handlers::new_tdg_handler::{create_file_sarif_output, create_sarif_output};

    match &analysis.project {
        Some(project) => create_sarif_output(project, &analysis.root),
        None => create_file_sarif_output(&analysis.score),
    }
}

/// Format TDG score as table
///
/// Rows are padded from their MEASURED width (colour escapes excluded); the
/// literal space runs this used to carry made the right border drift with the
/// content, e.g. `(A+)` and `(F)` rows closing in different columns.
fn format_tdg_score_table(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
    project: ProjectContext<'_>,
) -> Result<String> {
    use crate::tdg::formatters::boxdraw::{box_blank, box_bottom, box_row, box_separator, box_top};
    let mut output = String::new();
    let mut line = |text: String| {
        output.push_str(&text);
        output.push('\n');
    };

    // Header
    line(box_top());
    match &score.file_path {
        Some(file_path) => line(box_row(&format!(
            "TDG Score Report: {}",
            c::path(&file_path.display().to_string())
        ))),
        None => line(box_row("TDG Score Report")),
    }
    line(box_separator());

    // Overall score. The grade may be the F-grade-capped one, and if it is, the
    // next row says so: an unexplained `99.8/100 (B)` next to a `96.7/100 (A+)`
    // from the same command is not a report, it is a contradiction. The note
    // gets its own row because `box_row` clips at the frame width.
    if nothing_was_measured(project) {
        // An empty population supports no grade. This printed
        // `Overall Score: 0.0/100 (F)` over a directory pmat could not read one
        // file of — an F is a claim about a codebase's quality, and zero files
        // is the absence of evidence for any claim at all. The same run already
        // suppressed the component breakdown for exactly this reason; the
        // headline just never asked.
        line(box_row("Overall Score: not measured (0 files analyzed)"));
    } else {
        let grade_str = format_grade(score.grade);
        line(box_row(&format!(
            "Overall Score: {}/100 ({})",
            c::number(&format!("{:.1}", score.total)),
            c::grade(&grade_str)
        )));
        if let Some(note) = cap_note(project) {
            line(box_row(&format!("⚠ Grade {note}")));
        }
    }
    // A file that was walked but REFUSED is disclosed beside the score it is
    // missing from. `pmat tdg <dir>` on a crate whose only Rust file fails to
    // parse printed `Overall Score: 100.0/100 (A+)` over the one Python file
    // that survived — the refusal was an `eprintln!` on stderr and nothing in
    // the report said the headline covered a subset. Twin of the same row in
    // `tdg::formatters::format_project`.
    //
    // It must NAME them, not just count them (#983): this is the renderer whose
    // output the bug report pasted — `⚠ Not Graded: 159 file(s)` over a 78-crate
    // tree, with no way to learn which 159. The rows come from
    // `tdg::formatters::ungraded`, the single implementation shared with
    // `analyze tdg --format table` and `--format markdown`.
    if let Some(p) = project.filter(|p| !p.ungraded_files.is_empty()) {
        use crate::tdg::formatters::ungraded::{box_entry_budget, ungraded_rows};
        for (i, row) in ungraded_rows(&p.ungraded_files, Some(box_entry_budget()))
            .iter()
            .enumerate()
        {
            line(box_row(&if i == 0 {
                format!("⚠ {row}")
            } else {
                row.clone()
            }));
        }
    }
    line(box_row(&format!(
        "Language: {:?} (confidence: {}%)",
        score.language,
        c::number(&format!("{:.0}", score.confidence * 100.0))
    )));

    // A waiver that changes the verdict must be disclosed on every surface that
    // reports the verdict. The #279 exemption was visible ONLY in
    // `tdg check-quality --format json`; this box — the default output of
    // `pmat tdg <file>` — applied it in silence, so the reader of
    // `Overall Score: 25.2/100 (F)` had no way to learn the auto-fail this file
    // should have triggered had been waived. `tdg/formatters/human.rs` is the
    // twin of this renderer and carries the same disclosure.
    if score.has_critical_defects {
        line(box_row(&format!(
            "Critical Defects: {}",
            c::number(&score.critical_defects_count.to_string())
        )));
        if score.critical_defects_suppressed.is_some() {
            line(box_row("  auto-fail waived: untracked by git (#279)"));
        }
    }

    // Sprint 65: Git context (if available)
    if let Some(git) = git_context {
        line(box_blank());
        line(box_row("🔗 Git Context:"));
        line(box_row(&format!(
            "├─ Commit:  {}",
            c::number(&git.commit_sha_short)
        )));
        line(box_row(&format!("├─ Branch:  {}", c::path(&git.branch))));
        line(box_row(&format!("└─ Author:  {}", git.author_name)));
    }

    if include_components && nothing_was_measured(project) {
        // No file was analysed, so there is no breakdown. Printing the struct
        // defaults here showed 25/20/20/15/10/10 — full marks, summing to 100 —
        // under a total of 0.0.
        line(box_blank());
        line(box_row("📊 Breakdown: not measured (0 files analyzed)"));
    } else if include_components {
        line(box_blank());
        line(box_row("📊 Breakdown:"));
        for (label, value, max) in [
            ("├─ Structural:    ", score.structural_complexity, 25.0),
            ("├─ Semantic:      ", score.semantic_complexity, 20.0),
            ("├─ Duplication:   ", score.duplication_ratio, 20.0),
            ("├─ Coupling:      ", score.coupling_score, 15.0),
            ("├─ Documentation: ", score.doc_coverage, 10.0),
            ("└─ Consistency:   ", score.consistency_score, 10.0),
        ] {
            // c::score already renders "earned/max", so the row must NOT append
            // another "/25": the old template printed "25.0/25.0/25".
            line(box_row(&format!(
                "{label} {}",
                c::score(f64::from(value), max, 70.0, 40.0)
            )));
        }
    }

    line(box_bottom());
    Ok(output)
}

/// Format TDG score as JSON
fn format_tdg_score_json(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
    project: ProjectContext<'_>,
) -> Result<String> {
    // A machine consumer saw `{"total": 99.83, "grade": "B"}` with nothing to
    // explain why the grade disagreed with the number, and a full-marks
    // breakdown under `"total": 0.0` when no file could be parsed. Both facts
    // now travel with the score.
    let json_value = serde_json::json!({
        "file": score.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "language": format!("{:?}", score.language),
        "confidence": score.confidence,
        "files_analyzed": project.map(|p| p.total_files),
        "grade_capped": project.map(|p| p.grade_capped),
        "grade_uncapped": project
            .filter(|p| p.grade_capped)
            .and_then(crate::tdg::ProjectScore::uncapped_grade)
            .map(format_grade),
        "f_grade_count": project.map(|p| p.f_grade_count),
        "not_measured": nothing_was_measured(project),
        // Issue #1050. The duplication component measured only WITHIN each file,
        // so ten byte-identical files each scored the full 20/20 and their mean
        // did too — full marks for a tree `analyze duplicates` calls 100%
        // duplicated. The project-wide number now exists; it has to be legible
        // here or the fix is invisible to every machine consumer.
        //
        // `measured: false` is the case that matters: TDG grades languages the
        // clone engine has no tokenizer for (Go, Java, Ruby, Lua, …), and for
        // those the component is UNMEASURED, not clean. A null ratio beside a
        // stated reason is the only shape that cannot be misread as zero.
        "duplication": project.map(|p| serde_json::json!({
            "cross_file_ratio": p.cross_file_duplication_ratio,
            "measured": p.cross_file_duplication_ratio.is_some(),
            "unmeasured_reason": p.cross_file_duplication_unmeasured,
            // A ratio over PART of a tree is not a ratio for the tree: TDG
            // grades more languages than the clone engine tokenizes, so a mixed
            // repo measures only its readable subset. Publishing the size of
            // that subset is what stops the number being read as whole-tree.
            "files_measured": p.cross_file_duplication_coverage.map(|c| c.measured),
            "files_total": p.cross_file_duplication_coverage.map(|c| c.total),
            "covers_every_graded_file": p
                .cross_file_duplication_coverage
                .map(crate::tdg::project_score::CrossFileDuplicationCoverage::covers_every_graded_file),
        })),
        "score": {
            // Null, not 0.0/"F". A machine consumer averaging `total` over a
            // tree folded an unreadable directory in as a genuine zero, and one
            // testing `grade == "F"` could not tell "graded badly" from "never
            // graded". `grade_uncapped` is already nullable here, so a null
            // grade is a shape this document could always produce.
            "total": if nothing_was_measured(project) {
                serde_json::Value::Null
            } else {
                serde_json::json!(score.total)
            },
            "grade": if nothing_was_measured(project) {
                serde_json::Value::Null
            } else {
                serde_json::json!(format_grade(score.grade))
            },
            "breakdown": if include_components && !nothing_was_measured(project) {
                Some(serde_json::json!({
                    "structural_complexity": score.structural_complexity,
                    "semantic_complexity": score.semantic_complexity,
                    "duplication": score.duplication_ratio,
                    "coupling": score.coupling_score,
                    "documentation": score.doc_coverage,
                    "consistency": score.consistency_score,
                }))
            } else {
                None
            }
        },
        "git_context": git_context.map(|git| serde_json::json!({
            "commit_sha": git.commit_sha,
            "commit_sha_short": git.commit_sha_short,
            "branch": git.branch,
            "author_name": git.author_name,
            "author_email": git.author_email,
            "commit_timestamp": git.commit_timestamp.to_rfc3339(),
            "commit_message": git.commit_message,
            "tags": git.tags,
            "is_clean": git.is_clean,
            "uncommitted_files": git.uncommitted_files,
        }))
    });
    Ok(serde_json::to_string_pretty(&json_value)?)
}

/// Format TDG score as Markdown
fn format_tdg_score_markdown(
    score: &crate::tdg::TdgScore,
    _git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
    project: ProjectContext<'_>,
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# TDG Score Report\n\n");
    if let Some(file_path) = &score.file_path {
        output.push_str(&format!("**File**: `{}`\n\n", file_path.display()));
    }

    if nothing_was_measured(project) {
        output.push_str("**Overall Score**: not measured — 0 files analyzed\n");
    } else {
        output.push_str(&format!(
            "**Overall Score**: {:.1}/100 ({})\n",
            score.total,
            grade_headline(score, project)
        ));
    }
    output.push_str(&format!(
        "**Language**: {:?} (confidence: {:.0}%)\n\n",
        score.language,
        score.confidence * 100.0
    ));

    if include_components && nothing_was_measured(project) {
        output.push_str("## Component Breakdown\n\nNot measured — 0 files analyzed.\n");
    } else if include_components {
        output.push_str("## Component Breakdown\n\n");
        output.push_str("| Component | Score | Max |\n");
        output.push_str("|-----------|-------|-----|\n");
        output.push_str(&format!(
            "| Structural Complexity | {:.1} | 25 |\n",
            score.structural_complexity
        ));
        output.push_str(&format!(
            "| Semantic Complexity | {:.1} | 20 |\n",
            score.semantic_complexity
        ));
        output.push_str(&format!(
            "| Duplication | {:.1} | 20 |\n",
            score.duplication_ratio
        ));
        output.push_str(&format!(
            "| Coupling | {:.1} | 15 |\n",
            score.coupling_score
        ));
        output.push_str(&format!(
            "| Documentation | {:.1} | 10 |\n",
            score.doc_coverage
        ));
        output.push_str(&format!(
            "| Consistency | {:.1} | 10 |\n",
            score.consistency_score
        ));
    }

    Ok(output)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_comparison(
    comparison: crate::tdg::Comparison,
    format: TdgOutputFormat,
) -> Result<String> {
    if format == TdgOutputFormat::Table {
        use crate::tdg::formatters::boxdraw::{box_bottom, box_row, box_separator, box_top};
        let mut output = String::new();
        let mut line = |text: String| {
            output.push_str(&text);
            output.push('\n');
        };
        line(box_top());
        line(box_row("TDG Comparison"));
        line(box_separator());
        let grade1 = format_grade(comparison.source1.grade);
        let grade2 = format_grade(comparison.source2.grade);
        line(box_row(&format!(
            "Source 1: {} ({})",
            c::number(&format!("{:.1}", comparison.source1.total)),
            c::grade(&grade1)
        )));
        line(box_row(&format!(
            "Source 2: {} ({})",
            c::number(&format!("{:.1}", comparison.source2.total)),
            c::grade(&grade2)
        )));
        line(box_row(&format!(
            "Difference: {}",
            c::delta(f64::from(comparison.delta))
        )));
        line(box_row(&format!(
            "Winner: {}",
            c::label(&comparison.winner)
        )));
        line(box_bottom());
        Ok(output)
    } else {
        // For other formats, output as JSON
        let json_value = serde_json::json!({
            "source1": {
                "total": comparison.source1.total,
                "grade": format_grade(comparison.source1.grade),
            },
            "source2": {
                "total": comparison.source2.total,
                "grade": format_grade(comparison.source2.grade),
            },
            "difference": comparison.delta,
            "winner": comparison.winner
        });
        Ok(serde_json::to_string_pretty(&json_value)?)
    }
}

#[cfg(test)]
mod cap_disclosure_tests {
    use super::*;
    use crate::tdg::{Grade, ProjectScore, TdgScore};

    fn file_at(total: f32) -> TdgScore {
        let mut score = TdgScore {
            structural_complexity: total * 0.25,
            semantic_complexity: total * 0.20,
            duplication_ratio: total * 0.20,
            coupling_score: total * 0.15,
            doc_coverage: total * 0.10,
            consistency_score: total * 0.10,
            entropy_score: 0.0,
            ..TdgScore::default()
        };
        score.calculate_total();
        score
    }

    /// A project with one F-grade file: the printed score is A+ territory and
    /// the printed grade is B, and the released binary said nothing about why.
    fn capped_project() -> ProjectScore {
        let mut files = vec![file_at(100.0); 19];
        files.push(file_at(10.0));
        let project = ProjectScore::aggregate(files);
        assert!(project.grade_capped, "fixture must exercise the cap");
        project
    }

    #[test]
    fn table_names_the_cap_that_moved_the_grade() {
        let project = capped_project();
        let score = project.average();
        let rendered = format_tdg_score_table(&score, None, false, Some(&project)).expect("render");
        assert!(
            rendered.contains("capped from A+"),
            "the box must say the grade was capped, got:\n{rendered}"
        );
        assert!(
            rendered.contains("1 F-grade file"),
            "the box must say how many files caused it, got:\n{rendered}"
        );
    }

    /// R22: a walked-but-refused file left no trace in the report. `pmat tdg
    /// <dir>` on a crate whose only Rust file failed to parse printed
    /// `Overall Score: 100.0/100 (A+)` over the one file that survived.
    #[test]
    fn the_box_discloses_files_that_could_not_be_graded() {
        let mut project = ProjectScore::aggregate(vec![file_at(100.0)]);
        project.ungraded_files.push(crate::tdg::UngradedFile {
            path: "./src/main.rs".to_string(),
            reason: "cannot parse string into token stream".to_string(),
        });
        let score = project.average();

        let rendered = format_tdg_score_table(&score, None, false, Some(&project)).expect("render");
        assert!(
            rendered.contains("Not Graded: 1 file(s)"),
            "a 100.0/A+ headline over a subset must say so, got:\n{rendered}"
        );
    }

    /// REGRESSION (#983): this renderer — the one the bug report pasted —
    /// printed the COUNT and nothing else, so a reader of
    /// `⚠ Not Graded: 159 file(s)` could not tell which 159 or whether they
    /// mattered. It must name them, and the name must survive the 47-column
    /// frame: these paths share every leading directory, so a row clipped from
    /// the right identifies nothing.
    #[test]
    fn the_box_names_the_files_it_could_not_grade() {
        let mut project = ProjectScore::aggregate(vec![file_at(100.0)]);
        for name in ["arxiv_entries.rs", "coursera_entries.rs"] {
            project.ungraded_files.push(crate::tdg::UngradedFile {
                path: format!("/home/noah/src/aprender/crates/aprender-core/src/oracle/{name}"),
                reason: "expected `;`".to_string(),
            });
        }
        let score = project.average();

        let rendered = format_tdg_score_table(&score, None, false, Some(&project)).expect("render");
        for name in ["arxiv_entries.rs", "coursera_entries.rs"] {
            assert!(
                rendered.contains(name),
                "the box must name {name}, got:\n{rendered}"
            );
        }
    }

    /// The list is capped, and the cap says how many it hid and where the full
    /// list lives — a truncated list that does not say so is the same defect
    /// one level down.
    #[test]
    fn the_box_caps_the_list_and_points_at_the_json() {
        let mut project = ProjectScore::aggregate(vec![file_at(100.0)]);
        for i in 0..30 {
            project.ungraded_files.push(crate::tdg::UngradedFile {
                path: format!("src/frag_{i}.rs"),
                reason: "expected `;`".to_string(),
            });
        }
        let score = project.average();

        let rendered = format_tdg_score_table(&score, None, false, Some(&project)).expect("render");
        assert!(rendered.contains("Not Graded: 30 file(s)"), "{rendered}");
        assert!(rendered.contains("and 20 more"), "{rendered}");
        assert!(rendered.contains("json"), "{rendered}");
    }

    /// R23: the #279 waiver was disclosed only by `check-quality --format
    /// json`. `pmat tdg <file>` — this box — printed `Overall Score: 25.2/100
    /// (F)` and nothing else, so a reader could not tell that the auto-fail had
    /// been waived. This renderer is a twin of `tdg::formatters::format_human`;
    /// both carry the disclosure now.
    #[test]
    fn the_file_box_discloses_a_waived_critical_defect() {
        let mut score = crate::tdg::TdgScore {
            file_path: Some(std::path::PathBuf::from("src/untracked3.rs")),
            has_critical_defects: true,
            critical_defects_count: 3,
            critical_defects_suppressed: Some("untracked (#279)".to_string()),
            ..crate::tdg::TdgScore::default()
        };
        score.calculate_total();

        let rendered = format_tdg_score_table(&score, None, false, None).expect("render");
        assert!(
            rendered.contains("Critical Defects: 3"),
            "the box must report the defects it found, got:\n{rendered}"
        );
        assert!(
            rendered.contains("waived"),
            "the box must disclose the waiver that changed the verdict, got:\n{rendered}"
        );
    }

    #[test]
    fn json_carries_grade_capped_and_the_uncapped_grade() {
        let project = capped_project();
        let score = project.average();
        let rendered = format_tdg_score_json(&score, None, false, Some(&project)).expect("render");
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");

        assert_eq!(value["score"]["grade"], "B");
        assert_eq!(value["grade_capped"], true);
        assert_eq!(value["grade_uncapped"], "A+");
        assert_eq!(value["f_grade_count"], 1);
    }

    /// Issue #1050: a duplication verdict the detector could NOT reach must say
    /// so in the payload. A component that silently keeps its full 20/20 because
    /// nothing could measure it is the defect this whole change exists to
    /// remove, and a disclosure that never leaves `ProjectScore` is no
    /// disclosure at all — `--format json` is where a machine consumer looks.
    #[test]
    fn an_unmeasured_duplication_component_is_disclosed_in_json() {
        let mut project = ProjectScore::aggregate(vec![file_at(100.0), file_at(100.0)]);
        project.record_cross_file_duplication(
            &crate::tdg::cross_file_duplication::CrossFileDuplication::unmeasured(
                "no file among the 2 graded has a clone tokenizer",
                2,
            ),
        );
        let score = project.average();

        let json = format_tdg_score_json(&score, None, false, Some(&project)).expect("render");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(
            value["duplication"]["measured"], false,
            "an unreachable population must not be reported as measured"
        );
        assert!(
            value["duplication"]["cross_file_ratio"].is_null(),
            "no ratio may be invented, got {}",
            value["duplication"]["cross_file_ratio"]
        );
        assert!(
            value["duplication"]["unmeasured_reason"]
                .as_str()
                .is_some_and(|r| r.contains("clone tokenizer")),
            "the payload must say WHY, got {}",
            value["duplication"]["unmeasured_reason"]
        );
    }

    /// The measured case is equally explicit: `0.0` here means the detector ran
    /// and found nothing, which a reader must be able to tell apart from the
    /// null above.
    #[test]
    fn a_measured_duplication_ratio_reaches_json() {
        let mut project = ProjectScore::aggregate(vec![file_at(100.0)]);
        project.record_cross_file_duplication(
            &crate::tdg::cross_file_duplication::CrossFileDuplication::measured_at(0.42),
        );
        let score = project.average();

        let json = format_tdg_score_json(&score, None, false, Some(&project)).expect("render");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["duplication"]["measured"], true);
        assert_eq!(value["duplication"]["cross_file_ratio"], 0.42);
        assert!(value["duplication"]["unmeasured_reason"].is_null());
    }

    /// Partial coverage must be visible in the payload, not just on the struct.
    #[test]
    fn partial_duplication_coverage_reaches_json() {
        let mut project = ProjectScore::aggregate(vec![file_at(100.0), file_at(100.0)]);
        project.cross_file_duplication_ratio = Some(0.1);
        project.cross_file_duplication_coverage =
            Some(crate::tdg::project_score::CrossFileDuplicationCoverage {
                measured: 1,
                total: 2,
            });
        let score = project.average();

        let json = format_tdg_score_json(&score, None, false, Some(&project)).expect("render");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["duplication"]["files_measured"], 1);
        assert_eq!(value["duplication"]["files_total"], 2);
        assert_eq!(
            value["duplication"]["covers_every_graded_file"], false,
            "a ratio over half the tree must not present as whole-tree"
        );
    }

    /// An uncapped run must not grow a cap note out of nowhere.
    #[test]
    fn uncapped_grade_is_printed_bare() {
        let project = ProjectScore::aggregate(vec![file_at(100.0), file_at(100.0)]);
        assert!(!project.grade_capped);
        let score = project.average();
        assert_eq!(score.grade, Grade::APlus);
        let rendered = format_tdg_score_table(&score, None, false, Some(&project)).expect("render");
        assert!(rendered.contains("(A+)"), "got:\n{rendered}");
        assert!(!rendered.contains("capped"), "got:\n{rendered}");
    }

    /// `--include-components` over a directory where nothing could be analysed
    /// printed the full-marks defaults (25/20/20/15/10/10 = 100) under a total
    /// of 0.0, byte-identically to an empty directory.
    #[test]
    fn empty_analysis_emits_no_breakdown() {
        let project = ProjectScore::aggregate(vec![]);
        let score = project.average();

        let json = format_tdg_score_json(&score, None, true, Some(&project)).expect("render");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        // NOT 0.0, and NOT "F". This asserted `total == 0.0`, which pinned the
        // headline defect in place: a consumer averaging `total` across a tree
        // folded an unreadable directory in as a genuine zero, and one testing
        // `grade == "F"` could not tell "graded badly" from "never graded".
        assert!(
            value["score"]["total"].is_null(),
            "total must be null when no file was analyzed, got {}",
            value["score"]["total"]
        );
        assert!(
            value["score"]["grade"].is_null(),
            "grade must be null when no file was analyzed, got {}",
            value["score"]["grade"]
        );
        assert_eq!(value["not_measured"], true);
        assert!(
            value["score"]["breakdown"].is_null(),
            "breakdown must be null when no file was analyzed, got {}",
            value["score"]["breakdown"]
        );
        assert_eq!(value["files_analyzed"], 0);

        let table = format_tdg_score_table(&score, None, true, Some(&project)).expect("render");
        assert!(table.contains("not measured"), "got:\n{table}");
        assert!(!table.contains("25.0"), "got:\n{table}");
        // The HEADLINE, not just the breakdown: `Overall Score: 0.0/100 (F)`
        // was printed over a directory pmat could not read one file of.
        assert!(
            !table.contains("(F)"),
            "an empty population must not be graded F: {table}"
        );

        let md = format_tdg_score_markdown(&score, None, true, Some(&project)).expect("render");
        assert!(md.contains("Not measured"), "got:\n{md}");
        assert!(
            !md.contains("0.0/100"),
            "an empty population must not be scored 0.0/100: {md}"
        );
    }

    /// A single-file run has no project aggregate; nothing may be suppressed or
    /// annotated there.
    #[test]
    fn single_file_render_is_unchanged() {
        let score = file_at(96.0);
        let json = format_tdg_score_json(&score, None, true, None).expect("render");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(value["score"]["breakdown"].is_object());
        assert!(value["grade_capped"].is_null());
    }
}
