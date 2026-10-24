//! PRE receipt evaluation: gate-on-head + cargo-mutants + self-review (MT-1, MT-2),
//! path-derived review tier (MT-13), size gate (MT-11).
use serde::{Deserialize, Serialize};

pub const SIZE_LIMIT: usize = 400;
pub const MUTANT_TOOL: &str = "cargo-mutants";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Light,
    Standard,
    Sensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Off,
    Advisory,
    Enforce,
}

impl Mode {
    /// Unset or unrecognised means Off: the switch fails closed to "does nothing".
    pub fn parse(v: Option<&str>) -> Mode {
        match v.map(str::trim) {
            Some("advisory") => Mode::Advisory,
            Some("enforce") => Mode::Enforce,
            _ => Mode::Off,
        }
    }
    pub fn from_env() -> Mode {
        Mode::parse(std::env::var("PMAT_PRE_MODE").ok().as_deref())
    }
}

/// Path table from `.pmat/review-tiers.yaml` (restricted shape: `tier:` then `- "pattern"`).
#[derive(Debug, Default)]
pub struct TierTable {
    light: Vec<String>,
    sensitive: Vec<String>,
}

impl TierTable {
    pub fn parse(src: &str) -> TierTable {
        let mut t = TierTable::default();
        let mut cur = "";
        for line in src.lines().map(str::trim).filter(|l| !l.starts_with('#')) {
            if let Some(k) = line.strip_suffix(':') {
                cur = k;
            } else if let Some(p) = line.strip_prefix("- ") {
                let p = p.trim().trim_matches('"').to_string();
                match cur {
                    "light" => t.light.push(p),
                    "sensitive" => t.sensitive.push(p),
                    _ => {}
                }
            }
        }
        t
    }

    fn hit(pats: &[String], path: &str) -> bool {
        pats.iter().any(|p| match p.strip_prefix('*') {
            Some(ext) => path.ends_with(ext),
            None if p.ends_with('/') => path.starts_with(p.as_str()),
            None => path == p,
        })
    }

    /// Highest tier across paths. Sensitive beats Light even when both match one path.
    pub fn derive(&self, paths: &[&str]) -> Tier {
        let tier_of = |p: &&str| {
            if Self::hit(&self.sensitive, p) {
                Tier::Sensitive
            } else if Self::hit(&self.light, p) {
                Tier::Light
            } else {
                Tier::Standard
            }
        };
        paths.iter().map(tier_of).max().unwrap_or(Tier::Light)
    }
}

/// True only when every changed line of a unified diff is blank or a comment. Mechanical, never the author's word.
pub fn comment_only(diff: &str) -> bool {
    diff.lines()
        .filter(|l| {
            (l.starts_with('+') || l.starts_with('-'))
                && !l.starts_with("+++")
                && !l.starts_with("---")
        })
        .all(|l| {
            let b = l[1..].trim();
            b.is_empty()
                || b.starts_with("//")
                || b.starts_with('#')
                || b.starts_with("/*")
                || b.starts_with('*')
        })
}

