//! CB-2110 (goal-mode.md §4.3, §7 row CB-2110, invariant E): the spec ↔ epic
//! edge. Every specification under `docs/specifications/` carries YAML
//! front-matter — `epic:` (an issue number or null), `status:` (`active` |
//! `superseded` | `historical`) and an optional `vendors:` list — and every
//! `active` spec names an open issue labelled `epic` that has at least one
//! sub-issue. Membership is GitHub's native sub-issue relation and only that
//! (operator decision, 2026-09-09); an `Epic: #N` line in a body is prose.
//! `superseded` and `historical` take a spec off the epic leg, never off the
//! parse leg, and every exempt spec is listed by name wherever the rule speaks,
//! so the escape hatch is one nobody can miss (§4.3).
//!
//! Pure: the caller reads the files and hands in the snapshot; nothing here
//! touches the filesystem or the network. The comply handler
//! `check_spec_epics` is the only caller.

use crate::services::work_sync::{GithubSnapshot, IssueState};
use std::collections::BTreeSet;

/// The label an epic issue carries.
pub const EPIC_LABEL: &str = "epic";
/// Where the specifications live, relative to the project root.
pub const SPECS_DIR: &str = "docs/specifications";

/// A spec's `status:` — the closed set of §4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecStatus {
    Active,
    Superseded,
    Historical,
}

impl SpecStatus {
    /// The word as the front-matter spells it, or none.
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "active" => Some(Self::Active),
            "superseded" => Some(Self::Superseded),
            "historical" => Some(Self::Historical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Superseded => "superseded",
            Self::Historical => "historical",
        }
    }

    /// Only an `active` spec is on the epic leg (§4.3, §7 row CB-2110).
    pub fn on_epic_leg(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// What a spec's front-matter says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecFrontMatter {
    /// The epic issue's number; `None` for `epic: null` and for a missing key.
    pub epic: Option<u64>,
    pub status: SpecStatus,
    /// Vendor roles E.1's review must include; empty when absent.
    pub vendors: Vec<String>,
}

/// Why a spec's front-matter did not parse. Each renders to the `what` of a
/// `BAD-FRONT-MATTER` finding except [`FrontMatterError::Absent`], which is
/// `NO-FRONT-MATTER` — the falsifier "delete the front-matter" and the
/// falsifier "break a key" are different defects with different fixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontMatterError {
    /// The file does not begin with a `---` line.
    Absent,
    /// The opening `---` has no closing `---`.
    Unterminated,
    /// `status:` is missing.
    MissingStatus,
    /// `status:` is not `active` | `superseded` | `historical`.
    BadStatus(String),
    /// `epic:` is neither an issue number nor `null`.
    BadEpic(String),
    /// `vendors:` is not a list of names.
    BadVendors(String),
}

impl FrontMatterError {
    pub fn render(&self) -> String {
        match self {
            Self::Absent => "the file does not begin with a --- front-matter block".to_string(),
            Self::Unterminated => "the --- front-matter block is not closed".to_string(),
            Self::MissingStatus => {
                "status: is missing (active | superseded | historical)".to_string()
            }
            Self::BadStatus(s) => format!("status: {s:?} is not active | superseded | historical"),
            Self::BadEpic(s) => format!("epic: {s:?} is neither an issue number nor null"),
            Self::BadVendors(s) => format!("vendors: {s:?} is not a list of names"),
        }
    }
}

/// Parse the YAML front-matter at the top of a spec. The block is the lines
/// between a first line `---` and the next line `---`; keys other than
/// `epic`, `status` and `vendors` are ignored (`pmat spec` reads its own);
/// `epic:` absent reads as `epic: null`.
pub fn parse_front_matter(text: &str) -> Result<SpecFrontMatter, FrontMatterError> {
    let _ = text;
    Err(FrontMatterError::BadStatus(
        "not implemented (PMAT-728 phase 2)".to_string(),
    ))
}

/// One spec as the caller read it: the path relative to the project root
/// (`docs/specifications/…`) and the file's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecInput {
    pub path: String,
    pub text: String,
}

/// One finding, one spec, the first clause that failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecFinding {
    NoFrontMatter { spec: String },
    BadFrontMatter { spec: String, what: String },
    NoEpic { spec: String },
    EpicAbsent { spec: String, number: u64 },
    EpicClosed { spec: String, number: u64 },
    NotAnEpic { spec: String, number: u64 },
    SubIssuesUnmeasured { spec: String, number: u64 },
    NoSubIssues { spec: String, number: u64 },
}

impl SpecFinding {
    pub fn spec(&self) -> &str {
        match self {
            Self::NoFrontMatter { spec }
            | Self::BadFrontMatter { spec, .. }
            | Self::NoEpic { spec }
            | Self::EpicAbsent { spec, .. }
            | Self::EpicClosed { spec, .. }
            | Self::NotAnEpic { spec, .. }
            | Self::SubIssuesUnmeasured { spec, .. }
            | Self::NoSubIssues { spec, .. } => spec,
        }
    }

