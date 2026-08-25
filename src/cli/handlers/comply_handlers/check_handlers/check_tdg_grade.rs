// CB-200: TDG Grade Gate (#214)
//
// Reads the SQLite agent-context index and fails if definitions fall below a
// configurable minimum TDG grade (default A).
//
// The index is the project's own `.pmat/context.db` when it has one, and
// otherwise the copy pmat keeps for that project OUTSIDE it, under the user's
// cache directory, built here on demand. See `TdgIndex` for why the second
// exists: without it this gate could only ever fail on a machine that happened
// to have an index lying around, which is never a CI checkout (#1008).

use crate::models::comply_config::ComplyConfig;
use crate::tdg::grade::GRADE_VARIANTS;
use crate::tdg::Grade;
use std::path::{Path, PathBuf};

use super::types::*;

/// Convert a TDG grade letter to a numeric ordinal for comparison.
/// The spellings a floor ADMITS, computed from the proved order.
///
/// Returns `None` when the threshold is not a grade this codebase produces, so
/// an unreadable threshold fails the rule instead of yielding an empty set that
/// reads as "nothing violates".
///
/// This replaces a private `grade_ordinal` over `["A","B","C","D","F"]` with a
/// `_ => 5` catch-all. Both halves were wrong. The five-letter list could never
/// match a MODIFIED grade against `WHERE tdg_grade IN (...)`, so at a floor of
/// "A" the gate saw 247 violations and could not see 1,719. And the catch-all
/// ranked every modified grade WORSE THAN F, so `grades_below("A-")` was empty
/// and the caller returned Pass — `.pmat-metrics.toml:44` declares exactly that
/// spelling.
///
/// The order is `Grade`'s, anchored to the score bands in
/// `contracts/lean/Theorems/Tdg/Grade.lean::Grade_Rank_Anchored_To_Score`.
fn passing_spellings(min_grade: &str) -> Option<Vec<&'static str>> {
    let floor = Grade::from_variant_name(min_grade.trim())?;
    Some(
        GRADE_VARIANTS
            .iter()
            .copied()
            .filter(|g| Grade::from_variant_name(g).is_some_and(|x| x.meets_threshold(floor)))
            .collect(),
    )
}

struct TdgViolation {
    file_path: String,
    function_name: String,
    tdg_grade: String,
    complexity: u32,
    start_line: usize,
}

/// How many below-floor definitions this project has already agreed to carry,
/// read from `.pmat-gates.toml` `[tdg] baseline`.
///
/// CB-200 is a RATCHET, not a threshold, and the distinction is the whole
/// design. `.pmat-ratchet.toml`'s own header states it: "A number in a config
/// file that nobody re-runs is not a gate; it is a wish with a colon after it."
/// `.pmat-metrics.toml:45` is this repository's worked example —
/// `max_unwrap_calls = 100`, annotated `Current: 570`, in a tree measuring
/// 20,390: three numbers, no two agreeing, nothing reading the key, green
/// throughout.
///
/// This number is different in exactly one way, and it is the only way that
/// matters: it is RE-DERIVED on every run by the same query that produced it,
/// and compared. It can never quietly become a transcription. It is a record of
/// debt, not a permission to add more — a definition below the floor that
/// pushes the count past it fails the gate, closed, and the absolute count is
/// printed on every outcome so "passing" can never be read as "clean".
///
/// It does NOT touch `min_tdg_grade` and adds no exclude glob. Both of those
/// are threshold-lowering in disguise: they make debt invisible, where this
/// keeps every unit of it counted and reported.
#[derive(Debug, PartialEq, Eq)]
enum TdgBaseline {
    /// No `baseline` key. Zero tolerance — any violation fails, exactly as
    /// before. This is the default precisely so that ratcheting THIS repo
    /// cannot silently relax any other repo that runs pmat.
    Absent,
    /// The count this project last agreed to hold flat.
    Held(usize),
    /// The key is present and is not a count. Fails, for the same reason an
    /// unparseable `min_grade` fails: a bound nobody can read must not be read
    /// as a bound nothing exceeds.
    Unreadable(String),
}

struct TdgGateOverrides {
    min_grade: Option<String>,
    exclude: Vec<String>,
    baseline: TdgBaseline,
}

impl Default for TdgGateOverrides {
    /// No overrides: the floor comes from `.pmat.yaml`, nothing extra is
    /// excluded, and the gate has zero tolerance. An unreadable or unparsable
    /// `.pmat-gates.toml` lands here, which fails CLOSED — the excludes vanish
    /// and the baseline vanishes with them, so a typo cannot buy headroom.
    fn default() -> Self {
        Self {
            min_grade: None,
            exclude: Vec::new(),
            baseline: TdgBaseline::Absent,
        }
    }
}

/// A `baseline` value is a count or it is nothing. A string, a float, a
/// negative — anything that is not a non-negative integer — is reported rather
/// than rounded down to "no baseline", because "the key you wrote does nothing"
/// and "you configured zero tolerance" are opposite claims and look identical
/// from the outside.
fn parse_tdg_baseline(value: Option<&toml::Value>) -> TdgBaseline {
    let Some(value) = value else {
        return TdgBaseline::Absent;
    };
    match value.as_integer() {
        Some(n) if n >= 0 => TdgBaseline::Held(n as usize),
        _ => TdgBaseline::Unreadable(value.to_string()),
    }
}

fn load_tdg_gate_overrides(project_path: &Path) -> TdgGateOverrides {
    let path = project_path.join(".pmat-gates.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return TdgGateOverrides::default();
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        return TdgGateOverrides::default();
    };
    let tdg = table.get("tdg");
    let min_grade = tdg
        .and_then(|t| t.get("min_grade"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let exclude = tdg
        .and_then(|t| t.get("exclude"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let baseline = parse_tdg_baseline(tdg.and_then(|t| t.get("baseline")));
    TdgGateOverrides {
        min_grade,
        exclude,
        baseline,
    }
}

/// Is `.pmat/context.db` older than the sources it claims to describe?
///
/// `pub(crate)` because `services::tdg_baseline` asks the same question before
/// it treats CB-200's count as the count at HEAD. It CALLS this rather than
/// re-deriving it: two implementations of one predicate is how CB-200 went
/// blind for a release (see [`passing_spellings`]).
pub(crate) fn is_index_stale(project_path: &Path, db_path: &Path) -> bool {
    let db_mtime = match std::fs::metadata(db_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    for dir_name in ["src", "lib"] {
        let dir = project_path.join(dir_name);
        if !dir.exists() {
            continue;
        }
        if has_newer_source_file(&dir, db_mtime) {
            return true;
        }
    }
    false
}

fn has_newer_source_file(dir: &Path, threshold: std::time::SystemTime) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            has_newer_source_file(&path, threshold)
        } else {
            is_source_file(&path) && entry_is_newer(&entry, threshold)
        }
    })
}

/// Is a directory entry's mtime newer than `threshold`? (false on any I/O error)
fn entry_is_newer(entry: &std::fs::DirEntry, threshold: std::time::SystemTime) -> bool {
    entry
        .metadata()
        .and_then(|m| m.modified())
        .map(|mtime| mtime > threshold)
        .unwrap_or(false)
}

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext,
                "rs" | "ts"
                    | "tsx"
                    | "js"
                    | "jsx"
                    | "py"
                    | "go"
                    | "java"
                    | "kt"
                    | "swift"
                    | "c"
                    | "cpp"
                    | "cs"
            )
        })
}

// `rebuild_index` used to live here: CB-200 called
// `AgentContextIndex::build[_incremental]` and SAVED the result whenever
// `.pmat/context.db` was missing or stale, so merely ASKING for a compliance
// verdict wrote 160KB+ of index into the project under audit and, on a large
// repo, spent minutes doing it. Building the index is `pmat query`'s job; the
// gate now reads what exists and refuses honestly when nothing does (#939).

/// Check TDG grade gate against the SQLite index.
/// Query violations from the context database for grades below threshold
fn query_tdg_violations(
    db_path: &Path,
    passing_grades: &[&str],
) -> Result<Vec<TdgViolation>, ComplianceCheck> {
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Skip,
        message: format!("Failed to open context.db: {e}"),
        severity: Severity::Info,
    })?;
    let placeholders: Vec<String> = passing_grades
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    // NOT IN the PASSING set, never IN a failing list. `IN` answers "no row"
    // for a value it does not list and never an error, so an eleven-letter
    // writer and a five-letter reader coexisted silently for a release.
    // Enumerating what passes makes any future alphabet drift return a row that
    // must be classified, instead of a smaller number nobody can see.
    let sql = format!("SELECT file_path, function_name, tdg_grade, complexity, start_line FROM functions WHERE tdg_grade NOT IN ({})", placeholders.join(", "));
    let mut stmt = conn.prepare(&sql).map_err(|e| ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Skip,
        message: format!("Failed to query context.db: {e}"),
        severity: Severity::Info,
    })?;
    let params: Vec<&dyn rusqlite::types::ToSql> = passing_grades
        .iter()
        .map(|g| g as &dyn rusqlite::types::ToSql)
        .collect();
    stmt.query_map(params.as_slice(), |row| {
        Ok(TdgViolation {
            file_path: row.get(0)?,
            function_name: row.get(1)?,
            tdg_grade: row.get(2)?,
            complexity: row.get::<_, i64>(3)? as u32,
            start_line: row.get::<_, i64>(4)? as usize,
        })
    })
    .map(|iter| iter.filter_map(|r| r.ok()).collect())
    .map_err(|e| ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Skip,
        message: format!("Query failed: {e}"),
        severity: Severity::Info,
    })
}

/// Check if a violation should be excluded (test files or glob patterns)
fn is_tdg_violation_excluded(v: &TdgViolation, exclude_patterns: &[glob::Pattern]) -> bool {
    if v.file_path.contains("/tests/")
        || v.file_path.contains("/test/")
        || v.file_path.ends_with("_test.rs")
        || v.file_path.ends_with("_tests.rs")
    {
        return true;
    }
    let opts = glob::MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    exclude_patterns
        .iter()
        .any(|pat| pat.matches_with(&v.file_path, opts))
}

/// Which index CB-200 measured, and where it came from.
///
/// Two places, tried in this order, and the order is the whole of #1008:
///
/// 1. `<project>/.pmat/context.db` — the project has an index of its own,
///    built by `pmat query`. Read it, exactly as before. A developer's index
///    and CB-200's verdict must not be two different readings of one tree.
/// 2. pmat's own index for this project, under the user's cache directory
///    ([`comply_index_path`](crate::utils::pmat_cache_dir::comply_index_path)),
///    built here if it is missing or stale.
///
/// (2) is the fix. Before it, an audit refused to build an index at all —
/// correctly, because building it in the project writes `.pmat/context.db` and
/// `.pmat/context.idx` into a tree an auditor has no business writing to (#939)
/// — and the consequence was a gate that could only fail on a machine that
/// happened to have an index lying around. A fresh CI checkout never does, so
/// on the one machine whose verdict decides a merge, CB-200 measured nothing
/// and `is_compliant` (which tallies `Fail` only) read that silence as consent.
/// Same tree, same commit, index the only variable: 2 failing checks against 3.
///
/// The audited tree is still never written to. It is the CACHE that gets the
/// index, keyed by project path, and `git status --porcelain` in the audited
/// repository is empty afterwards — which is the property the refusal was
/// protecting and the only one worth keeping.
enum TdgIndex {
    /// `<project>/.pmat/context.db`, built by whoever ran `pmat query`.
    InProject(PathBuf),
    /// pmat's own index for this project, outside it.
    OutOfTree(PathBuf),
}

