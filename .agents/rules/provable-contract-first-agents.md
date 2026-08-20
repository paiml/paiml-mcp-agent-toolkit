# Agent Instruction: Provable-Contract-First Design

> **MANDATORY** for all agents and sub-agents operating in pmat-governed projects.
> Spec: [agent-integration.md](../specifications/components/agent-integration.md)

## The Rule

**NEVER write code before writing a contract.**

This applies to:
- Claude Code (primary agent)
- Sub-agents spawned by Claude Code
- MCP tool agents
- CI/CD automation agents
- Any autonomous system modifying source code

## Decision Tree

```
Is this a code change?
├── NO (docs, config, CI) → Proceed without contract
└── YES
    ├── New public function?
    │   └── Write YAML contract in ../provable-contracts/contracts/<project>/
    ├── Bug fix?
    │   └── Write contract.json in .pmat-work/<TICKET>/
    │       require: [reproducer], ensure: [fix verified]
    ├── Refactor?
    │   └── Write contract.json
    │       invariant: [behavior unchanged], ensure: [quality improved]
    ├── Performance?
    │   └── Write contract.json
    │       ensure: [benchmark target], invariant: [correctness preserved]
    └── Test addition?
        └── Write contract.json
            ensure: [coverage delta], require: [function exists]
```

## Step-by-Step (DETERMINISTIC)

### Step 1: Assess Current State

```bash
# What compliance checks currently pass?
pmat comply check

# What functions lack contracts?
pmat comply check 2>&1 | grep 'CB-1202\|CB-1208'

# What coverage gaps exist?
pmat query --coverage-gaps --limit 10 --exclude-tests
```

### Step 2: Write the Contract

For **new functions** (YAML in provable-contracts):

```yaml
name: <function-name>-v1
version: "1.0"
equations:
  - name: <function_name>
    preconditions:
      - "<domain-specific constraint on inputs>"
    postconditions:
      - "<domain-specific constraint on outputs>"
    proof_obligations:
      - "<property that must hold>"
    lean_theorem: "<lean proof reference>"  # Required for L5
```

For **work items** (contract.json):

```json
{
  "ticket_id": "<TICKET-ID>",
  "verification_level": "L3",
  "require": [
    {"description": "<precondition>", "evidence": "<command>"}
  ],
  "invariant": [
    {"description": "<what must not change>", "evidence": "<command>"}
  ],
  "ensure": [
    {"description": "<postcondition>", "evidence": "<command>"}
  ]
}
```

### Step 3: Validate Contract

```bash
# Contract well-formed?
pmat comply check

# For YAML contracts, also run:
# pv lint (if available)
```

### Step 4: Write Failing Test (RED)

Write a test that exercises the contract's postconditions.
The test MUST fail before implementation.

### Step 5: Implement

Write the minimum code to pass the test.

### Step 6: Verify

```bash
# All contract claims pass?
pmat comply check

# Coverage maintained?
pmat query --coverage-gaps --limit 5

# For work items:
pmat work complete <TICKET-ID>
```

## Anti-Patterns (NEVER DO)

| Anti-Pattern | Why It's Wrong | What To Do Instead |
|--------------|----------------|-------------------|
| Write code first, contract later | L0 paper-only; assertions become rubber stamps | Contract FIRST, always |
| Copy a generic contract | CB-1210 catches >70% identical preconditions | Write domain-specific claims |
| Skip `pmat comply check` | No verification = no proof | Run before AND after changes |
| Use `!input.is_empty()` as only precondition | E0 enforcement; CB-1214 flags it | Write real domain constraints |
| Sub-agent ignores parent contract | Breaks assume-guarantee chain | Sub-agent.require = parent.ensure |
| Autonomous agent at L0 | CB-1409 FAIL | Minimum L1, recommended L3+ |

## Multi-Agent Contract Chains

When spawning sub-agents, the parent MUST:

1. Define parent contract with `ensure` clauses
2. Each sub-agent's `require` MUST reference parent's `ensure`
3. Sub-agent's `ensure` feeds back into parent's verification

```
Parent Agent:
  ensure: ["module X has 95% coverage"]
  spawns: [sub-agent-A, sub-agent-B]

Sub-Agent A:
  require: ["module X compiles"]      ← parent precondition
  ensure: ["functions f1,f2 tested"]  ← partial parent goal

Sub-Agent B:
  require: ["functions f1,f2 tested"] ← sub-agent A's ensure
  ensure: ["module X has 95% coverage"] ← completes parent goal
```

## Commit Message Format

Agent commits MUST include contract metadata:

```
fix: CB-1208 binding names (Refs PMAT-032)

Contract: .pmat-work/PMAT-032/contract.json
Verification-Level: L3
Claims-Passed: 4/4
TDG-Score: 92.3
Co-Authored-By: Claude <noreply@anthropic.com>
```

## Quick Reference

| Question | Answer |
|----------|--------|
| Can I write code without a contract? | **NO** |
| What's the minimum verification level? | L1 (autonomous), L3 (recommended) |
| Where do function contracts go? | `../provable-contracts/contracts/<project>/` |
| Where do work contracts go? | `.pmat-work/<TICKET>/contract.json` |
| How do I check compliance? | `pmat comply check` |
| What if I'm a sub-agent? | Your `require` = parent's `ensure` |
