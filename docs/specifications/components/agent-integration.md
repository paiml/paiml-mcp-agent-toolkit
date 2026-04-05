# Agent Integration

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 10

## MANDATORY: Provable-Contract-First Agent Design

**Every agent or sub-agent operating within a pmat-governed project MUST follow
contract-first design. Code generation without a prior contract is a FAIL.**

This is not optional guidance — `pmat comply` enforces it via CB-1400..1410.

### The Agent Contract Thesis

If provable-contracts proves that `softmax(x)` sums to 1.0 for ALL inputs,
then an agent tasked with implementing `softmax` must FIRST write the contract
YAML, THEN write code that satisfies it. The same principle applies to every
agent task — bug fix, feature, refactor, or optimization.

**Formal basis:** Bruni et al. (2026). "Agent Behavioral Contracts."
arXiv:2602.22302. Extends Eiffel DbC to multi-agent systems.

### Contract-First Workflow (MANDATORY)

Every agent (Claude Code, sub-agents, MCP tool agents, CI agents) MUST follow
this sequence. Violations are caught by CB-1400.

```
Step 1: CONTRACT    →  Write contract (YAML or contract.json) with:
                        - preconditions (what must be true before)
                        - postconditions (what must be true after)
                        - invariants (what must never change)
                        - falsification claims (how to disprove)
                        - verification level (L0-L5)

Step 2: VALIDATE    →  pmat comply check (contract exists + well-formed)

Step 3: RED TEST    →  Write failing test that exercises the contract

Step 4: IMPLEMENT   →  Write code to pass the test

Step 5: VERIFY      →  pmat comply check (all claims pass)
                        pmat work complete (if work item exists)
```

**NEVER skip to Step 4.** An agent that writes code before writing a contract
is operating at L0 (paper-only) — the lowest verification level.

### Contract Types by Agent Task

| Task Type | Contract Location | Required Claims |
|-----------|-------------------|-----------------|
| New function | `../provable-contracts/contracts/<project>/` YAML | preconditions, postconditions, proof obligations |
| Bug fix | `.pmat-work/<TICKET>/contract.json` | require (reproducer), ensure (fix verified) |
| Refactor | `.pmat-work/<TICKET>/contract.json` | invariant (behavior unchanged), ensure (quality improved) |
| Feature | Both YAML + contract.json | Full DbC: require/ensure/invariant + YAML for public API |
| Performance | `.pmat-work/<TICKET>/contract.json` | ensure (benchmark target), invariant (correctness preserved) |
| Test addition | `.pmat-work/<TICKET>/contract.json` | ensure (coverage delta), require (function exists) |

### Verification Levels for Agent Work

Agents MUST declare and achieve a verification level:

| Level | Agent Requirement | Evidence |
|-------|-------------------|----------|
| L5 | Lean theorem for correctness | `lean_theorem:` in YAML |
| L4 | Property-based test (proptest/probar) | `proptest!` macro in test |
| L3 | Falsifiable claims with evidence commands | `evidence:` in contract.json |
| L2 | Compile-time enforcement (traits, types) | `#[contract]` annotation |
| L1 | Quality gates pass | `pmat comply check` COMPLIANT |
| L0 | Human review only | **FAIL for autonomous agents** |

**L0 is not acceptable for autonomous agents.** Any agent operating without
human oversight MUST achieve at minimum L1. Recommended: L3+.

---

## Agent Compliance Checks: CB-1400..1410

| Check | What it enforces | Severity |
|-------|------------------|----------|
| **CB-1400** | Agent contract exists before code change | FAIL |
| **CB-1401** | Contract has falsifiable claims (not just descriptions) | FAIL |
| **CB-1402** | Verification level declared and >= L1 for autonomous agents | FAIL |
| **CB-1403** | Assume-guarantee chain valid (multi-agent) | WARN |
| **CB-1404** | Agent used `pmat comply check` before completing task | WARN |
| **CB-1405** | Contract references (arXiv, spec section, oracle) present | INFO |
| **CB-1406** | Chain-of-thought audit trail recorded | INFO |
| **CB-1407** | Five Whys linked for defect-origin tasks | WARN |
| **CB-1408** | Evidence commands all executable and passing | FAIL |
| **CB-1409** | No code generated at L0 without human approval | FAIL |
| **CB-1410** | Sub-agent contracts compose (output ensures = input requires) | WARN |

