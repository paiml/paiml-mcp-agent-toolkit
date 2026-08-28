#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::grade::Grade;
use super::language_simple::Language;
use super::score::TdgScore;

/// A file that was walked but could NOT be graded, and why.
///
/// The skip used to exist ONLY as an `eprintln!` on stderr, which is not part of
/// a `--format json` payload and not part of an MCP response. `analyze tdg` on a
/// crate whose only Rust file fails to parse therefore reported
/// `{total_files: 1, average_score: 100.0, not_measured: []}` over the one
/// Python file that survived, and `pmat tdg . --min-grade A` printed
/// `Overall Score: 100.0/100 (A+)`: the average and the grade were computed over
/// the SURVIVORS and presented as the project's. A file that was refused has to
/// come back through the return value or every consumer silently averages a
/// subset it cannot see.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UngradedFile {
    /// Path as walked.
    pub path: String,
    /// Why it could not be graded (parse failure, unsupported language, …).
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Project score.
pub struct ProjectScore {
    pub files: Vec<TdgScore>,
    /// Mean TDG score over the files that carry CODE, or `None` when nothing
    /// was analysed. The population is not every entry in `files` — see
    /// `aggregate` for why documentation is excluded from it (#1090).
    ///
    /// GH #704: this was a bare `f32` defaulted to `0.0` for the empty case,
    /// and `average_grade` was derived from that default — so `analyze tdg` on
    /// an empty directory printed `Average Score: 0.0/100 (F)` next to
    /// `Total Files: 0`, in the table AND in the JSON, presenting a plausible
    /// default as a measurement of a project that was never measured. There
    /// was no value of `f32` that could express "not measured", so the type
    /// had to grow one. `None` serialises as `null` and is listed by name in
    /// `not_measured`, the same convention `quality_gate`/`analyze_deep_context`
    /// already use for unmeasurable fields.
    #[serde(default)]
    pub average_score: Option<f32>,
    /// Project grade, or `None` when nothing was analysed. See `average_score`.
    #[serde(default)]
    pub average_grade: Option<Grade>,
    /// Names of the fields above that could not be measured (empty when every
    /// field is a real measurement). A reader must never have to infer
    /// "not measured" from a plausible-looking zero.
    #[serde(default)]
    pub not_measured: Vec<String>,
    /// Number of files ANALYSED. Independent of how many entries `files`
    /// carries — see `files_reported`/`files_truncated`.
    pub total_files: usize,
    /// Keyed by a `BTreeMap`, not a `HashMap`: serde emits a map in iteration
    /// order, and `HashMap`'s is randomised per process. See the note on
    /// `grade_distribution` — the same defect, same fix.
    pub language_distribution: BTreeMap<Language, usize>,
    /// Grade distribution: count of files per grade (A+, A, ..., F)
    ///
    /// DETERMINISM (round-3 sweep): this was a `HashMap<Grade, usize>`, so
    /// `analyze tdg --format json` emitted the SAME counts under a different
    /// KEY ORDER on every run — 6 runs over an unchanged tree gave 6 orders
    /// (`["B","BMinus","BPlus","A","APlus","AMinus"]`,
    /// `["BMinus","BPlus","B","APlus","AMinus","A"]`, …) with a stable sum of
    /// 495. A byte-diff of two identical runs was therefore never empty, which
    /// is what makes a JSON baseline useless. `BTreeMap` emits in `Grade`'s
    /// declaration order (best grade first), which is also the order a reader
    /// expects.
    #[serde(default)]
    pub grade_distribution: BTreeMap<Grade, usize>,
    /// Count of F-grade files (critical quality issues) over EVERY graded file.
    ///
    /// It is exactly `grade_distribution[&Grade::F]` — `aggregate` reads it out
    /// of that map rather than counting a second time, so the two cannot drift.
    /// That equality is the point. #1090 narrowed `average_score` to the source
    /// files and narrowed this with it, which left a `pub`, serialized field
    /// silently meaning something new: `cli::handlers::new_tdg_handler` prints
    /// `**Grade Capped:** yes (N F-grade file(s))` from this number directly
    /// above a `## Grade Distribution` table whose F row comes from
    /// `grade_distribution`, and `tdg::quality_gate::f_grade::FGradeGate`
    /// counts F-grades over every file in the baseline. A tree with 2 F-grade
    /// `.rs` and 3 F-grade `.md` rendered "yes (2 F-grade file(s))" over an
    /// "F | 5" row, with nothing on screen to say the two counted different
    /// populations.
    ///
    /// #1090's measurement was that documentation must not move the MEAN, and
    /// that correction stays, confined to `average_score`. "How many F-grade
    /// files are there" has one answer here, in `grade_distribution`, and in
    /// the gate. The cost is deliberate and disclosed: an F-grade `.md` counts
    /// as an F-grade file and therefore still caps the grade at B, which is
    /// what `FGradeGate` fails the build over anyway.
    #[serde(default)]
    pub f_grade_count: usize,
    /// Whether grade was capped due to F-grade files
    #[serde(default)]
    pub grade_capped: bool,
    /// Number of entries actually present in `files`. Equals `total_files`
    /// unless `--top-files` truncated the list.
    #[serde(default)]
    pub files_reported: usize,
    /// True when `files` holds only the worst `--top-files` entries. A capped
    /// list is never presented as the whole population.
    #[serde(default)]
    pub files_truncated: bool,
    /// Which flag reduced `files` to the reported subset, when one did.
    ///
    /// The renderers hardcoded `(--top-files)` in the "Files Listed: 2 of 4"
    /// disclosure, so a run filtered by `--critical-only` — with `--top-files`
    /// never passed — blamed a flag the user had not used. The disclosure names
    /// the flag that actually applied, or says nothing when the list is whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_filter: Option<String>,
    /// Files that were walked but NOT graded — see [`UngradedFile`].
    ///
    /// `total_files`, `average_score` and `average_grade` describe the GRADED
    /// files only, so this list is what tells a reader the population those
    /// numbers were computed over is not the population that was walked.
    #[serde(default)]
    pub ungraded_files: Vec<UngradedFile>,
    /// Duplication measured ACROSS the graded files, in `0.0..=1.0`, or `None`
    /// when the clone detector could not be run over them (issue #1050).
    ///
    /// The duplication component used to be a per-file measurement only: ten
    /// byte-identical files each contain 0% *internal* duplication, so each
    /// scored the full 20/20 and their mean did too. `analyze duplicates` called
    /// the same tree 100% duplicated. The component awarded full marks exactly
    /// because it had no way to look across files.
    ///
    /// `None` is not 0.0. A ratio of zero is a measurement that found nothing;
    /// `None` says the measurement was never taken, and is named in
    /// `not_measured` alongside the reason so no reader has to infer it from a
    /// plausible-looking number.
    #[serde(default)]
    pub cross_file_duplication_ratio: Option<f64>,
    /// Why `cross_file_duplication_ratio` is `None`, when it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_file_duplication_unmeasured: Option<String>,
    /// How many of the graded files the clone detector could actually read, and
    /// how many the score covers, as `(measured, total)`.
    ///
    /// A ratio measured over PART of the tree is not a ratio for the tree. TDG
    /// grades Go, Java, Ruby, Lua and more; the clone engine tokenizes seven
    /// languages. On a mixed repo the duplication verdict describes only the
    /// subset it could read, and a reader who cannot see that subset's size will
    /// take it for the whole — the same over-claim as #1050 itself, one step
    /// smaller. `None` when no project walk ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_file_duplication_coverage: Option<CrossFileDuplicationCoverage>,
}

