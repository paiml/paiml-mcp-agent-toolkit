//! Diff-scoped complexity verdict for the generated pre-commit hook (BSE-12).
//!
//! The generated pre-commit hook used to measure WHOLE FILES: it ran
//! `pmat analyze complexity --file "$SRC_FILE"` over every staged file and
//! refused the commit when any function in that file was over threshold. A
//! one-line fix in an undebted function of a file carrying pre-existing debt
//! was therefore refused, and the only way to land the line was to decompose
//! functions the change never touched.
//!
//! The pre-commit hook is FEEDBACK, not the gate — `ci / gate` is what
//! enforces the thresholds on the merge path — so scoping the hook's verdict
//! to the diff removes no enforcement. It only stops charging a developer for
//! debt they did not write.
//!
//! The rule implemented here:
//!
//! * A function is TOUCHED when its measured `[line_start, line_end]` span in
//!   the NEW file overlaps a line added or modified by a staged hunk.
//! * A touched function is refused only when a metric exceeds the threshold
//!   AND exceeds the same function's value in the OLD file (`HEAD:<path>`),
//!   measured the same way — i.e. only when debt GREW.
//! * A function with no counterpart in the old file (new, renamed, split) is
//!   judged against the threshold alone, never as growth.
//! * Untouched functions never affect the verdict, over threshold or not.
//!
//! Function boundaries and metrics come from the one existing measurement path
//! ([`collect_functions`] + [`measure_block`] + [`FunctionSpans`]); this module
//! adds no second parser.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::services::accurate_complexity_analyzer::{collect_functions, measure_block};
use crate::services::source_line_index::{FunctionSpans, LineSpan};

/// One measured function: its span in the file and its two complexity metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredFn {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    /// 1-based inclusive span; `0` when the definition could not be located.
    pub line_start: u32,
    pub line_end: u32,
}

impl MeasuredFn {
    /// True when this function's span overlaps any of `ranges`.
    ///
    /// An unlocated span (`line_start == 0`) overlaps nothing: "not measured"
    /// must never render as "touched" or as "untouched" by accident, and the
    /// conservative reading for a verdict that only ever REFUSES is to leave
    /// it out of the touched set.
    #[must_use]
    pub fn overlaps(&self, ranges: &[TouchedRange]) -> bool {
        if self.line_start == 0 {
            return false;
        }
        ranges
            .iter()
            .any(|r| self.line_start <= r.end && self.line_end >= r.start)
    }
}

/// A 1-based inclusive run of lines in the NEW file that a hunk added or
/// modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchedRange {
    pub start: u32,
    pub end: u32,
}

/// The per-function limits the hook enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebtThresholds {
    pub max_cyclomatic: u32,
    pub max_cognitive: u32,
}

/// One reason to refuse: a touched function whose debt grew past a limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtGrowth {
    pub function: String,
    /// `"Cyclomatic"` or `"Cognitive"`.
    pub metric: &'static str,
    pub measured: u32,
    pub limit: u32,
    /// The same function's value in the old file, when it had a counterpart.
    pub previous: Option<u32>,
}

impl DebtGrowth {
    /// Render the offender the way the hook prints it: the function's name and
    /// both numbers — what it measures now and what it measured before.
    #[must_use]
    pub fn render(&self) -> String {
        match self.previous {
            Some(previous) => format!(
                "{} - {} {} > {} (was {})",
                self.function, self.metric, self.measured, self.limit, previous
            ),
            None => format!(
                "{} - {} {} > {} (new function, no previous measurement)",
                self.function, self.metric, self.measured, self.limit
            ),
        }
    }
}

/// The hook's verdict over one staged file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebtVerdict {
    Allowed,
    Refused(Vec<DebtGrowth>),
}

impl DebtVerdict {
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, DebtVerdict::Allowed)
    }

    /// Every offender line, in the order the hook prints them.
    #[must_use]
    pub fn rendered(&self) -> Vec<String> {
        match self {
            DebtVerdict::Allowed => Vec::new(),
            DebtVerdict::Refused(growths) => growths.iter().map(DebtGrowth::render).collect(),
        }
    }
}

