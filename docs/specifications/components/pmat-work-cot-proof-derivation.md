# Work Chain-of-Thought Proof Derivation

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 31

## Problem

`WorkContract { chain_of_thought: Vec<ChainOfThoughtStep>, ... }` today is
**write-only**. No code path reads it, validates it, or turns it into
evidence. Grep confirms: only writers exist in
`src/cli/handlers/work_contract_core.rs`; no consumer parses the string
content beyond displaying it in `pmat work status`.

Concrete evidence:
- `ChainOfThoughtStep { summary: String, rationale: String, timestamp: DateTime }` — prose only.
- No connection between a CoT step and any provable-contracts `proof_obligation`.
- An agent-generated ticket can write "Step 1: I assume the input is sorted"
  without that assumption ever becoming a `require` clause or a YAML
  precondition.
- Post-hoc auditors cannot trace which CoT step produced which
  falsification claim.

The CoT is an audit ledger that never got plumbed into the formal side. In
LLM-generated work (Component 10), this is a hallucination magnet:
assumptions silently inflate without ever being discharged.

## Goal

Structure each `ChainOfThoughtStep` so it exposes:

- an **assumption** it relies on (input to the reasoning),
- an **implication** it produces (output of the reasoning),
- an **evidence_method** describing how that implication could be
  falsified.

Then auto-derive:

- One `proof_obligation` in the ticket's generated YAML per step (Component 25 Phase 1),
- One `FalsifiableClaim` per step (Component 29 roster),
- Optionally one `require` or `ensure` clause per step (Component 30 codegen).

Chain integrity: step N's assumption must be discharged by step N-1's
implication or by a bound contract equation. Unmatched assumptions
surface as CB-1640 violations.

## Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfThoughtStep {
    pub id: String,                 // e.g. "CoT-1"
    pub summary: String,            // one-line human-readable
    pub assumption: Assumption,     // NEW — structured input
    pub implication: Implication,   // NEW — structured output
    pub evidence_method: EvidenceMethod,  // NEW — how to falsify
    pub discharged_by: DischargeSource, // NEW — which prior step or binding
    pub timestamp: DateTime<Utc>,
    pub author: Author,             // NEW — human | agent { model, run_id }
}

pub struct Assumption {
    pub predicate: String,          // "input array is sorted"
    pub expr: Option<String>,       // optional Rust expression
    pub references: Vec<String>,    // prior step IDs or binding equation names
}

pub struct Implication {
    pub predicate: String,          // "binary search terminates in O(log n)"
    pub expr: Option<String>,
}

pub enum EvidenceMethod {
    ExistingTest { path: PathBuf, name: String },
    NewFalsificationClaim { claim_id: String },
    BoundEquationPostcondition { equation: String },
    ReviewOnly { reviewer_sha: String },
}