/// How much of the graded tree the cross-file duplication verdict actually covers.
///
/// A NAMED struct, not the `(usize, usize)` tuple this started as, for two
/// reasons that turned out to be the same reason.
///
/// The differential-corpus gate flagged it: serialized as a tuple this is a raw
/// two-element array, so `cross_file_duplication_coverage[].len` was a numeric
/// leaf reading 2 for an empty project and 2 for a defect-rich one — a constant
/// masquerading as a measurement. It could have been allow-listed as "fixed
/// arity", and that would have been true and useless.
///
/// The better answer is that `[1, 2]` is a bad payload anyway: a consumer has to
/// know positionally which number is which, and getting it backwards silently
/// inverts the meaning. Named fields cannot be read backwards. Nothing consumed
/// this field yet — it was added in the same change — so naming it cost nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CrossFileDuplicationCoverage {
    /// Files the clone engine could tokenize and therefore measure.
    pub measured: usize,
    /// Files TDG graded. Always >= `measured`.
    pub total: usize,
}

impl CrossFileDuplicationCoverage {
    /// True when the verdict covers every graded file, so the ratio describes
    /// the whole tree rather than a subset of it.
    #[must_use]
    pub fn covers_every_graded_file(self) -> bool {
        self.measured == self.total
    }
}

/// Which language a project of this shape IS, or `None` when nothing was graded.
///
/// The winner used to be a plurality over EVERY graded file, documentation
/// included. `pmat tdg ~/src/pforge --format json` therefore reported
/// `"language": "Markdown"` for a cargo workspace — while `pmat tdg
/// ~/src/pforge/crates`, a subdirectory of that same tree, reported `"Rust"` —
/// because a doc-heavy Rust workspace holds more `.md` files than `.rs` files
/// at its root (issue #1073). `language` is a field consumers branch on, and it
/// can select language-specific grading, so a project whose code is Rust must
/// not be handed to a reader as a Markdown project.
///
/// The plurality is now taken over the languages that carry CODE, and falls
/// back to the whole distribution only when no source file was graded at all —
/// a docs-only tree really is a Markdown tree, and saying otherwise would be
/// the same false label pointing the other way.
///
/// The tie-break is unchanged and still load-bearing: `max_by_key` alone
/// returned whichever tied language iteration order happened to visit last, so
/// two `.rs` and two `.md` files made `tdg <dir>` alternate between "Rust" and
/// "Markdown" over unchanged input — 7 vs 5 out of 12 runs, at a byte-identical
/// score (GH #673). Ties resolve to the lowest `Language` discriminant.
fn dominant_language(distribution: &BTreeMap<Language, usize>) -> Option<Language> {
    let plurality = |source_only: bool| {
        distribution
            .iter()
            .filter(|(lang, _)| !source_only || lang.is_source_code())
            .max_by_key(|(&lang, &count)| (count, std::cmp::Reverse(lang as usize)))
            .map(|(&lang, _)| lang)
    };
    plurality(true).or_else(|| plurality(false))
}

/// The files a project's SCORE is computed over: the ones that carry code.
///
/// Issue #1090. The project score was an unweighted arithmetic mean over EVERY
/// graded file, and the AST walk grades documentation: `Policy::ast()` sets
/// `grades_markup`, so `is_gradable_extension` admits every `.md` and `.yaml`
/// in the tree (`file_discovery.rs`). A 217-byte Markdown stub grades 95.34/A+,
/// so each documentation file is a vote for ~95 that no amount of work on the
/// code can outweigh — an 85.0/A- fixture climbed 90.17 → 91.89 → 92.75 → 93.27
/// as four `.md` files landed beside it with the source untouched. Those rungs
/// are the unweighted mean of the one 85.0 file and k stubs at 95.34
/// ((85+95.34)/2 = 90.17, /3 = 91.89, /4 = 92.75, /5 = 93.27), so the FIRST
/// added `.md` is worth +5.17 on its own. (An earlier revision of this comment
/// read "a plain README.md on its own moved it 85.0 → 91.89"; 91.89 is the
/// second rung, k=2, transcribed onto the wrong row.) This is not a corner case:
/// pmat's own tree carries 719 tracked `.md` and 213 `.yaml` in the graded
/// population against 4,433 `.rs`, so the number a reader takes for "how good
/// is this code" is partly a count of how much prose sits next to it.
///
/// The predicate is the one `dominant_language` just above already applies,
/// for the same reason and the same issue family (#1073): documentation
/// describes a project, it is not what the project is written in.
///
/// The fallback is load-bearing, not defensive. A tree that really holds
/// nothing but documentation is a documentation tree and must still get a
/// grade — refusing to score it would replace one wrong answer with "not
/// measured", which is the mirror of the #1073 mislabel. It is also what keeps
/// `ProjectScore::aggregate(vec![score])` — the single-file path in
/// `mcp_pmcp::tool_functions::tdg_tools_storage` — answering for a lone `.md`
/// instead of degrading to `average_score: None`.
fn scoring_population(scores: &[TdgScore]) -> Vec<&TdgScore> {
    let source: Vec<&TdgScore> = scores
        .iter()
        .filter(|score| score.language.is_source_code())
        .collect();
    if source.is_empty() {
        scores.iter().collect()
    } else {
        source
    }
}

