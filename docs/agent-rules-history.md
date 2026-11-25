# Agent rules history (moved out of CLAUDE.md, 2026-11-25)

Rationale, measurements and reference detail that CLAUDE.md links to. The rules themselves stay in CLAUDE.md.

## Code search flags

- `--regex` Rust regex against name, signature, source; `--literal` exact match, no semantic ranking.
- `--case-sensitive` forces case (default smart-case like rg); `-i` / `--ignore-case`.
- `--exclude PATTERN` (like grep -v); `--exclude-file GLOB`; `--files-with-matches` (rg -l); `--count` (rg -c); `-A N` / `-B N` / `-C N`.
- `-G` / `--git-history` fuses commit history via RRF. `--churn` 90-day volatility, hot files >50%. `--duplicates` MinHash + LSH clones. `--entropy` low (<30%) boilerplate, high (>80%) unique. `--faults` batuta fault patterns (unwrap, panic, unsafe).
- Dropped example commands: `pmat query "serialize" --min-grade A`, `"cache" --max-complexity 10`, `"dispatch" --rank-by pagerank`, `"simd" --include-project ../trueno`, `"tokenize" --include-source`, `"fix memory leak" -G`, `"error handling" --coverage --limit 10`, `"handler" --exclude "test"`, `"Error" -i`, `"handler" --files-with-matches`, `"unwrap" --count`, `"panic" -A 3 -B 2`.
- MCP tools: `pmat_query_code` semantic search; `pmat_get_function` full function with metrics; `pmat_find_similar` similar functions (refactoring); `pmat_index_stats` index health.

## Build-Budget Metrics

- The only code hit for `lint_max_ms` is a test fixture (`src/cli/analysis_utilities/quality_gate_part2f.rs:41`); no hook, Makefile target or CI job reads any timing threshold. Check with `pmat query --literal "lint_max_ms" --limit 5` and `grep -n "lint_max_ms\|test_fast_max_ms\|coverage_max_ms\|deps_default_max" Makefile scripts/*.sh .github/workflows/*.yml`.
- Binary size: hardcoded `50 * 1024 * 1024` in `src/tests/binary_size.rs:40` = 52,428,800 vs the 50,000,000 the file declares, under a comment claiming they are "aligned". 2.43 MB apart, neither reads the other.
- `make lint` and `make test-fast` record nothing.
- `[exclude]` and `[entropy]` read by `pmat quality-gate` (`src/cli/analysis_utilities/quality_checks_part1_complexity.rs:120`, `quality_checks_part1_entropy.rs:220`). `provability_min` and `entropy_min_diversity` read by `src/cli/analysis_utilities/quality_gate_config.rs`; the only two `[thresholds]` keys anything parses.
- CB-2101 (`pmat comply coherence`) audits every scalar in `[thresholds]`, `[quality_gates]`, `[performance]` against a live measurement. At `331017130`: 17 thresholds, 0 FIRING, 1 VIOLATED, 16 VACUOUS. The VIOLATED one is `quality_gates.max_unwrap_calls = 100` against a measured 20,390. Every binding and its evidence is in `.pmat-ratchet.toml` under `[coherence.binding.*]`.

## .pmat-ratchet.toml

- `.pmat-metrics.toml` records budgets nothing reads; `.pmat-ratchet.toml` records BASELINES asserted by CB-2102 and the `--lib` test `the_committed_ratchet_holds_at_head`.
- A `git grep` pathspec that has rotted and a genuine zero are byte-identical at the shell (both print `0`, exit 1, no stderr), so the gate refuses to guess: 0 against a baseline above 0 is UNMEASURABLE.
- The gate RUNS each entry's command rather than reading the number, so a baseline cannot quietly become a transcription. "We could not measure it" must never read as "it did not regress".
- Why the two files differ: `.pmat-metrics.toml:45` declares `max_unwrap_calls = 100` with inline comment `Current: 570`, in a tree that measures 20,390 by the predicate `.pmat-ratchet.toml` pins. Three numbers, no two agree, green build, because nothing reads the key.

