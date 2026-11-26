//! The ticket quality rubric (FLOW-OWNER-001 §3, "testable-first").
//!
//! Seven items, each all-or-nothing, 100 points in total. The score is a pure
//! function of the ticket's text, labels, parent and milestone: no model call,
//! no clock, no I/O. The same input always gives the same score.

use crate::cli::handlers::work_handlers::work_add_triage::derive_from_body;
use crate::services::normalized_score::NormalizedScore;
use regex::Regex;
use serde::Serialize;
use std::fmt;
use std::sync::LazyLock;

/// The pass mark (decision D5): under it a ticket is refused or sent for rewrite.
pub const PASS_MARK: u32 = 70;
/// The most acceptance items a ticket that fits one PR carries.
pub const MAX_ACCEPTANCE_ITEMS: usize = 8;
/// The longest title the title item accepts, in characters.
pub const MAX_TITLE_CHARS: usize = 80;

/// What the rubric reads. Nothing else about a ticket can change its score.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ticket {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    /// The ticket is a sub-issue of an open epic or master.
    pub parent_open: bool,
    pub has_milestone: bool,
}

/// One rubric item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Acceptance,
    Falsifiable,
    ParentMilestone,
    Scope,
    Problem,
    Size,
    Title,
}

impl Item {
    /// Every item, in the order of the spec's table.
    pub const ALL: [Item; 7] = [
        Item::Acceptance,
        Item::Falsifiable,
        Item::ParentMilestone,
        Item::Scope,
        Item::Problem,
        Item::Size,
        Item::Title,
    ];

    /// The points the item is worth. The seven sum to 100.
    pub fn points(self) -> u32 {
        match self {
            Item::Acceptance => 25,
            Item::Falsifiable => 20,
            Item::ParentMilestone | Item::Scope => 15,
            Item::Problem | Item::Size => 10,
            Item::Title => 5,
        }
    }

    /// The stable id used in JSON and in comments.
    pub fn id(self) -> &'static str {
        match self {
            Item::Acceptance => "acceptance",
            Item::Falsifiable => "falsifiable",
            Item::ParentMilestone => "parent-milestone",
            Item::Scope => "scope",
            Item::Problem => "problem",
            Item::Size => "size",
            Item::Title => "title",
        }
    }

    /// What to add to a ticket that fails the item.
    pub fn missing(self) -> &'static str {
        match self {
            Item::Acceptance => {
                "acceptance criteria: a `- [ ]` checklist under an `## Acceptance` heading"
            }
            Item::Falsifiable => {
                "falsifiable check: a command, test name or file in a code span inside the acceptance section"
            }
            Item::ParentMilestone => {
                "parent and milestone: make it a sub-issue of an open epic or master, and set a milestone"
            }
            Item::Scope => "scope: name at least one repo-relative file path, or a `Scope:` line",
            Item::Problem => "problem statement: an `Observed` and an `Expected` section or line label",
            Item::Size => "size: at most 8 acceptance items, and a `kind:` label",
            Item::Title => {
                "title form: at most 80 characters, starting with a verb or a `kind(scope):` prefix"
            }
        }
    }

    /// Does `t` pass the item?
    pub fn passes(self, t: &Ticket) -> bool {
        match self {
            Item::Acceptance => !checklist(&t.body).is_empty(),
            Item::Falsifiable => has_falsifiable_check(&t.body),
            Item::ParentMilestone => t.parent_open && t.has_milestone,
            Item::Scope => has_scope(t),
            Item::Problem => {
                has_label_or_heading(&t.body, "observed")
                    && has_label_or_heading(&t.body, "expected")
            }
            Item::Size => checklist(&t.body).len() <= MAX_ACCEPTANCE_ITEMS && has_kind(&t.labels),
            Item::Title => title_is_well_formed(&t.title),
        }
    }
}

/// The points one item earned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemScore {
    pub item: &'static str,
    pub points: u32,
    pub max: u32,
}

/// The verdict of the rubric on one ticket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TicketScore {
    pub total: u32,
    pub pass_mark: u32,
    pub passed: bool,
    pub items: Vec<ItemScore>,
    /// One line per failed item, saying what to add.
    pub missing: Vec<String>,
}

