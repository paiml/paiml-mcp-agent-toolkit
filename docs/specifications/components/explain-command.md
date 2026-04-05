# Explain Command Specification

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Supplementary to Component 12 (CLI & HTTP API)

## Root-Cause Analysis: Why Users Can't Understand Check Results

Five Whys (2026-03-31):

1. **Why can't users understand what a check means?** Every scoring command
   outputs IDs like CB-1210, PV-05, TDG-A but there's no way to ask
   "what does CB-1210 mean?" without reading source code or the book.
2. **Why is there no explain mechanism?** Check descriptions are embedded
   as string literals in handler functions. No structured registry maps
   check IDs to descriptions, rationale, and remediation steps.
3. **Why is there no structured registry?** Checks were added incrementally
   across 60+ files. Each constructs its own `ComplianceCheck` inline.
4. **Why isn't the output message sufficient?** Messages tell you WHAT failed
   but not WHY it matters or HOW to fix it.
5. **Why does this matter now?** The enforcement chain grew to 60+ checks.
   Users need `--explain CB-1210` not Chapter 62 of the book.

**Root cause:** No centralized check registry with (id, description,
rationale, remediation) tuples.

## Design

### Interface

`--explain <PATTERN>` flag on all scoring commands:

```bash
# Explain a specific check
pmat comply --explain CB-1210
pmat comply --explain CB-12      # prefix match: shows all CB-12xx

# Explain scoring dimensions
pmat score --explain D1
pmat score --explain              # list all dimensions

# Explain TDG grades
pmat tdg --explain A
pmat tdg --explain                # list all grades

# Explain infra-score checks
pmat infra-score --explain PV-05
pmat infra-score --explain CI     # fuzzy match

# Explain rust-project-score categories
pmat rust-project-score --explain RT-01
```

### Output Format

```
CB-1210: Precondition Quality
═══════════════════════════════

What it checks:
  Scans YAML contract preconditions for diversity and flags
  mass-generated placeholder patterns.

Why it matters:
  Placeholder preconditions like `!input.is_empty()` provide zero
  domain-specific protection. Real preconditions catch real bugs.

FAIL when:
  • YAML precondition diversity < 30%
  • >5% of equations have only placeholder preconditions

How to fix:
  Replace placeholder preconditions with domain-specific expressions:
    Bad:  '!input.is_empty()'
    Good: 'x.iter().all(|v| v.is_finite())'

See also:
  • CB-1211 (Codegen Fidelity)
  • pmat-book Chapter 62: Provable Contracts
```

### Registry Structure

```rust
pub struct CheckExplanation {
    pub id: &'static str,
    pub name: &'static str,
    pub what: &'static str,
    pub why: &'static str,
    pub fail_when: &'static [&'static str],
    pub how_to_fix: &'static str,
    pub see_also: &'static [&'static str],
}
```

Static array `EXPLANATIONS: &[CheckExplanation]` in a new module
`src/explain.rs`. Lookup by exact match or prefix match on `id`.

### Scope

All scoring commands share the same registry:

| Command | ID Prefix | Examples |
|---------|-----------|---------|
| `comply` | CB-xxx | CB-120, CB-200, CB-500, CB-1210 |
| `comply` | PV-xxx | PV-01..PV-05 (also in infra-score) |
| `score` | D1-D5 | Contract scoring dimensions |
| `score` | CD1-CD5 | Codebase scoring dimensions |
| `tdg` | TDG-* | TDG-A+, TDG-A, TDG-B, etc. |
| `infra-score` | CI-xx, SEC-xx, PV-xx | Infrastructure categories |
| `rust-project-score` | RT-xx, CQ-xx, TS-xx | Rust tooling, code quality, testing |

### Implementation

1. Add `--explain <PATTERN>` optional arg to comply, score, tdg,
   infra-score, rust-project-score commands
2. When `--explain` is provided, print explanation and exit (no check run)
3. When `--explain` without value, list all available check IDs
4. Registry is a static `&[CheckExplanation]` — no runtime cost
5. Lookup: exact match first, then prefix match, then fuzzy

### Priority

| Phase | What | Checks |
|-------|------|--------|
| P0 | CB-1200..1214 | Provable contracts (new, users need guidance) |
| P0 | CB-120..127 | OIP Tarantula + coverage (most common failures) |
| P0 | CB-200 | TDG grade gate (frequent CI blocker) |
| P1 | CB-500..530 | Rust best practices (high volume) |
| P1 | PV-01..05 | Infra-score bonus |
| P2 | All remaining | D1-D5, CD1-CD5, TDG grades, RT-xx, etc. |

## References

### arXiv

- 2603.25773 — The Specification as Quality Gate (2026).
  Three hypotheses on AI-assisted code review. Argues that without
  external specification grounding, review checks code against itself.
  Directly motivates why `--explain` must reference specifications,
  not just restate the check logic.

- 2503.09002 — KNighter: LLM-Synthesized Static Analysis Checkers (2025).
  Demonstrates auto-generating checker explanations from code patterns.
  Applicable to future auto-generation of `--explain` entries from
  check implementation source.

- 2508.18816 — Dealing with SonarQube Cloud (Nachman et al. 2025).
  Lessons on quality gate fatigue. Users ignore checks they don't
  understand. `--explain` is the antidote to alert fatigue.

- 2504.12211 — Benchmarkable Components for AI Developer Tools (2025).
  Argues for mature benchmarking of developer experience. `--explain`
  is a DX feature that reduces time-to-understanding for quality gates.

### Foundational

- Wang et al. (2025). arXiv:2512.17540. Specification-grounded code
  review. Found 90.9% improvement in developer adoption when review
  suggestions are grounded in human-authored specifications.