impl ProjectScore {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Aggregate.
    pub fn aggregate(scores: Vec<TdgScore>) -> Self {
        let total_files = scores.len();
        // #1090: the score describes the CODE, so it is a mean over the source
        // files and not over everything the walk happened to grade — see
        // `scoring_population`. Empty only when `scores` itself is empty, so
        // the "was anything measured at all" question below is still
        // `total_files`.
        //
        // Scoped so the borrow of `scores` ends before `scores` is moved into
        // the returned value below.
        let average_score = {
            let scored = scoring_population(&scores);
            // GH #704: no files analysed means no average and no grade — not 0.0/F.
            (!scored.is_empty())
                .then(|| scored.iter().map(|s| s.total).sum::<f32>() / scored.len() as f32)
        };

        // The DISTRIBUTIONS stay a census of everything that was graded, and
        // deliberately so. #1073 left `language_distribution` whole for exactly
        // this reason — Markdown really is the plurality of a doc-heavy Rust
        // tree, and that fact stays readable even though it no longer decides
        // the label. `grade_distribution` is the same kind of statement, and
        // both renderers print each of its rows as a share of `total_files`
        // (`tdg::formatters::project` and `cli::handlers::new_tdg_handler`), so
        // narrowing the numerator alone would make the rows silently sum to
        // ~83% of the tree with nothing on screen to say why.
        let mut language_distribution = BTreeMap::new();
        let mut grade_distribution = BTreeMap::new();

        for score in &scores {
            *language_distribution.entry(score.language).or_insert(0) += 1;
            *grade_distribution.entry(score.grade).or_insert(0) += 1;
        }

        // …and `f_grade_count` is a ROW OF THAT CENSUS, read out of the map
        // instead of counted again, so the number the renderers print beside
        // the cap and the number in the F row of the distribution table cannot
        // be two different populations. See the field's own doc: #1090 narrowed
        // this to the source files along with the mean, which changed what a
        // `pub` serialized field meant without any renderer disclosing it.
        // The narrowing that #1090 actually measured is the MEAN, above.
        let f_grade_count = grade_distribution.get(&Grade::F).copied().unwrap_or(0);

        // GH #680: the project grade and the per-file grades must come from the
        // SAME mapping, and that mapping is `Grade::from_score` — the score and
        // nothing else. The first attempt at #680 unified them the other way
        // round, onto `TdgScore::grade_for(score, has_contract_coverage)`, which
        // capped at A-: a project averaging 100.0 reported average_grade
        // `AMinus`. Agreement on a wrong grade is still a wrong grade.
        //
        // F-GRADE CAPPING: any F-grade file caps the project grade at B. That
        // cap IS derived from measured input (a file actually graded F) and is
        // reported through `grade_capped`, so it stays.
        let uncapped_grade = average_score.map(Grade::from_score);
        let (average_grade, grade_capped) = match uncapped_grade {
            // Cap at B (score 79.9 equivalent) if any F-grades exist
            Some(g) if f_grade_count > 0 && g < Grade::B => (Some(Grade::B), true),
            other => (other, false),
        };

        // GH #704: an unmeasured field is null AND named here, so a reader
        // never has to infer "not measured" from a missing key.
        let not_measured = if total_files == 0 {
            vec!["average_score".to_string(), "average_grade".to_string()]
        } else {
            Vec::new()
        };

        Self {
            files: scores,
            average_score,
            average_grade,
            not_measured,
            total_files,
            language_distribution,
            grade_distribution,
            f_grade_count,
            grade_capped,
            files_reported: total_files,
            files_truncated: false,
            list_filter: None,
            ungraded_files: Vec::new(),
            // Set by `record_cross_file_duplication` when a PROJECT walk has
            // actually run the detector. `aggregate` alone cannot know: it is
            // handed scores, not files.
            cross_file_duplication_ratio: None,
            cross_file_duplication_unmeasured: None,
            cross_file_duplication_coverage: None,
        }
    }

    /// Record the project-wide duplication verdict on this score.
    ///
    /// Kept separate from `aggregate` because `aggregate` receives per-file
    /// scores and has no access to the files themselves — and because a caller
    /// that never ran the detector must not be able to imply that it did.
    pub(crate) fn record_cross_file_duplication(
        &mut self,
        measured: &crate::tdg::cross_file_duplication::CrossFileDuplication,
    ) {
        self.cross_file_duplication_ratio = measured.ratio;
        self.cross_file_duplication_coverage = Some(CrossFileDuplicationCoverage {
            measured: measured.files_measured,
            total: measured.files_total,
        });
        if let Some(reason) = measured.unmeasured_reason.clone() {
            self.cross_file_duplication_unmeasured = Some(reason);
            // GH #704's convention: an unmeasured field is null AND named, so
            // "could not measure" can never be read as "measured, and fine".
            let field = "cross_file_duplication_ratio".to_string();
            if !self.not_measured.contains(&field) {
                self.not_measured.push(field);
            }
        }
    }

    /// Keep only the `limit` WORST-scoring files in `files` (`limit == 0`
    /// keeps every file), always in a deterministic worst-first order.
    ///
    /// This is what `analyze tdg -n/--top-files` means. The flag used to be a
    /// complete no-op: `-n 5`, `-n 10` and `-n 100000` returned identical
    /// output, and `-n 3` on a 1593-file tree still emitted all 1593 entries.
    ///
    /// Truncation NEVER touches `total_files`, `grade_distribution`,
    /// `language_distribution`, `average_score` or `f_grade_count` — those stay
    /// whole-project — and it records `files_reported` + `files_truncated` so a
    /// capped list can never be mistaken for the full population.
    pub fn limit_to_worst_files(&mut self, limit: usize) {
        // Deterministic order regardless of filesystem walk order: worst score
        // first, ties broken by path.
        self.files.sort_by(|a, b| {
            a.total
                .partial_cmp(&b.total)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.file_path.cmp(&b.file_path))
        });

