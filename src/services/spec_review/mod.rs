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

pub mod record;
pub use record::{record, RecordRefusal, Recorded};

/// Where review artifacts live.
pub const REVIEW_DIR: &str = "docs/audits";

/// The five roles every active spec's review must carry (§6.1).
pub const BASE_ROLES: [&str; 5] = ["quality", "architecture", "security", "crux", "adversarial"];

/// `docs/audits/spec-<slug>-review.json` (§6.1).
#[derive(Debug, Clone, Deserialize)]
pub struct ReviewArtifact {
    pub spec: String,
    pub spec_sha256: String,
    /// Optional in the parse so its absence is `NO-PLAN`, never `BAD-REVIEW`.
    #[serde(default)]
    pub plan: Option<Plan>,
    pub lanes: Vec<Lane>,
    /// Recorded, not judged: §6.2 does not list it, and every lane PASS is
    /// agreement by construction.
    #[serde(default)]
    pub agreed: bool,
    /// Required: a misspelt `partial` must not default a partial review to
    /// complete.
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
pub fn slug(spec_path: &str) -> String {
    let under = spec_path
        .strip_prefix(crate::services::spec_epic::SPECS_DIR)
        .and_then(|rest| rest.strip_prefix('/'))
        .unwrap_or(spec_path);
    under.strip_suffix(".md").unwrap_or(under).replace('/', "-")
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
pub fn required_roles(vendors: &[String]) -> Vec<String> {
    let mut roles: Vec<String> = BASE_ROLES.iter().map(|r| (*r).to_string()).collect();
    for vendor in vendors {
        let role = format!("vendor:{vendor}");
        if !roles.contains(&role) {
            roles.push(role);
        }
    }
    roles
}

/// Is `role` in the closed set (§6.1)?
pub fn is_known_role(role: &str) -> bool {
    BASE_ROLES.contains(&role)
        || role
            .strip_prefix("vendor:")
            .is_some_and(|name| !name.is_empty() && !name.chars().any(char::is_whitespace))
}

/// CB-2111's judgement on one active spec: `artifact` is the review file's text,
/// or `None` when there is no file.
pub fn judge(
    spec_path: &str,
    spec_text: &str,
    vendors: &[String],
    artifact: Option<&str>,
) -> Vec<ReviewFinding> {
    let spec = spec_path.to_string();
    let Some(text) = artifact else {
        return vec![ReviewFinding::NoReview {
            spec,
            artifact: artifact_path(spec_path),
        }];
    };
    let review: ReviewArtifact = match serde_json::from_str(text) {
        Ok(review) => review,
        Err(e) => {
            return vec![ReviewFinding::BadReview {
                spec,
                artifact: artifact_path(spec_path),
                reason: e.to_string(),
            }];
        }
    };
    let mut findings = Vec::new();
    if review.spec != spec_path {
        findings.push(ReviewFinding::SpecMismatch {
            spec: spec.clone(),
            named: review.spec.clone(),
        });
    }
    let actual = sha256_hex(spec_text);
    if review.spec_sha256 != actual {
        findings.push(ReviewFinding::StaleReview {
            spec: spec.clone(),
            recorded: review.spec_sha256.clone(),
            actual,
        });
    }
    if review
        .plan
        .as_ref()
        .is_none_or(|plan| plan.sha256.trim().is_empty())
    {
        findings.push(ReviewFinding::NoPlan { spec: spec.clone() });
    }
    // An unrecognised role is reported once, as itself: its verdict is not a
    // lane's verdict, because it is not a lane (§6.1).
    for lane in &review.lanes {
        if !is_known_role(&lane.role) {
            findings.push(ReviewFinding::UnknownRole {
                spec: spec.clone(),
                role: lane.role.clone(),
            });
        } else if lane.verdict != "PASS" {
            findings.push(ReviewFinding::LaneNotPass {
                spec: spec.clone(),
                role: lane.role.clone(),
                verdict: lane.verdict.clone(),
            });
        }
    }
    for role in required_roles(vendors) {
        if !review.lanes.iter().any(|lane| lane.role == role) {
            findings.push(ReviewFinding::MissingRole {
                spec: spec.clone(),
                role,
            });
        }
    }
    if review.partial {
        findings.push(ReviewFinding::Partial { spec });
    }
    findings
}

#[cfg(test)]
mod tests;
