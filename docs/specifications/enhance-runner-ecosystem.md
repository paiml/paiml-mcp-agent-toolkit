# Enhanced Hook Runner Ecosystem for PMAT

**Status**: DRAFT - Peer Review In Progress
**Date**: 2025-12-21
**Pattern**: Toyota Way - Jidoka + Heijunka (Level Loading)
**Spec Version**: 1.1

## Executive Summary

Integrate a pre-commit hook runner into PMAT to complete the sovereign AI stack. This specification analyzes 5 popular single-binary approaches, proposes O(1) integration with existing PMAT infrastructure, and provides a 100-point Popperian falsification checklist.

**Goal**: Replace external `prek` dependency with native `pmat hooks run` capability.

---

## Part I: Comparative Analysis of Single-Binary Hook Runners

### Architecture Matrix

| Approach | Lang | Config | Parallelism | Env Mgmt | Binary Size | Complexity |
|----------|------|--------|-------------|----------|-------------|------------|
| **lefthook** | Go | YAML | Native goroutines | None | ~8 MB | Medium |
| **rusty-hook** | Rust | TOML | Manual (`&&`) | None | ~2 MB | Low |
| **cocogitto** | Rust | TOML | None | None | ~4 MB | Low |
| **cargo-husky** | Rust | Cargo.toml | Manual | None | 0 (build-time) | Minimal |
| **git-branchless** | Rust | None | Internal | None | ~15 MB | High |
| **pmat O(1)** | Rust | TOML | Rayon | uv/node/cargo | Existing | Medium |

### 1. Lefthook (Go) - Runtime Executor

**Architecture**: Single Go binary with goroutine-based parallelism.

```yaml
# lefthook.yml
pre-commit:
  parallel: true
  commands:
    lint:
      glob: "*.rs"
      run: cargo clippy
    fmt:
      glob: "*.rs"
      run: cargo fmt --check
```

**Strengths**:
- Native parallelism (5x faster than pre-commit Python)
- Mature ecosystem, well-documented
- File glob filtering

**Weaknesses**:
- No environment management (uv, node, etc.)
- External dependency (not sovereign)
- Go binary doesn't integrate with Rust toolchain

**Performance**: 1.016s vs pre-commit's 5.149s (benchmarked)

---

### 2. Rusty-Hook (Rust) - Minimal Executor

**Architecture**: Cargo dev-dependency, TOML config, shell delegation.

```toml
# .rusty-hook.toml
[hooks]
pre-commit = "cargo fmt --check && cargo clippy"
pre-push = "cargo test"
```

**Strengths**:
- Pure Rust, minimal footprint (~2 MB)
- Simple TOML configuration
- Zero runtime overhead after install

**Weaknesses**:
- No parallelism (sequential `&&` chaining)
- No file filtering
- No environment management

**Performance**: First run ~30s setup, subsequent runs near-instant.

---

### 3. Cocogitto (Rust) - Conventional Commits Specialist

**Architecture**: Specialized for commit message validation, libgit2 dependency.

```toml
# cog.toml
[git_hooks.commit-msg]
script = "cog verify --file $1"

[git_hooks.pre-push]
script = "cargo test"
```

**Strengths**:
- Best-in-class conventional commit enforcement
- Changelog generation
- Semantic versioning automation

**Weaknesses**:
- Not general-purpose hook runner
- No parallelism
- libgit2 dependency

**Performance**: Fast for commit-msg, not designed for complex pre-commit.

---

### 4. Cargo-Husky (Rust) - Build-Time Installer

**Architecture**: Uses `build.rs` to install static shell scripts.

```toml
# Cargo.toml
[dev-dependencies]
cargo-husky = { version = "1", features = ["run-cargo-test", "run-cargo-clippy"] }
```

**Strengths**:
- Zero runtime overhead (static scripts)
- Feature-flag configuration
- Extremely lightweight (1,000 LOC)

**Weaknesses**:
- Rust-only (no multi-language support)
- No parallelism
- Limited customization

**Performance**: Zero overhead - hooks are pre-generated shell scripts.

---

### 5. Git-Branchless Hooks (Rust) - Event-Driven

**Architecture**: SQLite event log, internal parallelism for rebase/graph ops.

**Strengths**:
- Fastest rebase implementation
- Changeset evolution tracking
- Multi-threaded internally