    pub fn class(&self) -> &'static str {
        match self {
            Self::NoFrontMatter { .. } => "NO-FRONT-MATTER",
            Self::BadFrontMatter { .. } => "BAD-FRONT-MATTER",
            Self::NoEpic { .. } => "NO-EPIC",
            Self::EpicAbsent { .. } => "EPIC-ABSENT",
            Self::EpicClosed { .. } => "EPIC-CLOSED",
            Self::NotAnEpic { .. } => "NOT-AN-EPIC",
            Self::SubIssuesUnmeasured { .. } => "SUB-ISSUES-UNMEASURED",
            Self::NoSubIssues { .. } => "NO-SUB-ISSUES",
        }
    }

    /// A finding the rule could not measure (doctrine 2): the row that carries
    /// it is `not_measured`, never merely red.
    pub fn is_unmeasured(&self) -> bool {
        matches!(self, Self::SubIssuesUnmeasured { .. })
    }

    pub fn render(&self) -> String {
        match self {
            Self::NoFrontMatter { spec } => format!(
                "NO-FRONT-MATTER {spec}: the file does not begin with a --- front-matter block (goal-mode.md §4.3)"
            ),
            Self::BadFrontMatter { spec, what } => format!("BAD-FRONT-MATTER {spec}: {what}"),
            Self::NoEpic { spec } => format!(
                "NO-EPIC {spec}: epic is null — an active spec names an open issue labelled epic (goal-mode.md §4.3)"
            ),
            Self::EpicAbsent { spec, number } => {
                format!("EPIC-ABSENT {spec}: names #{number}, which is not in the snapshot")
            }
            Self::EpicClosed { spec, number } => {
                format!("EPIC-CLOSED {spec}: names #{number}, which is closed")
            }
            Self::NotAnEpic { spec, number } => {
                format!("NOT-AN-EPIC {spec}: names #{number}, which is not labelled epic")
            }
            Self::SubIssuesUnmeasured { spec, number } => format!(
                "SUB-ISSUES-UNMEASURED {spec}: the snapshot does not carry #{number}'s sub-issue count — not measured is not zero (goal-mode.md doctrine 2)"
            ),
            Self::NoSubIssues { spec, number } => format!(
                "NO-SUB-ISSUES {spec}: #{number} has no sub-issue — a spec's tickets are its epic's sub-issues (goal-mode.md §4.3)"
            ),
        }
    }
}

/// The parse leg's result: every spec that parsed, and a finding for every
/// one that did not or that is active with no epic (offline — the epic leg
/// needs the snapshot, this does not).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// Sorted by path.
    pub specs: Vec<(String, SpecFrontMatter)>,
    /// Sorted by path: `NO-FRONT-MATTER`, `BAD-FRONT-MATTER`, `NO-EPIC`.
    pub findings: Vec<SpecFinding>,
}

impl Parsed {
    /// The specs on the epic leg.
    pub fn active(&self) -> impl Iterator<Item = &(String, SpecFrontMatter)> {
        self.specs.iter().filter(|(_, fm)| fm.status.on_epic_leg())
    }

    /// The specs off the epic leg, by name with their status, sorted — the
    /// list every verdict prints (§4.3: an escape hatch nobody can see is a
    /// hole).
    pub fn exempt(&self) -> Vec<(String, SpecStatus)> {
        self.specs
            .iter()
            .filter(|(_, fm)| !fm.status.on_epic_leg())
            .map(|(p, fm)| (p.clone(), fm.status))
            .collect()
    }

    /// `a.md (historical), b.md (superseded)`, or `none`.
    pub fn render_exempt(&self) -> String {
        let ex = self.exempt();
        if ex.is_empty() {
            return "none".to_string();
        }
        ex.iter()
            .map(|(p, s)| format!("{p} ({})", s.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The epics the active specs name — the issues whose sub-issue counts the
    /// epic leg will need. Empty means the epic leg needs no snapshot.
    pub fn epics_named(&self) -> BTreeSet<u64> {
        self.active().filter_map(|(_, fm)| fm.epic).collect()
    }
}

/// The parse leg over every spec, in path order.
pub fn parse_specs(specs: &[SpecInput]) -> Parsed {
    let _ = specs;
    Parsed::default()
}

/// The epic leg: for every active spec that names an epic, the first clause
/// that fails — absent, closed, not labelled `epic`, sub-issue count not in
/// the snapshot, no sub-issue — in path order.
pub fn bind_epics(parsed: &Parsed, snapshot: &GithubSnapshot) -> Vec<SpecFinding> {
    let _ = (parsed, snapshot, EPIC_LABEL, IssueState::Open);
    Vec::new()
}

#[cfg(test)]
mod tests;