/// Every function in `source`, measured through the existing complexity path.
///
/// Reuses [`collect_functions`] (which finds methods in `impl`/`trait`/`mod`
/// blocks, not only free functions) and [`FunctionSpans`] (which reads the real
/// closing brace) so this module never invents a boundary or a metric.
///
/// # Errors
/// Returns an error when `source` is not parseable Rust.
pub fn measure_source(source: &str) -> Result<Vec<MeasuredFn>> {
    let ast = syn::parse_file(source)?;
    let mut spans = FunctionSpans::from_source(source);
    let mut measured = Vec::new();
    for func in collect_functions(&ast.items) {
        let span = spans.take(&func.name).unwrap_or(LineSpan::UNKNOWN);
        let block = measure_block(&func.name, func.block);
        measured.push(MeasuredFn {
            name: func.name.clone(),
            cyclomatic: block.cyclomatic,
            cognitive: block.cognitive,
            line_start: span.start,
            line_end: span.end,
        });
    }
    Ok(measured)
}

/// The NEW-file line ranges named by a unified diff (`git diff --cached -U0`).
///
/// Reads the `+` side of each `@@ -a,b +c,d @@` header. A hunk with `+c,0` is a
/// pure deletion: no new line exists to overlap, so the two lines that now
/// straddle the removal are reported instead — the enclosing function WAS
/// modified and must be judged.
#[must_use]
pub fn parse_touched_ranges(diff: &str) -> Vec<TouchedRange> {
    diff.lines().filter_map(parse_hunk_header).collect()
}

fn parse_hunk_header(line: &str) -> Option<TouchedRange> {
    let rest = line.strip_prefix("@@ ")?;
    let body = rest.split(" @@").next()?;
    let plus = body.split_whitespace().find(|t| t.starts_with('+'))?;
    let (start, count) = parse_plus_token(plus)?;
    Some(range_from_hunk(start, count))
}

fn parse_plus_token(token: &str) -> Option<(u32, u32)> {
    let digits = token.strip_prefix('+')?;
    let mut parts = digits.split(',');
    let start: u32 = parts.next()?.parse().ok()?;
    let count: u32 = match parts.next() {
        Some(raw) => raw.parse().ok()?,
        None => 1,
    };
    Some((start, count))
}

fn range_from_hunk(start: u32, count: u32) -> TouchedRange {
    if count == 0 {
        // `@@ -10,3 +9,0 @@`: three lines removed after new-file line 9.
        let anchor = start.max(1);
        return TouchedRange {
            start: anchor,
            end: anchor + 1,
        };
    }
    TouchedRange {
        start,
        end: start + count - 1,
    }
}

/// The functions of `functions` whose spans overlap a touched range.
#[must_use]
pub fn touched_functions<'a>(
    functions: &'a [MeasuredFn],
    ranges: &[TouchedRange],
) -> Vec<&'a MeasuredFn> {
    functions.iter().filter(|f| f.overlaps(ranges)).collect()
}

/// The hook's verdict: a function of the diff's TOUCHED functions only.
///
/// `old_source` is the file as of `HEAD` (`None` for a file that did not exist
/// there — every one of its functions is then judged against the threshold
/// alone). `diff` is the output of `git diff --cached -U0 -- <path>`.
///
/// # Errors
/// Returns an error when either source is not parseable Rust.
pub fn diff_scoped_verdict(
    old_source: Option<&str>,
    new_source: &str,
    diff: &str,
    thresholds: DebtThresholds,
) -> Result<DebtVerdict> {
    let new_functions = measure_source(new_source)?;
    let old_functions = match old_source {
        Some(text) => measure_source(text)?,
        None => Vec::new(),
    };
    let ranges = parse_touched_ranges(diff);

    let mut growths = Vec::new();
    for func in touched_functions(&new_functions, &ranges) {
        let previous = old_functions.iter().find(|old| old.name == func.name);
        growths.extend(growth_for(func, previous, thresholds));
    }

    if growths.is_empty() {
        Ok(DebtVerdict::Allowed)
    } else {
        Ok(DebtVerdict::Refused(growths))
    }
}

fn growth_for(
    func: &MeasuredFn,
    previous: Option<&MeasuredFn>,
    thresholds: DebtThresholds,
) -> Vec<DebtGrowth> {
    let mut growths = Vec::new();
    let cyclomatic = grew(
        "Cyclomatic",
        &func.name,
        func.cyclomatic,
        previous.map(|p| p.cyclomatic),
        thresholds.max_cyclomatic,
    );
    let cognitive = grew(
        "Cognitive",
        &func.name,
        func.cognitive,
        previous.map(|p| p.cognitive),
        thresholds.max_cognitive,
    );
    growths.extend(cyclomatic);
    growths.extend(cognitive);
    growths
}