pub enum DischargeSource {
    PriorStep { id: String },                 // chain link
    BoundEquation { contract: String, equation: String },
    ExternalArtifact { url: String, sha: String },  // e.g. arXiv paper pinned by SHA
    Axiomatic { reason: String },              // leaf assumption, must have reason
}
```

Each field is optional only at the *lowest* authoring levels; an L3+
ticket cannot have `Assumption::expr = None` (CB-1643).

## Derivation Rules

Given N CoT steps with structured fields, emit:

1. **Proof obligations** into `contracts/work/<ID>.yaml`:
   ```yaml
   proof_obligations:
     - id: CoT-1-obligation
       statement: "binary search terminates in O(log n)"
       depends_on: [sorted_input]
       derived_from: CoT-1
   ```

2. **Falsifiable claims** into `contract.json` (Component 29):
   - When `evidence_method = NewFalsificationClaim { claim_id }`, auto-create
     the claim with source `InheritedFromCoT { step_id }`.
   - When `evidence_method = ExistingTest { path, name }`, attach a
     `FalsificationMethod::TestExecution { path, name }` entry.

3. **Require/ensure clauses** (Component 30):
   - `Assumption.expr` → `require` clause.
   - `Implication.expr` → `ensure` clause.
   - Clauses carry `source: ClauseSource::InheritedFromCoT { step_id }`.

Derivation is deterministic: regenerating from the same CoT produces
byte-identical output (enables CB-1646 SHA check).

## Chain Integrity Rule

Step N's `Assumption.references` must resolve to either:
- an earlier step's `Implication` (matched by predicate similarity ≥ 0.85,
  or identical `expr`),
- a bound equation's precondition,
- an axiomatic discharge with explicit `reason`.

Unmatched assumptions raise CB-1640. The reasoning process that the
ticket author claims is thus checked for coherence, not just displayed.

Predicate similarity: TF-IDF cosine over the
[semantic-search.md](semantic-search.md) vocabulary; falls back to exact
string match when vocabulary is missing. This is intentionally forgiving
for prose while strict for `expr` fields.

## CB-1640..1649: CoT Enforcement

| Check | Level | Enforces | Catches |
|-------|-------|----------|---------|
| CB-1640 | L3 | Every `Assumption.references` resolves to a prior step, binding, or `Axiomatic` | Unmatched assumptions (typical LLM failure) |
| CB-1641 | L3 | Each step has an `evidence_method` | Prose-only reasoning passing as proof |
| CB-1642 | L1 | `evidence_method = ExistingTest { path, name }` points at a real test | Fabricated test references |
| CB-1643 | L3 | L3+ tickets: every step has `Assumption.expr` or `Implication.expr` | Undischarged expressions |
| CB-1644 | L1 | `author = Agent { model, run_id }` runs are replayable — `run_id` present in agent audit log | Unverifiable LLM authorship |
| CB-1645 | L3 | Derived proof obligations present in `contracts/work/<ID>.yaml` | Skipped YAML derivation |
| CB-1646 | L1 | CoT derivation SHA matches last regenerate | Hand-edited generated obligations |
| CB-1647 | L3 | No orphan CoT steps — each chains to a final conclusion (DAG rooted at `implication = ticket.goal`) | Decorative CoT not contributing |
| CB-1648 | L4 | For L4 tickets, every `Axiomatic` discharge is either a bound equation invariant or a documented lemma | Unchecked axioms in formally verified work |
| CB-1649 | L5 | For L5 tickets, every step maps to a Lean theorem lemma | CoT must back the Lean proof |

## CLI Surface

### `pmat work cot add <ID> --summary <text> [--assumption ...] [--evidence ...]`

Appends a structured step. Interactive mode prompts for each field. Rejects
input violating CB-1640 at save time (early feedback).

```bash
pmat work cot add PMAT-530 \
  --summary "Rope encoding preserves vector norms" \
  --assumption "input is finite" --assumption-expr "x.is_finite()" \
  --implication "output preserves L2 norm" --implication-expr "(result - x).norm() < eps" \
  --evidence falsification:rope_norm_preservation_test \
  --references CoT-2,rope-kernel-v1/rope
```

### `pmat work cot derive <ID>`

Walks the structured CoT and (re)writes derived artifacts:
`contracts/work/<ID>.yaml` proof obligations, ticket falsification roster
entries, clauses. Idempotent — SHA-pinned per CB-1646.

### `pmat work cot audit <ID>`

Renders the CoT as a DAG, shows discharge source for each assumption,
flags CB-1640..1649 violations. Used by reviewers and by pre-commit.

### `pmat work cot replay <ID> --run-id <id>`

For agent-authored steps, fetches the agent run log (Component 10 audit
hook) and re-emits the same CoT. Produces diff against the persisted CoT;
nonzero diff = CB-1644 violation (agent nondeterminism on replay).

## Storage

```
.pmat-work/
└── PMAT-530/
    ├── contract.json              # cot: [ ChainOfThoughtStep, ... ]
    ├── cot-digest.json            # derivation SHA + step count
    ├── cot-dag.dot                # Graphviz DAG for review
    └── agent-runs/
        └── <run_id>.json          # agent audit for replay (CB-1644)