### CB-1400: Agent Contract Existence

**Detection:** On any commit that modifies `src/` or `crates/*/src/`:
1. Check that a corresponding contract exists in either:
   - `../provable-contracts/contracts/<project>/` (for new public functions)
   - `.pmat-work/<TICKET>/contract.json` (for work items)
2. FAIL if code changes exist without a contract
3. SKIP for test-only changes, documentation, or CI config

**Bypass:** `pmat comply check --allow-uncontracted` (must be explicit).

### CB-1409: No L0 Autonomous Code

**Detection:** If commit metadata contains `Co-Authored-By:` indicating an AI
agent, AND no contract.json or YAML contract references the changed files:
1. FAIL — autonomous agent generated code without a contract
2. The agent must retroactively create the contract and re-verify

This prevents the "write first, contract later" anti-pattern that
Li et al. (2025, arXiv:2510.12047) identified in LLM-generated code.

---

## Multi-Agent Assume-Guarantee Contracts

When multiple agents collaborate (architect + programmer, or parallel sub-agents),
their contracts form an assume-guarantee chain per Dardik & Kang (2509.06250):

```
Agent A (Architect):
  ensure: ["specification.md exists", "all requirements falsifiable"]
  output_invariant: "spec covers all acceptance criteria"

Agent B (Programmer):
  require: ["specification.md exists"]    ← Agent A's ensure
  ensure: ["all tests pass", "coverage >= 95%"]
  output_invariant: "implementation matches spec"

Agent C (Reviewer):
  require: ["all tests pass"]             ← Agent B's ensure
  ensure: ["pmat comply COMPLIANT", "no new SATD"]
```

**CB-1410** validates that `Agent[n].ensure` ⊇ `Agent[n+1].require`.

### Sub-Agent Contract Template

```json
{
  "agent_id": "sub-agent-coverage",
  "parent_agent": "main-agent",
  "task": "Improve test coverage for module X",
  "verification_level": "L3",
  "require": [
    {"description": "Module X compiles", "evidence": "cargo check"},
    {"description": "Current coverage baseline known", "evidence": "pmat query --coverage-gaps --limit 5"}
  ],
  "invariant": [
    {"description": "Existing tests still pass", "evidence": "cargo test"},
    {"description": "No new unwrap() calls", "evidence": "pmat query --literal 'unwrap()' --count"}
  ],
  "ensure": [
    {"description": "Coverage increased by >= 5%", "evidence": "cargo llvm-cov"},
    {"description": "No F-grade functions remain", "evidence": "pmat comply cb-200"}
  ],
  "chain_of_thought": [
    {"step": 1, "question": "Which functions have worst coverage?", "answer": "pmat query --coverage-gaps --rank-by impact"},
    {"step": 2, "question": "What patterns need testing?", "answer": "Error paths, edge cases from fault annotations"},
    {"step": 3, "question": "How to falsify coverage claim?", "answer": "cargo llvm-cov --fail-under-lines <target>"}
  ]
}
```

---

## Agent Context File Enforcement

### CLAUDE.md / GEMINI.md / AGENT.md

These files MUST include contract-first instructions. `pmat comply` (CB-130)
validates that agent context files contain required patterns:

**Required patterns in agent context files:**
- `provable-contract` or `contract-first` — design methodology reference
- `pmat comply` — enforcement command
- `NEVER.*code.*before.*contract` or equivalent — anti-pattern prohibition
- `verification_level` or `L0-L5` — ladder reference

CB-130 FAIL if agent context file exists but lacks contract-first patterns.

### AGENTS.md Protocol

Standard agent protocol declaration MUST include:

