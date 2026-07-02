//! MACS F3 (Component 32, implementing C31): structured chain-of-thought.
//!
//! Sub-spec: `docs/specifications/components/modern-agentic-coding-support.md`
//! Contract: `contracts/macs-cot-v1.yaml`
//!
//! The v2 step shape is `{id, assumption, implication, evidence_method,
//! discharged_by}`. Assumptions must be *discharged* — by a prior step's
//! implication, a bound equation, an environment fact, or a documented
//! axiom — and the discharge graph must be a DAG rooted in evidence
//! (CB-1640). Each step then derives one proof obligation and one
//! falsifiable claim with verbatim fields (CB-1658): reasoning becomes an
//! audit *ledger*, not an audit *log*.
//!
//! `check_chain` is pure (no I/O) — see the `checker is pure` invariant on
//! the MACS-008 card.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// What discharges a step's assumption (MACS-007).
///
/// Wire forms accepted:
/// - `"CoT-3"` — a prior step's implication
/// - `"macs-cot-v1#chain_integrity"` — a bound contract equation
/// - `"E5"` — a spec environment fact
/// - `{"Axiomatic": {"reason": "..."}}` — documented axiom (CB-1648's shape)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DischargeRef {
    /// A prior CoT step id, e.g. "CoT-1"
    Step(String),
    /// A bound provable-contract equation, e.g. "macs-cot-v1#chain_integrity"
    Equation {
        /// Contract name (left of `#`)
        contract: String,
        /// Equation key (right of `#`)
        equation: String,
    },
    /// A spec environment fact, e.g. "E5"
    EnvFact(String),
    /// Documented axiomatic discharge (reason must be non-empty prose)
    Axiomatic {
        /// Why this assumption needs no upstream discharge
        reason: String,
    },
}

impl DischargeRef {
    /// Parse the compact string form. Returns None for unrecognized shapes.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if let Some((contract, equation)) = s.split_once('#') {
            if !contract.is_empty() && !equation.is_empty() {
                return Some(Self::Equation {
                    contract: contract.to_string(),
                    equation: equation.to_string(),
                });
            }
            return None;
        }
        if s.starts_with("CoT-") && s.len() > 4 {
            return Some(Self::Step(s.to_string()));
        }
        if let Some(rest) = s.strip_prefix('E') {
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                return Some(Self::EnvFact(s.to_string()));
            }
        }
        None
    }

    /// Parse from a JSON value: compact string or CB-1648's tagged object.
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Self::parse(s),
            Value::Object(map) => {
                let axiomatic = map.get("Axiomatic")?;
                let reason = axiomatic
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                Some(Self::Axiomatic { reason })
            }
            _ => None,
        }
    }
}

impl Serialize for DischargeRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Step(id) => serializer.serialize_str(id),
            Self::EnvFact(fact) => serializer.serialize_str(fact),
            Self::Equation { contract, equation } => {
                serializer.serialize_str(&format!("{contract}#{equation}"))
            }
            Self::Axiomatic { reason } => {
                use serde::ser::SerializeMap;
                let mut outer = serializer.serialize_map(Some(1))?;
                outer.serialize_entry("Axiomatic", &serde_json::json!({ "reason": reason }))?;
                outer.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for DischargeRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        Self::from_value(&value)
            .ok_or_else(|| serde::de::Error::custom("invalid discharge ref: expected \"CoT-<id>\", \"<contract>#<equation>\", \"E<n>\", or {\"Axiomatic\": {...}}"))
    }
}

/// Normalized view of one CoT step used by the checker and deriver.
/// Both wire shapes normalize into this: MACS Appendix-A strings and the
/// C31 object form (`assumption: {text, references, expr}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CotStepView {
    /// Step id ("CoT-1"); legacy prose steps get "CoT-<n>" from `step`
    pub id: String,
    /// Assumption text (input to the reasoning)
    pub assumption: String,
    /// Explicit references extracted from an object-form assumption
    #[serde(default)]
    pub assumption_references: Vec<String>,
    /// Implication text (output of the reasoning)
    pub implication: String,
    /// How to falsify the implication
    pub evidence_method: String,
    /// Explicit discharge, if declared
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discharged_by: Option<DischargeRef>,
    /// False for steps migrated from the legacy `{step, question, answer}`
    /// prose shape — they are annotated L0 evidence, never dropped (MACS-007).
    pub structured: bool,
}