impl TdgIndex {
    fn db_path(&self) -> &Path {
        match self {
            Self::InProject(path) | Self::OutOfTree(path) => path,
        }
    }
}

/// Say where a measurement came from when it did NOT come from the project.
///
/// Appended to the verdict rather than folded into it: the in-project sentence
/// stays byte-identical, so this cannot move the output of any repository that
/// already has an index — and a reader who is surprised by a CB-200 result on a
/// machine with no `.pmat/` is told, in the verdict itself, which file was read.
fn note_index_provenance(check: ComplianceCheck, index: &TdgIndex) -> ComplianceCheck {
    let TdgIndex::OutOfTree(db) = index else {
        return check;
    };
    let message = format!(
        "{} [measured against {}, the index pmat keeps for this project outside it \u{2014} \
         the audited tree has no .pmat/context.db and was not written to (#1008)]",
        check.message,
        db.display()
    );
    ComplianceCheck { message, ..check }
}

/// Pick the index to read, building pmat's own copy when the project has none.
///
/// The project's own index wins whenever it exists, fresh or stale — a stale
/// one is reported as stale rather than replaced, because that is a decision
/// about the project's file and #1045 already settled it. Only the copy pmat
/// owns is rebuilt here, which is why this can rebuild at all.
fn resolve_tdg_index(project_path: &Path, cache_index: &Path) -> Result<TdgIndex, String> {
    let in_project = project_path.join(".pmat").join("context.db");
    if in_project.exists() {
        return Ok(TdgIndex::InProject(in_project));
    }
    let cached_db = cache_index.with_extension("db");
    if cached_db.exists() && !is_index_stale(project_path, &cached_db) {
        return Ok(TdgIndex::OutOfTree(cached_db));
    }
    match build_out_of_tree_index(project_path, cache_index) {
        Ok(()) => Ok(TdgIndex::OutOfTree(cached_db)),
        // A failed rebuild does not throw away a measurement already in hand: a
        // stale index describes an OLDER tree, `demote_pass_when_stale` refuses
        // to let that read as a pass, and the note names it. Reporting nothing
        // would be strictly less information.
        Err(reason) if cached_db.exists() => {
            crate::status_eprintln!("CB-200: keeping the previous index ({reason})");
            Ok(TdgIndex::OutOfTree(cached_db))
        }
        Err(reason) => Err(reason),
    }
}

/// Build the agent-context index for `project_path` at `index_path`, which is
/// outside `project_path`.
///
/// `AgentContextIndex::build` only walks and reads; `save` writes
/// `<index_path>/manifest.json` and `<index_path>.db`, and `index_path` is in
/// the user's cache. Nothing here touches the audited tree.
///
/// No staging layer is added on top of that, deliberately: `save_to_sqlite`
/// already builds into a process-unique scratch file and `rename`s it into
/// place, so two audits of one project racing here leave a whole index or the
/// previous one, never a half-populated `functions` table — which would
/// under-report violations and read as a pass.
///
/// An index with no definitions in it is an ERROR, not an empty pass. A
/// directory holding no code parses to zero functions, zero functions violate
/// no floor, and "nothing below grade A" is a sentence that would then be
/// printed over a project pmat never read a line of. Absence rendered as
/// success is this codebase's signature defect; the caller turns this Err into
/// the same "not measured" verdict an unbuildable index gets.
fn build_out_of_tree_index(project_path: &Path, index_path: &Path) -> Result<(), String> {
    crate::status_eprintln!(
        "CB-200: no index in {} \u{2014} building pmat's own at {} (the audited tree is not written to)",
        project_path.display(),
        index_path.display()
    );
    let index = crate::services::agent_context::AgentContextIndex::build(project_path)
        .map_err(|e| format!("no index could be built for this project ({e})"))?;
    if index.all_functions().is_empty() {
        return Err(
            "an index built from this project holds no definitions, so there is nothing to grade"
                .to_string(),
        );
    }
    crate::utils::pmat_cache_dir::ensure_parent_dir(index_path)
        .map_err(|e| format!("pmat's cache directory is not writable ({e})"))?;
    // Reclaim entries for projects that no longer exist before adding another
    // 79 MB one. Here rather than on the read path: this is the rare branch,
    // and a cache that is only ever added to is a disk leak with a nice name.
    crate::utils::pmat_cache_dir::sweep_idle_state(
        "index",
        crate::utils::pmat_cache_dir::STATE_MAX_IDLE,
    );
    index
        .save(index_path)
        .map_err(|e| format!("the index could not be saved to pmat's cache ({e})"))?;
    let db_path = index_path.with_extension("db");
    if !db_path.exists() {
        return Err(format!(
            "the index was built but {} was not written",
            db_path.display()
        ));
    }
    Ok(())
}

/// A threshold that cannot be parsed must not be read as a threshold nothing
/// violates. Decided from the CONFIG alone, so it is answered before any index
/// is looked for: "no index" must not launder a broken floor into a skip.
fn unparseable_floor_verdict(min_grade: &str) -> ComplianceCheck {
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "minimum grade {min_grade:?} is not a grade this codebase produces (known: {}). \
             A threshold that cannot be parsed must not be read as a threshold nothing violates.",
            GRADE_VARIANTS.join(", ")
        ),
    )
}

/// The floor admits every grade the codebase produces, so this verdict is
/// derived from the CONFIG and not from the index: no row, fresh or stale, can
/// violate it. It therefore carries no staleness note and is not demoted — the
/// invariant being kept is that a `Pass` never rests on a stale reading, and
/// this one rests on no reading at all. It is also answered before the index is
/// resolved, so a floor of "F" never pays for an index build it cannot use.
fn floor_admits_everything_verdict(min_grade: &str) -> ComplianceCheck {
    tdg_check(
        CheckStatus::Pass,
        Severity::Info,
        format!("Minimum grade {min_grade} \u{2014} no grades below threshold"),
    )
}

/// Every glob a violation may be excluded by: `.pmat.yaml`'s, then
/// `.pmat-gates.toml`'s. A pattern that does not compile is dropped, which
/// keeps a typo from excluding everything.
fn tdg_exclude_patterns(
    comply_config: &ComplyConfig,
    overrides: &TdgGateOverrides,
) -> Vec<glob::Pattern> {
    comply_config
        .thresholds
        .tdg_exclude_paths
        .iter()
        .map(String::as_str)
        .chain(overrides.exclude.iter().map(String::as_str))
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_tdg_grade_gate(
    project_path: &Path,
    comply_config: &ComplyConfig,
) -> ComplianceCheck {
    check_tdg_grade_gate_with_index(
        project_path,
        comply_config,
        &crate::utils::pmat_cache_dir::comply_index_path(project_path),
    )
}

/// CB-200 with the out-of-tree index location passed in.
///
/// A parameter and not an environment variable for the tests' sake: a test that
/// has to `set_var` to stay out of the developer's real cache is a test that
/// cannot run in parallel with any other, and this file has forty that do.
/// `$PMAT_CACHE_DIR` still moves the default — see
/// [`user_cache_root`](crate::utils::pmat_cache_dir::user_cache_root) — it is
/// just not how a test gets a private one.
fn check_tdg_grade_gate_with_index(
    project_path: &Path,
    comply_config: &ComplyConfig,
    cache_index: &Path,
) -> ComplianceCheck {
    let overrides = load_tdg_gate_overrides(project_path);
    let min_grade = overrides
        .min_grade
        .as_deref()
        .unwrap_or(&comply_config.thresholds.min_tdg_grade);
    let Some(passing_grades) = passing_spellings(min_grade) else {
        return unparseable_floor_verdict(min_grade);
    };
    if passing_grades.len() == GRADE_VARIANTS.len() {
        return floor_admits_everything_verdict(min_grade);
    }
    let index = match resolve_tdg_index(project_path, cache_index) {
        Ok(index) => index,
        Err(reason) => return absent_index_verdict(&overrides.baseline, &reason),
    };
    // A stale index still measures something REAL — an OLDER tree — and it is a
    // rebuild that would measure nothing. For the project's own index the
    // staleness is reported instead of silently repaired (#939, #1045); the
    // copy pmat owns was rebuilt above unless the rebuild failed, so reaching
    // here with a stale one means the note is carrying real news.
    let stale = is_index_stale(project_path, index.db_path());
    let violations = match query_tdg_violations(index.db_path(), &passing_grades) {
        Ok(v) => v,
        Err(check) => return check,
    };
    let exclude_patterns = tdg_exclude_patterns(comply_config, &overrides);
    let filtered: Vec<&TdgViolation> = violations
        .iter()
        .filter(|v| !is_tdg_violation_excluded(v, &exclude_patterns))
        .collect();
    let verdict = tdg_grade_verdict(
        &filtered,
        violations.len(),
        min_grade,
        &overrides.baseline,
        if stale { STALE_INDEX_NOTE } else { "" },
    );
    note_index_provenance(demote_pass_when_stale(verdict, stale), &index)
}

/// What a stale index costs the reader, and the command that actually fixes it.
///
/// The advice used to be `pmat query "x"`, which does not refresh an index that
/// already exists (#1045). `load_or_build_index`
/// (`cli/handlers/query_handler/indexing.rs:251`) BUILDS only when both
/// `.pmat/context.idx` and `.pmat/context.db` are missing; otherwise it loads
/// and updates in memory, and `maybe_save_incremental` writes that back only
/// when more than 50 files or 5% of the index changed (#212). Edit a handful of
/// files and the db is never rewritten — so a reader following the advice saw
/// the identical stale verdict, with the identical advice, indefinitely.
/// `--rebuild-index` forces the rebuild and the save.
///
/// This is not academic: `.pmat-gates.toml`'s CB-200 baseline was first
/// committed 216 too high because it was derived against an index that was
/// stale by 265 definitions, and nothing in the loop said so loudly enough.
const STALE_INDEX_NOTE: &str = " (index is stale: source files are newer than .pmat/context.db, \
     so this count describes an OLDER tree - run `pmat query \"x\" --rebuild-index` to refresh; \
     a plain `pmat query` will NOT rewrite an index that already exists unless more than 50 \
     files or 5% of it changed)";

/// A stale index may report debt, but it may never report a clean bill.
///
/// CB-200 reads `.pmat/context.db`. When sources are newer than the db, the
/// count is a measurement of an older tree: the definitions it names may
/// already be fixed, and — the direction that matters — the ones added since
/// are not in it at all. Passing on that is this project's signature defect,
/// absence rendered as success, and it is how the ratchet baseline was first
/// banked 216 units too high.
///
/// `Warn`, not `Fail`, and the choice is deliberate on both sides:
///
/// * not `Pass`, because `retain_blocking_checks` (check.rs:270) switches on
///   `CheckStatus` alone and drops every `Pass`, so under
///   `comply check --failures-only` — the exact invocation `quality-gate.yml`
///   runs — the sentence saying the measurement is stale would be discarded.
///   `Warn` survives into `report.summary.warn`, which is tallied before the
///   list is narrowed.
/// * not `Fail`, because a stale index is not evidence of a regression, and a
///   gate that goes red the moment anyone edits a file after indexing would be
///   turned off within a day. `--strict` still escalates warnings.
///
/// A stale `Fail` or `Warn` is left exactly as it is: staleness never makes a
/// verdict more lenient, only less conclusive.
fn demote_pass_when_stale(check: ComplianceCheck, stale: bool) -> ComplianceCheck {
    if !stale || check.status != CheckStatus::Pass {
        return check;
    }
    ComplianceCheck {
        status: CheckStatus::Warn,
        severity: Severity::Warning,
        message: format!(
            "NOT A VERDICT ON THE CURRENT TREE - the index is stale, so this is not a pass: {}",
            check.message
        ),
        ..check
    }
}

/// CB-200 measured nothing at all: neither the project's index nor pmat's own
/// could be read, and `reason` says which failed and how.
///
/// `Skip` for a project that never opted into the ratchet. It has recorded no
/// baseline to hold, and pmat must not fail every fresh clone of every repo
/// that runs `comply check` — around 40 of 155 checks only run where some
/// state exists, and a blanket "unmeasured is a failure" rule turns a fresh
/// clone into a wall of red and makes failure counts incomparable between
/// machines. That is a different defect, not a fix for this one.
///
/// `Fail` for a project that RECORDED a `[tdg] baseline`. `Skip` is not counted
/// by `ComplianceReport::is_compliant`, which tallies `Fail` only, so a
/// declared ratchet that could not be measured used to report success. A
/// ratchet that did not run has not held; it has not been asked. "We could not
/// measure it" must never read as "it did not regress" — the rule
/// `.pmat-ratchet.toml` already applies to its own baselines, where an
/// UNMEASURABLE metric fails rather than passes.
///
/// Since #1008 this is a genuinely exceptional path rather than the normal one:
/// an absent index is now built out-of-tree (see [`TdgIndex`]), so getting here
/// means the project holds no code to grade, or pmat has nowhere writable to
/// keep an index.
fn absent_index_verdict(baseline: &TdgBaseline, reason: &str) -> ComplianceCheck {
    const HOW: &str = "CB-200 builds its own index OUTSIDE the audited project \
                       (under $PMAT_CACHE_DIR, else the platform cache directory) so that a \
                       fresh checkout can be measured without being written to (#1008) \
                       \u{2014} this run could not. Point $PMAT_CACHE_DIR at a writable \
                       directory, or build an index in the project with `pmat query \"x\" \
                       --rebuild-index`, then re-run.";
    match baseline {
        TdgBaseline::Absent => tdg_check(
            CheckStatus::Skip,
            Severity::Info,
            format!(
                "Not measured: {reason}, and this project records no \
                 `[tdg] baseline` for CB-200 to hold. {HOW}"
            ),
        ),
        TdgBaseline::Held(b) => tdg_check(
            CheckStatus::Fail,
            Severity::Error,
            format!(
                "Not measured: {reason}, so the recorded `[tdg] baseline` of {b} \
                 was never checked. A ratchet that did not run has not held - an unmeasured \
                 gate must not report success. {HOW}"
            ),
        ),
        TdgBaseline::Unreadable(raw) => unreadable_baseline_verdict(raw),
    }
}

/// The most anyone reads before scrolling. Both listings stop here.
const TDG_MAX_LISTED: usize = 10;

/// Worst first, and deterministically so.
///
/// The listing used to be `filtered.iter().take(10)` over an unordered SQLite
/// scan with no `ORDER BY`. Two runs of the same gate over the same database
/// printing a different ten is indistinguishable, to the reader, from the tree
/// having changed — and on a flat distribution of 1,905 across 1,052 files
/// there is nothing else in the message to tell them apart.
///
/// `Grade`'s derived `Ord` runs best-to-worst (`APlus` first), so the worst
/// grade is the LARGEST; a spelling the scale does not know sorts above even
/// `F`, because a violation nobody can rank is the one most worth looking at.
fn rank_tdg_violations<'a>(filtered: &[&'a TdgViolation]) -> Vec<&'a TdgViolation> {
    fn severity_rank(v: &TdgViolation) -> usize {
        Grade::from_variant_name(&v.tdg_grade).map_or(usize::MAX, |g| g as usize)
    }
    let mut ranked: Vec<&TdgViolation> = filtered.to_vec();
    ranked.sort_by(|a, b| {
        severity_rank(b)
            .cmp(&severity_rank(a))
            .then_with(|| b.complexity.cmp(&a.complexity))
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.start_line.cmp(&b.start_line))
    });
    ranked
}