**Weaknesses**:
- Not a general hook runner (specialized workflow)
- No user-configurable hooks
- Heavy binary (~15 MB)

**Performance**: Excellent for internal operations, not applicable to general hooks.

---

### 6. PMAT O(1) Approach - Proposed Integration

**Architecture**: Extend existing `pmat hooks` with runner capability, leverage O(1) metric caching.

```toml
# .pmat-hooks.toml
[runner]
parallel = true
fail_fast = true

[hooks.pre-commit]
stages = ["lint", "test", "quality"]

[hooks.pre-commit.lint]
run = "cargo clippy -- -D warnings"
glob = "**/*.rs"
cache = "o1"  # Use O(1) metric validation

[hooks.pre-commit.test]
run = "cargo test --lib"
cache = "o1"

[hooks.pre-commit.quality]
run = "pmat validate-metrics"
cache = "none"  # Always run (instant)
```

**Unique Strengths**:
- **O(1) Caching**: Skip re-execution if inputs unchanged (Merkle-hash based)
- **Sovereign**: No external dependencies
- **Environment Management**: Leverage uv (Python), node, cargo
- **Quality Integration**: Native TDG, complexity, SATD checks
- **Rayon Parallelism**: Rust-native thread pool

---

## Part II: Proposed Architecture

### Design Principles (Toyota Way)

| Principle | Application |
|-----------|-------------|
| **Jidoka** | Automated quality detection, stop on first failure |
| **Heijunka** | Level-load hook execution across cores |
| **Muda** | O(1) caching eliminates redundant work |
| **Poka-yoke** | Config validation prevents misconfiguration |
| **Genchi Genbutsu** | Measure actual hook performance |

### Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    pmat hooks run                        │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Config      │  │ O(1) Cache  │  │ Env Manager │     │
│  │ Parser      │  │ Validator   │  │ (uv/node)   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                │             │
│         ▼                ▼                ▼             │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Parallel Executor (Rayon)           │   │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐ │   │
│  │  │ lint   │  │ test   │  │ fmt    │  │ custom │ │   │
│  │  └────────┘  └────────┘  └────────┘  └────────┘ │   │
│  └─────────────────────────────────────────────────┘   │
│                          │                              │
│                          ▼                              │
│  ┌─────────────────────────────────────────────────┐   │
│  │           Result Aggregator + Reporter           │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### O(1) Cache Integration

Leverage existing `.pmat-metrics/` infrastructure with Merkle-tree-like properties:

```rust
fn should_run_hook(hook: &Hook) -> bool {
    if hook.cache == CacheMode::None {
        return true;  // Always run
    }

    // O(N) calculation, but N is small (changed files only) in incremental mode
    // or O(1) retrieval if using FS monitor / git index SHA
    let input_hash = compute_merkle_root(&hook.glob, &hook.files);
    let cached_hash = read_cached_hash(&hook.name);

    // O(1) comparison
    input_hash != cached_hash
}
```

**Cache Hit**: Skip execution, report cached result (~5ms)
**Cache Miss**: Execute hook, update cache
**Mechanism**: The "O(1)" claim refers to the *decision* complexity once state is known. By relying on Git's internal index SHA or a fast Merkle walk of changed files, we avoid re-reading all content.

---

## Part III: Peer-Reviewed Citations

### [1] Static Analysis Tool Usability (ACM ISSTA 2022)

> Nachtigall, M., Schlichtig, M., & Bodden, E. (2022). "A Large-Scale Study of Usability Criteria Addressed by Static Analysis Tools." *ACM SIGSOFT International Symposium on Software Testing and Analysis*, 532-543. DOI: 10.1145/3533767.3534374

**Relevance**: >50% of tools have poor warning messages; workflow integration neglected. PMAT hooks must prioritize actionable feedback.

### [2] Incremental Build Benefits (ACM ICSE 2022)

> Randrianaina, G.A., et al. (2022). "On the Benefits and Limits of Incremental Build of Software Configurations." *ACM/IEEE International Conference on Software Engineering*, 1584-1596. DOI: 10.1145/3510003.3510190

**Relevance**: Incremental builds achieve 88.5% faster execution. O(1) caching applies same principle to hook execution.

### [3] CI/CD Pipeline Optimization (MDPI 2025)

> "CI/CD Pipeline Optimization Using AI: A Systematic Mapping Study." *MDPI Journal*, 2025.

