//! The project-level half of the TDG duplication component (issue #1050).
//!
//! Every duplication scorer TDG has ever had takes ONE file's source and looks
//! for repeated token sequences inside it. That is a real measurement, and at
//! single-file scope it is the only one available. What it cannot see is the
//! commonest kind of duplication there is: the same code sitting in two
//! different files. Ten byte-identical files each contain 0% internal
//! duplication, so each scored the full 20/20, and the project score — a mean of
//! those per-file components — scored 20/20 as well. The component awarded full
//! marks precisely because it had no way to look.
//!
//! Measured on the pre-fix binary, over a tree of ten distinct files and the
//! same tree with a byte-identical copy of every file added:
//!
//! ```text
//!            analyze duplicates   tdg duplication component
//! clean10          0.0%           20.0 / 20
//! cloned20        88.2%           20.0 / 20
//! ```
//!
//! This module closes that gap with `DuplicateDetectionEngine` (MinHash + LSH).
//!
//! ## It does NOT agree numerically with `analyze duplicates`, and cannot
//!
//! An earlier draft of this comment claimed the two "agree by construction
//! rather than by coincidence". That is false, and it was caught by an
//! adversarial review before it shipped. `analyze duplicates`' headline number
//! comes from `find_duplicate_blocks` — hash-bucketed LINE BLOCKS over
//! `ProjectFileDiscovery`'s walk. `DuplicateDetectionEngine` appears there only
//! in `find_structural_similarities`, as a supplementary near-miss pass, with a
//! different config and a different denominator. Measured:
//!
//! ```text
//! tree       analyze duplicates      tdg cross_file_ratio
//! pmat/src   22.025% (4004 files)     7.709% (1569 files)
//! duende     70.005% (  79 files)    83.579% (  57 files)
//! pepita      7.832%                 15.174% (20/30)
//! pzsh       14.096% (  29 files)    15.353% (16/46)
//! ```
//!
//! They agree only on degenerate 0%/100% fixtures. Two things drive the gap:
//! the denominators differ (TDG measures over the files it GRADED, which is a
//! subset), and block-hashing and MinHash-LSH answer different questions.
//!
//! So do not read the two numbers as the same quantity, and do not "fix" one to
//! match the other without deciding which question you meant to ask. What this
//! module guarantees is ORDERING, not equality: a tree with more cross-file
//! duplication loses more of the component than one with less.
//!
//! ## What it does NOT do
//!
//! It does not touch single-file analysis. `pmat tdg <one file>` still reports
//! within-file duplication, because cross-file duplication is undefined for a
//! population of one and the within-file number is a legitimate measurement at
//! that scope. The defect was never that the number existed; it was that a
//! within-file number was presented as a PROJECT verdict.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::services::duplicate_detector::{
    DuplicateDetectionConfig, DuplicateDetectionEngine, Language as CloneLanguage,
};
use crate::tdg::score::TdgScore;

/// Cross-file duplication over the files a project score covers.
#[derive(Debug, Default)]
pub(crate) struct CrossFileDuplication {
    /// Project-wide duplicated-line ratio in `0.0..=1.0`, or `None` when the
    /// detector could not be run at all. `None` is never rendered as 0.0: a
    /// ratio of zero is a measurement that found nothing, and "we could not
    /// look" must not read the same as "we looked and it was clean".
    pub(crate) ratio: Option<f64>,
    /// Why `ratio` is `None`, in the words a reader gets.
    pub(crate) unmeasured_reason: Option<String>,
    /// Duplicated-line ratio per file, keyed by the path as walked.
    per_file: HashMap<PathBuf, f64>,
    /// How many of the graded files the detector could actually read.
    pub(crate) files_measured: usize,
    /// How many files the project score covers.
    pub(crate) files_total: usize,
}

impl CrossFileDuplication {
    /// True when the detector ran over every file the score covers.
    pub(crate) fn is_complete(&self) -> bool {
        self.ratio.is_some() && self.files_measured == self.files_total
    }

    /// A verdict the detector could not reach, carrying the reason.
    ///
    /// The reason is not optional. "Unmeasured" without a cause is the same
    /// dead end as a plausible zero: a reader learns that the number is missing
    /// but not whether that is expected (a Go tree) or a fault to chase.
    pub(crate) fn unmeasured(reason: impl Into<String>, files_total: usize) -> Self {
        Self {
            ratio: None,
            unmeasured_reason: Some(reason.into()),
            per_file: HashMap::new(),
            files_measured: 0,
            files_total,
        }
    }

    /// A verdict with a measured project-wide ratio and no per-file attribution.
    /// Renderer tests use this; `measure` builds the real thing.
    #[cfg(test)]
    pub(crate) fn measured_at(ratio: f64) -> Self {
        Self {
            ratio: Some(ratio),
            unmeasured_reason: None,
            per_file: HashMap::new(),
            files_measured: 1,
            files_total: 1,
        }
    }
}