/// Extract text + references from either a plain string or the C31 object
/// form `{text, references[], expr}`.
fn text_and_refs(value: Option<&Value>) -> (String, Vec<String>) {
    match value {
        Some(Value::String(s)) => (s.clone(), Vec::new()),
        Some(Value::Object(map)) => {
            let text = map
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let references = map
                .get("references")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            (text, references)
        }
        _ => (String::new(), Vec::new()),
    }
}

/// Normalize a raw `chain_of_thought` entry into a checkable view.
/// Structured (v2) steps carry assumption/implication/evidence_method;
/// legacy prose steps migrate with `structured: false` (annotated, not
/// dropped — `cot::legacy_prose_migrates_annotated_L0`).
pub fn parse_step(index: usize, step: &Value) -> CotStepView {
    let structured = step.get("assumption").is_some()
        || step.get("implication").is_some()
        || step.get("evidence_method").is_some()
        || step.get("discharged_by").is_some();

    let id = step
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            step.get("step")
                .and_then(Value::as_u64)
                .map(|n| format!("CoT-{n}"))
        })
        .unwrap_or_else(|| format!("CoT-{}", index + 1));

    if !structured {
        // Legacy prose migration: question becomes the assumption text,
        // answer the implication; evidence_method is annotated as migrated.
        let question = step
            .get("question")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let answer = step
            .get("answer")
            .and_then(Value::as_str)
            .unwrap_or_default();
        return CotStepView {
            id,
            assumption: question.to_string(),
            assumption_references: Vec::new(),
            implication: answer.to_string(),
            evidence_method: "MIGRATED-L0: prose step predates the v2 schema".to_string(),
            discharged_by: None,
            structured: false,
        };
    }

    let (assumption, assumption_references) = text_and_refs(step.get("assumption"));
    let (implication, _) = text_and_refs(step.get("implication"));
    let evidence_method = step
        .get("evidence_method")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let discharged_by = step.get("discharged_by").and_then(DischargeRef::from_value);

    CotStepView {
        id,
        assumption,
        assumption_references,
        implication,
        evidence_method,
        discharged_by,
        structured: true,
    }
}

/// Parse a contract's `chain_of_thought` array into checkable views.
pub fn parse_steps(contract: &Value) -> Vec<CotStepView> {
    contract
        .get("chain_of_thought")
        .and_then(Value::as_array)
        .map(|steps| {
            steps
                .iter()
                .enumerate()
                .map(|(i, s)| parse_step(i, s))
                .collect()
        })
        .unwrap_or_default()
}

/// A CB-1640 chain-integrity violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cb1640Violation {
    /// Offending step id
    pub step_id: String,
    /// Violation class: duplicate-id | unresolved-ref | cycle | undischarged
    pub kind: String,
    /// Human-readable detail
    pub detail: String,
}

impl std::fmt::Display for Cb1640Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]: {}", self.step_id, self.kind, self.detail)
    }
}

/// Inline references in assumption text: `CoT-<id>`, `E<n>`, `<name>#<eq>`.
fn inline_refs(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for word in text.split(|c: char| !(c.is_alphanumeric() || "#-_.".contains(c))) {
        let token = word.trim_matches(|c: char| c == '.' || c == '-');
        if token.starts_with("CoT-") && token.len() > 4 {
            refs.push(token.to_string());
        } else if token.len() >= 2
            && token.starts_with('E')
            && token[1..].chars().all(|c| c.is_ascii_digit())
        {
            refs.push(token.to_string());
        }
    }
    refs
}