```yaml
capabilities:
  - pmat_comply_check    # Can run compliance validation
  - pmat_query_code      # Can search code semantically
  - pmat_work_start      # Can create work contracts

contract_policy: "contract-first"   # MANDATORY
min_verification_level: "L1"        # Minimum for autonomous operation
```

---

## MCP Tools Exposed

| Tool | Description | Contract Relevance |
|------|-------------|-------------------|
| `pmat_query_code` | Semantic code search | Find functions needing contracts |
| `pmat_get_function` | Get function with full metrics | Assess contract coverage |
| `pmat_find_similar` | Find similar functions | Reuse existing contract patterns |
| `pmat_index_stats` | Index health and statistics | Verify index freshness |
| `pmat_comply_check` | Run compliance checks | **Primary contract enforcement** |
| `pmat_tdg_score` | Get TDG score for function | Quality input for contracts |

### Agent MCP Workflow

An agent using pmat MCP tools MUST:

1. `pmat_comply_check` — assess current compliance state
2. `pmat_query_code` — find relevant functions and existing contracts
3. Write contract (YAML or contract.json)
4. Implement code
5. `pmat_comply_check` — verify compliance after changes

---

## Sub-Agent Architecture

### Actix Actor Framework

Multi-agent workflows using Actix actors:
- Each analysis phase runs as a contracted actor
- Message-passing for inter-phase communication with assume-guarantee
- Supervision trees for fault tolerance
- Actor contracts define message schemas and postconditions

### Architect-Programmer Protocol (SWE-CI)

Dual-agent workflow for requirement-driven development:
1. **Architect agent**: Analyzes test gaps, produces requirements AND contract
2. **Programmer agent**: Implements against the contract, validates all claims
3. **Review pass**: `pmat comply check` validates full chain

See [swe-ci-evolution.md](swe-ci-evolution.md) for full specification.

---

## Claude Code Skills

### Registered Skills

| Skill | Trigger | Contract Requirement |
|-------|---------|---------------------|
| `/quality` | Code quality analysis | Reports contract coverage gaps |
| `/comply` | Compliance check | **Primary enforcement entry point** |
| `/search` | Semantic code search | Finds uncontracted functions |
| `/evolve` | Evolution analysis | Tracks contract maturity over time |
| `/kaizen` | Continuous improvement | Creates contract-first work items |

---

## Enforcement Integration Points

### Pre-Commit Hook

```bash
# Validates agent commits have contracts
pmat comply check --agent-contracts
```

### CI/CD Gate

```yaml
# quality-gate.yml
- name: Agent Contract Enforcement
  run: pmat comply check --checks agent-contracts --fail-on-warn
```

### pmat score Integration

Agent contract compliance feeds into the composite score:
- DBC sub-score includes agent contract coverage
- Comply sub-score includes CB-1400..1410 results
- Zero agent contracts in an agent-modified file = zero DBC for that file

---

## Key Files

| File | Purpose |
|------|---------|
| `src/mcp_server/` | MCP server implementation |
| `src/mcp_pmcp/` | PMCP protocol handler |
| `CLAUDE.md` / `AGENT.md` (planned) | Agent protocol declaration (PMAT-540) |
| `CLAUDE.md` | Claude Code agent instructions |
| `src/cli/handlers/comply_handlers/` | Compliance check handlers |
| `src/cli/handlers/work_contract.rs` | DbC v5.0 contract engine |

## References

### arXiv
- Bruni et al. (2026). arXiv:2602.22302. Agent Behavioral Contracts.
- Dardik & Kang (2025). arXiv:2509.06250. Assume-Guarantee reasoning.
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts.
- Yang et al. (2025). arXiv:2509.11787. CodeCureAgent auto-fix.

### Foundational
- Meyer, B. (1988). *Object-Oriented Software Construction*. Design by Contract.
- Ohno, T. (1988). *Toyota Production System*. Jidoka.
- Popper, K. (1959). *The Logic of Scientific Discovery*. Falsificationism.

### Consolidated from
- agents, claude-agent-integration, claude-code-agent-mode-spec,
  claude-skills-spec-v1, claude-sub-agents-spec-actix