/// The clone engine's language for a path, or `None` when it has no tokenizer
/// for that extension.
///
/// A SUPERSET of `cli::analysis::duplicates_detection`'s table — not the same
/// one, which an earlier version of this comment claimed. That table has `ts`,
/// `js`, `c`, `cpp|cc|cxx`; this adds `tsx`, `jsx`, `h`, `hpp`. The difference
/// is deliberate (a `.tsx` file is TypeScript and the engine tokenizes it) but
/// it is a difference, and describing it as sameness would send the next reader
/// to the wrong file to change it.
///
/// TDG grades more languages than the clone engine tokenizes (Go, Java, Ruby,
/// Lua, SQL, Lean, …); those files are counted as UNMEASURED rather than as
/// clean, which is the whole point of #1050.
fn clone_language(path: &Path) -> Option<CloneLanguage> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Some(CloneLanguage::Rust),
        Some("ts" | "tsx") => Some(CloneLanguage::TypeScript),
        Some("js" | "jsx") => Some(CloneLanguage::JavaScript),
        Some("py") => Some(CloneLanguage::Python),
        Some("c" | "h") => Some(CloneLanguage::C),
        Some("cpp" | "cc" | "cxx" | "hpp") => Some(CloneLanguage::Cpp),
        Some("kt" | "kts") => Some(CloneLanguage::Kotlin),
        _ => None,
    }
}

/// Lines that carry code, by the SAME rule the per-file scorer
/// (`analyze_duplication_ast`) uses for its denominator: non-blank, not a
/// comment. Both ratios therefore sit on one scale, so a file cannot be scored
/// differently depending on which of the two paths reached it.
fn code_line_count(source: &str) -> usize {
    source
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
        .count()
}

/// Run the project-wide clone detector over `files`.
///
/// `files` is the gradable population the project score is computed over, so
/// the duplication verdict covers exactly the files the score claims to cover.
pub(crate) fn measure(files: &[PathBuf]) -> CrossFileDuplication {
    let files_total = files.len();

    let mut sources: Vec<(PathBuf, String, CloneLanguage)> = Vec::new();
    for path in files {
        let Some(language) = clone_language(path) else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        sources.push((path.clone(), source, language));
    }

    if sources.is_empty() {
        return CrossFileDuplication::unmeasured(
            format!(
                "no file among the {files_total} graded has a clone tokenizer \
                 (supported: rs, ts, js, py, c, cpp, kt)"
            ),
            files_total,
        );
    }

    let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());
    let report = match engine.detect_duplicates(&sources) {
        Ok(report) => report,
        Err(error) => {
            return CrossFileDuplication::unmeasured(
                format!("clone detection failed: {error}"),
                files_total,
            )
        }
    };

    // Distinct duplicated LINE NUMBERS per file. A set, not a sum: a line
    // covered by two overlapping clone groups is one duplicated line, and
    // summing instance lengths (which is what `compute_hotspots` does for its
    // severity ranking) can report more duplicated lines than the file has.
    let mut duplicated_lines: HashMap<&Path, HashSet<usize>> = HashMap::new();
    for group in &report.groups {
        for instance in &group.fragments {
            let lines = duplicated_lines.entry(instance.file.as_path()).or_default();
            for line in instance.start_line..=instance.end_line {
                lines.insert(line);
            }
        }
    }

    let mut per_file = HashMap::with_capacity(sources.len());
    for (path, source, _) in &sources {
        let code_lines = code_line_count(source);
        if code_lines == 0 {
            continue;
        }
        let duplicated = duplicated_lines.get(path.as_path()).map_or(0, HashSet::len);
        let ratio = (duplicated as f64 / code_lines as f64).clamp(0.0, 1.0);
        per_file.insert(path.clone(), ratio);
    }

    CrossFileDuplication {
        ratio: Some(report.summary.duplication_ratio.clamp(0.0, 1.0)),
        unmeasured_reason: None,
        per_file,
        files_measured: sources.len(),
        files_total,
    }
}

/// The penalty a duplication ratio earns, in points off the component.
///
/// DEVIATION, recorded deliberately. The brief specified the curve
/// `(ratio * 40.0).min(20.0)` from `impl Scorer for DuplicationDetector` in
/// `src/tdg/scorers/duplication_scoring.rs`. That file is NOT COMPILED — neither
/// `src/tdg/scorers/` nor `src/tdg/analyzer.rs` is declared as a module anywhere
/// in the crate, and `ScorerSet` has no reference outside them. The scorer this
/// build actually runs is `TdgAnalyzerAst::analyze_duplication_ast`, whose curve
/// is `(ratio * 20.0).min(20.0)` above a 10% floor. Matching the LIVE curve is
/// what delivers the property the brief asked for — one scale whichever path
/// reaches a tree — so that is the curve used here. Adopting the dead file's
/// steeper curve would have created the very split it was meant to prevent.
///
/// The 10% floor comes from the same live scorer. It is a noise floor, not a
/// silence: a tree measured below it still reports its measured ratio through
/// `ProjectScore::cross_file_duplication_ratio`, so "measured 8%, under the
/// floor" is legible and never presented as "not duplicated".
fn penalty_for(ratio: f64, weight: f32) -> f32 {
    if ratio <= 0.1 {
        return 0.0;
    }
    ((ratio as f32) * 20.0).min(weight)
}