```

`cot-dag.dot` is regenerated on every `cot derive`; reviewers can render it
to inspect reasoning structure.

## Interaction with Other Specs

- [Component 14 work-management.md](work-management.md)
  §Chain-of-Thought Audit Trail is the current informal definition this
  spec formalizes.
- [Component 10 agent-integration.md](agent-integration.md)
  provable-contract-first design: LLM-generated CoT steps must carry
  `Author::Agent { model, run_id }` and pass replay.
- [Component 22 provable-contracts.md](provable-contracts.md)
  `proof_obligations` is the target format for derivation.
- [Component 29 pmat-work-falsification-unification.md](pmat-work-falsification-unification.md)
  consumes CoT-derived claims into the roster.
- [Component 30 pmat-work-compile-time-codegen.md](pmat-work-compile-time-codegen.md)
  consumes CoT `expr` fields into `require` / `ensure` clauses.

## LLM Hallucination Countermeasures

Drawing from Li et al. (2025, arXiv:2510.12047) and Bruni et al. (2026,
arXiv:2602.22302):

- **Structural fields kill hand-wave.** Requiring `Assumption.predicate +
  references` forces the agent to name its input; unnamed inputs become
  `Axiomatic` discharges that reviewers can accept or reject explicitly.
- **Replay gate.** CB-1644 means an agent-authored CoT must produce the
  same step sequence on re-run. Agents that "creatively re-reason" fail.
- **Semantic-entropy gate.** Steps whose `Implication.predicate` exceeds
  an entropy threshold (Farquhar et al. 2024, Nature) without matching
  any existing contract obligation trigger a warning — agents padding
  out reasoning get flagged.

## Migration

- Existing CoT steps are string-only. `pmat work migrate --cot-structure`
  parses legacy entries with a best-effort field extractor (NER on
  "assume/therefore/because"), writes `cot-migration.json` for review.
- Legacy steps without structured fields are tagged
  `EvidenceMethod::ReviewOnly`, author unknown, treated as L0.
- CB-1640..1649 enter warn mode for 60 days (longer than other components
  since CoT rewrite is high-touch), then fail.

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_contract_core.rs` | Restructure `ChainOfThoughtStep` |
| `src/cli/handlers/work_cot_derivation.rs` | NEW — derive YAML / claims / clauses |
| `src/cli/handlers/work_cot_audit.rs` | NEW — DAG build, CB-1640..1649 |
| `src/services/semantic_match.rs` | Shared predicate-matcher (reused across components) |
| `src/cli/handlers/comply_handlers/check_handlers/check_cot_proof.rs` | NEW — CB-1640..1649 |

## References

### Internal
- [work-management.md](work-management.md) §Chain-of-Thought Audit Trail
- [agent-integration.md](agent-integration.md) CB-1400..1410 agent contracts
- [provable-contracts.md](provable-contracts.md) proof_obligations format
- [pmat-work-falsification-unification.md](pmat-work-falsification-unification.md) claim consumer
- [pmat-work-compile-time-codegen.md](pmat-work-compile-time-codegen.md) clause consumer

### Foundational
- Lakatos, I. (1976). *Proofs and Refutations*. Proof as a lemma-discharging process — motivates assumption-chain validity.
- Gentzen, G. (1935). *Investigations into Logical Deduction*. Natural deduction as the formal model for CoT-to-obligation translation.

### arXiv
- Wei et al. (2022). arXiv:2201.11903. Chain-of-Thought prompting — original LLM construct.
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts — empirical motivation for structured CoT.
- Farquhar et al. (2024). Nature. Semantic entropy — motivates CB padding-detection.
- Bruni et al. (2026). arXiv:2602.22302. Agent behavioral contracts — replay/verification framework adopted here.