**Relevance**: 92 papers analyzed; AI techniques effective for test stage optimization. Hook runners benefit from intelligent scheduling.

### [4] Developer Productivity with AI (arXiv 2023)

> "The Impact of AI on Developer Productivity: Evidence from GitHub Copilot." *arXiv:2302.06590*, February 2023.

**Relevance**: 55.8% faster task completion with AI assistance. Fast hooks reduce context-switching overhead.

### [5] Shift-Left Testing (IEEE Software 2022)

> "Shift-Left Testing: Early Defect Detection in Agile Development." *IEEE Software*, 2022.

**Relevance**: Early defect detection reduces rework costs 10-100x. Pre-commit hooks are the ultimate shift-left.

### [6] The SPACE of Developer Productivity (ACM Queue 2021)

> Forsgren, N., et al. (2021). "The SPACE of Developer Productivity." *ACM Queue*, 19(1), 20-48. DOI: 10.1145/3454122.3454124

**Relevance**: Emphasizes "Flow" and "Efficiency". Fast, reliable hooks maintain developer flow state (the "S" and "E" in SPACE) by reducing wait times and cognitive load.

### [7] Monorepo at Scale (CACM 2016)

> Potvin, R., & Levenberg, J. (2016). "Why Google Stores Billions of Lines of Code in a Single Repository." *Communications of the ACM*, 59(7), 78-87. DOI: 10.1145/2854146

**Relevance**: Supports PMAT's unified "sovereign stack" approach. Centralized tooling in a monorepo (or large repo) reduces dependency divergence and enforces consistent quality gates across all languages.

### [8] Modern Build Systems (ICSE 2018)

> Mokhov, A., et al. (2018). "Build Systems à la Carte." *Proceedings of the 40th International Conference on Software Engineering*, 705-716. DOI: 10.1145/3180155.3180181

**Relevance**: Defines the theory of "Cloud Build Systems" and "Early Cutoff". PMAT's O(1) caching implements the "Early Cutoff" principle—stopping execution if inputs have not changed—essential for scalable hook performance.

---

## Part IV: 100-Point Popperian Falsification Checklist

### Methodology

Following Karl Popper's falsificationism: a specification is scientific only if it can be proven false. Each item below is a **falsifiable claim** that must be tested.

---

### A. Core Functionality (20 points)

| # | Falsifiable Claim | Test Method | Pass |
|---|-------------------|-------------|------|
| 1 | `pmat hooks run` executes pre-commit hooks | Run command, verify execution | ☐ |
| 2 | Hooks execute in parallel when `parallel=true` | Measure wall time < sum of individual times | ☐ |
| 3 | `fail_fast=true` stops on first failure | Introduce failing hook, verify subsequent hooks skipped | ☐ |
| 4 | `fail_fast=false` runs all hooks | Introduce failing hook, verify all hooks run | ☐ |
| 5 | Glob patterns filter files correctly | `*.rs` excludes `.ts` files | ☐ |
| 6 | Hook exit code 0 = pass | Verify successful hook reports pass | ☐ |
| 7 | Hook exit code non-0 = fail | Verify failing hook reports fail | ☐ |
| 8 | Hook stdout captured | Output visible in terminal | ☐ |
| 9 | Hook stderr captured | Error output visible in terminal | ☐ |
| 10 | Hook timeout terminates stuck process | Set 1s timeout, sleep 10s in hook | ☐ |
| 11 | Missing hook command = clear error | Typo in command shows helpful message | ☐ |
| 12 | Empty hooks section = no-op | No hooks defined, command succeeds | ☐ |
| 13 | `--hook <name>` runs specific hook | Only named hook executes | ☐ |
| 14 | `--all-files` ignores staged filter | Runs on entire repo | ☐ |
| 15 | `--files <path>` runs on specific files | Only specified files checked | ☐ |
| 16 | Working directory is repo root | Hook CWD is git root | ☐ |
| 17 | Environment variables passed to hooks | `$HOME`, `$PATH` available | ☐ |
| 18 | Custom env vars configurable | `env = { FOO = "bar" }` works | ☐ |
| 19 | Hook inherits shell | `/bin/sh` or configured shell used | ☐ |
| 20 | Multiple hook types supported | pre-commit, pre-push, commit-msg | ☐ |