/// CB-1640 chain-integrity checker (MACS-008): every assumption must be
/// discharged, the discharge graph must be a DAG, and roots must be
/// evidence (E-facts, bound equations, documented axioms, or a non-empty
/// falsification `evidence_method`). Pure — no I/O.
pub fn check_chain(steps: &[CotStepView]) -> Vec<Cb1640Violation> {
    let mut violations = Vec::new();
    let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();

    // Duplicate ids break resolution.
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        if !seen.insert(*id) {
            violations.push(Cb1640Violation {
                step_id: (*id).to_string(),
                kind: "duplicate-id".to_string(),
                detail: "step id appears more than once".to_string(),
            });
        }
    }

    // Collect step->step edges and per-step discharge sources.
    let index_of = |id: &str| ids.iter().position(|s| *s == id);
    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); steps.len()];
    for (i, step) in steps.iter().enumerate() {
        let mut sources = 0usize;
        let mut refs: Vec<String> = Vec::new();
        match &step.discharged_by {
            Some(DischargeRef::Step(target)) => refs.push(target.clone()),
            Some(DischargeRef::Axiomatic { reason }) => {
                if reason.trim().is_empty() {
                    violations.push(Cb1640Violation {
                        step_id: step.id.clone(),
                        kind: "undischarged".to_string(),
                        detail: "Axiomatic discharge requires a non-empty reason".to_string(),
                    });
                } else {
                    sources += 1; // documented axiom is a root
                }
            }
            Some(DischargeRef::Equation { .. }) | Some(DischargeRef::EnvFact(_)) => {
                sources += 1; // terminal evidence roots
            }
            None => {}
        }
        for r in step
            .assumption_references
            .iter()
            .cloned()
            .chain(inline_refs(&step.assumption))
        {
            match DischargeRef::parse(&r) {
                Some(DischargeRef::Step(target)) => refs.push(target),
                Some(_) => sources += 1, // E-fact / equation reference in text
                None => {}
            }
        }

        for target in refs {
            match index_of(&target) {
                Some(j) if j == i => violations.push(Cb1640Violation {
                    step_id: step.id.clone(),
                    kind: "cycle".to_string(),
                    detail: "step discharges itself".to_string(),
                }),
                Some(j) => {
                    edges[i].push(j);
                    sources += 1;
                }
                None => violations.push(Cb1640Violation {
                    step_id: step.id.clone(),
                    kind: "unresolved-ref".to_string(),
                    detail: format!("reference '{target}' does not name a step in this chain"),
                }),
            }
        }

        // Evidence-rooted step: a non-empty falsification method anchors a
        // step with no upstream reference (the §3.1 CoT-1 pattern).
        if sources == 0 && step.evidence_method.trim().is_empty() {
            violations.push(Cb1640Violation {
                step_id: step.id.clone(),
                kind: "undischarged".to_string(),
                detail: "no discharge source and no evidence_method".to_string(),
            });
        }
    }

    // Cycle detection over step->step edges (iterative DFS, 3-color).
    let mut color = vec![0u8; steps.len()]; // 0 white, 1 grey, 2 black
    for start in 0..steps.len() {
        if color[start] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        color[start] = 1;
        while let Some(&mut (node, ref mut edge_idx)) = stack.last_mut() {
            if *edge_idx < edges[node].len() {
                let next = edges[node][*edge_idx];
                *edge_idx += 1;
                match color[next] {
                    0 => {
                        color[next] = 1;
                        stack.push((next, 0));
                    }
                    1 => violations.push(Cb1640Violation {
                        step_id: steps[next].id.clone(),
                        kind: "cycle".to_string(),
                        detail: format!("discharge graph cycles through '{}'", steps[node].id),
                    }),
                    _ => {}
                }
            } else {
                color[node] = 2;
                stack.pop();
            }
        }
    }

    violations
}

