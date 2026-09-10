//! goal-mode.md §6 (E.1): the spec review artifact, and CB-2111's judgement of it.
//!
//! The gate reads a FILE, `docs/audits/spec-<slug>-review.json`, and never invokes
//! a model (§6.1). It checks, offline: the file parses; `spec` names the spec it
//! reviews; `spec_sha256` equals the spec's hash NOW (the §7 falsifier: append one
//! space to the spec); `plan` is present with a non-empty `sha256`; every required
//! role is present (the five, plus `vendor:<name>` for each vendor in the spec's
//! front-matter); every lane is PASS; `partial: true` is red; and an unrecognised
//! role is an error, never an extra lane (§6.1).
//!
//! What it buys, exactly (§6.2): a skipped review becomes an auditable lie somebody
//! wrote down instead of an omission nobody can see. It is not evidence that the
//! review happened, or that its lanes were independent minds.

use serde::Deserialize;

/// Where review artifacts live.
pub const REVIEW_DIR: &str = "docs/audits";

/// The five roles every active spec's review must carry (§6.1).
pub const BASE_ROLES: [&str; 5] = ["quality", "architecture", "security", "crux", "adversarial"];

/// `docs/audits/spec-<slug>-review.json` (§6.1).
#[derive(Debug, Clone, Deserialize)]
pub struct ReviewArtifact {
    pub spec: String,
    pub spec_sha256: String,
    #[serde(default)]
    pub plan: Option<Plan>,
    #[serde(default)]
    pub lanes: Vec<Lane>,
    #[serde(default)]
    pub agreed: bool,
    #[serde(default)]
    pub partial: bool,
}

/// The plan the review judged.
#[derive(Debug, Clone, Deserialize)]
pub struct Plan {
    #[serde(default)]
    pub tool: String,
    #[serde(default, rename = "ref")]
    pub reference: String,
    #[serde(default)]
    pub sha256: String,
}

/// One reviewer's verdict.
#[derive(Debug, Clone, Deserialize)]
pub struct Lane {
    pub role: String,
    #[serde(default)]
    pub executor: String,
    pub verdict: String,
    #[serde(default)]
    pub summary: String,
}

/// Why an active spec's review does not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewFinding {
    NoReview {
        spec: String,
        artifact: String,
    },
    BadReview {
        spec: String,
        artifact: String,
        reason: String,
    },
    SpecMismatch {
        spec: String,
        named: String,
    },
    StaleReview {
        spec: String,
        recorded: String,
        actual: String,
    },
    NoPlan {
        spec: String,
    },
    UnknownRole {
        spec: String,
        role: String,
    },
    MissingRole {
        spec: String,
        role: String,
    },
    LaneNotPass {
        spec: String,
        role: String,
        verdict: String,
    },
    Partial {
        spec: String,
    },
}

impl ReviewFinding {
    /// The class name a verdict counts by.
    pub fn class(&self) -> &'static str {
        match self {
            ReviewFinding::NoReview { .. } => "NO-REVIEW",
            ReviewFinding::BadReview { .. } => "BAD-REVIEW",
            ReviewFinding::SpecMismatch { .. } => "SPEC-MISMATCH",
            ReviewFinding::StaleReview { .. } => "STALE-REVIEW",
            ReviewFinding::NoPlan { .. } => "NO-PLAN",
            ReviewFinding::UnknownRole { .. } => "UNKNOWN-ROLE",
            ReviewFinding::MissingRole { .. } => "MISSING-ROLE",
            ReviewFinding::LaneNotPass { .. } => "LANE-NOT-PASS",
            ReviewFinding::Partial { .. } => "PARTIAL",
        }
    }

    /// The spec the finding is about.
    pub fn spec(&self) -> &str {
        match self {
            ReviewFinding::NoReview { spec, .. }
            | ReviewFinding::BadReview { spec, .. }
            | ReviewFinding::SpecMismatch { spec, .. }
            | ReviewFinding::StaleReview { spec, .. }
            | ReviewFinding::NoPlan { spec }
            | ReviewFinding::UnknownRole { spec, .. }
            | ReviewFinding::MissingRole { spec, .. }
            | ReviewFinding::LaneNotPass { spec, .. }
            | ReviewFinding::Partial { spec } => spec,
        }
    }

    /// One line a reader can act on.
    pub fn render(&self) -> String {
        match self {
            ReviewFinding::NoReview { spec, artifact } => format!("NO-REVIEW {spec}: no {artifact}"),
            ReviewFinding::BadReview { spec, artifact, reason } => format!("BAD-REVIEW {spec}: {artifact} does not parse ({reason})"),
            ReviewFinding::SpecMismatch { spec, named } => format!("SPEC-MISMATCH {spec}: the review names {named}"),
            ReviewFinding::StaleReview { spec, recorded, actual } => format!("STALE-REVIEW {spec}: reviewed at {recorded}, the file is {actual} now"),
            ReviewFinding::NoPlan { spec } => format!("NO-PLAN {spec}: the review carries no plan sha256"),
            ReviewFinding::UnknownRole { spec, role } => format!("UNKNOWN-ROLE {spec}: `{role}` is not a review role (quality, architecture, security, crux, adversarial, vendor:<name>)"),
            ReviewFinding::MissingRole { spec, role } => format!("MISSING-ROLE {spec}: no `{role}` lane"),
            ReviewFinding::LaneNotPass { spec, role, verdict } => format!("LANE-NOT-PASS {spec}: the `{role}` lane says {verdict}"),
            ReviewFinding::Partial { spec } => format!("PARTIAL {spec}: the review is marked partial"),
        }
    }
}

/// `docs/specifications/components/cli-api.md` → `components-cli-api`.
pub fn slug(_spec_path: &str) -> String {
    String::new()
}

/// Where the review of `spec_path` lives.
pub fn artifact_path(spec_path: &str) -> String {
    format!("{REVIEW_DIR}/spec-{}-review.json", slug(spec_path))
}

/// The hex sha256 of `text`: what `spec_sha256` records.
pub fn sha256_hex(text: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The five base roles, then `vendor:<name>` for each front-matter vendor.
pub fn required_roles(_vendors: &[String]) -> Vec<String> {
    Vec::new()
}

/// Is `role` in the closed set (§6.1)?
pub fn is_known_role(_role: &str) -> bool {
    true
}

/// CB-2111's judgement on one active spec: `artifact` is the review file's text,
/// or `None` when there is no file.
pub fn judge(
    _spec_path: &str,
    _spec_text: &str,
    _vendors: &[String],
    _artifact: Option<&str>,
) -> Vec<ReviewFinding> {
    Vec::new()
}

#[cfg(test)]
mod tests;