---

### B. O(1) Caching (20 points)

| # | Falsifiable Claim | Test Method | Pass |
|---|-------------------|-------------|------|
| 21 | Cache hit skips execution | No file changes, hook not re-run | ☐ |
| 22 | Cache miss triggers execution | File changed, hook runs | ☐ |
| 23 | Hash computation is stable | Same inputs = same hash | ☐ |
| 24 | Hash changes on file modification | Edit file, hash differs | ☐ |
| 25 | Hash changes on file addition | Add file, hash differs | ☐ |
| 26 | Hash changes on file deletion | Delete file, hash differs | ☐ |
| 27 | Cache validation <10ms | Benchmark cache check | ☐ |
| 28 | Cache stored in `.pmat-metrics/` | Verify file location | ☐ |
| 29 | Cache format is JSON | Parseable JSON files | ☐ |
| 30 | `cache = "none"` always runs | No caching for marked hooks | ☐ |
| 31 | Stale cache (>7d) triggers warning | Old cache shows warning | ☐ |
| 32 | `--no-cache` bypasses caching | Force re-execution | ☐ |
| 33 | Cache respects glob pattern | Only glob-matched files in hash | ☐ |
| 34 | Cache invalidates on config change | Edit hook config, re-runs | ☐ |
| 35 | Parallel hooks have independent caches | Each hook cached separately | ☐ |
| 36 | Cache survives git operations | Checkout branch, cache persists | ☐ |
| 37 | Cache can be cleared | `pmat hooks cache clear` works | ☐ |
| 38 | Cache size reported | `pmat hooks cache size` shows bytes | ☐ |
| 39 | Corrupted cache handled gracefully | Delete half of cache file, no crash | ☐ |
| 40 | Cache directory auto-created | Fresh repo, directory created on first run | ☐ |

---

### C. Configuration (15 points)

| # | Falsifiable Claim | Test Method | Pass |
|---|-------------------|-------------|------|
| 41 | TOML config parsed correctly | Valid TOML loads without error | ☐ |
| 42 | Invalid TOML = clear error | Syntax error shows line number | ☐ |
| 43 | Missing config = sensible defaults | No config file, command still works | ☐ |
| 44 | Config schema validated | Unknown key shows warning | ☐ |
| 45 | Config supports inheritance | `extends = "base.toml"` works | ☐ |
| 46 | Per-hook overrides work | Hook-specific settings override global | ☐ |
| 47 | `--config <path>` uses custom config | Alternate config file used | ☐ |
| 48 | Config hot-reload not needed | Config read fresh each run | ☐ |
| 49 | Relative paths resolved from repo root | `./scripts/lint.sh` works | ☐ |
| 50 | Absolute paths supported | `/usr/bin/cargo` works | ☐ |
| 51 | `~` expanded in paths | `~/bin/tool` expands | ☐ |
| 52 | Config supports comments | `# comment` ignored | ☐ |
| 53 | Config supports multiline strings | TOML `"""` syntax works | ☐ |
| 54 | Config validates hook names | Reserved names rejected | ☐ |
| 55 | Config example in docs | README shows full example | ☐ |

---

### D. Environment Management (15 points)

| # | Falsifiable Claim | Test Method | Pass |
|---|-------------------|-------------|------|
| 56 | Python hooks use `uv` if available | `uv run ruff` preferred | ☐ |
| 57 | Node hooks use system node | `node` resolved from PATH | ☐ |
| 58 | Rust hooks use cargo | `cargo` commands work | ☐ |
| 59 | Virtual env activated for Python | `.venv` respected | ☐ |
| 60 | `language = "python"` sets up env | Auto-detects Python runtime | ☐ |
| 61 | `language = "node"` sets up env | Auto-detects Node runtime | ☐ |
| 62 | `language = "rust"` sets up env | Auto-detects Cargo | ☐ |
| 63 | `language = "system"` uses raw shell | No env setup | ☐ |
| 64 | Missing runtime = clear error | "Python not found" message | ☐ |
| 65 | Runtime version logged | Shows `Python 3.12.0` on verbose | ☐ |
| 66 | `additional_dependencies` installed | Extra packages available | ☐ |
| 67 | Dependency cache persists | Second run uses cached deps | ☐ |
| 68 | Env isolation between hooks | Hook A's deps don't affect B | ☐ |
| 69 | `require_serial = true` disables parallel | Marked hooks run sequentially | ☐ |
| 70 | Env setup time logged | Shows setup duration on verbose | ☐ |

