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

/// `vendors:` as a flow list — `[]` or `[a, b]`, quotes stripped; a bare
/// scalar is [`FrontMatterError::BadVendors`]. The block form (`- name`
/// lines) is collected by [`ParseState::process_line`].
fn parse_vendors(val: &str) -> Result<Vec<String>, FrontMatterError> {
    let val = val.trim();
    if val.is_empty() || val == "[]" {
        return Ok(Vec::new());
    }
    if val.starts_with('[') && val.ends_with(']') {
        let inner = val[1..val.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for v in inner.split(',') {
            out.push(v.trim().trim_matches('\'').trim_matches('"').to_string());
        }
        return Ok(out);
    }
    Err(FrontMatterError::BadVendors(val.to_string()))
}

/// `epic:` — `null`, `~`, empty or absent is `None`; plain digits are the
/// issue number; anything else is [`FrontMatterError::BadEpic`] with the raw
/// text (`#123` is a reference, not a number — §4.3 names an issue).
fn parse_epic(val: Option<&str>) -> Result<Option<u64>, FrontMatterError> {
    match val {
        Some("null") | Some("~") | Some("") | None => Ok(None),
        Some(v) => match v.parse::<u64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => Err(FrontMatterError::BadEpic(v.to_string())),
        },
    }
}

/// The three keys as the block is read line by line; `in_vendors_block` is
/// true while a `vendors:` with no value on its line is collecting `- name`
/// lines.
struct ParseState {
    epic_val: Option<String>,
    status_val: Option<String>,
    vendors_val: Vec<String>,
    in_vendors_block: bool,
}

impl ParseState {
    fn process_line(&mut self, line: &str) -> Result<(), FrontMatterError> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        if self.in_vendors_block {
            if let Some(item) = trimmed.strip_prefix("- ") {
                let v = item.trim().trim_matches('\'').trim_matches('"');
                self.vendors_val.push(v.to_string());
                return Ok(());
            }
            if !line.starts_with(' ') {
                self.in_vendors_block = false;
            }
        }

        if let Some(rest) = line.strip_prefix("epic:") {
            self.in_vendors_block = false;
            self.epic_val = Some(rest.trim().trim_matches('\'').trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("status:") {
            self.in_vendors_block = false;
            self.status_val = Some(rest.trim().trim_matches('\'').trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("vendors:") {
            let val = rest.trim();
            if val.is_empty() {
                self.in_vendors_block = true;
            } else {
                self.vendors_val = parse_vendors(val)?;
            }
        } else {
            self.in_vendors_block = false;
        }
        Ok(())
    }
}

/// Parse the YAML front-matter at the top of a spec. The block is the lines
/// between a first line exactly `---` and the next line exactly `---` (CRLF
/// tolerated); simple `key: value` lines, with `#` comments and blank lines
/// skipped and keys other than `epic`, `status` and `vendors` ignored (`pmat
/// spec` reads its own); `epic:` absent reads as `epic: null`. No YAML crate
/// on purpose: three keys, no feature coupling.
pub fn parse_front_matter(text: &str) -> Result<SpecFrontMatter, FrontMatterError> {
    if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
        return Err(FrontMatterError::Absent);
    }

    let mut lines = text.lines();
    lines.next(); // Skip the first `---`

    let mut state = ParseState {
        epic_val: None,
        status_val: None,
        vendors_val: Vec::new(),
        in_vendors_block: false,
    };
    let mut found_end = false;

    for line in lines {
        if line == "---" {
            found_end = true;
            break;
        }
        state.process_line(line)?;
    }

    if !found_end {
        return Err(FrontMatterError::Unterminated);
    }

    let status_str = state.status_val.ok_or(FrontMatterError::MissingStatus)?;
    let status = SpecStatus::parse(&status_str).ok_or(FrontMatterError::BadStatus(status_str))?;
    let epic = parse_epic(state.epic_val.as_deref())?;

    Ok(SpecFrontMatter {
        epic,
        status,
        vendors: state.vendors_val,
    })
}

/// One spec as the caller read it: the path relative to the project root
/// (`docs/specifications/…`) and the file's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecInput {
    pub path: String,
    pub text: String,
}

/// A violation of invariant E (goal-mode.md §4.3) found in a spec.
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
            Self::NoFrontMatter { spec } => format!("NO-FRONT-MATTER {spec}: the file does not begin with a --- front-matter block (goal-mode.md §4.3)"),
            Self::BadFrontMatter { spec, what } => format!("BAD-FRONT-MATTER {spec}: {what}"),
            Self::NoEpic { spec } => format!("NO-EPIC {spec}: epic is null — an active spec names an open issue labelled epic (goal-mode.md §4.3)"),
            Self::EpicAbsent { spec, number } => format!("EPIC-ABSENT {spec}: names #{number}, which is not in the snapshot"),
            Self::EpicClosed { spec, number } => format!("EPIC-CLOSED {spec}: names #{number}, which is closed"),
            Self::NotAnEpic { spec, number } => format!("NOT-AN-EPIC {spec}: names #{number}, which is not labelled epic"),
            Self::SubIssuesUnmeasured { spec, number } => format!("SUB-ISSUES-UNMEASURED {spec}: the snapshot does not carry #{number}'s sub-issue count — not measured is not zero (goal-mode.md doctrine 2)"),
            Self::NoSubIssues { spec, number } => format!("NO-SUB-ISSUES {spec}: #{number} has no sub-issue — a spec's tickets are its epic's sub-issues (goal-mode.md §4.3)"),
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
    let mut parsed = Parsed::default();