/// Fold cross-file duplication into the per-file duplication components.
///
/// A file's component becomes the WORSE of its two measurements — the
/// within-file duplication `analyze_duplication_ast` already found, and the
/// cross-file duplication measured here. Taking the worse of the two means
/// project analysis can only ever lower a file's duplication score relative to
/// analysing that file alone, never raise it: whatever `pmat tdg <file>` says
/// about a file, `pmat tdg <dir>` will not claim it is cleaner.
pub(crate) fn apply(scores: &mut [TdgScore], measured: &CrossFileDuplication, weight: f32) {
    if measured.ratio.is_none() {
        return;
    }

    for score in scores.iter_mut() {
        let Some(path) = score.file_path.as_ref() else {
            continue;
        };
        let Some(&ratio) = measured.per_file.get(path) else {
            continue;
        };

        // Recovered rather than re-derived: whatever the within-file scorer
        // decided is exactly what this file already lost.
        let within_penalty = (weight - score.duplication_ratio).max(0.0);
        let cross_penalty = penalty_for(ratio, weight);

        score.duplication_ratio = (weight - within_penalty.max(cross_penalty)).max(0.0);
        score.calculate_total();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_ratio_costs_the_whole_component() {
        assert!((penalty_for(1.0, 20.0) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_penalty_never_exceeds_the_component_weight() {
        for percent in 0..=100 {
            let penalty = penalty_for(f64::from(percent) / 100.0, 20.0);
            assert!(
                (0.0..=20.0).contains(&penalty),
                "ratio {percent}% produced penalty {penalty}, outside the 0..=20 weight"
            );
        }
    }

    #[test]
    fn a_ratio_under_the_noise_floor_costs_nothing() {
        assert!((penalty_for(0.05, 20.0)).abs() < f32::EPSILON);
        assert!(penalty_for(0.5, 20.0) > 0.0);
    }

    #[test]
    fn code_lines_ignore_blanks_and_comments() {
        let source = "fn a() {}\n\n// note\n/* block */\nfn b() {}\n";
        assert_eq!(code_line_count(source), 2);
    }

    #[test]
    fn unknown_extensions_have_no_clone_tokenizer() {
        assert!(clone_language(Path::new("a.go")).is_none());
        assert!(clone_language(Path::new("a.lua")).is_none());
        assert!(clone_language(Path::new("a.rs")).is_some());
    }

    /// A tree of languages the clone engine cannot tokenize must come back
    /// UNMEASURED, never as a clean 0.0 ratio.
    #[test]
    fn a_tree_the_engine_cannot_read_is_unmeasured_not_clean() {
        let measured = measure(&[PathBuf::from("a.go"), PathBuf::from("b.lua")]);
        assert!(measured.ratio.is_none(), "must not report a ratio");
        assert!(!measured.is_complete());
        assert_eq!(measured.files_total, 2);
        assert_eq!(measured.files_measured, 0);
        let reason = measured.unmeasured_reason.expect("a reason");
        assert!(reason.contains("clone tokenizer"), "got {reason}");
    }

    /// An unmeasured result must leave every component exactly as it found it —
    /// it must not invent a penalty any more than it may award full marks.
    #[test]
    fn an_unmeasured_result_changes_no_score() {
        let mut scores = vec![TdgScore {
            duplication_ratio: 20.0,
            file_path: Some(PathBuf::from("a.go")),
            ..TdgScore::default()
        }];
        let before = scores[0].clone();
        apply(&mut scores, &CrossFileDuplication::default(), 20.0);
        assert_eq!(scores[0], before);
    }

    /// Project analysis may only ever lower a file's duplication component,
    /// never raise it above what single-file analysis reported.
    #[test]
    fn cross_file_analysis_never_improves_a_files_duplication_score() {
        let path = PathBuf::from("dirty.rs");
        // A file the within-file scorer already docked to 4.0/20.
        let mut scores = vec![TdgScore {
            duplication_ratio: 4.0,
            file_path: Some(path.clone()),
            ..TdgScore::default()
        }];
        let measured = CrossFileDuplication {
            ratio: Some(0.2),
            unmeasured_reason: None,
            // A cross-file penalty (0.2 * 20 = 4.0) SMALLER than the within-file
            // one (16.0), so the worse of the two must win.
            per_file: HashMap::from([(path, 0.2)]),
            files_measured: 1,
            files_total: 1,
        };
        apply(&mut scores, &measured, 20.0);
        assert!(
            (scores[0].duplication_ratio - 4.0).abs() < f32::EPSILON,
            "the within-file penalty was the worse one and must stand, got {}",
            scores[0].duplication_ratio
        );
    }
}