/// Up to `limit` offenders, worst first, with a truncation line that names how
/// many were not shown. `limit` of 0 still lists one: a Fail that names nothing
/// is a number the reader cannot act on.
fn tdg_offender_listing(filtered: &[&TdgViolation], limit: usize) -> String {
    let limit = limit.clamp(1, TDG_MAX_LISTED);
    let ranked = rank_tdg_violations(filtered);
    let mut details: Vec<String> = ranked
        .iter()
        .take(limit)
        .map(|v| {
            format!(
                "    {}:{} {} [{}] (complexity: {})",
                v.file_path, v.start_line, v.function_name, v.tdg_grade, v.complexity
            )
        })
        .collect();
    if ranked.len() > limit {
        details.push(format!("    ... and {} more", ranked.len() - limit));
    }
    details.join("\n")
}

/// How many distinct files the surviving violations span — the shape of the
/// debt, not just its size. 1,905 in one file is a refactor; 1,905 across 1,052
/// files is a policy, and the reader cannot tell which from a count alone.
fn tdg_violation_file_count(filtered: &[&TdgViolation]) -> usize {
    filtered
        .iter()
        .map(|v| v.file_path.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

fn tdg_check(status: CheckStatus, severity: Severity, message: String) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status,
        message,
        severity,
    }
}

/// CB-200's verdict over a completed measurement.
///
/// Four outcomes, and which one you get depends only on the baseline the
/// project recorded:
///
/// ```text
///   baseline unreadable      -> Fail   (a bound nobody can read is not a bound)
///   no baseline              -> Fail on any violation      (unchanged)
///   count <= baseline        -> Pass, carrying count AND baseline
///   count >  baseline        -> Fail, naming the excess and the offenders
/// ```
fn tdg_grade_verdict(
    filtered: &[&TdgViolation],
    queried_rows: usize,
    min_grade: &str,
    baseline: &TdgBaseline,
    staleness: &str,
) -> ComplianceCheck {
    match baseline {
        TdgBaseline::Unreadable(raw) => unreadable_baseline_verdict(raw),
        TdgBaseline::Absent => zero_tolerance_verdict(filtered, queried_rows, min_grade, staleness),
        TdgBaseline::Held(b) if filtered.len() > *b => {
            over_baseline_verdict(filtered, *b, min_grade, staleness)
        }
        TdgBaseline::Held(b) => within_baseline_verdict(filtered, *b, min_grade, staleness),
    }
}

fn unreadable_baseline_verdict(raw: &str) -> ComplianceCheck {
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "`[tdg] baseline` in .pmat-gates.toml is {raw}, which is not a count. A ratchet \
             baseline that cannot be read must not be read as a baseline nothing exceeds \
             \u{2014} write a non-negative integer, or delete the key to restore zero tolerance."
        ),
    )
}

/// A project that records no baseline gets exactly what it got before: any
/// surviving violation fails. The message is unchanged, byte for byte, so
/// adding a ratchet here cannot move another repository's output.
fn zero_tolerance_verdict(
    filtered: &[&TdgViolation],
    queried_rows: usize,
    min_grade: &str,
    staleness: &str,
) -> ComplianceCheck {
    let count = filtered.len();
    if count == 0 {
        return tdg_check(
            CheckStatus::Pass,
            Severity::Info,
            format!(
                "All non-test functions meet minimum grade {min_grade}{}{staleness}",
                if queried_rows == 0 {
                    String::new()
                } else {
                    format!(" ({queried_rows} test/excluded functions skipped)")
                }
            ),
        );
    }
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "{count} function(s) below minimum grade {min_grade}{staleness}\n{}",
            tdg_offender_listing(filtered, TDG_MAX_LISTED)
        ),
    )
}

/// At or under the recorded baseline: the gate passes, and says why it is not
/// clean while doing so.
///
/// `Warn`, not `Pass`, and the reason is mechanical rather than aesthetic.
///
/// `Pass` was the first choice — held-flat debt must not block a release, which
/// is the whole point of a ratchet — with `Severity::Warning` carrying the "not
/// clean" signal. That does not work: `retain_blocking_checks` (check.rs:270)
/// switches on `CheckStatus` ALONE and drops `Pass` unconditionally, ignoring
/// `Severity` entirely. `quality-gate.yml` runs
/// `pmat comply check --failures-only`, so the one line saying "1,904
/// definitions are below the floor" was discarded by the exact invocation CI
/// uses. A gate that hides its own debt from the only place anyone reads it is
/// how 1,904 accumulated unseen in the first place.
///
/// `Warn` does not block either — `exit_policy` (check.rs:241) only turns
/// warnings into a non-zero code under `--strict` — but it IS counted in
/// `report.summary.warn`, and the summary is deliberately tallied before the
/// list is narrowed, so the count survives `--failures-only`. The debt is
/// therefore always reachable, which was the requirement.
///
/// A clean tree at baseline 0 still reports `Pass`/`Info`: nothing is being
/// held, so there is nothing to warn about.
fn within_baseline_verdict(
    filtered: &[&TdgViolation],
    baseline: usize,
    min_grade: &str,
    staleness: &str,
) -> ComplianceCheck {
    let count = filtered.len();
    let slack = baseline - count;
    let slack_note = if slack == 0 {
        String::new()
    } else {
        format!(
            " The tree is {slack} under the recorded baseline: lower `[tdg] baseline` to \
             {count} in .pmat-gates.toml to bank it \u{2014} a baseline the tree has already \
             beaten is headroom for new debt."
        )
    };
    if count == 0 {
        return tdg_check(
            CheckStatus::Pass,
            if slack == 0 {
                Severity::Info
            } else {
                Severity::Warning
            },
            format!(
                "0 definitions below minimum grade {min_grade}, against a recorded baseline of \
                 {baseline}.{slack_note}{staleness}"
            ),
        );
    }
    tdg_check(
        CheckStatus::Warn,
        Severity::Warning,
        format!(
            "{count} definition(s) below minimum grade {min_grade} across {} file(s), at the \
             recorded baseline of {baseline} \u{2014} this is debt held flat, not a clean tree. \
             Any new definition below {min_grade} fails this gate.{slack_note}{staleness}",
            tdg_violation_file_count(filtered)
        ),
    )
}