        if limit > 0 && limit < self.files.len() {
            self.files.truncate(limit);
            self.files_truncated = true;
            self.list_filter = Some("--top-files".to_string());
        } else {
            self.files_truncated = false;
        }
        self.files_reported = self.files.len();
    }

    /// The grade `average_score` maps to BEFORE the F-grade cap.
    ///
    /// `average_grade` alone is not enough to render an honest headline: when
    /// `grade_capped` is true the printed grade is deliberately worse than the
    /// printed score, and a reader needs to be told what it would otherwise
    /// have been.
    ///
    /// `None` when nothing was analysed — see `average_score` (GH #704).
    #[must_use]
    pub fn uncapped_grade(&self) -> Option<Grade> {
        self.average_score.map(Grade::from_score)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Calculate the average.
    pub fn average(&self) -> TdgScore {
        if self.files.is_empty() {
            // No files analyzed — return zero score, not perfect score.
            //
            // The COMPONENTS have to be zeroed by hand: `..TdgScore::default()`
            // seeds every component at its category maximum (25/20/20/15/10/10,
            // which is exactly 100), so `pmat tdg <dir> --include-components`
            // over a directory where every file failed to parse printed a
            // full-marks breakdown summing to 100 beside a total of 0.0 — and
            // an EMPTY directory printed the byte-identical breakdown, which is
            // how you can tell it measured nothing. Nothing was measured here,
            // so nothing is claimed.
            return TdgScore {
                total: 0.0,
                grade: crate::tdg::Grade::F,
                confidence: 0.0,
                structural_complexity: 0.0,
                semantic_complexity: 0.0,
                duplication_ratio: 0.0,
                coupling_score: 0.0,
                doc_coverage: 0.0,
                consistency_score: 0.0,
                entropy_score: 0.0,
                ..TdgScore::default()
            };
        }

        let mut avg = TdgScore::default();
        let count = self.files.len() as f32;

        // Reported, not applied: contract coverage no longer rewrites a grade
        // (see `Grade::from_score`), but the aggregate should still say whether
        // every file it covers was contract-covered.
        avg.has_contract_coverage = self.files.iter().all(|s| s.has_contract_coverage);

        avg.structural_complexity = self
            .files
            .iter()
            .map(|s| s.structural_complexity)
            .sum::<f32>()
            / count;
        avg.semantic_complexity = self
            .files
            .iter()
            .map(|s| s.semantic_complexity)
            .sum::<f32>()
            / count;
        avg.duplication_ratio = self.files.iter().map(|s| s.duplication_ratio).sum::<f32>() / count;
        avg.coupling_score = self.files.iter().map(|s| s.coupling_score).sum::<f32>() / count;
        avg.doc_coverage = self.files.iter().map(|s| s.doc_coverage).sum::<f32>() / count;
        avg.consistency_score = self.files.iter().map(|s| s.consistency_score).sum::<f32>() / count;
        avg.entropy_score = self.files.iter().map(|s| s.entropy_score).sum::<f32>() / count;
        avg.confidence = self.files.iter().map(|s| s.confidence).sum::<f32>() / count;

        // Which language this project IS.
        if let Some(lang) = dominant_language(&self.language_distribution) {
            avg.language = lang;
        }

        avg.calculate_total();

        // The project has exactly ONE score, and it is `average_score` — the
        // mean of the per-file totals that `aggregate` already computed and
        // that `analyze tdg --format json` serialises.
        //
        // `calculate_total()` above re-derives a total from the *component*
        // means instead, and the two disagree whenever any file hit the
        // >100 → /1.1 rescale branch: on a two-file fixture the components
        // said 87.75 while `average_score` said 82.75, so `pmat tdg --format
        // json` printed 87.75 and the SARIF/JSON project properties printed
        // 82.75 for the same run. The components stay as the breakdown; the
        // headline number and its grade come from the aggregate.
        //
        // GH #704 made both fields `Option`, but this arm only runs with
        // `files` non-empty, where `aggregate` always produced a `Some`. If a
        // hand-built score somehow reaches here without one, the
        // component-derived `calculate_total()` figures above are left in
        // place rather than overwritten with a stand-in.
        if let Some(score) = self.average_score {
            avg.total = score;
            avg.grade = self
                .average_grade
                .unwrap_or_else(|| crate::tdg::Grade::from_score(score));
        }
        avg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Comparison.
pub struct Comparison {
    pub source1: TdgScore,
    pub source2: TdgScore,
    pub delta: f32,
    pub improvement_percentage: f32,
    pub winner: String,
    pub improvements: Vec<String>,
    pub regressions: Vec<String>,
}

impl Comparison {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(source1: TdgScore, source2: TdgScore) -> Self {
        let delta = source2.total - source1.total;
        let improvement_percentage = if source1.total > 0.0 {
            (delta / source1.total) * 100.0
        } else {
            0.0
        };

        let winner = if source2.total > source1.total {
            source2
                .file_path
                .as_ref()
                .map_or_else(|| "source2".to_string(), |p| p.display().to_string())
        } else {
            source1
                .file_path
                .as_ref()
                .map_or_else(|| "source1".to_string(), |p| p.display().to_string())
        };

        let mut improvements = Vec::new();
        let mut regressions = Vec::new();

        if source2.structural_complexity > source1.structural_complexity {
            improvements.push(format!(
                "Structural complexity improved by {:.1}",
                source2.structural_complexity - source1.structural_complexity
            ));
        } else if source2.structural_complexity < source1.structural_complexity {
            regressions.push(format!(
                "Structural complexity degraded by {:.1}",
                source1.structural_complexity - source2.structural_complexity
            ));
        }

        if source2.semantic_complexity > source1.semantic_complexity {
            improvements.push(format!(
                "Semantic complexity improved by {:.1}",
                source2.semantic_complexity - source1.semantic_complexity
            ));
        } else if source2.semantic_complexity < source1.semantic_complexity {
            regressions.push(format!(
                "Semantic complexity degraded by {:.1}",
                source1.semantic_complexity - source2.semantic_complexity
            ));
        }

        if source2.duplication_ratio > source1.duplication_ratio {
            improvements.push(format!(
                "Code duplication reduced by {:.1}",
                source2.duplication_ratio - source1.duplication_ratio
            ));
        } else if source2.duplication_ratio < source1.duplication_ratio {
            regressions.push(format!(
                "Code duplication increased by {:.1}",
                source1.duplication_ratio - source2.duplication_ratio
            ));
        }

        if source2.doc_coverage > source1.doc_coverage {
            improvements.push(format!(
                "Documentation coverage improved by {:.1}",
                source2.doc_coverage - source1.doc_coverage
            ));
        } else if source2.doc_coverage < source1.doc_coverage {
            regressions.push(format!(
                "Documentation coverage decreased by {:.1}",
                source1.doc_coverage - source2.doc_coverage
            ));
        }

        Self {
            source1,
            source2,
            delta,
            improvement_percentage,
            winner,
            improvements,
            regressions,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod no_contradiction_tests {
    use super::*;

    fn file_score(total: f32, language: Language, has_contract_coverage: bool) -> TdgScore {
        // Drive the real scoring path so `grade` is whatever pmat would print.
        let mut score = TdgScore {
            language,
            has_contract_coverage,
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

    /// GH #680: a single-file project reported files[0].grade "AMinus" and
    /// average_grade "APlus" for the same score — two grades for one number in
    /// one document.
    #[test]
    fn single_file_project_grade_equals_that_files_grade() {
        let file = file_score(99.7, Language::Rust, false);
        let expected = file.grade;
        let project = ProjectScore::aggregate(vec![file]);

        assert_eq!(project.total_files, 1);
        assert_eq!(
            project.average_grade,
            Some(expected),
            "average_grade must equal the only file's grade"
        );
        assert_eq!(
            project.average_grade,
            Some(project.files[0].grade),
            "the same score must map to the same grade in both places"
        );
    }

    /// GH #680, second round. This test previously asserted the OPPOSITE —
    /// `assert_eq!(project.average_grade, Grade::AMinus)` for two files each
    /// totalling 100.0 — i.e. it pinned the defect. Contract coverage is
    /// unmeasured for any project without `contracts/binding.yaml`, so that cap
    /// made `APlus`/`A` unreachable at any score. Rewritten to assert the
    /// corrected contract: a perfect project grades A+ whether or not contract
    /// coverage was measured, in both the file and the project position.
    #[test]
    fn perfect_project_reaches_the_top_grade_regardless_of_contract_coverage() {
        for coverage in [false, true] {
            let files = vec![
                file_score(100.0, Language::Rust, coverage),
                file_score(100.0, Language::Rust, coverage),
            ];
            let project = ProjectScore::aggregate(files);
            assert_eq!(
                project.average_grade,
                Some(Grade::APlus),
                "100.0 must grade A+ (has_contract_coverage={coverage})"
            );
            assert_eq!(project.files[0].grade, Grade::APlus);
            assert_eq!(project.average().grade, Grade::APlus);
        }
    }

    /// The grade is a function of the score and of nothing else: flipping an
    /// unrelated input must not move it.
    #[test]
    fn grade_depends_only_on_the_score() {
        for total in [100.0, 97.0, 92.0, 87.0, 81.0, 60.0, 10.0] {
            let with = file_score(total, Language::Rust, true);
            let without = file_score(total, Language::Rust, false);
            assert_eq!(
                with.grade, without.grade,
                "contract coverage must not change the grade of {total}"
            );
            assert_eq!(with.grade, Grade::from_score(with.total));
        }
    }

    /// GH #673: identical input flipped between Rust and Markdown run to run
    /// because the winner of a tie came out of HashMap iteration order.
    /// Deterministic over many freshly built maps, and source beats docs.
    #[test]
    fn tied_language_distribution_is_deterministic() {
        for _ in 0..64 {
            let files = vec![
                file_score(90.0, Language::Rust, true),
                file_score(90.0, Language::Markdown, true),
                file_score(90.0, Language::Rust, true),
                file_score(90.0, Language::Markdown, true),
            ];
            let project = ProjectScore::aggregate(files);
            assert_eq!(
                project.average().language,
                Language::Rust,
                "a 2-2 Rust/Markdown tie must always resolve the same way"
            );
        }
    }

    /// DETERMINISM (round-3 sweep): `analyze tdg --format json` emitted
    /// `grade_distribution` in a different KEY ORDER on every run — 6 runs over
    /// an unchanged tree gave 6 orders (`["B","BMinus","BPlus","A","APlus",
    /// "AMinus"]`, `["BMinus","BPlus","B","APlus","AMinus","A"]`, …) while the
    /// counts summed to a stable 495 every time. Serde emits a map in iteration
    /// order and `HashMap`'s is randomised per process.
    ///
    /// Each iteration aggregates a FRESH set of scores, so each builds a fresh
    /// map: the in-process stand-in for "run the binary again".
    #[test]
    fn distribution_key_order_is_stable_across_fresh_aggregations() {
        fn serialized() -> (String, String) {
            let files = vec![
                file_score(99.0, Language::Rust, false),
                file_score(96.0, Language::Python, false),
                file_score(92.0, Language::Rust, false),
                file_score(88.0, Language::Markdown, false),
                file_score(84.0, Language::TypeScript, false),
                file_score(81.0, Language::Go, false),
                file_score(75.0, Language::Rust, false),
            ];
            let project = ProjectScore::aggregate(files);
            (
                serde_json::to_string(&project.grade_distribution).unwrap(),
                serde_json::to_string(&project.language_distribution).unwrap(),
            )
        }

        let (grades, languages) = serialized();
        // More than one key, or the test would pass vacuously.
        assert!(grades.matches(':').count() > 1, "grades: {grades}");
        assert!(languages.matches(':').count() > 1, "languages: {languages}");

        for i in 0..10 {
            assert_eq!(
                serialized(),
                (grades.clone(), languages.clone()),
                "iteration {i}: identical input must serialize byte-identically"
            );
        }

        // Ordered by grade, best first — a reader's order, not a hash seed's.
        let order: Vec<&str> = grades
            .split('"')
            .filter(|piece| {
                !piece.is_empty() && piece.chars().next().is_some_and(char::is_alphabetic)
            })
            .collect();
        // GH #703: read the spellings off `Grade` itself rather than restating
        // them. This list used to be hardcoded to the Rust variant names, which
        // is one of the things that pinned serde's variant-name output in place
        // while every other surface printed "A+"/"A-".
        let all_grades_best_first: Vec<String> =
            Grade::all().iter().map(ToString::to_string).collect();
        let mut expected = order.clone();
        expected.sort_by_key(|name| {
            all_grades_best_first
                .iter()
                .position(|g| g.as_str() == *name)
                .expect("known grade")
        });
        assert_eq!(order, expected, "grade keys must be in grade order");
    }

    /// `pmat tdg <dir> --include-components` over a directory where nothing
    /// could be analysed printed the FULL-MARKS component defaults
    /// (25/20/20/15/10/10, summing to 100) beside `"total": 0.0`, and an empty
    /// directory printed the byte-identical breakdown — proof it was the struct
    /// default and not a measurement.
    #[test]
    fn empty_project_claims_no_component_measurements() {
        let empty = ProjectScore::aggregate(vec![]);
        let avg = empty.average();

        assert_eq!(avg.total, 0.0);
        assert_eq!(avg.grade, Grade::F);
        assert_eq!(avg.confidence, 0.0);
        for (name, value) in [
            ("structural", avg.structural_complexity),
            ("semantic", avg.semantic_complexity),
            ("duplication", avg.duplication_ratio),
            ("coupling", avg.coupling_score),
            ("documentation", avg.doc_coverage),
            ("consistency", avg.consistency_score),
            ("entropy", avg.entropy_score),
        ] {
            assert_eq!(
                value, 0.0,
                "{name} claims {value} points for a project where no file was analysed"
            );
        }
        // The old defect in one line: the components must not sum to full marks
        // under a total of zero.
        let breakdown_sum = avg.structural_complexity
            + avg.semantic_complexity
            + avg.duplication_ratio
            + avg.coupling_score
            + avg.doc_coverage
            + avg.consistency_score;
        assert_eq!(breakdown_sum, 0.0, "breakdown sums to {breakdown_sum}");
    }

    /// The F-grade cap makes the grade non-monotone in the score on purpose, so
    /// the aggregate has to be able to say what the grade would have been.
    #[test]
    fn capped_project_reports_the_grade_it_would_have_had() {
        let mut files = vec![file_score(100.0, Language::Rust, false); 19];
        files.push(file_score(10.0, Language::Rust, false));
        let project = ProjectScore::aggregate(files);

        assert_eq!(project.f_grade_count, 1);
        assert!(
            project.grade_capped,
            "one F-grade file must cap the project"
        );
        assert_eq!(project.average_grade, Some(Grade::B));
        assert_eq!(
            project.uncapped_grade(),
            project.average_score.map(Grade::from_score),
            "the uncapped grade is the score's own band"
        );
        assert!(
            project.uncapped_grade().expect("20 files were analysed") < Grade::B,
            "the fixture must actually exercise the cap"
        );
    }

    /// GH #704: an empty analysis has no score and no grade.
    ///
    /// `analyze tdg` over a directory with nothing to grade used to report
    /// `Average Score: 0.0/100 (F)` beside `Total Files: 0` — in the table and
    /// byte-for-byte in the JSON — which reads as "measured, and terrible".
    /// The unmeasured fields are now `None` (serialised `null`) and named in
    /// `not_measured`.
    #[test]
    fn empty_aggregate_reports_no_score_and_no_grade() {
        let project = ProjectScore::aggregate(vec![]);

        assert_eq!(project.total_files, 0);
        assert_eq!(
            project.average_score, None,
            "0 files analysed cannot yield an average score"
        );
        assert_eq!(
            project.average_grade, None,
            "0 files analysed cannot yield a grade"
        );
        assert_eq!(project.uncapped_grade(), None);
        assert_eq!(
            project.not_measured,
            vec!["average_score".to_string(), "average_grade".to_string()],
            "unmeasured fields must be disclosed by name"
        );

        let json = serde_json::to_value(&project).expect("ProjectScore serialises");
        assert!(
            json["average_score"].is_null(),
            "average_score must be null, got {}",
            json["average_score"]
        );
        assert!(
            json["average_grade"].is_null(),
            "average_grade must be null, got {}",
            json["average_grade"]
        );
    }

    /// The measured path keeps reporting real numbers with an empty
    /// `not_measured` list.
    #[test]
    fn measured_aggregate_reports_score_and_grade() {
        let project = ProjectScore::aggregate(vec![file_score(85.0, Language::Rust, false)]);

        assert_eq!(project.average_score, Some(85.0));
        assert!(project.average_grade.is_some());
        assert!(
            project.not_measured.is_empty(),
            "nothing is unmeasured when a file was analysed"
        );
    }

    /// Issue #1090: documentation was pooled into the project score.
    ///
    /// `aggregate` took an unweighted mean over EVERY graded file, and the AST
    /// walk grades `.md` and `.yaml` (`Policy::ast()` sets `grades_markup`).
    /// A 217-byte Markdown stub grades 95.34/A+, so an 85.0/A- fixture climbed
    /// 90.17 → 91.89 → 92.75 → 93.27 as four `.md` files landed beside it with
    /// the source untouched — measured on the released binary, and each rung is
    /// the unweighted mean of the 85.0 file and k stubs at 95.34, so one `.md`
    /// alone is the first rung, 85.0 → 90.17.
    ///
    /// The fixture mirrors that measurement: one 85.0 Rust file, four Markdown
    /// files and one YAML file at the grade a doc stub really gets.
    #[test]
    fn documentation_cannot_move_the_project_score() {
        let source = file_score(85.0, Language::Rust, false);
        let expected = source.total;

        let mut files = vec![source];
        files.extend(vec![file_score(95.34, Language::Markdown, false); 4]);
        files.push(file_score(95.34, Language::Yaml, false));

        let project = ProjectScore::aggregate(files);

        assert_eq!(
            project.total_files, 6,
            "every graded file is still counted and still listed"
        );
        assert_eq!(
            project.average_score,
            Some(expected),
            "five documentation files must not move a one-file Rust project's score"
        );

        // The distribution stays a census of what was graded — the docs are
        // still visible, they just no longer vote on the score.
        assert_eq!(project.language_distribution[&Language::Markdown], 4);
        assert_eq!(project.language_distribution[&Language::Yaml], 1);
    }

    /// COUNTER-TEST for the one above: the fix must be "documentation does not
    /// vote", not "extra files do not vote". The identical fixture with the
    /// five extras written in Rust moves the score exactly as it always did.
    #[test]
    fn more_source_files_still_move_the_project_score() {
        let mut files = vec![file_score(85.0, Language::Rust, false)];
        files.extend(vec![file_score(95.34, Language::Rust, false); 4]);
        files.push(file_score(95.34, Language::Rust, false));

        let average = ProjectScore::aggregate(files)
            .average_score
            .expect("six Rust files were analysed");
        assert!(
            average > 93.0,
            "five more source files at 95.34 must pull an 85.0 project up, got {average}"
        );
    }

    /// COUNTER-TEST bounding the correction the other way. "Prefer source" must
    /// not become "refuse to score a tree that has none": a documentation tree
    /// really is a documentation tree, and answering `average_score: null` for
    /// it would replace one wrong answer with "not measured".
    ///
    /// This also guards the single-score caller
    /// `ProjectScore::aggregate(vec![score])` in
    /// `mcp_pmcp::tool_functions::tdg_tools_storage`, which can legitimately be
    /// handed one `.md` file.
    #[test]
    fn a_docs_only_project_is_still_scored() {
        for language in [Language::Markdown, Language::Yaml, Language::Unknown] {
            let files = vec![file_score(95.34, language, false); 3];
            let expected = files[0].total;
            let project = ProjectScore::aggregate(files);

            assert!(
                project.average_score.is_some(),
                "a {language}-only tree must still be scored"
            );
            let average = project.average_score.expect("asserted on the line above");
            assert!(
                (average - expected).abs() < 0.01,
                "a {language}-only tree scores what its files score, got {average}"
            );
            assert!(project.average_grade.is_some(), "{language}");
            assert!(
                project.not_measured.is_empty(),
                "{language}: the score was measured, so nothing is disclosed as unmeasured"
            );
        }

        let lone_doc = ProjectScore::aggregate(vec![file_score(95.34, Language::Markdown, false)]);
        assert!(
            lone_doc.average_score.is_some(),
            "a single-file aggregate over one .md must not degrade to null"
        );
    }

    /// #1090 narrows the MEAN and nothing else: `f_grade_count` is a census row.
    ///
    /// The fix that stopped documentation voting on the score narrowed
    /// `f_grade_count` along with it, which changed what a `pub`, serialized
    /// field meant without any renderer disclosing the change. This fixture
    /// separates the two claims on one input — an F-grade `.md` beside perfect
    /// Rust — because they pull in opposite directions and only one of them was
    /// ever measured:
    ///
    /// * the `.md` must NOT move `average_score` (#1090, the measured defect);
    /// * it must still BE an F-grade file everywhere the payload counts them —
    ///   `grade_distribution`, `f_grade_count`, and
    ///   `tdg::quality_gate::f_grade::FGradeGate`, which counts over every file
    ///   in the baseline and fails the build on this one.
    ///
    /// The earlier version of this test asserted `f_grade_count == 0` and
    /// `!grade_capped` on this fixture — and `new_tdg_handler` prints its
    /// `**Grade Capped:**` line only when `grade_capped`, so the report carried
    /// no cap line at all above an "F | 1" row of its own distribution table.
    #[test]
    fn an_f_grade_document_is_counted_even_though_it_cannot_move_the_mean() {
        let mut files = vec![file_score(100.0, Language::Rust, false); 19];
        files.push(file_score(10.0, Language::Markdown, false));
        let project = ProjectScore::aggregate(files);

        // #1090, unchanged: prose does not vote on the code's mean.
        assert_eq!(
            project.average_score,
            Some(100.0),
            "an F-grade Markdown file must not drag the code's mean down"
        );
        assert_eq!(
            project.uncapped_grade(),
            Some(Grade::APlus),
            "the score's own band is unaffected by the document"
        );

        // …and it is still an F-grade file, in every count the payload carries.
        assert_eq!(
            project.grade_distribution.get(&Grade::F),
            Some(&1),
            "the census must show the F-grade Markdown file"
        );
        assert_eq!(
            project.f_grade_count, 1,
            "f_grade_count is the census row, not a second population"
        );
        assert!(
            project.grade_capped,
            "an F-grade file caps the grade — the same file FGradeGate fails on"
        );
        assert_eq!(project.average_grade, Some(Grade::B));
        assert_eq!(project.files.len(), 20, "and it is still listed");
    }

    /// The reported symptom, as a fixture: 2 F-grade `.rs` and 3 F-grade `.md`.
    ///
    /// `cli::handlers::new_tdg_handler` prints `**Grade Capped:** yes (N
    /// F-grade file(s))` from `f_grade_count` and, a few lines below it in the
    /// same document, a `## Grade Distribution` table whose F row is
    /// `grade_distribution[&Grade::F]`. While the two counted different
    /// populations that report read "yes (2 F-grade file(s))" over an "F | 5"
    /// row, under labels a reader takes for the same quantity.
    ///
    /// `aggregate` now reads `f_grade_count` out of `grade_distribution`, so
    /// the equality is structural; this pins it against a population where the
    /// two used to differ.
    #[test]
    fn the_cap_line_and_the_f_row_report_one_population() {
        let mut files = vec![file_score(100.0, Language::Rust, false); 10];
        files.extend(vec![file_score(10.0, Language::Rust, false); 2]);
        files.extend(vec![file_score(10.0, Language::Markdown, false); 3]);

        let project = ProjectScore::aggregate(files);

        assert_eq!(
            project.grade_distribution.get(&Grade::F),
            Some(&5),
            "the census counts every F-grade file that was graded"
        );
        assert_eq!(
            project.f_grade_count,
            project
                .grade_distribution
                .get(&Grade::F)
                .copied()
                .unwrap_or(0),
            "the capped line and the F row must not be two different counts"
        );
        assert_eq!(project.f_grade_count, 5);

        // The mean is still the source files' — 10 at 100.0 and 2 at 10.0 —
        // with the three documents excluded from it and only from it.
        let average = project.average_score.expect("12 source files were graded");
        assert!(
            (average - 85.0).abs() < 0.01,
            "the mean is the source files' own, got {average}"
        );
    }

    /// A clear majority still wins; the tiebreak only settles equal counts.
    ///
    /// UPDATED for #1073, not deleted. The claim — "the tiebreak settles equal
    /// counts and nothing else" — is still exactly right, and still worth a
    /// test. The FIXTURE was wrong: it made the claim with two Markdown files
    /// against one Rust file and asserted `Markdown`, which is the defect
    /// #1073 reports, written down as the expected answer. A project's
    /// `language` is now the plurality among the languages that carry CODE
    /// (`dominant_language`), so a majority of DOCS proves nothing about the
    /// tiebreak between source languages.
    ///
    /// Restated between two source languages, where the tiebreak actually
    /// operates: Python is the majority and wins, even though `Rust` sorts
    /// ahead of it and would win a tie.
    #[test]
    fn language_majority_still_wins_over_the_tiebreak() {
        let files = vec![
            file_score(90.0, Language::Python, true),
            file_score(90.0, Language::Python, true),
            file_score(90.0, Language::Rust, true),
        ];
        let project = ProjectScore::aggregate(files);
        assert_eq!(project.average().language, Language::Python);
    }

    /// A file that was walked but refused must reach the payload.
    ///
    /// `analyze tdg --format json` on the broken fixture answered
    /// `{total_files: 1, average_score: 100.0, not_measured: []}` over the one
    /// Python file that parsed, with `src/main.rs` — the crate's only Rust file
    /// — gone without trace: the refusal was an `eprintln!` and stderr is not
    /// part of a JSON payload.
    #[test]
    fn ungraded_files_are_serialised_so_a_json_consumer_sees_the_skip() {
        let mut project = ProjectScore::aggregate(vec![file_score(100.0, Language::Python, true)]);
        project.ungraded_files.push(UngradedFile {
            path: "./src/main.rs".to_string(),
            reason: "cannot parse string into token stream".to_string(),
        });

        let json = serde_json::to_value(&project).expect("ProjectScore serialises");
        let listed = json["ungraded_files"]
            .as_array()
            .expect("ungraded_files must be an array on the wire");
        assert_eq!(listed.len(), 1, "{json}");
        assert_eq!(listed[0]["path"], "./src/main.rs");
        assert!(
            listed[0]["reason"].as_str().is_some_and(|r| !r.is_empty()),
            "a skip must carry its reason: {json}"
        );
    }

    /// A whole analysis with nothing refused says so with an empty list, never
    /// by omitting the key.
    #[test]
    fn a_complete_analysis_reports_an_empty_ungraded_list() {
        let project = ProjectScore::aggregate(vec![file_score(85.0, Language::Rust, false)]);
        let json = serde_json::to_value(&project).expect("ProjectScore serialises");
        assert_eq!(
            json["ungraded_files"].as_array().map(Vec::len),
            Some(0),
            "the key must always be present: {json}"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod project_language_is_the_dominant_source_language_tests {
    use super::*;

    fn graded(language: Language) -> TdgScore {
        TdgScore {
            language,
            structural_complexity: 20.0,
            semantic_complexity: 16.0,
            duplication_ratio: 16.0,
            coupling_score: 12.0,
            doc_coverage: 8.0,
            consistency_score: 8.0,
            entropy_score: 0.0,
            ..TdgScore::default()
        }
    }

    /// Issue #1073. `pmat tdg ~/src/pforge --format json` reported
    /// `"language": "Markdown"` for a cargo workspace, while
    /// `pmat tdg ~/src/pforge/crates` — a SUBDIRECTORY of the same tree —
    /// reported `"Rust"`. The winner was a plurality over every graded file,
    /// and a doc-heavy Rust workspace has more `.md` files than `.rs` files.
    #[test]
    fn docs_outnumbering_code_does_not_make_the_project_a_markdown_project() {
        let mut files = vec![graded(Language::Rust); 3];
        files.extend(vec![graded(Language::Markdown); 10]);

        let project = ProjectScore::aggregate(files);

        // The distribution is untouched — Markdown really is the plurality, and
        // that fact stays readable.
        assert_eq!(project.language_distribution[&Language::Markdown], 10);
        assert_eq!(
            project.average().language,
            Language::Rust,
            "the project's language is the language its CODE is written in"
        );
    }

    /// The counter-test that bounds the correction. A tree that really holds
    /// nothing but documentation must still be reported as Markdown: "prefer
    /// source" must not become "never say Markdown", which would be a second
    /// false label in the other direction.
    #[test]
    fn a_tree_with_no_source_files_is_still_labelled_by_what_it_holds() {
        let project = ProjectScore::aggregate(vec![graded(Language::Markdown); 4]);
        assert_eq!(project.average().language, Language::Markdown);

        let project = ProjectScore::aggregate(vec![graded(Language::Yaml); 2]);
        assert_eq!(project.average().language, Language::Yaml);
    }

    /// …and the plurality still decides BETWEEN source languages: preferring
    /// source code must not degrade into "whichever source language sorts
    /// first".
    #[test]
    fn the_plurality_still_decides_among_source_languages() {
        let mut files = vec![graded(Language::Python); 9];
        files.push(graded(Language::Rust));
        files.extend(vec![graded(Language::Markdown); 40]);

        assert_eq!(
            ProjectScore::aggregate(files).average().language,
            Language::Python
        );
    }
}