    let mut sorted_specs = specs.to_vec();
    sorted_specs.sort_by(|a, b| a.path.cmp(&b.path));

    for spec in &sorted_specs {
        match parse_front_matter(&spec.text) {
            Ok(fm) => {
                if fm.status.on_epic_leg() && fm.epic.is_none() {
                    parsed.findings.push(SpecFinding::NoEpic {
                        spec: spec.path.clone(),
                    });
                }
                parsed.specs.push((spec.path.clone(), fm));
            }
            Err(FrontMatterError::Absent) => {
                parsed.findings.push(SpecFinding::NoFrontMatter {
                    spec: spec.path.clone(),
                });
            }
            Err(e) => {
                parsed.findings.push(SpecFinding::BadFrontMatter {
                    spec: spec.path.clone(),
                    what: e.render(),
                });
            }
        }
    }

    parsed
}

/// The epic leg: for every active spec that names an epic, the first clause
/// that fails — absent, closed, not labelled `epic`, sub-issue count not in
/// the snapshot, no sub-issue — in path order.
pub fn bind_epics(parsed: &Parsed, snapshot: &GithubSnapshot) -> Vec<SpecFinding> {
    let mut findings = Vec::new();

    for (path, fm) in parsed.active() {
        if let Some(number) = fm.epic {
            let issue = snapshot.issues.iter().find(|i| i.number == number);
            match issue {
                None => findings.push(SpecFinding::EpicAbsent {
                    spec: path.clone(),
                    number,
                }),
                Some(issue) => {
                    if issue.state == IssueState::Closed {
                        findings.push(SpecFinding::EpicClosed {
                            spec: path.clone(),
                            number,
                        });
                    } else if !issue.labels.contains(&EPIC_LABEL.to_string()) {
                        findings.push(SpecFinding::NotAnEpic {
                            spec: path.clone(),
                            number,
                        });
                    } else if issue.sub_issues.is_none() {
                        findings.push(SpecFinding::SubIssuesUnmeasured {
                            spec: path.clone(),
                            number,
                        });
                    } else if issue.sub_issues == Some(0) {
                        findings.push(SpecFinding::NoSubIssues {
                            spec: path.clone(),
                            number,
                        });
                    }
                }
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests;
