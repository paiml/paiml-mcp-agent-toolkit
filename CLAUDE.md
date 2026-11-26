# Claude Code Configuration

Git workflow, branches, PRs: global instructions. Long reference blocks and rationale: `docs/agent-rules-history.md` (read it before running any command block named below).

## Dependencies (80/20 Batuta Stack)
- **MANDATORY: minimize external dependencies; 80% batuta stack, 20% external max.**
- Before ANY external dep (math, ML, compute): 1. CHECK BATUTA STACK FIRST (`pmat query "YourFeature" --limit 5`); 2. BUILD IF CLOSE (70%+: extend it); 3. EXTERNAL ONLY AS LAST RESORT, document why.
- Replacement table: docs/agent-rules-history.md "Reference blocks". `nalgebra-sparse` removed (`Cargo.toml:251`): do not re-add.

## Code Search
- **NEVER use grep/glob for code search. ALWAYS use `pmat query`.** Add `--faults` when auditing unwrap/panic/unsafe. Exploring code to write tests: `pmat query --include-source`, NOT `Read`/`cat`/`grep`. Commands: docs/agent-rules-history.md "Reference blocks".
- **MANDATORY coverage work: `pmat query --coverage-gaps`. NEVER `make coverage` or raw `cargo llvm-cov` output.**
- grep only for non-code files (TOML, YAML, Markdown) and one-off line matches.

## pmat-book (Jidoka)
- **MANDATORY before any release or version bump: `make validate-book`** (Ch 13 must always pass).
- **MANDATORY: book updates MUST be pushed with code changes**; pre-push hook from `bash scripts/install-git-hooks.sh` BLOCKS `git push` until they are. Before `cargo publish`: book pushed, validate passes. Detail: docs/agent-rules-history.md "Reference blocks".

## Verify before commit
- **MANDATORY: run `pmat verify --format json` before every commit** (ok:true = safe; ok:null = no verdict). `pmat quality-gate` misses clippy + tests. Loop: `docs/agent-instructions/autonomous-verify-loop.md`.

## Build budgets and ratchet
- **Do not treat `[thresholds]` in `.pmat-metrics.toml` as gates**; they are RECORDED, NOT ENFORCED. Real pre-commit gate set is `pmat verify`. Emergency bypass: `git commit --no-verify`.
- Numbers that CAN fail a build: `.pmat-ratchet.toml` (CB-2102). A metric may only get better; raising a baseline needs a `justification`. Run `pmat comply ratchet`; `--lower` rewrites baselines the tree beat. A metric at 0 vs baseline above 0 is UNMEASURABLE and FAILS.
- Why: nothing reads most keys (0 FIRING, 1 VIOLATED, 16 VACUOUS at `331017130`). History: docs/agent-rules-history.md "Reference blocks"

## Documentation accuracy
- **MANDATORY for README.md, CLAUDE.md, GEMINI.md, AGENT.md: after editing, run `pmat context` then `pmat validate-readme --fail-on-contradiction` (`--deep-context` REQUIRED) and the dead-path/bad-command checks.** Commands: docs/agent-rules-history.md "Reference blocks". Only the pre-commit hook runs this; no CI job does.
- Why: pre-fix revision had 8 dead paths, 1 bad command.

## Quality tools
- bashrs: bash/Makefiles should pass `make lint-makefile` (non-blocking). `cargo install bashrs`.
- **Use `cargo llvm-cov` exclusively. NEVER use cargo-tarpaulin.**
- `#[ignore]` tests are silently unmeasured: measure the count, never quote it; fix and remove the attribute, or record why it stays next to it. Do not work on master.
- `pmat five-whys` (aliases `why`, `debug-whys`) **is the ONLY acceptable debugging method**; `--auto-analyze` and `--context` do nothing.
- `pmat rust-project-score` (289 points, 11 categories); `--full` for all checks.
- **Renacer golden tracing MANDATORY for transpilers, distributed systems, multi-process, cross-language work**: validate golden traces before completing (`renacer.toml`).
- trueno-graph is ACTIVE (CSR graph, PageRank): `src/services/context_graph.rs`, `src/tdg/tdg_graph.rs`.
- Agent instructions: follow `docs/agent-instructions/` (`docs/agent-instructions/pmat-work-ux-fixes.md`, `docs/agent-instructions/pmat-work-quality-principles.md`). Stack docs: `batuta oracle --rag-stats`, `--rag "your question here"`.
- Compliance CB-130 (`pmat comply check`): index `.pmat/context.idx` fresh (<24h); CLAUDE.md contains `pmat query`, `NEVER use grep`, `--faults`.