/// Over the recorded baseline: closed, naming how many and by how much.
///
/// The caveat about the listing is not hedging. A baseline is a COUNT, so it
/// cannot identify WHICH definitions are new — only that there are more than
/// there were. Presenting the worst-graded survivors as "the ones you just
/// added" would be a claim the measurement does not support, and the reader
/// would chase the wrong functions.
fn over_baseline_verdict(
    filtered: &[&TdgViolation],
    baseline: usize,
    min_grade: &str,
    staleness: &str,
) -> ComplianceCheck {
    let count = filtered.len();
    let over = count - baseline;
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "{count} definition(s) below minimum grade {min_grade} \u{2014} {over} OVER the \
             recorded baseline of {baseline}. A ratchet holds only if new debt is refused: fix \
             {over}, or revert what added them. Raising `[tdg] baseline` is not the fix \u{2014} \
             a baseline may only go down.{staleness}\n    (the baseline is a count, not a \
             roster, so these are the worst-graded survivors, not necessarily the ones just \
             added)\n{}",
            tdg_offender_listing(filtered, over)
        ),
    )
}

/// Evaluate a single custom score definition and return the compliance check
fn evaluate_custom_score(
    project_path: &Path,
    score_def: &crate::models::comply_config::CustomScoreDefinition,
) -> ComplianceCheck {
    let check_name = format!("CB-1100: Custom Score [{}]", score_def.id);
    let output = match std::process::Command::new("sh")
        .args(["-c", &score_def.command])
        .current_dir(project_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return ComplianceCheck {
                name: check_name,
                status: CheckStatus::Skip,
                message: format!("Failed to run command: {e}"),
                severity: Severity::Info,
            }
        }
    };
    if !output.status.success() {
        return ComplianceCheck {
            name: check_name,
            status: CheckStatus::Fail,
            message: format!(
                "{}: command failed (exit {})",
                score_def.name,
                output.status.code().unwrap_or(-1)
            ),
            severity: Severity::from(score_def.severity),
        };
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual_score = match extract_score_from_output(&stdout) {
        Some(s) => s,
        None => {
            return ComplianceCheck {
                name: check_name,
                status: CheckStatus::Skip,
                message: format!(
                    "{}: could not parse score from command output",
                    score_def.name
                ),
                severity: Severity::Info,
            }
        }
    };
    match score_def.min_score {
        Some(min) if actual_score < min => ComplianceCheck {
            name: check_name,
            status: CheckStatus::Fail,
            message: format!(
                "{}: score {:.1} below minimum {:.1}",
                score_def.name, actual_score, min
            ),
            severity: Severity::from(score_def.severity),
        },
        Some(min) => ComplianceCheck {
            name: check_name,
            status: CheckStatus::Pass,
            message: format!(
                "{}: score {:.1} (min: {:.1})",
                score_def.name, actual_score, min
            ),
            severity: Severity::Info,
        },
        None => ComplianceCheck {
            name: check_name,
            status: CheckStatus::Pass,
            message: format!("{}: score {:.1}", score_def.name, actual_score),
            severity: Severity::Info,
        },
    }
}