/// Derivation output for one step (MACS-009): one proof obligation and one
/// falsifiable claim, fields copied verbatim — no paraphrase drift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Derived {
    /// "PO-<step id>"
    pub obligation_id: String,
    /// The step's implication, verbatim
    pub statement: String,
    /// The step's evidence_method, verbatim (claim.method)
    pub method: String,
    /// The step id this derivation came from
    pub step_id: String,
}

/// Derive one obligation + claim per step (contracts/macs-cot-v1.yaml#
/// derivation_complete: |claims| = |steps| and |obligations| = |steps|).
pub fn derive(step: &CotStepView) -> Derived {
    Derived {
        obligation_id: format!("PO-{}", step.id),
        statement: step.implication.clone(),
        method: step.evidence_method.clone(),
        step_id: step.id.clone(),
    }
}

/// Render the derivation artifact for a ticket. Deterministic: steps are
/// emitted in input order, output depends only on the inputs
/// (`derive::stable_output_two_runs`). With `emit_clauses` the optional
/// C30 require/ensure clause section is included (gated by flag on the
/// CLI — `derive::optional_clause_codegen_gated_by_flag`).
pub fn render_derivation(ticket: &str, steps: &[CotStepView], emit_clauses: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Auto-generated by `pmat work cot derive` from .pmat-work/{ticket}/contract.json\n"
    ));
    out.push_str(&format!(
        "# MACS-009 (Component 32) — do not edit by hand\nticket: \"{ticket}\"\n"
    ));
    // metadata.references keeps the artifact well-formed under `pv lint`
    // (contracts/ is scanned recursively; every doc there needs a source ref).
    out.push_str("metadata:\n");
    out.push_str("  version: \"1.0.0\"\n");
    out.push_str("  kind: schema\n");
    out.push_str(&format!(
        "  description: \"Auto-derived proof obligations + falsifiable claims for {ticket}\"\n"
    ));
    out.push_str("  references:\n");
    out.push_str(&format!("    - \".pmat-work/{ticket}/contract.json\"\n"));
    out.push_str("proof_obligations:\n");
    for step in steps {
        let d = derive(step);
        out.push_str(&format!("- id: {}\n", yaml_quote(&d.obligation_id)));
        out.push_str(&format!("  statement: {}\n", yaml_quote(&d.statement)));
        out.push_str(&format!("  evidence_method: {}\n", yaml_quote(&d.method)));
    }
    out.push_str("falsifiable_claims:\n");
    for step in steps {
        let d = derive(step);
        out.push_str(&format!("- hypothesis: {}\n", yaml_quote(&d.statement)));
        out.push_str(&format!("  method: {}\n", yaml_quote(&d.method)));
        out.push_str(&format!("  from_step: {}\n", yaml_quote(&d.step_id)));
    }
    if emit_clauses {
        out.push_str("clauses:\n");
        for step in steps {
            out.push_str(&format!(
                "- kind: ensure\n  id: {}\n",
                yaml_quote(&format!("ensure.{}", step.id))
            ));
            out.push_str(&format!(
                "  description: {}\n",
                yaml_quote(&step.implication)
            ));
        }
    }
    out
}

fn yaml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Canonical SHA-256 of a contract's `chain_of_thought` (CB-1646's digest).
/// Single source of truth — the comply check delegates here.
pub fn canonical_cot_sha(contract: &Value) -> String {
    let cot = contract
        .get("chain_of_thought")
        .cloned()
        .unwrap_or(Value::Null);
    let mut buf = String::new();
    canonicalize_value(&cot, &mut buf);
    let mut hasher = Sha256::new();
    hasher.update(buf.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Emit `v` as canonical JSON: object keys sorted lexicographically, no
/// insignificant whitespace (matches the CB-1646 checker's algorithm).
pub fn canonicalize_value(v: &Value, out: &mut String) {
    match v {
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonicalize_value(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String((*k).clone()).to_string());
                out.push(':');
                canonicalize_value(&map[*k], out);
            }
            out.push('}');
        }
        scalar => out.push_str(&scalar.to_string()),
    }
}

#[cfg(test)]
#[path = "work_cot_tests.rs"]
mod tests;