## Documentation Accuracy

- `pmat quality-gate --checks docs-accuracy` once survived in this file for releases; it exits 2 with a clap error. `validate-readme` checks no command at all.
- `validate-readme` silently skipped a dead path whose filename contained parentheses, so a clean run is a floor, not a proof. On the pre-fix revision the two checks printed 8 dead paths and 1 bad command.
- Validation uses Semantic Entropy (Nature 2024) and the MIND framework (IJCAI 2025): capability claims vs AST, file paths, function/module refs, external URLs (404 detection).
- The pmat-invocation check drops this snippet's own regex text with `grep -vE`.

## Test Coverage

- At `fcb1eb45d` there were 341 `#[ignore]` in `src/` and 816 in `tests/`. An earlier revision claimed "~94 total (82 in src/)" with a 14-row category breakdown; both were off by more than 10x and the breakdown was not reconstructible, so it was deleted.
- An earlier revision said "always work on master", contradicting the global instructions (feature branch + PR).

## Five Whys

- `--auto-analyze` and `--context <FILE>` print `Warning: ... is not yet implemented. Flag ignored.` and give byte-identical output (verified by diffing full JSON with and without).
- Evidence weights v2 (PMAT-510, `calculate_confidence` in `src/services/five_whys_analyzer.rs`): IssueLocation 35%, Complexity 25%, SATD 20%, GitChurn 15%, EvoScoreTrajectory 15%, CoverageDelta 15%, ManualInspection 15%, DeadCode 10%. TDG weighted 0% (redundant with complexity+churn). Without an issue-specific IssueLocation item confidence is capped (`NO_ISSUE_EVIDENCE_CEILING`): other sources are repo-wide metrics identical whatever issue you typed.
- Command: `pmat five-whys "Test failures" --format json -o out.json`, `pmat five-whys "Perf regression" --format markdown`, `pmat why "Memory leak in cache" --depth 3`.

## Rust Project Score

289 points over 11 categories, 15 peer-reviewed papers (2022-2025). Categories: Rust Tooling & CI/CD (130), Code Quality (26), Testing (20), Known Defects (20), Formal Verification (16), Documentation (15), Reproducibility (15), Build Performance (15), Dependency Health (12), Performance & Benchmarking (10), GPU/SIMD Quality (10). Extra commands: `pmat rust-project-score --format json -o score.json` (CI/CD), `--failures-only`. Spec `docs/specifications/components/repo-health.md`; code `src/services/rust_project_score/`.

## Renacer

renacer 0.10.2 has exactly two subcommands, `validate` and `visualize`; `renacer capture` and `renacer validate --all` were documented here and both error out.

## trueno-graph

Context generation: `build_context_graph()` is called at `src/services/context_impl/visitor.rs:483`; every cached project analysis builds a `ProjectContextGraph`. Tests: `context_graph_tests.rs` plus `context_graph_coverage_{core,extended,fixtures}.rs`; TDG tests `src/tdg/tdg_graph_tests.rs`. Architecture: HashMap (O(1) lookups) + CSR graph (PageRank) + bidirectional NodeId mapping. Key insight: CSR graphs only track nodes with edges; `num_nodes()` returns node_map.len(), not graph.num_nodes().

## Dependency table

Reviewed externals: `roaring` (specialized bitmap, no batuta equivalent yet), `rand` (foundational, may move to trueno), `rayon` (foundational parallel iterator). Batuta crates and what they replace: see CLAUDE.md.

## Original phrasings kept for the record

- `pmat verify` gives a "green here ⇒ green in CI" guarantee: it runs the gate set CI enforces (format, complexity, satd, clippy, tests); the pre-commit hook and `pmat quality-gate` miss clippy + tests.
- Ratchet: "we could not measure it" must never read as "it did not regress".
- Search-flag table row: `pmat query "cache" --exclude-file "tests"` (flag " --exclude-file " takes a glob).