/// CB-1100: Custom Project Scores
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_custom_scores(project_path: &Path) -> Vec<ComplianceCheck> {
    let config = match crate::models::comply_config::PmatYamlConfig::load(project_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    if config.scoring.custom_scores.is_empty() {
        return vec![];
    }
    config
        .scoring
        .custom_scores
        .iter()
        .map(|s| evaluate_custom_score(project_path, s))
        .collect()
}

fn extract_score_from_output(output: &str) -> Option<f64> {
    for line in output.lines() {
        let line = line.trim();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(score) = json.get("score").and_then(|s| s.as_f64()) {
                return Some(score);
            }
        }
    }
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SCORE:") {
            if let Ok(score) = rest.trim().parse::<f64>() {
                return Some(score);
            }
        }
    }
    None
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_tdg_grade {
    use super::*;
    use crate::models::comply_config::ComplyConfig;
    use std::path::PathBuf;

    /// The floor admits exactly the grades at least as good as it.
    ///
    /// Replaces `test_grade_ordinal` and `test_grades_below`, which pinned the
    /// defect rather than the rule: they asserted `grade_ordinal("X") == 5`
    /// (the catch-all that ranked every unknown, and every MODIFIED, grade
    /// worse than F) and `grades_below("A") == ["B","C","D","F"]` (the
    /// five-letter blindness). Both were true of the old code and both were
    /// the bug.
    #[test]
    fn passing_set_is_the_up_set_of_the_floor() {
        assert_eq!(
            passing_spellings("A+").expect("A+ is a canonical grade spelling"),
            vec!["A+"]
        );
        assert_eq!(
            passing_spellings("A").expect("A is a canonical grade spelling"),
            vec!["A+", "A"]
        );
        // The spelling that used to produce an EMPTY failing set and a Pass.
        assert_eq!(
            passing_spellings("A-").expect("A- is a canonical grade spelling"),
            vec!["A+", "A", "A-"]
        );
        assert_eq!(
            passing_spellings("B").expect("B is a canonical grade spelling"),
            vec!["A+", "A", "A-", "B+", "B"]
        );
        // Only F admits everything, and that is the one honest vacuous floor.
        assert_eq!(
            passing_spellings("F")
                .expect("F is a canonical grade spelling")
                .len(),
            GRADE_VARIANTS.len()
        );
    }

    /// Passing and failing partition the scale at every floor — the property
    /// the five-letter table did not have, proved for all eleven floors in
    /// `contracts/lean/Theorems/Tdg/Grade.lean::Grade_Partition`.
    #[test]
    fn passing_and_failing_partition_the_scale() {
        for floor in GRADE_VARIANTS {
            let passing = passing_spellings(floor).expect("canonical spelling parses");
            let failing: Vec<_> = GRADE_VARIANTS
                .iter()
                .filter(|g| !passing.contains(g))
                .collect();
            assert_eq!(
                passing.len() + failing.len(),
                GRADE_VARIANTS.len(),
                "floor {floor} does not partition the scale"
            );
        }
    }

    /// An unreadable threshold yields `None`, never an empty set. An empty set
    /// is what the caller reads as "nothing violates".
    #[test]
    fn an_unparseable_floor_is_none_not_empty() {
        for bad in ["X", "", " ", "A--", "Q", "E"] {
            assert!(
                passing_spellings(bad).is_none(),
                "{bad:?} must not parse as a grade floor"
            );
        }
        // Counter-test: every canonical spelling still parses, so the guard did
        // not become a spelling police.
        for good in GRADE_VARIANTS {
            assert!(passing_spellings(good).is_some(), "{good} must parse");
        }
    }

    #[test]
    fn test_missing_db_returns_skip() {
        let tmp = PathBuf::from("/tmp/pmat-test-tdg-missing-db");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(&tmp, &config);
        assert_eq!(result.status, CheckStatus::Skip);
        assert!(
            result.message.contains("Not measured"),
            "{}",
            result.message
        );
    }

    /// #939: CB-200 used to BUILD and SAVE the agent-context index whenever
    /// `.pmat/context.db` was missing or stale — so asking for a compliance
    /// verdict wrote `.pmat/context.db` and `.pmat/context.idx` into the
    /// project being audited (and spent minutes doing it on a large repo).
    ///
    /// It still never writes there, and that is what this test is for. What
    /// #1008 changed is where the index comes from when the project has none:
    /// pmat's own copy, outside the tree. The `Skip` this used to assert WAS
    /// the defect — a gate that declines to measure a fresh checkout cannot
    /// run in the one place a verdict decides a merge.
    #[test]
    fn cb200_never_builds_an_index_inside_the_audited_project() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::create_dir_all(tmp.path().join("src")).expect("create src");
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn f() {}\n").expect("write source");

        let result = check_tdg_grade_gate_with_index(
            tmp.path(),
            &ComplyConfig::default(),
            &cache.path().join("context.idx"),
        );

        assert!(
            !tmp.path().join(".pmat").exists(),
            "comply check must not write .pmat/ into the project it audits"
        );
        assert_ne!(
            result.status,
            CheckStatus::Skip,
            "#1008: a tree pmat could read must be measured, not skipped: {}",
            result.message
        );
        assert!(
            cache.path().join("context.db").exists(),
            "the index it read must be the one outside the audited tree"
        );
    }

    /// A stale index is still a real measurement — it is reported, with the
    /// staleness named, rather than silently repaired by a write.
    #[test]
    fn cb200_reports_staleness_instead_of_rebuilding() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch(
            "CREATE TABLE functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, \
             function_name TEXT NOT NULL, tdg_grade TEXT NOT NULL DEFAULT 'A', \
             complexity INTEGER NOT NULL DEFAULT 1, start_line INTEGER NOT NULL DEFAULT 0)",
        )
        .expect("schema");
        drop(conn);

        // A source file newer than the db makes the index stale.
        //
        // The mtime is SET, not raced for. Writing the source after the db and
        // trusting wall-clock ordering failed on CI with "fixture must be
        // stale": both files landed in the same filesystem timestamp tick, so
        // `is_index_stale` correctly saw nothing newer and the fixture never
        // reached the behaviour under test. Filesystem timestamp granularity is
        // coarser than the microseconds between two writes, and varies by
        // filesystem — which is exactly the kind of thing that passes on the
        // author's machine and fails in CI.
        std::fs::create_dir_all(tmp.path().join("src")).expect("create src");
        let src = tmp.path().join("src/lib.rs");
        std::fs::write(&src, "pub fn f() {}\n").expect("write source");
        let db_mtime = std::fs::metadata(&db_path)
            .and_then(|m| m.modified())
            .expect("db mtime");
        std::fs::File::options()
            .write(true)
            .open(&src)
            .and_then(|f| f.set_modified(db_mtime + std::time::Duration::from_secs(60)))
            .expect("set source mtime a clear minute past the db");
        assert!(
            is_index_stale(tmp.path(), &db_path),
            "fixture must be stale"
        );

        let before = std::fs::metadata(&db_path).expect("meta").len();
        let result = check_tdg_grade_gate(tmp.path(), &ComplyConfig::default());
        let after = std::fs::metadata(&db_path).expect("meta").len();

        assert_eq!(before, after, "the audit rewrote the index it was reading");
        assert!(
            !pmat_dir.join("context.idx").exists(),
            "comply check must not build .pmat/context.idx"
        );
        assert!(
            result.message.contains("index is stale"),
            "staleness must be reported, not silently repaired: {}",
            result.message
        );
    }

    #[test]
    fn test_grade_f_no_violations() {
        let tmp = PathBuf::from("/tmp/pmat-test-tdg-grade-f");
        let mut config = ComplyConfig::default();
        config.thresholds.min_tdg_grade = "F".to_string();
        let result = check_tdg_grade_gate(&tmp, &config);
        assert!(result.status == CheckStatus::Skip || result.status == CheckStatus::Pass);
    }

    #[test]
    fn test_in_memory_db_with_violations() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/core.rs', 'good_fn', 'A', 5, 10)", []).expect("insert A");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/legacy.rs', 'bad_fn', 'D', 42, 20)", []).expect("insert D");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/awful.rs', 'terrible_fn', 'F', 60, 30)", []).expect("insert F");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/tests/helpers.rs', 'test_helper', 'D', 35, 40)", []).expect("insert test D");
        drop(conn);
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result
            .message
            .contains("2 function(s) below minimum grade A"));
        assert!(result.message.contains("src/legacy.rs:20 bad_fn [D]"));
        assert!(result.message.contains("src/awful.rs:30 terrible_fn [F]"));
        assert!(!result.message.contains("test_helper"));
    }

    #[test]
    fn test_tdg_exclude_paths() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('vendor/lib.rs', 'vendor_fn', 'D', 40, 10)", []).expect("insert vendor D");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/main.rs', 'main_fn', 'D', 30, 5)", []).expect("insert src D");
        drop(conn);
        let mut config = ComplyConfig::default();
        config.thresholds.tdg_exclude_paths = vec!["vendor/*".to_string()];
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("1 function(s)"));
        assert!(result.message.contains("main_fn"));
        assert!(!result.message.contains("vendor_fn"));
    }

    #[test]
    fn test_all_pass_with_good_grades() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/lib.rs', 'good_fn', 'A', 3, 1)", []).expect("insert A");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/util.rs', 'ok_fn', 'B', 8, 10)", []).expect("insert B");
        drop(conn);
        let mut config = ComplyConfig::default();
        config.thresholds.min_tdg_grade = "B".to_string();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("meet minimum grade B"));
    }

    #[test]
    fn test_pmat_gates_toml_min_grade_override() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/lib.rs', 'good_fn', 'A', 3, 1)", []).expect("insert A");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/util.rs', 'ok_fn', 'B', 8, 10)", []).expect("insert B");
        drop(conn);
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nmin_grade = \"B\"\n",
        )
        .expect("write gates toml");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_pmat_gates_toml_exclude_override() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/core_generated.rs', 'gen_fn', 'D', 40, 10)", []).expect("insert generated D");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/real.rs', 'real_fn', 'D', 30, 5)", []).expect("insert real D");
        drop(conn);
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nexclude = [\"**/*_generated.rs\"]\n",
        )
        .expect("write gates toml");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("1 function(s)"));
        assert!(result.message.contains("real_fn"));
        assert!(!result.message.contains("gen_fn"));
    }

    #[test]
    fn test_is_index_stale_no_db() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        assert!(is_index_stale(
            tmp.path(),
            &tmp.path().join("nonexistent.db")
        ));
    }

    #[test]
    fn test_is_index_stale_fresh_db() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        std::fs::write(src_dir.join("lib.rs"), "fn main() {}").expect("write src");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let db_path = tmp.path().join("context.db");
        std::fs::write(&db_path, "").expect("write db");
        assert!(!is_index_stale(tmp.path(), &db_path));
    }

    #[test]
    fn test_is_index_stale_outdated_db() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let db_path = tmp.path().join("context.db");
        std::fs::write(&db_path, "").expect("write db");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        std::fs::write(src_dir.join("lib.rs"), "fn main() {}").expect("write src");
        assert!(is_index_stale(tmp.path(), &db_path));
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("foo.rs")));
        assert!(is_source_file(Path::new("foo.py")));
        assert!(is_source_file(Path::new("foo.ts")));
        assert!(!is_source_file(Path::new("foo.txt")));
        assert!(!is_source_file(Path::new("foo.toml")));
        assert!(!is_source_file(Path::new("Makefile")));
    }

    // ── CB-200 as a RATCHET ──────────────────────────────────────────────
    //
    // The gate was BLIND until 2026-08-20: a five-letter reader against an
    // eleven-letter writer, so it saw 247 violations and could not see 1,719.
    // Every historical "CB-200 passed" came from that version. Once it could
    // see, it measured 1,905 below-A definitions across 1,052 files, max 12 per
    // file — a flat distribution with no hotspot and no bounded refactor.
    //
    // The two ways to make that green are both threshold-lowering in disguise:
    // drop `min_tdg_grade`, or add an exclude glob. One of them (187f506885) is
    // how a past "pass" was manufactured. Neither is done here. The gate holds
    // the count flat instead, refuses any increase closed, and prints the
    // absolute number on every outcome so that passing can never be mistaken
    // for clean.

    /// A project with `.pmat/context.db` holding exactly these rows.
    pub(super) fn tdg_fixture(rows: &[(&str, &str, &str, u32, usize)]) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let conn = rusqlite::Connection::open(pmat_dir.join("context.db")).expect("open db");
        conn.execute_batch(
            "CREATE TABLE functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, \
             function_name TEXT NOT NULL, tdg_grade TEXT NOT NULL DEFAULT 'A', \
             complexity INTEGER NOT NULL DEFAULT 1, start_line INTEGER NOT NULL DEFAULT 0)",
        )
        .expect("schema");
        for (file, name, grade, complexity, line) in rows {
            conn.execute(
                "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, \
                 start_line) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![file, name, grade, complexity, line],
            )
            .expect("insert row");
        }
        tmp
    }

    pub(super) fn write_gates(tmp: &tempfile::TempDir, body: &str) {
        std::fs::write(tmp.path().join(".pmat-gates.toml"), body).expect("write gates toml");
    }

    pub(super) fn judge(tmp: &tempfile::TempDir) -> ComplianceCheck {
        check_tdg_grade_gate(tmp.path(), &ComplyConfig::default())
    }

    /// The key is OPTIONAL, and its absence is not a softer gate.
    ///
    /// This is the counter-test that bounds the whole change: pmat runs against
    /// other repositories, and none of them has agreed to anything. A project
    /// that records no baseline must get zero tolerance, byte for byte, whether
    /// it has a `.pmat-gates.toml` with no `baseline` key or no file at all.
    #[test]
    fn without_a_baseline_key_any_violation_still_fails() {
        let no_file = tdg_fixture(&[("src/a.rs", "bad", "D", 30, 5)]);
        let verdict = judge(&no_file);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("1 function(s) below minimum grade A"),
            "the no-baseline message must be unchanged: {}",
            verdict.message
        );

        let other_keys = tdg_fixture(&[("src/a.rs", "bad", "D", 30, 5)]);
        write_gates(&other_keys, "[tdg]\nexclude = [\"nothing/**\"]\n");
        let verdict = judge(&other_keys);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("1 function(s) below minimum grade A"),
            "a [tdg] table without a baseline is still zero tolerance: {}",
            verdict.message
        );
    }

    /// At the baseline the gate PASSES — and must not read as clean.
    ///
    /// "1905 below A" printed at `Info` next to a genuinely empty tree is how
    /// 1,905 accumulated unseen. The count, the baseline and the word "debt"
    /// are all load-bearing, and so is `Severity::Warning` on a `Pass`.
    /// Held debt must survive `--failures-only`, which is the ONLY invocation
    /// CI runs.
    ///
    /// This is the counter-test for the status choice, and it exists because the
    /// obvious simplification — "it passed, so return `Pass`" — is wrong in a way
    /// nothing else here would catch. `retain_blocking_checks` (check.rs) matches
    /// on `CheckStatus` ALONE and drops `Pass` unconditionally; `Severity` is not
    /// consulted. `quality-gate.yml` runs `pmat comply check --failures-only`, so
    /// a `Pass` verdict deletes the one line reporting that 1,904 definitions sit
    /// below the floor, from the only report anyone reads.
    ///
    /// `Warn` does not block — `exit_policy` only escalates warnings under
    /// `--strict` — but it is counted into `summary.warn`, and the summary is
    /// tallied BEFORE the list is narrowed. That is what keeps the number
    /// reachable.
    ///
    /// RED: change `within_baseline_verdict`'s non-empty branch back to
    /// `CheckStatus::Pass` and this fails with `left: Pass, right: Warn`.
    #[test]
    fn held_debt_is_warn_so_failures_only_cannot_hide_it() {
        let tmp = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
        write_gates(&tmp, "[tdg]\nbaseline = 5\n");
        let verdict = judge(&tmp);

        assert_eq!(
            verdict.status,
            CheckStatus::Warn,
            "held debt reported as Pass is dropped by `--failures-only`; the \
             count must stay reachable. Message was: {}",
            verdict.message
        );

        // ...and the counter-test: a genuinely clean tree is still a Pass, so
        // "always Warn" cannot satisfy the assertion above.
        let clean = tdg_fixture(&[]);
        write_gates(&clean, "[tdg]\nbaseline = 5\n");
        let clean_verdict = judge(&clean);
        assert_eq!(
            clean_verdict.status,
            CheckStatus::Pass,
            "nothing is being held, so there is nothing to warn about: {}",
            clean_verdict.message
        );
    }

    #[test]
    fn at_the_baseline_it_passes_carrying_the_count_and_the_baseline() {
        let tmp = tdg_fixture(&[
            ("src/a.rs", "one", "D", 30, 5),
            ("src/b.rs", "two", "C-", 20, 9),
        ]);
        write_gates(&tmp, "[tdg]\nbaseline = 2\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert_eq!(
            verdict.severity,
            Severity::Warning,
            "held debt at Info reads as clean: {}",
            verdict.message
        );
        assert!(
            verdict
                .message
                .contains("2 definition(s) below minimum grade A"),
            "the absolute count must lead: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("recorded baseline of 2"),
            "the baseline must be named: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("not a clean tree"),
            "passing at baseline must not read as passing clean: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("2 file(s)"),
            "the shape of the debt is part of the report: {}",
            verdict.message
        );
    }

    /// One over is a failure, named as one over.
    #[test]
    fn one_definition_over_the_baseline_fails_and_says_by_how_much() {
        let tmp = tdg_fixture(&[
            ("src/a.rs", "one", "D", 30, 5),
            ("src/b.rs", "two", "C-", 20, 9),
            ("src/c.rs", "three", "F", 44, 2),
        ]);
        write_gates(&tmp, "[tdg]\nbaseline = 2\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert_eq!(verdict.severity, Severity::Error);
        assert!(
            verdict
                .message
                .contains("3 definition(s) below minimum grade A"),
            "the absolute count must lead even on a failure: {}",
            verdict.message
        );
        assert!(
            verdict
                .message
                .contains("1 OVER the recorded baseline of 2"),
            "the excess must be named: {}",
            verdict.message
        );
        // Exactly `over` offenders are listed, worst first, and the rest are
        // counted rather than dropped.
        assert!(
            verdict.message.contains("src/c.rs:2 three [F]"),
            "the worst survivor must be named: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("... and 2 more"),
            "the unlisted remainder must be counted: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("may only go down"),
            "the fix must not read as 'raise the baseline': {}",
            verdict.message
        );
    }

    /// Under the baseline is a pass that asks to be banked.
    ///
    /// A baseline the tree has already beaten is headroom for new debt, which
    /// is the failure mode `.pmat-ratchet.toml`'s `--lower` job exists to
    /// prevent. CB-200 has no such job, so it asks in the message instead.
    #[test]
    fn under_the_baseline_passes_and_asks_for_the_baseline_to_be_lowered() {
        let tmp = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
        write_gates(&tmp, "[tdg]\nbaseline = 5\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert!(
            verdict.message.contains("4 under the recorded baseline"),
            "slack must be reported: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("lower `[tdg] baseline` to 1"),
            "the message must name the number to bank: {}",
            verdict.message
        );
    }

    /// A clean tree under a stale baseline reports the slack, not silence.
    #[test]
    fn a_clean_tree_still_reports_a_baseline_it_has_outgrown() {
        let tmp = tdg_fixture(&[("src/a.rs", "fine", "A", 3, 1)]);
        write_gates(&tmp, "[tdg]\nbaseline = 5\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Pass, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("0 definitions below minimum grade A"),
            "{}",
            verdict.message
        );
        assert!(
            verdict.message.contains("lower `[tdg] baseline` to 0"),
            "a baseline of 5 over an empty tree is pure headroom: {}",
            verdict.message
        );
    }

    /// A baseline nobody can read is not a baseline nothing exceeds.
    ///
    /// The same rule `passing_spellings` already enforces for `min_grade`: the
    /// unreadable value is REPORTED, never quietly rounded to "no baseline",
    /// because "your key does nothing" and "you chose zero tolerance" are
    /// opposite claims that look identical from outside.
    #[test]
    fn an_unreadable_baseline_fails_closed_and_names_itself() {
        for bad in ["\"many\"", "-1", "1.5", "true", "[1905]"] {
            let tmp = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
            write_gates(&tmp, &format!("[tdg]\nbaseline = {bad}\n"));
            let verdict = judge(&tmp);
            assert_eq!(
                verdict.status,
                CheckStatus::Fail,
                "baseline = {bad} must fail: {}",
                verdict.message
            );
            assert!(
                verdict.message.contains("is not a count"),
                "baseline = {bad} must name itself: {}",
                verdict.message
            );
        }
        // Counter-test: the guard did not become a "no baseline may be small"
        // rule. Zero is a real baseline — it is zero tolerance, said out loud.
        let clean = tdg_fixture(&[("src/a.rs", "fine", "A", 3, 1)]);
        write_gates(&clean, "[tdg]\nbaseline = 0\n");
        assert_eq!(judge(&clean).status, CheckStatus::Pass);
        let dirty = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
        write_gates(&dirty, "[tdg]\nbaseline = 0\n");
        let verdict = judge(&dirty);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("1 OVER the recorded baseline of 0"),
            "{}",
            verdict.message
        );
    }

    /// The over-correction this must NOT become: a baseline that hides debt.
    ///
    /// A generous baseline buys silence in exactly one place — the pass/fail
    /// verdict. It must not raise the floor, must not exclude a path, and must
    /// not remove a single definition from the count. Nothing here may read as
    /// the sentence a genuinely clean tree gets.
    #[test]
    fn a_baseline_holds_debt_flat_without_hiding_any_of_it() {
        let tmp = tdg_fixture(&[
            ("src/a.rs", "one", "D", 30, 5),
            ("src/b.rs", "two", "C-", 20, 9),
            ("src/c.rs", "three", "B+", 12, 3),
        ]);
        write_gates(&tmp, "[tdg]\nbaseline = 100\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert!(
            verdict.message.contains("3 definition(s)"),
            "every unit of debt stays counted: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("minimum grade A"),
            "the floor is still A — a baseline is not a lowered threshold: {}",
            verdict.message
        );
        // B+ is below A and is still counted: the baseline did not quietly
        // narrow the alphabet the way the five-letter reader did.
        assert!(
            !verdict
                .message
                .contains("All non-test functions meet minimum grade"),
            "held debt must never borrow the clean tree's sentence: {}",
            verdict.message
        );
    }

    /// The listing is deterministic, worst first.
    ///
    /// It used to be `take(10)` off a `SELECT` with no `ORDER BY`. On a flat
    /// 1,905-across-1,052-files distribution there is nothing else in the
    /// message to distinguish "the tree changed" from "SQLite scanned in a
    /// different order".
    #[test]
    fn the_offender_listing_is_worst_first_and_stable() {
        let tmp = tdg_fixture(&[
            ("src/mild.rs", "mild", "B+", 11, 1),
            ("src/worst.rs", "worst", "F", 9, 1),
            ("src/bad_simple.rs", "bad_simple", "D", 4, 1),
            ("src/bad_complex.rs", "bad_complex", "D", 90, 1),
        ]);
        let verdict = judge(&tmp);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);

        for name in ["worst", "bad_complex", "bad_simple", "mild"] {
            assert!(
                verdict.message.contains(name),
                "{name} must be listed: {}",
                verdict.message
            );
        }
        let order: Vec<usize> = ["worst", "bad_complex", "bad_simple", "mild"]
            .iter()
            .map(|name| verdict.message.find(name).unwrap_or(usize::MAX))
            .collect();
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(
            order, sorted,
            "worst grade first, then highest complexity: {}",
            verdict.message
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_stale_and_absent_index {
    use super::tests_tdg_grade::{judge, tdg_fixture, write_gates};
    use super::*;
    use crate::models::comply_config::ComplyConfig;

    /// Make `tmp`'s index stale by planting a source file a clear minute newer
    /// than `.pmat/context.db`.
    ///
    /// The mtime is SET, not raced for. Writing the source after the db and
    /// trusting wall-clock ordering fails on filesystems whose timestamp
    /// granularity is coarser than the microseconds between two writes — both
    /// files land in the same tick, `is_index_stale` correctly sees nothing
    /// newer, and the fixture silently never reaches the behaviour under test.
    fn make_stale(tmp: &tempfile::TempDir) {
        let db_path = tmp.path().join(".pmat").join("context.db");
        std::fs::create_dir_all(tmp.path().join("src")).expect("create src");
        let src = tmp.path().join("src/lib.rs");
        std::fs::write(&src, "pub fn f() {}\n").expect("write source");
        let db_mtime = std::fs::metadata(&db_path)
            .and_then(|m| m.modified())
            .expect("db mtime");
        std::fs::File::options()
            .write(true)
            .open(&src)
            .and_then(|f| f.set_modified(db_mtime + std::time::Duration::from_secs(60)))
            .expect("set source mtime a clear minute past the db");
        assert!(
            is_index_stale(tmp.path(), &db_path),
            "fixture must be stale before the behaviour under test can be reached"
        );
    }

    /// Plant a source file OLDER than the db, so the fixture is provably fresh
    /// while still containing sources — the counter-fixture to `make_stale`.
    fn make_fresh(tmp: &tempfile::TempDir) {
        let db_path = tmp.path().join(".pmat").join("context.db");
        std::fs::create_dir_all(tmp.path().join("src")).expect("create src");
        let src = tmp.path().join("src/lib.rs");
        std::fs::write(&src, "pub fn f() {}\n").expect("write source");
        let db_mtime = std::fs::metadata(&db_path)
            .and_then(|m| m.modified())
            .expect("db mtime");
        std::fs::File::options()
            .write(true)
            .open(&src)
            .and_then(|f| f.set_modified(db_mtime - std::time::Duration::from_secs(60)))
            .expect("set source mtime a clear minute before the db");
        assert!(
            !is_index_stale(tmp.path(), &db_path),
            "counter-fixture must be fresh"
        );
    }

    /// #1045. A clean count taken from a stale index is not a clean tree: the
    /// definitions added since the index was built are not in it AT ALL, so
    /// "nothing below the floor" is a statement about yesterday.
    ///
    /// RED, with `demote_pass_when_stale` reduced to `check` (the identity it
    /// replaced):
    /// ```text
    /// a clean count over a STALE index must not be a Pass, got Pass:
    ///   All non-test functions meet minimum grade A (index is stale: ...)
    /// ```
    #[test]
    fn a_stale_index_may_not_report_a_pass() {
        let tmp = tdg_fixture(&[("src/lib.rs", "fine", "A", 3, 1)]);
        make_stale(&tmp);
        let verdict = judge(&tmp);
        assert_ne!(
            verdict.status,
            CheckStatus::Pass,
            "a clean count over a STALE index must not be a Pass, got Pass:\n  {}",
            verdict.message
        );
        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("NOT A VERDICT ON THE CURRENT TREE"),
            "the reader must be told the verdict is not about their tree: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("index is stale"),
            "the reason must survive the demotion: {}",
            verdict.message
        );
    }

    /// The counter-test that bounds the demotion. Staleness is the trigger, not
    /// the presence of sources or the mere act of reading an index: an index
    /// NEWER than every source still reports a clean tree as clean.
    ///
    /// Without this, `demote_pass_when_stale` returning `Warn` unconditionally
    /// would pass every other test in this module.
    #[test]
    fn a_fresh_index_still_reports_a_pass() {
        let tmp = tdg_fixture(&[("src/lib.rs", "fine", "A", 3, 1)]);
        make_fresh(&tmp);
        let verdict = judge(&tmp);
        assert_eq!(
            verdict.status,
            CheckStatus::Pass,
            "a fresh index over a clean tree must still Pass: {}",
            verdict.message
        );
        assert!(
            !verdict.message.contains("index is stale"),
            "a fresh index must not be described as stale: {}",
            verdict.message
        );
    }

    /// Staleness makes a verdict LESS conclusive, never more lenient. A count
    /// over the floor is still a Fail — those definitions exist somewhere in
    /// history and the fix is to look, not to shrug.
    #[test]
    fn a_stale_index_does_not_soften_a_failure() {
        let tmp = tdg_fixture(&[("src/lib.rs", "bad", "D", 30, 5)]);
        make_stale(&tmp);
        let verdict = judge(&tmp);
        assert_eq!(
            verdict.status,
            CheckStatus::Fail,
            "staleness must not downgrade a Fail: {}",
            verdict.message
        );
    }

    /// #1045's second half: the advice printed beside a stale index must name a
    /// command that actually refreshes one.
    ///
    /// `pmat query "x"` does not. `load_or_build_index`
    /// (`cli/handlers/query_handler/indexing.rs:251`) rebuilds only when BOTH
    /// `.pmat/context.idx` and `.pmat/context.db` are absent; with an index
    /// present it updates in memory and `maybe_save_incremental` persists that
    /// only past 50 changed files or 5% of the index. Following the old advice
    /// after editing a handful of files left the db byte-identical and the next
    /// run printed the same "index is stale" with the same useless remedy.
    ///
    /// RED, restoring the old text: `the stale-index advice must name
    /// --rebuild-index`.
    #[test]
    fn the_stale_advice_names_a_command_that_rebuilds() {
        let tmp = tdg_fixture(&[("src/lib.rs", "fine", "A", 3, 1)]);
        make_stale(&tmp);
        let verdict = judge(&tmp);
        assert!(
            verdict.message.contains("--rebuild-index"),
            "the stale-index advice must name --rebuild-index, because a plain \
             `pmat query` does not rewrite an index that already exists: {}",
            verdict.message
        );
    }

    /// #1008. `.pmat/` is gitignored and no CI leg builds an index, so an
    /// absent index answered `Skip` — and `is_compliant` counts `Fail` only.
    /// A project that RECORDED a ratchet baseline therefore had a gate that
    /// could only fail on a machine which happened to have an index lying
    /// around: unenforceable exactly where it decides a merge.
    ///
    /// RED, with `absent_index_verdict` returning the old unconditional `Skip`:
    /// ```text
    /// a recorded baseline that was never checked must not report success,
    /// got Skip
    /// ```
    #[test]
    fn an_absent_index_fails_a_project_that_recorded_a_baseline() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nbaseline = 1688\n",
        )
        .expect("write gates toml");
        let verdict = check_tdg_grade_gate(tmp.path(), &ComplyConfig::default());
        assert_eq!(
            verdict.status,
            CheckStatus::Fail,
            "a recorded baseline that was never checked must not report success, got {:?}: {}",
            verdict.status,
            verdict.message
        );
        assert!(
            verdict.message.contains("1688"),
            "the unchecked baseline must be named: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("Not measured"),
            "the failure is 'unmeasured', not 'violated': {}",
            verdict.message
        );
        assert!(
            !tmp.path().join(".pmat").exists(),
            "#939: an audit must not build an index inside the tree it audits"
        );
    }

    /// The counter-test that bounds #1008's fix, and it is the one that matters
    /// most: pmat runs `comply check` against repositories that never opted
    /// into this ratchet. A project with no `[tdg] baseline` — no
    /// `.pmat-gates.toml` at all, or one without the key — must still be told
    /// "not measured" and must NOT be failed for it.
    ///
    /// Without this, "make the absent index fail" is a one-line change that
    /// reddens every fresh clone of every project pmat has ever been pointed at.
    #[test]
    fn an_absent_index_still_skips_a_project_with_no_baseline() {
        let nothing = tempfile::tempdir().expect("create tempdir");
        let verdict = check_tdg_grade_gate(nothing.path(), &ComplyConfig::default());
        assert_eq!(
            verdict.status,
            CheckStatus::Skip,
            "a project holding no baseline has nothing to fail: {}",
            verdict.message
        );

        let no_key = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            no_key.path().join(".pmat-gates.toml"),
            "[tdg]\nexclude = [\"nothing/**\"]\n",
        )
        .expect("write gates toml");
        let verdict = check_tdg_grade_gate(no_key.path(), &ComplyConfig::default());
        assert_eq!(
            verdict.status,
            CheckStatus::Skip,
            "a [tdg] table without a baseline records no ratchet: {}",
            verdict.message
        );
    }

    /// An unreadable baseline is a Fail with or without an index. It was
    /// already a Fail once the index was read; routing the absent-index case
    /// through the same verdict keeps the two answers identical rather than
    /// letting "no index" launder a broken config into a Skip.
    #[test]
    fn an_unreadable_baseline_fails_even_with_no_index() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nbaseline = \"lots\"\n",
        )
        .expect("write gates toml");
        let verdict = check_tdg_grade_gate(tmp.path(), &ComplyConfig::default());
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict.message.contains("not a count"),
            "{}",
            verdict.message
        );
    }

    /// The demotion is a funnel, not a special case bolted onto one verdict:
    /// the ratchet's own clean-at-baseline `Pass` goes through it too.
    #[test]
    fn the_ratchet_clean_pass_is_demoted_when_stale() {
        let tmp = tdg_fixture(&[("src/lib.rs", "fine", "A", 3, 1)]);
        write_gates(&tmp, "[tdg]\nbaseline = 0\n");
        make_stale(&tmp);
        let verdict = judge(&tmp);
        assert_eq!(
            verdict.status,
            CheckStatus::Warn,
            "0-against-baseline-0 over a stale index is not a Pass: {}",
            verdict.message
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_index_outside_the_audited_tree {
    use super::*;
    use std::process::Command;

    /// A tree with one definition no floor of A can admit, and one that passes,
    /// so a verdict of "everything is fine" and a verdict of "nothing was read"
    /// are distinguishable from each other.
    ///
    /// The grade is driven by complexity alone. It used to carry two
    /// self-admitted-debt comments as well, until `.pmat-ratchet.toml`'s
    /// `satd_markers_src_comments` went red at HEAD: that metric greps comment
    /// lines across `src/*.rs`, and a raw string literal inside a test is still
    /// a line in a file. A fixture is not a licence to move a baseline — nor is
    /// a comment about one, which is why this paragraph does not spell the four
    /// words out either.
    const AWFUL: &str = r#"
pub fn fine(a: u32) -> u32 { a + 1 }

pub fn awful(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let mut t = 0;
    for i in 0..a {
        if i % 2 == 0 {
            for j in 0..b {
                if j % 3 == 0 {
                    while t < c {
                        if t % 5 == 0 { t += 1; } else if t % 7 == 0 { t += 2; } else { t += 3; }
                        match t % 4 {
                            0 => t += 1,
                            1 => t += 2,
                            2 => { if d > 0 { t += 3 } else { t -= 1 } }
                            _ => t += 4,
                        }
                    }
                } else if j % 5 == 0 { t -= 1; } else if j % 7 == 0 { t += j; } else { t += 2; }
            }
        } else if i % 3 == 0 {
            t += 2;
        } else {
            for k in 0..d { if k > 3 { t += k } else if k > 1 { t -= k } else { t += 1 } }
        }
    }
    t
}
"#;

    pub(super) fn committed_cargo_project() -> tempfile::TempDir {
        let d = git_project_with_code();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write manifest");
        git(d.path(), &["add", "-A"]);
        git(
            d.path(),
            &[
                "-c",
                "user.email=t@t",
                "-c",
                "user.name=t",
                "commit",
                "--no-verify",
                "-qm",
                "manifest",
            ],
        );
        d
    }

    pub(super) fn porcelain_of(p: &Path) -> String {
        porcelain(p)
    }

    pub(super) fn build_index_in(p: &Path) {
        build_in_project_index(p)
    }

    fn project_with_code() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("create tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("create src");
        std::fs::write(dir.path().join("src/lib.rs"), AWFUL).expect("write source");
        dir
    }

    /// The index `pmat query` would leave in the project — built the same way,
    /// so side B of the A/B is the real thing and not a hand-written fixture.
    fn build_in_project_index(project: &Path) {
        let index_path = project.join(".pmat").join("context.idx");
        std::fs::create_dir_all(project.join(".pmat")).expect("create .pmat");
        crate::services::agent_context::AgentContextIndex::build(project)
            .expect("build index")
            .save(&index_path)
            .expect("save index");
    }

    /// The verdict without the provenance note, which is the only part of the
    /// message that is *supposed* to differ between the two sides.
    fn verdict_core(check: &ComplianceCheck) -> String {
        check
            .message
            .split(" [measured against")
            .next()
            .unwrap_or_default()
            .to_string()
    }

    /// `$PMAT_CACHE_DIR`, restored on drop so a failing assertion cannot leak
    /// it into the rest of the suite.
    pub(super) struct CacheDirGuard(Option<std::ffi::OsString>);

    /// `$PMAT_CACHE_DIR` for the duration of one test, for the sibling module
    /// that runs the whole compliance report rather than one check.
    pub(super) fn cache_dir_guard(dir: &Path) -> CacheDirGuard {
        CacheDirGuard::pointing_at(dir)
    }

    impl CacheDirGuard {
        fn pointing_at(dir: &Path) -> Self {
            let previous = std::env::var_os(crate::utils::pmat_cache_dir::CACHE_DIR_ENV);
            std::env::set_var(crate::utils::pmat_cache_dir::CACHE_DIR_ENV, dir);
            Self(previous)
        }
    }

    impl Drop for CacheDirGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(v) => std::env::set_var(crate::utils::pmat_cache_dir::CACHE_DIR_ENV, v),
                None => std::env::remove_var(crate::utils::pmat_cache_dir::CACHE_DIR_ENV),
            }
        }
    }

    fn git(dir: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect("git must be runnable")
    }

    fn porcelain(dir: &Path) -> String {
        String::from_utf8_lossy(&git(dir, &["status", "--porcelain"]).stdout).into_owned()
    }

    /// `--template=` keeps a developer's global hook template out of the
    /// fixture; `--no-verify` on the commit keeps a global `core.hooksPath`
    /// from running this repository's own gates inside a two-file tempdir.
    fn git_project_with_code() -> tempfile::TempDir {
        let dir = project_with_code();
        assert!(
            git(dir.path(), &["init", "-q", "--template=", "."])
                .status
                .success(),
            "git init failed"
        );
        git(dir.path(), &["add", "-A"]);
        assert!(
            git(
                dir.path(),
                &[
                    "-c",
                    "user.email=t@t",
                    "-c",
                    "user.name=t",
                    "commit",
                    "--no-verify",
                    "-qm",
                    "fixture",
                ],
            )
            .status
            .success(),
            "commit failed"
        );
        dir
    }

    /// #1008's own A/B, at the check that showed it: one tree, one commit, the
    /// index the only variable. The two sides must return the SAME verdict.
    ///
    /// Reported on `paiml/rmedia` at `ffe4f68` as 2 failing checks without an
    /// index against 3 with one — a gate that structurally could not fail on a
    /// fresh CI checkout, which is the only checkout whose verdict decides a
    /// merge.
    ///
    /// RED, against the code this replaces:
    /// ```text
    /// thread 'a_tree_gets_the_same_verdict_with_or_without_an_index_of_its_own'
    ///   panicked at check_tdg_grade.rs:
    /// a project with no index of its own was not measured at all (Skip) - #1008:
    /// a gate that can only run where an index happens to exist cannot run in CI
    /// ```
    #[test]
    #[serial_test::serial]
    fn a_tree_gets_the_same_verdict_with_or_without_an_index_of_its_own() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let _env = CacheDirGuard::pointing_at(cache.path());
        let project = project_with_code();
        let config = ComplyConfig::default();

        // A: no `.pmat/` anywhere in the tree, exactly as a fresh checkout.
        let without = check_tdg_grade_gate(project.path(), &config);
        assert!(
            !project.path().join(".pmat").exists(),
            "#939: the audited tree must not be written to, index or no index"
        );
        assert_ne!(
            without.status,
            CheckStatus::Skip,
            "a project with no index of its own was not measured at all (Skip) - #1008: \
             a gate that can only run where an index happens to exist cannot run in CI. \
             Message was: {}",
            without.message
        );

        // B: the same tree, with the index `pmat query` would have built.
        build_in_project_index(project.path());
        let with = check_tdg_grade_gate(project.path(), &config);

        assert_eq!(
            without.status, with.status,
            "the same tree got two different verdicts:\n  without: {}\n  with:    {}",
            without.message, with.message
        );
        assert_eq!(
            verdict_core(&without),
            verdict_core(&with),
            "the same tree was measured differently depending on where its index lived"
        );
        assert!(
            with.message.split(" [measured against").count() == 1,
            "a project's own index is read in place and says nothing about a cache: {}",
            with.message
        );
        assert!(
            without.message.contains("[measured against"),
            "a verdict from outside the tree must name the file it read: {}",
            without.message
        );
    }

    /// The fixture is load-bearing: if it graded clean, the A/B above would be
    /// comparing two Passes and could not tell a measurement from a shrug.
    #[test]
    #[serial_test::serial]
    fn the_out_of_tree_measurement_can_actually_fail() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let project = project_with_code();

        let verdict = check_tdg_grade_gate_with_index(
            project.path(),
            &ComplyConfig::default(),
            &cache.path().join("context.idx"),
        );

        assert_eq!(
            verdict.status,
            CheckStatus::Fail,
            "a definition below the floor must fail from the out-of-tree index too: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("awful"),
            "the offender must be named: {}",
            verdict.message
        );
    }

    /// THE counter-test. Building an index must not manufacture a verdict out
    /// of a tree pmat never read a line of.
    ///
    /// Zero definitions violate no floor, so the naive version of this fix
    /// answers "no functions below minimum grade A" — a clean bill of health
    /// for a directory with no code in it, which is worse than the skip it
    /// replaced. An empty measurement is not a measurement.
    #[test]
    fn a_tree_with_no_definitions_is_not_measured_into_a_pass() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let project = tempfile::tempdir().expect("create tempdir");
        std::fs::write(project.path().join("README.md"), "# no code here\n").expect("write");

        let verdict = check_tdg_grade_gate_with_index(
            project.path(),
            &ComplyConfig::default(),
            &cache.path().join("context.idx"),
        );

        assert_eq!(
            verdict.status,
            CheckStatus::Skip,
            "a tree with nothing to grade must not report a pass: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("no definitions"),
            "the reason must say what was missing: {}",
            verdict.message
        );
        assert!(
            !cache.path().join("context.db").exists(),
            "an empty index must not be cached as if it were a measurement"
        );

        // And the same nothing, with a ratchet recorded against it, fails
        // rather than passes: an unmeasured baseline has not held.
        std::fs::write(
            project.path().join(".pmat-gates.toml"),
            "[tdg]\nbaseline = 7\n",
        )
        .expect("write gates toml");
        let recorded = check_tdg_grade_gate_with_index(
            project.path(),
            &ComplyConfig::default(),
            &cache.path().join("context.idx"),
        );
        assert_eq!(recorded.status, CheckStatus::Fail, "{}", recorded.message);
        assert!(
            recorded.message.contains("Not measured"),
            "{}",
            recorded.message
        );
    }

    /// The property the refusal to build was protecting, asked of git rather
    /// than of a filename: after CB-200 has measured a repository, that
    /// repository's `git status` is empty.
    #[test]
    #[serial_test::serial]
    fn measuring_a_repository_leaves_its_git_status_clean() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let _env = CacheDirGuard::pointing_at(cache.path());
        let project = git_project_with_code();
        assert_eq!(porcelain(project.path()), "", "fixture must start clean");

        let verdict = check_tdg_grade_gate(project.path(), &ComplyConfig::default());

        assert_ne!(
            verdict.status,
            CheckStatus::Skip,
            "the point is that it measured: {}",
            verdict.message
        );
        assert_eq!(
            porcelain(project.path()),
            "",
            "CB-200 dirtied the repository it audited"
        );
        assert!(
            !project.path().join(".pmat").exists(),
            "not even an ignored .pmat/ - the tree is not pmat's to write to"
        );
    }

    /// The other counter-test: a project that VERSIONS its own `.pmat/` must
    /// keep it. The out-of-tree index is a fallback for a tree that has none,
    /// never a replacement for one that does — nothing here may ignore, move,
    /// clobber or shadow a committed index.
    #[test]
    #[serial_test::serial]
    fn a_committed_index_is_read_in_place_and_left_alone() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let _env = CacheDirGuard::pointing_at(cache.path());
        let project = git_project_with_code();
        build_in_project_index(project.path());
        git(project.path(), &["add", "-f", ".pmat/context.db"]);
        assert!(
            git(
                project.path(),
                &[
                    "-c",
                    "user.email=t@t",
                    "-c",
                    "user.name=t",
                    "commit",
                    "--no-verify",
                    "-qm",
                    "we version our index",
                ],
            )
            .status
            .success(),
            "commit failed"
        );
        let before = std::fs::metadata(project.path().join(".pmat/context.db"))
            .and_then(|m| m.modified())
            .expect("db mtime");

        let verdict = check_tdg_grade_gate(project.path(), &ComplyConfig::default());

        assert!(
            git(
                project.path(),
                &["ls-files", "--error-unmatch", ".pmat/context.db"],
            )
            .status
            .success(),
            "the committed index was dropped from the index"
        );
        let after = std::fs::metadata(project.path().join(".pmat/context.db"))
            .and_then(|m| m.modified())
            .expect("db mtime");
        assert_eq!(before, after, "the committed index was rewritten");
        assert!(
            !verdict.message.contains("[measured against"),
            "a committed index must be READ, not shadowed by a cached copy: {}",
            verdict.message
        );
        assert!(
            !crate::utils::pmat_cache_dir::comply_index_path(project.path())
                .with_extension("db")
                .exists(),
            "no out-of-tree index may be built for a project that has one"
        );
        // Asked of THIS project's cache entry and not of the cache root: the
        // root is shared, so `<root>/comply` merely existing says only that
        // some other project was measured, which is not this test's business
        // and made it fail under a full parallel run.
    }

    /// The cache is a cache: a second run reads what the first one built
    /// instead of paying for the walk again.
    #[test]
    fn a_built_index_is_reused_by_the_next_run() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let index_path = cache.path().join("context.idx");
        let project = project_with_code();
        let config = ComplyConfig::default();

        let first = check_tdg_grade_gate_with_index(project.path(), &config, &index_path);
        let built = std::fs::metadata(index_path.with_extension("db"))
            .and_then(|m| m.modified())
            .expect("db mtime");

        let second = check_tdg_grade_gate_with_index(project.path(), &config, &index_path);
        let after = std::fs::metadata(index_path.with_extension("db"))
            .and_then(|m| m.modified())
            .expect("db mtime");

        assert_eq!(
            built, after,
            "the second run rebuilt an index it could reuse"
        );
        assert_eq!(first.status, second.status);
        assert_eq!(first.message, second.message);
    }

    /// …and it is a cache of THIS tree. Edit the sources and the next run
    /// measures the edit, or the gate would hold a baseline against a tree that
    /// no longer exists — which is how `.pmat-gates.toml`'s own baseline was
    /// first banked 216 units too high.
    #[test]
    fn an_edit_after_the_build_is_measured_not_ignored() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let index_path = cache.path().join("context.idx");
        let project = project_with_code();
        let config = ComplyConfig::default();

        let before = check_tdg_grade_gate_with_index(project.path(), &config, &index_path);
        assert_eq!(before.status, CheckStatus::Fail, "{}", before.message);

        // The offending definition is deleted, and the CACHED INDEX is aged a
        // clear minute rather than the edit being raced against it: filesystem
        // timestamp granularity is coarser than the microseconds between two
        // writes, which is the kind of thing that passes locally and fails in
        // CI. Ageing the index rather than post-dating the source also keeps
        // the fixture honest — a source file with an mtime in the future is
        // newer than any index that could ever be built from it.
        let src = project.path().join("src/lib.rs");
        std::fs::write(&src, "pub fn fine(a: u32) -> u32 { a + 1 }\n").expect("write source");
        let db = index_path.with_extension("db");
        let db_mtime = std::fs::metadata(&db)
            .and_then(|m| m.modified())
            .expect("db mtime");
        std::fs::File::options()
            .write(true)
            .open(&db)
            .and_then(|f| f.set_modified(db_mtime - std::time::Duration::from_secs(60)))
            .expect("age the cached index a clear minute");
        assert!(
            is_index_stale(project.path(), &db),
            "fixture must be stale before the run under test"
        );

        let after = check_tdg_grade_gate_with_index(project.path(), &config, &index_path);
        assert_eq!(
            after.status,
            CheckStatus::Pass,
            "the fix was not seen: a stale cached index was reported as the current tree: {}",
            after.message
        );
    }

    /// ADVERSARIAL PROBE (temporary): does the cached index notice an edit when
    /// the code lives under crates/*/src rather than <project>/src?
    #[test]
    fn adversarial_probe_workspace_layout_staleness() {
        let cache = tempfile::tempdir().expect("cache");
        let index_path = cache.path().join("context.idx");
        let project = tempfile::tempdir().expect("proj");
        std::fs::create_dir_all(project.path().join("crates/foo/src")).expect("mkdir");
        let src = project.path().join("crates/foo/src/lib.rs");
        std::fs::write(&src, AWFUL).expect("write");

        let before =
            check_tdg_grade_gate_with_index(project.path(), &ComplyConfig::default(), &index_path);
        eprintln!(
            "PROBE top_level_src_exists = {}",
            project.path().join("src").exists()
        );
        eprintln!("PROBE before = {:?} :: {}", before.status, before.message);

        std::fs::write(&src, "pub fn fine(a: u32) -> u32 { a + 1 }\n").expect("write");
        let db = index_path.with_extension("db");
        let m = std::fs::metadata(&db)
            .and_then(|x| x.modified())
            .expect("db mtime");
        std::fs::File::options()
            .write(true)
            .open(&db)
            .and_then(|f| f.set_modified(m - std::time::Duration::from_secs(600)))
            .expect("age the index");
        eprintln!(
            "PROBE is_index_stale_after_edit = {}",
            is_index_stale(project.path(), &db)
        );

        let after =
            check_tdg_grade_gate_with_index(project.path(), &ComplyConfig::default(), &index_path);
        eprintln!("PROBE after = {:?} :: {}", after.status, after.message);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_the_reported_ab {
    use super::super::check::compute_compliance_report;
    use super::super::types::{CheckStatus, ComplianceReport};
    use super::tests_index_outside_the_audited_tree as fixture;

    fn names(report: &ComplianceReport, want: CheckStatus) -> Vec<String> {
        let mut found: Vec<String> = report
            .checks
            .iter()
            .filter(|c| c.status == want)
            .map(|c| c.name.clone())
            .collect();
        found.sort();
        found
    }

    /// #1008 as it was reported: the WHOLE compliance report, one tree, one
    /// commit, the index the only variable. The set of FAILING checks must be
    /// the same on both sides.
    ///
    /// The report on `paiml/rmedia` at `ffe4f68` was 2 failing checks without
    /// an index and 3 with — CB-200 the difference. The exit code happened to
    /// be 1 either way there, because other checks were failing; had CB-200
    /// been the only violation, the same commit would have exited 0 in CI and 1
    /// on a developer's machine.
    ///
    /// Measured here, on this fixture, RED against the code this replaces:
    /// ```text
    /// A fail=3  B fail=4   only_with_index: ["CB-200: TDG Grade Gate"]
    /// ```
    /// and after:
    /// ```text
    /// A fail=4  B fail=4   only_with_index: []
    /// ```
    ///
    /// It asserts the SET and not the count, and it names the differing checks
    /// when it fails, because the interesting failure is a new check that can
    /// only be measured where somebody has run `pmat query` — this defect
    /// arriving again somewhere else.
    #[test]
    #[serial_test::serial]
    fn the_failing_checks_do_not_depend_on_whether_an_index_exists() {
        let cache = tempfile::tempdir().expect("create cache dir");
        let _env = fixture::cache_dir_guard(cache.path());
        let project = fixture::committed_cargo_project();

        let without = compute_compliance_report(project.path()).expect("report without an index");
        let without_failing = names(&without, CheckStatus::Fail);
        assert_eq!(
            fixture::porcelain_of(project.path()),
            "",
            "auditing the project dirtied it"
        );

        fixture::build_index_in(project.path());
        let with = compute_compliance_report(project.path()).expect("report with an index");
        let with_failing = names(&with, CheckStatus::Fail);

        let only_with: Vec<&String> = with_failing
            .iter()
            .filter(|n| !without_failing.contains(n))
            .collect();
        let only_without: Vec<&String> = without_failing
            .iter()
            .filter(|n| !with_failing.contains(n))
            .collect();
        assert!(
            only_with.is_empty() && only_without.is_empty(),
            "the same tree failed a different set of checks depending on whether an index \
             happened to exist.\n  fails only WITH an index:    {only_with:?}\n  \
             fails only WITHOUT an index: {only_without:?}"
        );
        assert!(
            without_failing.iter().any(|n| n.starts_with("CB-200")),
            "the fixture must actually violate CB-200, or this proves nothing. Failing: \
             {without_failing:?}"
        );
    }
}