impl NormalizedScore for TicketScore {
    fn raw(&self) -> f64 {
        f64::from(self.total)
    }

    fn max_raw(&self) -> f64 {
        100.0
    }
}

impl fmt::Display for TicketScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verdict = if self.passed { "PASS" } else { "NEEDS REWRITE" };
        writeln!(
            f,
            "ticket score: {}/100 (pass mark {}) {verdict}",
            self.total, self.pass_mark
        )?;
        for i in &self.items {
            writeln!(f, "  {:>2}/{:<2} {}", i.points, i.max, i.item)?;
        }
        for m in &self.missing {
            writeln!(f, "  missing: {m}")?;
        }
        Ok(())
    }
}

/// Score `t` against the rubric; `pass_mark` only decides `passed`.
pub fn score(t: &Ticket, pass_mark: u32) -> TicketScore {
    let mut items = Vec::new();
    let mut missing = Vec::new();
    for item in Item::ALL {
        let ok = item.passes(t);
        items.push(ItemScore {
            item: item.id(),
            points: if ok { item.points() } else { 0 },
            max: item.points(),
        });
        if !ok {
            missing.push(item.missing().to_string());
        }
    }
    let total = items.iter().map(|i| i.points).sum();
    TicketScore {
        total,
        pass_mark,
        passed: total >= pass_mark,
        items,
        missing,
    }
}

/// The filing gate (decision D5): a ticket under the pass mark is refused, and the
/// refusal names what is missing.
pub fn refuse_low_score(t: &Ticket, pass_mark: u32) -> anyhow::Result<TicketScore> {
    let s = score(t, pass_mark);
    if !s.passed {
        anyhow::bail!(
            "refused: the ticket scores {}/100, under the pass mark {pass_mark} (FLOW-OWNER-001) — nothing was linked or written. Missing:\n  - {}",
            s.total,
            s.missing.join("\n  - ")
        );
    }
    Ok(s)
}

/// The checklist items (`[ ] x`, `[x] x`) of the acceptance section.
pub fn checklist(body: &str) -> Vec<String> {
    derive_from_body(body)
        .acceptance
        .into_iter()
        .filter(|l| is_checklist_item(l))
        .collect()
}

fn is_checklist_item(line: &str) -> bool {
    ["[ ]", "[x]", "[X]"]
        .iter()
        .any(|b| line.strip_prefix(b).is_some_and(|r| !r.trim().is_empty()))
}

static CODE_SPAN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`[^`\s][^`]*`").expect("CODE_SPAN is a valid regex"));

/// A code span or a fence opener inside the acceptance section.
fn has_falsifiable_check(body: &str) -> bool {
    derive_from_body(body)
        .acceptance
        .iter()
        .any(|l| l.starts_with("```") || CODE_SPAN.is_match(l))
}

fn has_kind(labels: &[String]) -> bool {
    labels.iter().any(|l| {
        l.to_ascii_lowercase()
            .strip_prefix("kind:")
            .is_some_and(|k| !k.trim().is_empty())
    })
}

/// A heading (`## Observed`) or a line label (`Observed:`, `**Observed:**`) for `word`.
fn has_label_or_heading(body: &str, word: &str) -> bool {
    body.lines().any(|line| {
        let heading = line.trim_start().starts_with('#');
        let norm = line
            .trim()
            .trim_start_matches(['#', '*', '_', '`', '-', '>', ' '])
            .to_ascii_lowercase();
        norm.strip_prefix(word).is_some_and(|rest| {
            heading
                || rest
                    .trim_start_matches(['*', '_', '`', ' '])
                    .starts_with(':')
        })
    })
}

static WIDE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z0-9_./~*<>{}$:#@%=+-]+").expect("WIDE_TOKEN is a valid regex")
});
static LINE_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?::\d+)+$").expect("LINE_SUFFIX is a valid regex"));
static FILE_EXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.[A-Za-z][A-Za-z0-9]{0,7}$").expect("FILE_EXT is a valid regex")
});

/// Is `tok` a clean repo-relative path: only path characters, at least one `/`, not
/// absolute, not a URL, no `..`, no placeholder or glob.
fn is_relative_path(tok: &str) -> bool {
    let clean = tok
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'));
    let starts_inside = tok.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '.');
    clean && starts_inside && tok.contains('/') && !tok.contains("//") && !tok.contains("..")
}