/// Over the limit AND worse than before — the two halves of "debt grew".
///
/// A touched function that is over the limit but no worse than it was is
/// allowed: the developer did not write that debt, and `ci / gate` still
/// refuses to let the file's total escape.
fn grew(
    metric: &'static str,
    name: &str,
    measured: u32,
    previous: Option<u32>,
    limit: u32,
) -> Option<DebtGrowth> {
    if measured <= limit {
        return None;
    }
    if previous.is_some_and(|before| measured <= before) {
        return None;
    }
    Some(DebtGrowth {
        function: name.to_string(),
        metric,
        measured,
        limit,
        previous,
    })
}

/// The staged verdict for one file, read out of git itself.
///
/// This is the adapter the CLI entry point calls: it performs the three reads
/// the rule above needs and does no judging of its own.
///
/// * NEW source is `git show :<path>` — the INDEX, not the working tree. The
///   hook judges what is being COMMITTED; an unstaged edit sitting in the
///   working tree is not part of this commit and must not change the verdict.
/// * OLD source is `git show HEAD:<path>`, `None` when the file has no
///   counterpart there (added, renamed, or an unborn branch).
/// * The diff is `git diff --cached -U0 -- <path>`.
///
/// # Errors
/// Returns an error when git is unusable, when `path` is not staged (an
/// absence must never render as "no violations"), when the diff cannot be
/// read, or when either source is not parseable Rust.
pub fn staged_verdict(
    repo_root: &Path,
    path: &Path,
    thresholds: DebtThresholds,
) -> Result<DebtVerdict> {
    ensure_rust_source(path)?;
    let spec = path.to_string_lossy().replace('\\', "/");
    let staged = format!(":{spec}");
    let new_source = git_read(repo_root, &["show", &staged])?.ok_or_else(|| {
        anyhow!(
            "{spec} is not staged: `git show {staged}` found no blob, so the \
             diff-scoped check has nothing to judge"
        )
    })?;
    let old_source = git_read(repo_root, &["show", &format!("HEAD:{spec}")])?;
    // A failed diff read is an error, never an empty range list: no ranges
    // means no touched functions means Allowed, which would be a pass this
    // code never measured.
    let diff = git_read(repo_root, &["diff", "--cached", "-U0", "--", &spec])?
        .ok_or_else(|| anyhow!("`git diff --cached -U0 -- {spec}` failed in {repo_root:?}"))?;
    diff_scoped_verdict(old_source.as_deref(), &new_source, &diff, thresholds)
}

/// [`staged_verdict`] for a path as the user typed it: finds the repository
/// and the path's name inside it, then judges.
///
/// # Errors
/// As [`staged_verdict`], plus when `path` is not inside a git repository.
pub fn staged_verdict_for_file(path: &Path, thresholds: DebtThresholds) -> Result<DebtVerdict> {
    let (root, relative) = locate_in_repo(path)?;
    staged_verdict(&root, &relative, thresholds)
}

/// The repository root containing `path`, and `path` as git names it.
fn locate_in_repo(path: &Path) -> Result<(PathBuf, PathBuf)> {
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("{path:?} names no file"))?;
    let root = git_read(&dir, &["rev-parse", "--show-toplevel"])?
        .ok_or_else(|| anyhow!("{path:?} is not inside a git repository"))?;
    let root = canonical(Path::new(root.trim()));
    let relative = canonical(&dir)
        .strip_prefix(&root)
        .map(Path::to_path_buf)
        .map_err(|_| anyhow!("{path:?} is not under the repository root {root:?}"))?
        .join(name);
    Ok((root, relative))
}

/// Symlinked temp roots (`/tmp` on macOS) make the string forms of the same
/// directory differ; compare the resolved forms, and fall back to the input
/// when it cannot be resolved (a staged-but-deleted path has no inode).
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The measurement path is `syn`, so a non-Rust file cannot be graded here.
/// Saying so is the point: a caller that silently allowed every `.py` file
/// would be reporting a verdict it never reached.
fn ensure_rust_source(path: &Path) -> Result<()> {
    if path.extension().is_some_and(|ext| ext == "rs") {
        return Ok(());
    }
    Err(anyhow!(
        "diff-scoped complexity is measured with the Rust analyzer; {path:?} is not a .rs file"
    ))
}

/// `git -C <dir> <args>`: `Ok(None)` when git EXITED non-zero (a missing
/// object), `Err` only when git could not be run at all.
fn git_read(dir: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new("git")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .output()
        .with_context(|| format!("running `git {}` in {dir:?}", args.join(" ")))?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
}
#[cfg(test)]
#[path = "hook_debt_scope_tests.rs"]
mod hook_debt_scope_tests;