---

### E. Integration (15 points)

| # | Falsifiable Claim | Test Method | Pass |
|---|-------------------|-------------|------|
| 71 | `pmat hooks install` installs git hooks | `.git/hooks/pre-commit` created | ☐ |
| 72 | Installed hook invokes `pmat hooks run` | Commit triggers pmat | ☐ |
| 73 | `pmat hooks uninstall` removes hooks | `.git/hooks/pre-commit` removed | ☐ |
| 74 | Existing hooks backed up | `.git/hooks/pre-commit.bak` created | ☐ |
| 75 | TDG enforcement integrated | `pmat hooks enforce-tdg` works | ☐ |
| 76 | bashrs linting integrated | Bash files linted via bashrs | ☐ |
| 77 | Metric recording integrated | Hook results cached in `.pmat-metrics/` | ☐ |
| 78 | CI mode supported | `--ci` disables interactive prompts | ☐ |
| 79 | Verbose mode shows details | `--verbose` prints hook commands | ☐ |
| 80 | Quiet mode minimal output | `--quiet` only shows failures | ☐ |
| 81 | JSON output supported | `--format json` for CI parsing | ☐ |
| 82 | Exit code reflects status | 0 = all pass, 1 = failure | ☐ |
| 83 | `--no-verify` bypass documented | Emergency escape hatch | ☐ |
| 84 | Monorepo support | Multiple `.pmat-hooks.toml` files | ☐ |
| 85 | Staged files passed to hooks | `$STAGED_FILES` env var set | ☐ |

---

### F. Performance & Reliability (15 points)

| # | Falsifiable Claim | Test Method | Pass |
|---|-------------------|-------------|------|
| 86 | Cold start <100ms | Benchmark from zero state | ☐ |
| 87 | Warm start <50ms | Benchmark with cache | ☐ |
| 88 | O(1) validation <10ms | Benchmark cache check only | ☐ |
| 89 | Parallel speedup measurable | N hooks, wall time < N × single | ☐ |
| 90 | Memory usage <100MB | Profile with large repo | ☐ |
| 91 | No memory leaks | Run 1000 times, memory stable | ☐ |
| 92 | Handles 1000+ files | Large repo doesn't crash | ☐ |
| 93 | Handles deep nesting | 50-level directory works | ☐ |
| 94 | Handles special characters | Filenames with spaces, unicode | ☐ |
| 95 | Handles symlinks | Symlinked files processed | ☐ |
| 96 | Handles missing .git | Non-git directory = clear error | ☐ |
| 97 | Handles detached HEAD | Works in CI detached state | ☐ |
| 98 | Handles bare repo | Bare repo = clear error | ☐ |
| 99 | Signal handling (SIGINT) | Ctrl+C cleanly terminates | ☐ |
| 100 | Concurrent git operations safe | Multiple terminals don't corrupt | ☐ |

---

## Part V: Implementation Phases

### Phase 1: Core Runner (Sprint N)
- Config parser (TOML)
- Sequential executor
- Basic git integration

### Phase 2: Parallelism (Sprint N+1)
- Rayon thread pool
- Dependency graph for ordering
- `fail_fast` support

### Phase 3: O(1) Caching (Sprint N+2)
- Hash computation
- Cache storage/retrieval
- Staleness detection

### Phase 4: Environment Management (Sprint N+3)
- uv integration
- Node detection
- Cargo integration

### Phase 5: Polish (Sprint N+4)
- Monorepo support
- Migration from prek
- Documentation

---

## Part VI: Migration Path from prek

```bash
# Current (external prek)
prek run --all-files

# Future (native pmat)
pmat hooks run --all-files

# Config migration
mv .pre-commit-config.yaml .pmat-hooks.toml  # With conversion script
```

**Compatibility**: Provide `pmat hooks migrate` to convert prek YAML to PMAT TOML.

---

## Decision Required

**Option A**: Full integration (all 5 phases)
**Option B**: Minimal viable (phases 1-2 only, no caching)
**Option C**: Defer to future sprint

**Recommendation**: Option A - Full integration aligns with sovereign stack goal.

---

**Awaiting feedback before implementation.**