## Reference blocks moved from CLAUDE.md (second trim, 2026-11-25)

### Code search and coverage commands
```bash
pmat query "error handling" --limit 10            # by intent
pmat query "unwrap" --faults --exclude-tests      # fault patterns
pmat query "dispatch" --churn --duplicates --entropy --faults -G   # full enrichment
pmat query --regex "fn\s+handle_\w+" --limit 10   # like rg -e
pmat query --literal "unwrap()" --limit 10        # like rg -F
pmat query --coverage-gaps --limit 30 --exclude-tests
pmat query --coverage-gaps --rank-by impact --limit 20
pmat query "parse" --coverage --uncovered-only --limit 10
```
Flags: `--exclude`, `--exclude-file`, `-i`, `--files-with-matches`, `--count`, `-A/-B/-C`, `-G`, `--churn`, `--duplicates`, `--entropy`, `--faults`. MCP tools: `pmat_query_code`, `pmat_get_function`, `pmat_find_similar`, `pmat_index_stats`.
Coverage workflow: run `--coverage-gaps`; pick highest impact (missed_lines * pagerank / complexity); read with `pmat query "function_name" --include-source --limit 1`; write tests for uncovered lines; re-run.
grep is acceptable only for non-code files (TOML, YAML, Markdown) and quick one-off line matches while debugging.

### pmat-book policy detail
- Critical chapters in parallel (Ch 05, 07, 13, 14; `scripts/validate-pmat-book.sh:28`), 60s timeout each. Chapter 13 (Multi-Language) must always pass.
- Book checkout `/home/noah/src/pmat-book`; override `PMAT_BOOK_DIR=<path>`. Absent dir = script skips, not fails: green proves nothing.
- Enforced only by hooks from `bash scripts/install-git-hooks.sh`: pre-commit warns (`scripts/install-git-hooks.sh:128`); pre-push BLOCKS `git push` until pmat-book commits are pushed (`scripts/install-git-hooks.sh:238`). `pmat hooks install` hooks have neither check: grep `.git/hooks/pre-push` for `pmat-book`.
- Workflow: update pmat-book, push to main (deploys Pages), push code. Before `cargo publish`: book pushed, `make validate-book` passes, Pages deployed.

### pmat verify
```bash
pmat verify --format json          # ok:true safe to commit; ok:null = no verdict (see not_measured[])
pmat verify --fix                  # auto-apply cargo fmt / clippy --fix first
pmat verify --skip clippy,tests    # fast inner loop
```
Loop: edit -> pmat verify --format json -> fix on red -> commit on green. Spec `docs/specifications/pmat-verify-autonomous-preflight.md`; `docs/agent-instructions/autonomous-verify-loop.md`.

### Build-budget and ratchet detail
- Budgets: lint <=150s, test-fast <=360s, coverage <=600s, binary <=50MB, deps <=3,000. Only `make coverage` and `make build-release` call `scripts/record-metric.sh`. Binary size is enforced by a hardcoded value in `src/tests/binary_size.rs:40`.
- Really read: `[exclude]`, `[entropy]`, `provability_min`, `entropy_min_diversity`. `pmat comply coherence` (`--format json`) shows FIRING, VIOLATED or VACUOUS per threshold; bindings in `.pmat-ratchet.toml` `[coherence.binding.*]`.
- Ratchet: BASELINES asserted by comply rule CB-2102 and `--lib` test `the_committed_ratchet_holds_at_head`. A metric measuring 0 against a baseline above 0 is UNMEASURABLE (set `zero_is_reachable = true` when the zero is real) and FAILS. Each entry carries its reproducing command. Engine `src/services/metrics_ratchet/`; contract `contracts/comply-ratchet-v1.yaml`.
```bash
pmat comply ratchet            # judge baselines; non-zero on regression
pmat comply ratchet --lower    # rewrite baselines the tree already beat
```