/// Waiver = a `PMAT-Size-Waiver: <reason>` trailer with a non-empty reason.
pub fn has_waiver(message: &str) -> bool {
    message.lines().any(|l| {
        l.strip_prefix("PMAT-Size-Waiver:")
            .is_some_and(|r| !r.trim().is_empty())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreReceipt {
    pub head: String,
    pub gate_pass: bool,
    pub mutant_tool: String,
    pub mutants_total: u32,
    pub mutants_killed: u32,
    pub self_review: bool,
}

#[derive(Debug)]
pub struct PrInput<'a> {
    pub head: &'a str,
    pub paths: &'a [&'a str],
    pub diff: &'a str,
    pub changed_lines: usize,
    pub message: &'a str,
    pub receipt: Option<&'a PreReceipt>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Skipped,
    Recorded,
    Refused,
}

#[derive(Debug)]
pub struct Outcome {
    pub tier: Tier,
    pub violations: Vec<String>,
    pub action: Action,
}

pub fn evaluate(table: &TierTable, pr: &PrInput, mode: Mode) -> Outcome {
    let mut tier = table.derive(pr.paths);
    if tier == Tier::Standard && comment_only(pr.diff) && !pr.diff.is_empty() {
        tier = Tier::Light;
    }
    let mut v = Vec::new();
    if pr.changed_lines > SIZE_LIMIT {
        if has_waiver(pr.message) {
            tier = Tier::Sensitive; // MT-11: a waiver forces the Sensitive tier
        } else {
            v.push(format!(
                "MT-11: {} changed lines > {SIZE_LIMIT}, no PMAT-Size-Waiver",
                pr.changed_lines
            ));
        }
    }
    match pr.receipt {
        None => v.push("MT-1: no pre-receipt".into()),
        Some(r) => {
            if r.head != pr.head {
                v.push("MT-1: pre-receipt is for another head".into());
            }
            if !r.gate_pass || !r.self_review {
                v.push("MT-1: gate or self-review not passed".into());
            }
            if r.mutant_tool != MUTANT_TOOL
                || r.mutants_total == 0
                || r.mutants_killed != r.mutants_total
            {
                v.push("MT-2: mutants not tool-generated or not all killed".into());
            }
        }
    }
    let action = match mode {
        Mode::Off => Action::Skipped,
        Mode::Advisory => Action::Recorded,
        Mode::Enforce if v.is_empty() => Action::Recorded,
        Mode::Enforce => Action::Refused,
    };
    Outcome {
        tier,
        violations: v,
        action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = include_str!("../../.pmat/review-tiers.yaml");
    fn good(head: &str) -> PreReceipt {
        PreReceipt {
            head: head.into(),
            gate_pass: true,
            mutant_tool: MUTANT_TOOL.into(),
            mutants_total: 4,
            mutants_killed: 4,
            self_review: true,
        }
    }
    fn run(
        paths: &[&str],
        diff: &str,
        lines: usize,
        msg: &str,
        r: Option<&PreReceipt>,
        mode: Mode,
    ) -> Outcome {
        let t = TierTable::parse(TABLE);
        evaluate(
            &t,
            &PrInput {
                head: "h1",
                paths,
                diff,
                changed_lines: lines,
                message: msg,
                receipt: r,
            },
            mode,
        )
    }

    #[test]
    fn mt13_workflow_is_sensitive_and_highest_wins() {
        let t = TierTable::parse(TABLE);
        assert_eq!(
            t.derive(&["README.md", ".github/workflows/ci.yml"]),
            Tier::Sensitive
        );
        assert_eq!(t.derive(&["docs/a.txt", "README.md"]), Tier::Light);
        assert_eq!(t.derive(&["src/x.rs", "README.md"]), Tier::Standard);
    }

    #[test]
    fn mt13_comment_only_is_mechanical() {
        let mixed = "+// note\n+let x = 1;\n";
        let o = run(
            &["src/x.rs"],
            mixed,
            2,
            "",
            Some(&good("h1")),
            Mode::Advisory,
        );
        assert_eq!(
            o.tier,
            Tier::Standard,
            "code in a 'comment' diff must not be Light"
        );
        let o = run(
            &["src/x.rs"],
            "+// note\n-# old\n",
            2,
            "",
            Some(&good("h1")),
            Mode::Advisory,
        );
        assert_eq!(o.tier, Tier::Light);
    }

    #[test]
    fn mt11_size_gate_and_waiver_forces_sensitive() {
        let r = good("h1");
        assert_eq!(
            run(&["src/x.rs"], "+a\n", 600, "", Some(&r), Mode::Enforce).action,
            Action::Refused
        );
        let o = run(
            &["docs/a.md"],
            "+a\n",
            600,
            "x\n\nPMAT-Size-Waiver: big move",
            Some(&r),
            Mode::Enforce,
        );
        assert_eq!((o.action, o.tier), (Action::Recorded, Tier::Sensitive));
        assert!(
            !has_waiver("PMAT-Size-Waiver:   "),
            "empty reason is no waiver"
        );
    }

    #[test]
    fn mt1_mt2_receipt_must_match_head_and_kill_all_tool_mutants() {
        let p = ["src/x.rs"];
        assert!(run(&p, "+a\n", 5, "", None, Mode::Enforce).violations[0].starts_with("MT-1"));
        assert_eq!(
            run(&p, "+a\n", 5, "", Some(&good("OLD")), Mode::Enforce).action,
            Action::Refused
        );
        let mut hand = good("h1");
        hand.mutant_tool = "hand".into();
        assert_eq!(
            run(&p, "+a\n", 5, "", Some(&hand), Mode::Enforce).action,
            Action::Refused
        );
        let mut survivor = good("h1");
        survivor.mutants_killed = 3;
        assert_eq!(
            run(&p, "+a\n", 5, "", Some(&survivor), Mode::Enforce).action,
            Action::Refused
        );
        assert_eq!(
            run(&p, "+a\n", 5, "", Some(&good("h1")), Mode::Enforce).action,
            Action::Recorded
        );
    }

    #[test]
    fn switch_off_by_default_advisory_never_refuses() {
        assert_eq!(Mode::parse(None), Mode::Off);
        assert_eq!(Mode::parse(Some("garbage")), Mode::Off);
        let bad = run(&["src/x.rs"], "+a\n", 900, "", None, Mode::Off);
        assert_eq!(bad.action, Action::Skipped);
        let adv = run(&["src/x.rs"], "+a\n", 900, "", None, Mode::Advisory);
        assert_eq!((adv.action, adv.violations.len()), (Action::Recorded, 2));
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;

    #[test]
    fn mode_parse_recognises_each_value_and_trims() {
        assert_eq!(Mode::parse(Some("advisory")), Mode::Advisory);
        assert_eq!(Mode::parse(Some(" enforce ")), Mode::Enforce);
        assert_eq!(Mode::parse(Some("")), Mode::Off);
    }

    fn receipt(gate_pass: bool, self_review: bool) -> PreReceipt {
        PreReceipt {
            head: "h".into(),
            gate_pass,
            mutant_tool: MUTANT_TOOL.into(),
            mutants_total: 3,
            mutants_killed: 3,
            self_review,
        }
    }

    fn violations(lines: usize, r: &PreReceipt) -> Vec<String> {
        let pr = PrInput {
            head: "h",
            paths: &["src/x.rs"],
            diff: "+a\n",
            changed_lines: lines,
            message: "",
            receipt: Some(r),
        };
        evaluate(&TierTable::default(), &pr, Mode::Advisory).violations
    }

    #[test]
    fn size_limit_is_inclusive() {
        assert_eq!(TierTable::default().derive(&[]), Tier::Light);
        let ok = receipt(true, true);
        assert!(violations(SIZE_LIMIT, &ok).is_empty());
        assert_eq!(violations(SIZE_LIMIT + 1, &ok).len(), 1);
    }

    #[test]
    fn either_gate_or_self_review_missing_is_a_violation() {
        assert_eq!(violations(1, &receipt(false, true)).len(), 1);
        assert_eq!(violations(1, &receipt(true, false)).len(), 1);
        assert!(violations(1, &receipt(true, true)).is_empty());
    }
}