/// Every repo-relative path the text names, in order, without repeats. A trailing
/// `:line` is dropped, and so is sentence punctuation.
pub fn named_paths(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for m in WIDE_TOKEN.find_iter(text) {
        let tok = m.as_str().trim_end_matches(['.', ':', '-']);
        let tok = LINE_SUFFIX.replace(tok, "");
        let tok = tok.trim_start_matches("./");
        if is_relative_path(tok) && !out.iter().any(|p| p == tok) {
            out.push(tok.to_string());
        }
    }
    out
}

/// The named paths that are files: the last segment has an extension.
pub fn named_files(text: &str) -> Vec<String> {
    named_paths(text)
        .into_iter()
        .filter(|p| FILE_EXT.is_match(p))
        .collect()
}

static SCOPED_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*[A-Za-z]+\([^)\s][^)]*\)!?:\s*\S").expect("SCOPED_PREFIX is a valid regex")
});

/// Scope is named by a file or directory path, a `kind(scope):` title, or a
/// `Scope:` / `Component:` line with a value.
fn has_scope(t: &Ticket) -> bool {
    let path = named_paths(&t.body)
        .iter()
        .any(|p| FILE_EXT.is_match(p) || p.ends_with('/') || p.matches('/').count() >= 2);
    path || SCOPED_PREFIX.is_match(&t.title)
        || ["scope", "component"]
            .iter()
            .any(|w| has_value_label(&t.body, w))
}

/// A `word: value` line whose value is not empty.
fn has_value_label(body: &str, word: &str) -> bool {
    body.lines().any(|line| {
        let norm = line
            .trim()
            .trim_start_matches(['*', '_', '`', '-', '>', ' '])
            .to_ascii_lowercase();
        norm.strip_prefix(word)
            .map(|r| r.trim_start_matches(['*', '_', '`', ' ']))
            .and_then(|r| r.strip_prefix(':'))
            .is_some_and(|v| !v.trim_matches(['*', '_', '`', ' ']).is_empty())
    })
}

/// Imperative verbs a title may start with. A fixed list keeps the item deterministic.
const VERBS: &[&str] = &[
    "add",
    "allow",
    "audit",
    "avoid",
    "block",
    "build",
    "bump",
    "change",
    "check",
    "clean",
    "close",
    "compute",
    "convert",
    "count",
    "create",
    "cut",
    "declare",
    "delete",
    "deploy",
    "derive",
    "detect",
    "disable",
    "document",
    "drop",
    "emit",
    "enable",
    "enforce",
    "ensure",
    "expose",
    "extend",
    "extract",
    "fail",
    "file",
    "fix",
    "fold",
    "gate",
    "generate",
    "guard",
    "handle",
    "harden",
    "implement",
    "improve",
    "include",
    "install",
    "keep",
    "land",
    "limit",
    "link",
    "list",
    "load",
    "lock",
    "log",
    "make",
    "measure",
    "merge",
    "migrate",
    "move",
    "pin",
    "port",
    "prevent",
    "print",
    "prove",
    "publish",
    "read",
    "record",
    "reduce",
    "refactor",
    "refuse",
    "reject",
    "remove",
    "rename",
    "replace",
    "report",
    "require",
    "restore",
    "retire",
    "retry",
    "revert",
    "rewrite",
    "route",
    "run",
    "score",
    "set",
    "ship",
    "show",
    "skip",
    "speed",
    "split",
    "stabilise",
    "stabilize",
    "stop",
    "support",
    "sync",
    "teach",
    "test",
    "tighten",
    "track",
    "trim",
    "turn",
    "unify",
    "update",
    "upgrade",
    "use",
    "validate",
    "verify",
    "wire",
    "wrap",
    "write",
    "bound",
    "name",
    "raise",
    "lower",
    // conventional-commit types that are not verbs
    "feat",
    "docs",
    "ci",
    "chore",
    "perf",
    "style",
];

fn title_is_well_formed(title: &str) -> bool {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > MAX_TITLE_CHARS {
        return false;
    }
    let first = title
        .split(|c: char| !c.is_ascii_alphabetic())
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    SCOPED_PREFIX.is_match(title) || VERBS.contains(&first.as_str())
}