### Documentation accuracy
```bash
pmat context --output deep_context.md --format llm-optimized      # ~25s
pmat validate-readme \
    --targets README.md CLAUDE.md \
    --deep-context deep_context.md \
    --fail-on-contradiction --verbose                             # --deep-context REQUIRED
```
- No docs-accuracy check on `pmat quality-gate`; `pmat validate-readme` is the whole surface. Runs only in the pre-commit hook from `bash scripts/install-git-hooks.sh` (`scripts/install-git-hooks.sh:98`, blocking on contradictions); in no `.github/workflows/` job, not in `pmat hooks install` hooks. It can skip refs (a clean run is a floor).
- After editing CLAUDE.md run both checks, expected output nothing:
```bash
grep -oE '`[A-Za-z0-9_./()-]+\.(md|rs|sh|toml)(:[0-9]+)?`' CLAUDE.md \
  | tr -d '`' | sed 's/:[0-9]*$//' | sort -u \
  | while read -r p; do test -e "$p" || echo "DEAD PATH: $p"; done
{ sed -e ':a' -e '/\\$/{N;s/\\\n//;ba}' CLAUDE.md | grep -oE '^pmat [^#|>]+'
  grep -oE '`pmat [^`]+`' CLAUDE.md | tr -d '`'
} | sed 's/[[:space:]]*$//' | grep -vE '[][^…\\]' | sort -u \
  | while read -r c; do eval "$c --help" >/dev/null 2>&1 || echo "BAD COMMAND: $c"; done
```

### Other command blocks
- bashrs: `make lint-makefile` (ends `|| true`, `Makefile:898`, non-blocking; suppressions `.bashrsignore`); `bashrs lint Makefile --ignore MAKE003,MAKE006,MAKE010,MAKE012,MAKE017,MAKE018`; `make lint-scripts` is deno/TypeScript, no bashrs; `cargo install bashrs`; bugs https://github.com/paiml/bashrs/issues
- `#[ignore]` count (measure, never quote): `git grep -cE '^[[:space:]]*#\[ignore' -- src | awk -F: '{n+=$2} END{print n+0}'` (same for `tests`); authoritative: `env -u RUST_MIN_STACK cargo test --lib -- --ignored --list`.
- Five whys: `pmat five-whys "Stack overflow in parser"` (default `--depth 5`); options `--depth <1-10>`, `--format <text|json|markdown>`, `--output <FILE>`, `--path <PATH>`; `--auto-analyze` and `--context <FILE>` do nothing.
- Rust score: `pmat rust-project-score` (~2-3 min), `--full` (~10-15 min); spec `docs/specifications/components/repo-health.md`, code `src/services/rust_project_score/`.
- Renacer: `renacer validate --generate <DIR> -- <command>` (baseline); `renacer validate --baseline <DIR> -- <command>` (compare).
- trueno-graph: `analyze_project_with_cache()` (`src/services/context_impl/visitor.rs:473`) calls `build_context_graph()`; `src/services/context_graph.rs`; `src/tdg/tdg_graph.rs`.
- Stack docs: `batuta oracle --rag-index` (once), `--rag "your question here"`, `--rag-stats`, `--rag-index-force`; `--rag-index --force` errors.
- Batuta replacements: aprender (not nalgebra/linfa/smartcore), trueno (not ndarray), trueno-graph (not petgraph), trueno-db (not polars/datafusion), trueno-rag (not qdrant/milvus), trueno-viz (not plotters), trueno-zram-core (not lz4/zstd), bashrs (not shellcheck), probar (not quickcheck), presentar-core (not ratatui). Reviewed externals kept: `roaring`, `rand`, `rayon`; `nalgebra-sparse` removed (`Cargo.toml:251`).
