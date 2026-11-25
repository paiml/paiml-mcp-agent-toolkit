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
