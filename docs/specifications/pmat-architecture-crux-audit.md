# PMAT Architecture, Performance & Competitive (CRUX) Audit

## 1. Provenance

> **Audited at.** Audited against `paiml/paiml-mcp-agent-toolkit` (crate `pmat`) at
> `origin/master` **`01fba4f65`** (v3.34.0, clean), and **written against `512c60139`**,
> which merged the 3.35.0 release PR #1108 mid-audit. The full delta between the two is
> `CHANGELOG.md, Cargo.lock, Cargo.toml, README.md, mcp.json` — **zero lines under `src/`,
> `tests/`, `.github/` or `build.rs`** (`git diff --stat 01fba4f65 512c60139`). One of those five
> files is itself the subject of an item: the `mcp.json` delta is the version string alone
> (`git diff 01fba4f65 512c60139 -- mcp.json` → `"version": "3.34.0"` → `"3.35.0"`), so CRUX-09's
> schema findings are unaffected. Every code finding below therefore holds at master as written;
> where a *number* moved between the two commits it is recorded as a **CORRECTION**, not a
> refutation. **A number that differs because two commands measured different file sets is not a
> correction and must not be filed as one** — every `git grep` count in §11 carries its pathspec for
> that reason (see the `include!` entry, where `-- 'src/**/*.rs'` and `-- src` differ by 6 sites
> because the glob silently drops the 14 top-level `src/*.rs` files).
>
> **Method.** A 12-dimension research fleet (231 findings, 91 candidate enhancements,
> 2.8 M tokens, 1,176 tool calls) across module architecture, MCP/HTTP/CLI surface parity,
> performance hot paths, measured performance, build/CI, dependency health, product UX,
> competitive landscape, pmat-on-pmat self-quality, prior-audit reconciliation,
> index/persistence/determinism, and error/exit-code/public-API. Merged to **20 candidates**,
> then put through **3-lens adversarial verification** — EMPIRICAL, NOVELTY, VALUE — whose
> decisive lens **re-ran each candidate's own acceptance test against the freshly built
> binary at HEAD** rather than reading the research report. 20 of 20 survived and 47 corrections
> were returned. A second pass then re-read the merge against the twelve dimension reports and
> recovered **12 further candidates the merge had dropped with no written reason** (CRUX-21..32),
> plus 26 factual corrections — including three acceptance snippets that were **green on the
> unfixed binary** and one "correction" that was a pathspec artefact rather than a number moving.
> **The backlog is 32 items**; every correction from both passes is applied below.
>
> **Access.** Better-than-live: a local clone `git diff`-verified against master; a freshly
> built release binary of the audited commit at
> `/mnt/nvme-raid0/coverage/paiml-mcp-agent-toolkit/release/pmat`
> (`pmat --version` → `3.34.0 / commit: 01fba4f6554742ae690fa00131444ddf722a5334 / worktree: clean`,
> 54,633,288 B) plus the independently installed `~/.cargo/bin/pmat` 3.34.0 as a control;
> live `gh api` (branch protection, check-runs, secrets, Dependabot alerts, issues); live
> crates.io; WebSearch/WebFetch for every competitor cell; and `strace` for execve/openat
> attribution. `cargo metadata` / `cargo tree` resolve without compiling; no
> `cargo build|test|clippy|llvm-cov` was run by the audit (the three release-build timings
> are read from a prior instrumented run, `research/build{1,2,3}.*`).
>
> **Artefact custody.** Every `research/` citation in §5, §8.19 and §11 resolved only inside the
> ephemeral per-session scratch directory the audit ran in. That directory is gone; the evidence is
> not. The timing artefacts were copied into this repository at **`docs/audits/2026-09-crux/`**
> before the session ended — `m/*.time` and `m/*.meta` (one pair per measured command, carrying wall
> clock, max RSS, CPU%, exit status, the exact command line and the `uptime` at that instant),
> `build{1,2,3}.{time,stderr}`, and the four harness scripts that produced them. **Read
> `research/<x>` below as `docs/audits/2026-09-crux/<x>`.** Two classes were deliberately not
> preserved: the command *payloads* (`m/*.out`, `m/*.err`) because several are tens of MB and every
> number cited here comes from the `.time`/`.meta` pair, and the 8.67 MB `ctx.md` that is itself
> §8.18's evidence — its size is the finding, and `wc -c` on a regenerated copy reproduces it. A
> citation that cannot be resolved in that directory is **UNVERIFIED**, not ARTEFACT. As a second
> hedge, the load average at each measurement's instant is also inlined beside the §5.1 table and
> the §5.5 build rows, so the one thing an artefact could still have told you — *was this measured
> under contention* — survives even the loss of the directory.
>
> **Machine.** One shared 48-core / 125 GB NVMe host that **also runs the self-hosted CI
> runner pool**, under load average ~24 at the start of the audit and 0.76–32.14 across the
> measurement window. Every timing below was taken under that load, and every `.meta` file
> records the `uptime` at its instant. Sub-second wall-clock deltas on this box are
> scheduling noise; CPU-seconds and CPU% are the robust figures and are what the findings
> rest on.
>
> **Scope note.** This audit does **not** re-derive gate integrity from scratch; that is
> `docs/specifications/fable-review.md` (July 2026, HEAD `f4ce4f980`). §7 reconciles all 24
> of its backlog items and all 19 checkable claims of issue #1114 against this tree.

---

## 2. Executive summary — the load-bearing findings

1. **The command every agent must run before every commit is vacuous in its most common state.**
   CLAUDE.md makes `pmat verify` MANDATORY and promises "green here ⇒ green in CI". On the clean
   tree it returns `{"ok": true, "stages_measured": 2}`, exit 0, with complexity reporting
   `ok: null, not_applicable: "no Rust files changed vs HEAD, so nothing was measured"` — while
   `pmat quality-gate` on the identical tree exits 1 with **35 blocking violations** — of which the
   proposed fix makes verify decline to assert over **1** (28 are entropy, which is not a verify stage
   at all; 5 of the 6 complexity blockers are at or below the repo's own configured `max_complexity`
   and are a *separate* over-reporting defect, CRUX-24; 1 of 3 SATD findings is unambiguous debt). The
   35 is the size of the divergence, not a count of genuine violations, and this document uses it
   only that way. One line:
   `src/cli/verify.rs:194` `ok: !failed && measured > 0`, whose `not_measured` guard fires only at
   `measured == 0`. The file's own doc comment at `:66-76` states the opposite contract.

2. **`quality-gate` renders three unmeasured dimensions as clean.** On an uncompilable crate it
   prints `Checking dead code... 0 violations found` and emits no `not_measured` anywhere, while
   `analyze dead-code` on the same tree returns `{"not_measured": true, "reason": "Cargo check
   failed"}` — the analyzer's state is discarded at `quality_checks_part1_dead_code.rs:68`. Its
   coverage check accepts any cache it finds, including this repo's gitignored one **114 commits**
   behind HEAD covering 209 of 4,036 files. And `duplicate_violations: 0` on a tree where
   `analyze duplicates` measures **21.67 % / 33,801 clone blocks** — the gate's "duplicate" is a
   whole-file hash, the analyzer's is a block.

3. **A validator that certifies input it could not read — and a report that says nothing about input
   it did.** `pmat config --validate` prints `Configuration is valid / No issues detected` and exits 0
   for a schema-invalid `pmat.toml`, for `not even toml ][` and for **no config at all**: three inputs
   it could not read, certified via silent fallback to the defaults. A fourth case is byte-identical
   for a *different* reason — the repo's own fully valid `pmat.toml` parses fine (no stderr warning)
   and still produces the same bytes, because the statistics block is hardcoded
   (`configuration_handlers_validation.rs:118-122` prints the literals `Sections: 7` and
   `Total Settings: ~50`), so the verdict carries no evidence of what was validated. Since #1105 merged, `quality-gate` emits a *blocking* violation for the
   same bytes: two commands, one binary, opposite verdicts.

4. **One `Cargo.toml` line has disabled clap's usage, error-context and suggestions for four years.**
   `Cargo.toml:226`. **Not** "since the crate shipped": it was introduced by `a36329bc6` (2025-06-01,
   "comprehensive test coverage improvements and compilation fixes"), which rewrote
   `clap = { version = "4.5", features = ["derive", "env"] }` into the `default-features = false`
   form (`git log -S'clap = { version = "4.5", default-features = false'`). `git show
   v0.1.9:server/Cargo.toml` still has default features ON, and v0.1.5–v0.1.9 were cut 2025-05-28 —
   before the regression. Every release from 2025-06-01 carries it. Measured: **0 of 71** subcommands print a usable `Usage:` line;
   `--bogus-flag` returns 34 bytes that never name the flag; a mistyped subcommand gets no
   suggestion. It survived because **14 sites assert the substring `"Usage:"`**, which the broken
   output still contains — one of them pmat's own product code
   (`src/docs_enforcement/cli_checker_validation.rs:74`), so pmat certifies its own empty help.

5. **`build.rs:21` declares a path outside the repo, so there is no incremental build.**
   `cargo:rerun-if-changed=../assets/demo/`, a fossil of the `server/` layout deleted by
   `2aa5832a1`. Cargo names it itself — the **only** stale item in the fingerprint log — and a
   no-op `cargo build --release` costs **55.28 s wall / 263.5 CPU-s at 499 %**, twice measured.

6. **The index CLAUDE.md mandates over grep can serve deleted code forever.** `check_mtime_reuse`
   skips read+SHA256 for any file whose mtime predates `built_at` and republishes the old checksum
   unverified. Reproduced: after a content swap with a backdated mtime, `pmat query` returned a
   **deleted** function with its deleted source and missed the live one, and the manifest checksum
   did not match disk. The fast path covers 4,477 of 4,477 files here. Independently: no walk is
   sorted (order follows `readdir`), and `analyze churn --format json` serialises a `HashMap` —
   5 distinct outputs in 10 runs.

7. **The two MCP transports have different JSON-RPC error contracts.** Six probe shapes, **six**
   divergences: stdio `-32602` vs HTTP `-32603` (×3), `-32601` id-echoed vs `-32700` `id: null`,
   and `{"jsonrpc":"1.0"}` rejected on stdio but answered `200` with the full tool list over HTTP.
   Three fresh HTTP servers give three tool orderings. The guard,
   `cli_mcp_surface_parity_tests.rs:86`, is `include_str!` + `.contains()` — it binds no socket.

8. **The packaged `mcp.json` is wrong for 19 of 19 tools; 6 cannot be called as described.**
   `tool_schema()` (`tool_manifest.rs:131-167`) invents a shape from the tool *name*.
   `manifest_descriptions_match_handler_metadata` pins the prose; nothing pins the schema.

9. **`include!` defeats pmat's own headline metric by 32.5 points.** `pmat tdg` on
   `src/cli/analysis_utilities/mod.rs` → **93.4 (A)**; the same code flattened through its 54-file
   closure → **60.9 (C)**. Reproduced with two independent flatteners and a synthetic probe, so it
   is a product defect every pmat *user* can exploit. The directory walk is worse:
   `pmat tdg src/cli/analysis_utilities` drops 44 of 70 files and reports `files_ungraded: 0,
   not_measured: false`.

10. **489 of 4,433 tracked `.rs` files hold 8,313 tests no build compiles**, and pmat ships the
    0.32 s instrument that says so, wired to nothing: `grep -rn reachability .github/workflows/
    Makefile` returns nothing. `src/tests/` alone is 188 files / 76,461 lines / 4,335 tests, 82 of
    them named `coverage_boost_*`.

11. **Six CI checks compute a number and cannot fail a build**: the coverage floor was `skipped` on
    all four sampled green master runs, MSRV is verified only `on: release: published` and blind to
    a direct dev-dep needing 1.93.1 against a declared 1.91.0, the required `feature-gate` context
    *prints* a Dependabot conclusion its own leg says it did not measure, three duplicate-dependency
    artifacts are inert, and `cargo-semver-checks` is in no workflow.

12. **Competitively, the differentiators are real and the flagship is unusable at scale.** Five
    checks have no competitor equivalent found (`reachability`, `unrun-tests`, `vacuous-tests`,
    `comply coherence`, `comply ratchet`) — while `pmat context` emits **8,670,750 bytes**
    (~2.17 M tokens, ESTIMATE) with no `--max-tokens` on the CLI or the MCP tool, against an
    incumbent shipping a 1k-token budgeted repo map since 2023.

**Four more that the first merge dropped without a reason and that §8 now carries** (the full
recovery is §3, Stage 2b): **(13)** every `cargo install pmat` binary reports `commit: unknown`
while the SHA it needs sits in `.cargo_vcs_info.json` inside the same tarball — the defect whose
*symptom* items 5 and 11 above already use as evidence (**CRUX-21**); **(14)** `analyze duplicates`
defaults to a mode costing **16.67 s / 1.69 GiB** against `exact`'s **1.61 s / 596 MiB** for **6.9 %**
more findings, and `--min-lines` cannot bound it (**CRUX-22**); **(15)** `src/protocol/` (2,047
lines) and `src/state/` (3,896) are `pub mod` in the default build with nothing calling them, and
**all three** analyzers pmat sells as differentiators are blind to the class (**CRUX-23**); and
**(16)** the complexity gate announces a limit of 30 and blocks at 28, which is why five of the six
complexity blockers in item 1's "35" comply with the repo's own configuration (**CRUX-24**).

---

## 3. Method and falsification discipline

**Stage 1 — 12 parallel dimension agents**, each read-only, each required to carry `path:line`
or an exact command with its output for every claim, to state what observation would refute the
claim, and to say whether it looked. 231 findings, 91 candidate enhancements.

**Stage 2 — merge.** Deduplicated to 20 candidates, with 13 items explicitly **dropped with a
written reason** (§9) rather than silently pruned, and 3 issues nominated as closable (§10).

**Stage 2b — audit of the merge itself, which is where the largest single result came from.** The
merge was re-read against all twelve dimension reports, asking only "what survived its own report's
falsification and then appears in neither §8 nor §9.2?". Twelve did: the build-provenance defect
whose *symptom* §5.4 and §8.19 already use as evidence; the largest measured performance lever in
the corpus; two `pub mod`s nothing calls; a gate that blocks two below the limit it announces; a
comply check reporting the population it did not inspect; the `pmat query` peer-semantics inversion;
two CRUX-scorecard failures; two written gates nothing can run; the package-headroom exclusion; the
doc-citation checker; the TLS pin; and the context parse path. They are CRUX-21..32. **The merge's
failure mode was silent omission, not over-inclusion** — which is the same defect class the audit's
subject matter is about, committed by the audit, and it is recorded here rather than smoothed over.

**Stage 3 — 3-lens adversarial verification.** Each of the 20 was independently attacked on
three axes:

- **EMPIRICAL** — re-run the candidate's own acceptance test against the built binary at HEAD.
  Not "does the report say so", but "does the assertion fail today, in my shell".
- **NOVELTY** — is it already shipped, already fixed between `01fba4f65` and `512c60139`,
  already on fable-review §5's do-not-do list, or already an open issue? (Duplicating an *open*
  issue is not a refutation; duplicating a *closed and fixed* one is.)
- **VALUE** — is the fix at the cause or the symptom, does it strengthen or weaken a gate, does
  it add a dependency the batuta doctrine forbids, and **can the acceptance test be gamed**?

**What the discipline actually caught.** Nothing was refuted outright — all 20 survive — but the
verification returned **47 corrections**, and their distribution is the interesting result:

| correction class | count | example |
|---|---|---|
| stated root cause wrong (defect real) | 4 | CRUX-01(b): strict SATD misses `TODO(CB-128):` because of the **separator**, not the comment's position; `classifier.rs:244-250` requires `rest.strip_prefix(':')` while Standard accepts `[':','(','[','!']`. The proposed "handle trailing comments" fix would have repaired **neither** repo marker. |
| **acceptance test that could not fail** | 3 | CRUX-06's static leg: a `... \| while read p; do [ -e "$p" ] \|\| echo ...; done` pipeline exits with the status of `echo` — it printed `MISSING: ../assets/demo/` **and exited 0**. Wired into CI unchanged it would have been one more gate that certifies. |
| acceptance test gameable by a lazy fix | 9 | CRUX-02 leg 3 passes on `duplicate_violations = max(1, whole_file_dupes)`; CRUX-08 asserts only *equality* between transports, so deleting `repair_outbound` from stdio makes both equally bad and the gate green; CRUX-11(a) passes if the size-sensitive `structural_complexity` term is simply deleted. |
| acceptance leg already fixed (premise stale) | 2 | CRUX-02 leg 2 asserted the coverage check is silent when no report exists — `quality_gate_part2a.rs:20-46` already emits an explicit "was NOT measured" violation and exits 1. The surviving defect is narrower: **the gate accepts any cache it finds**. |
| acceptance leg unrunnable as written | 4 | `pmat comply ratchet --format json` → `error: unexpected argument found` (no `--format` on that subcommand); `quality_proxy {"mode":"blocking"}` → `-32602 Invalid mode` (the enum is strict/advisory/auto_fix). |
| anti-vacuity control that would fail a *correct* fix | 3 | CRUX-02's control demands `not_measured` be empty on a healthy crate — but a healthy crate legitimately has no coverage data, so the correct fix fails its own control. |
| acceptance leg demanding a **regression** | 1 | CRUX-10's polyglot leg demanded identical SATD counts for byte-identical bytes as `.rs`/`.py`/`.sh`. Measured both ways: `# TODO:` scores 0/1/1 and `// TODO:` scores 1/0/0 — SATD detection is correctly language-aware in *both* directions, and enforcing the leg would make pmat flag `n = a // TODO_COUNT` as debt. Leg deleted. |
| number drift / transcription | 11 | run 1 of `pmat score` is 10,237,248 kB = **9.76 GiB**, not the 9.53 GiB two reports carried (a kB→MiB slip — the same slip recurred in four more RSS figures and is corrected in §5.4); the `--format` sweep is **0 of 71**, not 1 of 71; the SARIF-formatter census is **44** non-test files, not 60 (§8.15). |
| **pathspec artefact mistaken for drift** | 1 | "`include!` sites are 2,289 at HEAD, not 2,295" was filed here as a correction and is **withdrawn**. Both numbers are the same tree measured with two pathspecs, and neither moved: `git grep -c 'include!(' -- 'src/**/*.rs'` → 2,289 at *both* commits, `-- src` → 2,295 at *both*. `src/**/*.rs` silently drops the 14 top-level `src/*.rs` files (4,019 of 4,033), so 2,289 is an undercount, and §1's own zero-lines-under-`src/` delta makes the "moved between commits" framing self-contradictory. **2,295 is the number.** Every git-grep count in §11 now records its pathspec. |
| wrong file cited for a real fact | 6 | CRUX-13 cited `quality_proxy_analysis.rs:247` (a doc comment about the scanner) for the SATD walk; the code to change is `detection_file_discovery.rs:180-206`. |

**The single most instructive result** is the correction class "acceptance test that could not
fail", found three times in an audit whose subject is gates that cannot fail. A falsifier is a
gate; it inherits every failure mode of the gates it judges. Every acceptance test in §8 has
therefore been re-written to (a) exit non-zero, not merely print, (b) carry an explicit
anti-vacuity control naming the lazy fix it blocks, and (c) where the fix could be satisfied by
*measuring less*, pin the work done (a file count, a provenance object, a richness floor).

**What this audit did not do.** No `cargo build/test/clippy/llvm-cov` was run (read-only budget),
so: the post-fix build timings in CRUX-06, the `--features mutation-testing` build in CRUX-16,
the two `cargo clippy`-backed ratchet metrics, and `pmat rust-project-score` are UNVERIFIED and
marked as such. `pmat comply check` was never run — it saturates this machine (~744 % CPU) — so
comply hermeticity claims are inherited from the prior audit, not re-measured.

---

## 4. Architecture findings

**A1 — The single-crate layout is not the problem, and a workspace split is not the fix.** Only
**66,876 lines (6.0 % of `src/`)** sit in modules with out-degree zero, while `cli` (360 k) and
`services` (357 k) are **64 % of the tree and mutually cyclic** — 970 refs one way, 92 the other,
across 14 module-level cycles. The cheap prerequisite: **62 of those 92 `services → cli` edges are
two misplaced leaf modules** — `src/cli/colors.rs` (26,882 B, **zero `crate::` dependencies of its
own**, used by 134 files / 382 refs) accounts for 47 and `cli::language_analyzer` for 15. Moving
`colors` to `src/term/colors.rs` deletes half the cycle as a pure rename. It fixes no measurable
failure and blocks nothing on the §8 list, so it is a prerequisite chain, not a work item (§9).

**A2 — `include!` is the tree's defining architectural fact, and it defeats pmat's own metric.**
`git grep -c 'include!(' -- src` → **2,295 sites** at HEAD — unchanged at `01fba4f65`; the narrower
`-- 'src/**/*.rs'` gives 2,289 because that glob drops the 14 top-level `src/*.rs` files (4,019 of
4,033), which is a pathspec artefact and not drift (§3) — and resolving every target over all of
`src/` gives **2,061 fragment files / 528,656 lines**, i.e. **48 % of the 1,110,514
lines under `src/` are not modules** — `include!` splices text, so there is no privacy boundary, no
separate compilation, no `mod`-level `#[cfg]`. Module size recomputed as *host + transitive closure*:

```
  lines files   host  path
  16789    54    395  src/cli/analysis_utilities/mod.rs
  14956    39   1507  src/cli/handlers/comply_handlers/check_handlers/check.rs
   8521    24    475  src/cli/handlers/work_contract.rs
   5390    10     39  src/mcp_pmcp/tool_functions/mod.rs
```

`analysis_utilities` is **16,789 lines** — 60 % *larger* than the 10,478 #1114 filed, because the
remediation was to shard it into fragments to satisfy a per-file line gate:
`src/cli/analysis_utilities/mod.rs:26` reads `// TDG handlers - extracted for file health (CB-040)`
directly above `include!("tdg.rs")`, and 48 sites tree-wide carry that rationale. `mod.rs` uses
`include!` 25 times and `mod` 7 times, all 7 `#[cfg(test)]`. The correct form —
`#[path = "..."] mod` — is already used **174 times**, so the tree knows it.
Measured consequences: the same code grades **A (93.4) split and C (60.9) flattened** (§8.11); **55 %**
of pmat's self-measured complexity comes from a `heuristic_include_fragment` fallback
(`{ast: 1802, heuristic_include_fragment: 2231, heuristic: 3}` over 4,036 files) rather than the AST;
`pmat context` cannot parse 28 files because a fragment is deliberately brace-unbalanced
(`work_tests_part1.rs`: 56 `{`, 53 `}`) — a fact pmat's own source documents at
`src/services/vacuous_tests.rs:119`; and because `#[cfg(test)]` sits inside the included file rather
than at the call site, rustc lexes and parses **~190,342 lines of test source on every release build**.

**A3 — 37 % of `src/` is test code compiled into the library crate, and it ships.** 1,195 files /
413,004 lines / 11.79 MB of 38.06 MB are test-named; `Cargo.toml`'s `exclude` drops `/docs/`,
`/scripts/` and `/rust-docs/` but **nothing under `src/`**, so all of it goes into the published
tarball. The often-quoted "94 % of crates.io's 10 MiB ceiling" is **not a current measurement**: it is
transcribed from the comment at `Cargo.toml:22-25`, which records the size that *motivated* the
existing exclusions ("The package was 9.4 MiB against crates.io's 10 MiB ceiling — 94 % — so one more
asset would have made the crate unpublishable"). The current packed size is **UNVERIFIED** (it needs
`cargo package`, outside this audit's budget). What *is* measured is the live constraint: the
`package-size` job hard-fails at `mib >= 9.0` (`feature-matrix.yml:506-507`, "at or past the 9.0 MiB
budget"), i.e. the real headroom is 9.0 MiB, not 10.
33,505 test functions compile into **one** `--lib` binary, so `cargo test --lib` gets no cross-binary
parallelism — the flip side of the `autotests = false` consolidation.

**A4 — The public API is ~10,752 `pub` items behind 39 root `pub mod`, with 1 `#[doc(hidden)]` — and two of those 39 are dead.**
`unreachable_pub = "warn"` (`Cargo.toml:1001`) is structurally near-vacuous: every internal module is
`pub mod` at the root, so almost every `pub` item is reachable by construction. **The consequence was
measured, not merely feared.** `src/protocol/` (13 files, **2,047 lines**, `pub mod protocol;` at
`src/lib.rs:168`) has **zero** references outside its own subtree
(`git grep -lE 'crate::protocol\b|pmat::protocol\b' -- src tests | grep -v '^src/protocol/'` → 0) and
`src/state/` (**3,896 lines**, `src/lib.rs:185`) is referenced only from
`tests/integration_test.rs.disabled`. Both are in the DEFAULT build, so ~5,900 shipped lines are
invisible to `unreachable_pub`, to `analyze dead-code` (5 dead functions / 0 dead modules across
3,228 files) and to `analyze reachability` (which correctly calls them reachable — the build does
compile them). No pmat check can currently see this class; **CRUX-23** proposes the one that can. There is **no
`cargo-semver-checks` in any CI job** and, measured live, **zero reverse dependencies on crates.io**
(`{"meta":{"total":0}}`) — which is why §8.14's semver leg is split out and *not* made required.

**A5 — Error handling has one type in practice and seventy on paper.** 2,684 `anyhow` tokens, 889
`anyhow::Result`, 609 `anyhow!`, 354 `bail!` against **70 declared `*Error` enums**, of which
`PmatError` — the advertised consolidation — has **zero production callers** (317 of its 355
occurrences are its own tests; not re-exported from `lib.rs`). #1114's "650+ anyhow references" is a
4× undercount, and its "the robust `PmatError` enum is bypassed" is wrong in an interesting way: it
was never connected.

**A6 — MCP advertises 19 tools against 71 CLI subcommands; only one family has a forcing function.**
`src/cli/analyze_mcp_exposure.rs:109` emits a **total match with no catch-all**, so adding an
`AnalyzeCommands` variant without declaring `Tool` / `CliOnly(reason)` / `Backlog(issue)` is a compile
error — 37 rows, 9/6/22, against a `BACKLOG_CEILING = 22` ratchet. That machinery covers **1 of 71**
top-level `Commands` variants; the template for extending it exists at
`src/cli/command_wire_names.rs`.

**A7 — Four `McpRequest`/`McpResponse` definitions and four tool inventories.**
`src/mcp_integration/types.rs:66,82`; `src/demo/adapters/mcp.rs:14,23`;
`src/agents_md/bridge.rs:97,153`; `src/models/mcp_types.rs:3,12`. Two dead inventories
(`src/handlers/initialize_tools_*.rs`, 21 tools reachable only from `src/tests/e2e_full_coverage.rs`;
`src/contracts/mcp_simple/handler.rs:89`, 8 tools reachable only from its own tests) name analyzers
the live registry marks `Backlog` — so reading the source tells you pmat serves analyzers it does not.

**A8 — Design work that is written and not compiled.** The "parse every file in parallel ONCE and
share one AST cache" design and the whole `src/tdg/scorers/` subsystem — **4,368 lines of the most
performance-relevant code in the tree** — are unreachable. Static hot-path reasoning about this
codebase is wrong by default; every performance claim in §5 is backed by `strace` or `/usr/bin/time`,
not by reading.

---

## 5. Performance findings

### 5.1 The measured table

Harness `research/measure.sh` / `measure2.sh`; every row is one exec of the HEAD binary under
`timeout N /usr/bin/time -v`, with the load average recorded per run in `research/m/<name>.meta`.
`/usr/bin/time`'s "Maximum resident set size" covers the process **and its reaped children** —
which matters for `score` (§5.4).

| run | command | dir | exit | wall | max RSS | CPU% | user s | sys s | stdout B |
|---|---|---|---|---|---|---|---|---|---|
| version1/2 | `--version` | repo | 0 | 0:00.00 | 15.0 MB | 237 / 171 % | 0.00 | 0.01 | 77 |
| help1 | `--help` | repo | 0 | 0:00.00 | 15.3 MB | 150 % | 0.00 | 0.01 | 9,117 |
| fix_complexity | `analyze complexity --format json` | 3-file fixture | 0 | 0:00.21 | 24.3 MB | **12 %** | 0.00 | 0.02 | 3,278 |
| fix_complexity2 | same, 2nd exec | fixture | 0 | 0:00.01 | 24.3 MB | 144 % | 0.00 | 0.01 | 3,361 |
| fix_satd | `analyze satd --format json` | fixture | 0 | 0:00.01 | 20.6 MB | 140 % | 0.00 | 0.01 | 696 |
| fix_deadcode | `analyze dead-code --format json` | fixture | 0 | 0:00.11 | 84.3 MB | 98 % | 0.04 | 0.07 | 1,149 |
| repo_query1 | `query "error handling" --limit 5` | repo | 0 | 0:00.25 | 92.2 MB | 97 % | 0.14 | 0.10 | 5,701 |
| repo_satd | `analyze satd --format json` | repo | 0 | 0:00.17 | 21.2 MB | 102 % | 0.10 | 0.07 | 1,262 |
| repo_deadcode | `analyze dead-code --format json` | repo | 0 | 0:01.06 | 19.6 MB | **34 %** | 0.05 | 0.32 | 4,058 |
| repo_complexity | `analyze complexity --format json` | repo | 0 | 0:01.50 | 54.9 MB | 114 % | 1.44 | 0.27 | 727,864 |
| repo_qualitygate | `quality-gate --format json` | repo | **1** | 0:02.00 | 55.9 MB | 107 % | 1.82 | 0.33 | 34,045 |
| repo_verify | `verify --skip clippy,tests --format json` | repo | 0 | 0:02.73 | 300.7 MB | 97 % | 2.35 | 0.30 | 627 |
| repo_tdg | `tdg . --format json` | repo | 0 | 0:05.08 | 526.0 MB | **1768 %** | 58.98 | 30.89 | 12,808 |
| repo_context | `context --format llm-optimized -o ctx.md` | repo | 0 | 0:05.99 | 278.1 MB | 119 % | 5.25 | 1.92 | 8,670,750 (file) |
| repo_dag | `analyze dag --format json` | repo | **2** | 0:00.00 | 16.1 MB | 171 % | 0.00 | 0.01 | 0 |
| repo_dag_fixed | `analyze dag -o /dev/null` | repo | 0 | 0:01.47 | 217.4 MB | 313 % | — | — | (mermaid) |
| repo_duplicates | `analyze duplicates --format json` | repo | 0 | 0:16.22 | **1,731 MiB** | **1995 %** | 218.15 | 105.59 | 46,641 |
| dup_exact | `analyze duplicates --detection-type exact` | repo | 0 | **0:01.61** | **596 MiB** | **102 %** | — | — | 35,342 blocks |
| dup_all | `analyze duplicates` (default `all`), paired back-to-back with the row above | repo | 0 | 0:16.67 | 1.69 GiB | 1988 % | — | — | 37,785 blocks |
| repo_score | `score` | repo | 0 | **1:19.85** | **10,237,248 kB = 9.76 GiB** | 270 % | 179.12 | 36.93 | 721 |
| repo_score2 | `score` (2nd run, same tree) | repo | 0 | 1:16.30 | **13,329,792 kB = 12.71 GiB** | 290 % | 179.83 | 41.59 | 721 |
| cold_query1 | `query "error handling" --limit 5` | 4,033-file cold corpus | 0 | **0:14.86** | 208.5 MB | **99 %** | — | — | — |
| cold_query2/3 | same, index now present | same | 0 | 0:00.15 / 0:00.17 | 89.8 MB | 103 % | — | — | — |

The last two rows are one paired probe from a second harness (`research/dup_probe.txt`), run
back-to-back on the same tree and the same binary; they are the measurement behind **CRUX-22**.
`--detection-type all` is the documented default, and it costs **10× the wall clock, ~216 extra
CPU-seconds and ~3× the RSS for 2,443 more blocks (+6.9 %)**. `--min-lines 20` cuts
`blocks_analyzed` 4.2× and changes wall and RSS *not at all* (17.28 s / 1.68 GiB), so the cost is
not in the block list.

**Measurement conditions, inlined so they survive the artefact directory (§1).** 1-minute load
average at each run's instant, from `research/m/<name>.meta`: `version1/2`, `help1`,
`fix_*`, `repo_query1/2` **32.14**; `repo_complexity`, `repo_deadcode`, `repo_satd` **0.76**;
`repo_tdg` **0.78**; `repo_dag`, `repo_duplicates` **3.21**; `repo_context` **3.32**;
`repo_qualitygate`, `repo_verify` **3.52**; `repo_dag_fixed` **2.71**; `cold_query1/2/3` **3.26 /
3.22 / 3.22**; `repo_score` **8.62**; `repo_score2` **7.18**. Sub-second wall deltas on this box
are scheduling noise at any of those loads; CPU-seconds and CPU % are the robust figures.

Three rows deserve their own note:

- **`analyze dag` exits 2 in 0.00 s because `--format` does not exist on it** — the only
  `analyze` subcommand without one (see CRUX-15). Re-run with the flag it accepts it costs
  1.47 s / 313 %, and emits Mermaid only.
- **`quality-gate` is a write command**: `💾 Persisted 37 violations to .pmat/context.db`.
- **The exit 1 is real but its `35` must be read with §8.24 beside it.** Of the 35 blocking
  violations, 28 are entropy, and **five of the six complexity blockers are at or below the repo's
  own configured `max_complexity = 30`** — the gate derives an undisclosed warn band at `limit - 2`
  and counts warnings as blocking. Quote 35 as the size of the verify-vs-gate divergence, never as a
  count of genuine violations.
- **The 20× fixture gap is scheduling noise, not a cache.** Run 1 spent **0.02 s of CPU across
  0.21 s of wall clock** at load 32.14; run 2 did *more* work (10 of 13 files vs 1 of 4) in
  0.01 s. A cache would show as fewer CPU-seconds; CPU-seconds are flat. Hypothesis raised and
  refuted. Fixed startup cost is ≈10 ms.

### 5.2 Parallelism: most commands use one of 48 cores

Ordered by CPU% from the table above: `analyze duplicates` 1995 % (~20 cores), `tdg .` 1768 %
(~18), `analyze dag` 313 %, `score` 270–290 %, then a cliff — `context` 119 %,
`analyze complexity` 114 %, `quality-gate` 107 %, **the cold index build 99 % (exactly one core)**,
warm `query` ~100 %, `verify` 97 %. The parallel machinery exists and works at 1995 %; the two
most user-visible latencies are the two that do not use it. Root cause for the complexity walk is
one line: `src/cli/analysis_utilities/quality_checks_part4.rs:153` carries the comment
`// PERFORMANCE OPTIMIZATION: Process files in parallel batches` over a bare
`futures::future::try_join_all` at `:168` — no `spawn`, no `spawn_blocking`, no rayon. `try_join_all`
polls a chunk's futures concurrently **on one task**, and the only await point in
`analyze_complexity_file` is `tokio::fs::read_to_string`
(`src/services/accurate_complexity_analyzer_core.rs:7`), which dispatches to the blocking pool. So the
file *reads* do overlap while every parse/annotate is serialised on that one task — which is exactly
what 110–113 % CPU looks like, against 1995 % for the rayon-based `analyze duplicates`. rayon is used
at 33 sites in 19 production files, none of them these walks.

**Scaling is linear at this size.** The quadratic behaviour in this project's history (#1059) was
looked for on a second corpus (1.12× the lines): `analyze duplicates` ratio 1.074 CPU / 1.107 RSS,
`analyze complexity` 1.19 CPU. Hypothesis raised and refuted at this scale.

### 5.3 Read amplification

`strace -f -e trace=openat`, `analyze complexity -p src/graph` (stderr: "Successfully analyzed
42 file(s)"): **113 openat on `.rs` paths, 87 successful, 26 ENOENT, 49 distinct files** —
`parallel_louvain.rs` and `builder.rs` opened 7× each, `types.rs` 6×. Two causes, both located:
the walk reads the content at `quality_checks_part4.rs:464` and then hands a *path* to
`src/cli/language_analyzer/mod.rs:34` → `src/services/ast_rust_compat.rs:22` →
`src/services/accurate_complexity_analyzer_core.rs:7`, which does
`tokio::fs::read_to_string(path)` again; and `is_included_by_sibling`
(`src/cli/language_analyzer/mod.rs:78-107`) strips one `_`-segment at a time and calls
`file_contains` — a **blocking** `std::fs::read_to_string` inside an async fn — twice per
candidate. `pmat context -p src/graph` is worse: 776 openat, 176 successful over the same 42
files. The live AST cache cannot compensate — `AstCacheStrategy::max_size()` is **100**
(`src/services/cache/strategies_ast.rs:105-107`) — but note it has only two callers, both in
`src/services/context_impl/visitor.rs`, so raising it would not move the complexity number
(a correction the VALUE lens returned against the original proposal).

### 5.4 Memory, and the hidden child

`pmat score` peaks at **9.76 GiB and 12.71 GiB** across two runs of the identical command in the
identical tree — a 30 % swing, at load 7.18 with 110 GB free, so not contention. Against
`.pmat-metrics.toml:59 max_memory_usage_mb = 512` that is **19×–26×**.

The cause is a child pmat never mentions: `src/cli/handlers/score_handler.rs:582` is
`std::process::Command::new("pmat").args(["comply","check","--format","json"])` with
`.stderr(Stdio::null())`, no timeout, and both failure arms collapsing to `COMPLY_UNMEASURED`.
`strace -f -e trace=execve` in a two-file scratch crate proves the consequence:

```
3500107 execve("/mnt/nvme-raid0/.../release/pmat", [..., "score"], ...) = 0
3500205 execve("/home/noah/.cargo/bin/pmat", ["pmat","comply","check","--format","json"], ...) = 0
```

The freshly built binary graded its own repo using a *different build* — `~/.cargo/bin/pmat`
reports `commit: unknown` and is three days older. This is a defect class this repo has already
fixed twice (`#956`, `#897`) and pinned with a source-level test at
`src/services/spec_falsification_tests.rs:564-575` whose comment reads "three call sites
regressed to `Command::new("pmat")` once already, and each one flips a verdict when PATH
changes" — the pin's glob simply does not cover `score_handler.rs`. Note also that
`check.rs:483-487` already prints comply's own "~N GB peak" disclosure to **stderr**, which
score sends to `/dev/null`.

Peak RSS against the 512 MB budget, from the same sweep, converted from `/usr/bin/time`'s kbytes at
1 kB = 1024 B so that these agree with §5.1: `analyze duplicates` **1,772,556 kB = 1,731 MiB =
1.69 GiB** (3.4×), `tdg .` **538,668 kB = 526 MiB** (1.03×), `verify` **307,924 kB = 301 MiB**,
`context` **284,820 kB = 278 MiB**, `query` (warm) 92 MiB. (An earlier draft carried 1.65 GiB /
513 MiB / 294 MiB / 272 MiB here — each the true MiB divided by 1.0243, i.e. kB read as 1000 bytes
and then converted to MiB. The `score` figures in the paragraph above were already correct.)

### 5.5 Build cost

| build | what it was | wall | user CPU | CPU% | max RSS |
|---|---|---|---|---|---|
| build1 | first `cargo build --release` of the session | 2:24.77 | 514.32 s | 368 % | 6.79 GiB |
| build2 | **immediately after build1, zero source changes** | **0:56.12** | 265.82 s | 496 % | 4.26 GiB |
| build3 | same again, `CARGO_LOG=…fingerprint=info` | **0:55.28** | 263.46 s | 499 % | 4.24 GiB |

`build3.sh` stat'd `.git/index` and `.git/HEAD` before and after: byte-identical. Cargo gives
the reason itself, and it is a single item:

```
dirty: FsStatusOutdated(StaleItem(MissingFile {
    path: "/home/noah/src/paiml-mcp-agent-toolkit/../assets/demo/" }))
stale: missing ".../../assets/demo/"
```

That path is `build.rs:21`. Line 20 already declares the real `assets/demo/`. **A no-op *release*
build costs 55 s wall / 4.4 CPU-minutes** — measured at 1-minute load **26.82** (build2) and
**26.44** (build3), i.e. under the shared CI pool, with build1 started at load 7.64
(`research/build{1,2,3}.start`).

**Scope that claim precisely.** All three timings are `cargo build --release`. The debug cost is
**UNVERIFIED**: the stale fingerprint forces the same rebuild-and-relink in any profile, but a debug
relink is not a 263-CPU-second optimised one, and no debug no-op build was measured. In particular
`pmat verify` is **not** a beneficiary of the release number and may not be one at all: its test
stage is `cargo test --lib` in debug (`src/cli/verify.rs:510-511`, with `RUST_MIN_STACK=8388608`),
its `cargo()` helper is a plain `cargo` (`:299`), and its clippy stage short-circuits on a receipt
before executing cargo at all — `verify_lint_receipt::is_proven` → `StageResult::ran(true, …)` at
`src/cli/verify.rs:553-558`, whose own comment quotes "~0.3s instead of the measured 1m06s". So
"every `pmat verify` pays 55 s" is false as stated; what is true is that every *release* build in
this tree, local or CI, pays it, and every debug `cargo build|test|clippy` pays an unmeasured
fraction of the same relink.

Two honest caveats the verification added. First, cargo **short-circuits on the first stale
item**, so items declared after line 21 — including `.git/index` at `build.rs:1678` — were never
evaluated; any `git add`/`git status`-after-edit re-triggers the same relink, so the win is "a
git-quiet tree builds incrementally", not "no-op builds are free forever". Second, CI blast
radius is limited: all 14 cache sites use `Swatinem/rust-cache@v2`, which evicts the workspace
crate anyway.

### 5.6 CI cost and the enforced band

One push to master at the audited commit: **43 jobs / 7.67 runner-hours / 49.0 min critical
path** (`gh api .../check-runs`). Required contexts: `ci / gate` 49.0 min (job duration 0.1 min),
`feature-gate` 24.9, `pmat score` 17.4, `provable ladder` 16.1, `docs build` 2.1. Longest jobs:
`ci / test` 32.9 m, `ci / coverage` 32.9 m, `mutants` 31.3 m (informational), shards 18.3–24.8 m,
`ci / lint` 17.9 m, `binary size band` 14.0 m.

**32 % of the critical path is queue wait for a six-second job.** `ci / gate` is a pure
`${{ needs.*.result }}` string comparison with no checkout and no cargo, and it runs on
`runs-on: [self-hosted, clean-room]` — the same 48-core box as `ci / test` and `ci / coverage`.
Two of the last four green master runs paid 15.7 and 18.8 minutes of queue for it. The fix is
one `runs-on:` line in `paiml/.github`, not in this repo (§9).

**26 of the last 60 master `ci.yml` runs were CANCELLED (43 %)** by
`concurrency: cancel-in-progress: true`, so master is verified by the *last* push in a burst.
This does not weaken the merge gate (different concurrency group); it weakens post-merge
verification.

Binary size is the one number that can genuinely fail a build, and only one of the three
declared limits is real:

| declared limit | where | value | verdict vs 54,633,288 | enforced? |
|---|---|---|---|---|
| `binary_max_bytes` | `.pmat-metrics.toml:12` | 50,000,000 | **over by 9.27 %** | **No** — nothing reads the key |
| `MAX_SIZE_BYTES` | `src/tests/binary_size.rs:40` | 52,428,800 | over by 2.2 MB | **N/A — the file no longer exists** |
| `EXPECTED_BYTES ± FAIL_PCT` | `tests/modules/binary_size_band.rs` | 55,000,000 ±20 % | inside; drift 0.67 % | **Yes**, via `feature-matrix.yml:651-652` into `feature-gate`'s needs |


### 5.7 Link surface, and the five denominators

Two facts belong beside §5.5 and §5.6, because they change how the numbers there should be read.

**(1) The link surface is 114 units against a 280 MiB rlib.** `cargo metadata --no-deps` counts
**137 targets**: 1 lib, 2 bins, 113 examples, 11 tests, 9 benches, 1 build script. Of those, 96
examples carry no `required-features` and so build on the default feature set; with 7 default-feature
test targets, 9 benches and 2 bins that is **114 separate link units** for
`cargo clippy --all-targets` — the CI lint command — each linking a `libpmat-*.rlib` measured at
**294,094,466 B (280 MiB)** and **299,361,140 B (285 MiB)** for the two hashes present.
`examples/` is 1.2 MB of source producing 96 binaries against that rlib. §5.6 records `ci / lint`
at 17.9 minutes and offers no explanation; this is the candidate. **How much of the 17.9 min the
114 links account for is UNVERIFIED** — the unit count and the rlib sizes are measured, the
attribution is not, and measuring it needs a `cargo` run this audit did not make.

Related, and measured: only **9** files sit at `tests/*.rs` while **258** `.rs` files live under
`tests/` (`find tests -name '*.rs' | wc -l`); the other 249 are modules reached from `tests/all.rs`,
which is the whole point of `autotests = false`. Three of the nine top-level files are declared
nowhere and are therefore **never compiled** — `tests/golden_trace_validation.rs` (whose own header
says `//! Run with: cargo test --test golden_trace_validation`, a target that does not exist),
`tests/lib.rs`, `tests/zero_warnings_tdd.rs`.

**(2) Five commands, five different denominators for one tree.** Every per-file rate quoted anywhere
in §5 inherits this caveat:

| command | analysed | denominator it reports |
|---|---|---|
| `analyze complexity` | 4,521 | 5,374 seen |
| `analyze dead-code` | 3,228 | 4,433 discovered |
| `analyze dag` | 4,487 | 5,481 scanned ("Limited to 10000 for performance") |
| `query` index | 4,477 files / 24,413 functions | — |
| cold `query` index (`src/` only) | 4,030 files / 23,563 functions | — |

**No two of these are comparable**, so no two throughput figures are either. The cause is
structural, not a rounding difference: complexity and dead-code are **two independent discovery
layers that share no code.** Path A (complexity) goes through the shared service —
`src/cli/handlers/complexity_handlers/analysis.rs:326` → `analyze_project_files`
(`quality_checks_part4.rs:106`) → `ProjectFileDiscovery::discover_files()`
(`src/services/file_discovery_walker.rs:7`) — filters by every supported language and applies **no**
test-file exclusion. Path B (dead-code) does not use the service at all: it hand-rolls a
`WalkBuilder` at `src/services/cargo_dead_code_analyzer/analysis.rs:112-118` (Rust only, plus
`is_excluded_source` at `:175-193`, which drops `tests/`, `examples/`, `benches/` and anything
matching `is_test_file_name` at `:570-580` — **on by default**) and a *third* walk at
`src/services/cargo_dead_code_analyzer/parsing.rs:293-299` purely to count lines, re-reading every
file at `:319`. Controlled proof on a 10-file polyglot fixture: complexity reports 9 analysed of 10,
dead-code reports 4 (6 with `--include-tests`).

This matters to **CRUX-13**, whose leg (c) asserts denominator parity between `analyze satd` and
`analyze complexity`. That leg treats complexity's denominator as the oracle. It is the *broadest*
of the five, which is why it is the right target for a SATD walk — but it is not a repo-wide truth,
and the ticket must say so. Unifying the two discovery layers is **not proposed** here (§9.2).

---

---

## 6. Competitive landscape

Every vendor cell below is sourced to a URL fetched **2026-09-02**; nothing is from prior
belief. `?` means "I could not source it" and is never a guess. pmat cells are measured against
the HEAD binary except where marked ESTIMATE.

### 6a. Platform code-quality suites

| | **pmat 3.34.0** | SonarQube/Cloud | CodeScene | Qlty (Code Climate successor) | DeepSource | Codacy |
|---|---|---|---|---|---|---|
| Languages | 20 claimed (`README.md:51`) | 35+ commercial / 20+ Community | 30+ | 40+ langs, 70+ plugins | 30+ | 40+ |
| Incremental / PR-diff | **P** — `analyze incremental-coverage` only; `quality-gate` has no `--diff` | Y — "Clean as You Code" | Y — PR gates on Code Health decline | Y (Cloud) | Y — inline PR comments | Y |
| SARIF upload | **P** — **17** `analyze` subcommands + `pmat tdg` emit valid SARIF; **0 CI steps upload**; the unique checks cannot emit it | Y | ? | ? | ? | ? |
| IDE / LSP | **N** — 0 LSP strings in the binary | Y | Y (VS Code / JetBrains, real-time) | ? | ? | ? |
| Baseline + ratchet | **Y, strongest in class** — `comply ratchet` re-RUNS each baseline's command; raising one needs a justification checked against the prior committed file; unmeasurable ⇒ FAIL | Y (new-code baseline) | Y | ? | ? | ? |
| Mutation testing | **N** — `pmat mutate` → exit 2 | N | N | N | N | N |
| Debt score comparable to TDG | Y — TDG A+..F, 6 metrics | Y (SQALE) | Y — Code Health, 25–30 factors | Y — A–F, debt ratio | Y | Y |
| AI-context / repo map w/ token budget | **N** | N (MCP context augmentation instead) | N | N | N | N |
| MCP tools | **19**, flat, no toolsets, no read-only mode | **40+** across named toolsets | ≥2 named; count ? | ? | ? | ? |
| Licence | MIT | SSAL v1.0 for the MCP server; Community free | proprietary SaaS | CLI BSL-1.1 → DOSP, free incl. commercial | proprietary | proprietary |
| Price | $0 | Community $0; Developer ~$720/yr | €18 / €27 per active author/mo | CLI $0; Team $15/contributor/mo | $24/user/mo | $15/user/mo |
| Install footprint | **54.6 MB binary** (measured) [^cache] | JVM server / Docker | SaaS | ? | SaaS | SaaS |
| Offline, no account | **Y — unique among the six** | N | N | CLI: Y | N | N |

[^cache]: The cache is **not** a first-run cost and the audit has no measurement of one. On *this*
repository — the most heavily dogfooded pmat checkout in existence — `.pmat/` had reached 728 MB
(`workspace.db` 582 MB, `context.db` 68 MB, `context.db.bak` 52 MB) beside a 48 MB source tree, and
growth is unbounded and never pruned (§9.2 item 6). n=1, after months of use; do not read it as what
a new user pays.

Sources: https://www.sonarsource.com/solutions/reduce-technical-debt/ · https://docs.sonarsource.com/sonarqube-server/user-guide/about-new-code · https://www.sonarsource.com/products/sonarqube/mcp-server/ · https://github.com/SonarSource/sonarqube-mcp-server · https://appsecsanta.com/sonarqube · https://codescene.com/product · https://codescene.com/product/code-health-mcp · https://codescene.com/pricing · https://github.com/qltysh/qlty · https://docs.qlty.sh/migration/overview · https://codeclimate.com/blog/code-climate-quality-is-now-qlty-software · https://dev.to/rahulxsingh/deepsource-vs-codacy-code-quality-tools-compared-2026-52dd

### 6b. Security / SAST

| | **pmat** | Semgrep | GitHub CodeQL |
|---|---|---|---|
| Languages | 20 claimed | 30+ | 12 GA incl. **Rust GA** (2021 & 2024 editions) |
| Diff-aware | P | Y (~10 s PR scans) | Y |
| SARIF upload | P (emits, never uploads) | Y | Y (native) |
| Licence | MIT | LGPL-2.1 (CLI) | proprietary; free code scanning for public repos |
| Price | $0 | ~$30/committer/mo (2026) | ~$30/committer/mo; $0 public repos |

Sources: https://github.com/semgrep/semgrep · https://konvu.com/compare/semgrep-vs-codeql · https://codeql.github.com/docs/codeql-overview/supported-languages-and-frameworks/ · https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/uploading-a-sarif-file-to-github

### 6c. Rust / single-purpose CLIs

| | **pmat** | rust-code-analysis | tokei / scc | cargo-mutants | cargo-semver-checks | cargo-geiger | cargo-llvm-cov |
|---|---|---|---|---|---|---|---|
| Scope | everything (71 subcommands) | 11 metrics × 10 langs | LOC + (scc) complexity/COCOMO | mutation | SemVer API breaks | `unsafe` per dep | coverage |
| pmat has an equivalent | — | Y (`analyze complexity`) | Y | **N** | **N** | **N** | P |
| Diff mode | N on gates | N | N | **Y `--in-diff`** + `--shard k/n` | N | N | N |
| Speed (published/measured) | 1.50 s complexity over 1.1 M lines (measured) | ? | scc 23.0 ms vs tokei 74.0 ms on Valkey (hyperfine, 32-core, 2026-03-05) | ? | ? | `--forbid-only` "seconds" | ? |

Sources: https://github.com/mozilla/rust-code-analysis · https://mozilla.github.io/rust-code-analysis/metrics.html · https://github.com/boyter/scc · https://boyter.org/posts/sloc-cloc-code-performance/ · https://mutants.rs/in-diff.html · https://mutants.rs/shards.html · https://mutants.rs/baseline.html · https://crates.io/crates/cargo-semver-checks · https://rust-lang.github.io/rust-project-goals/2026/cargo-semver-checks.html · https://github.com/geiger-rs/cargo-geiger · https://github.com/taiki-e/cargo-llvm-cov

### 6d. AI-context / repo-map / MCP context servers

| | **pmat** | Aider repo map | Serena MCP | Cursor index | Context7 | GitHub MCP | repo-context-mcp / repowise / jCodeMunch |
|---|---|---|---|---|---|---|---|
| Token budget | **N** | **Y `--map-tokens`, default 1k** | N (symbol-scoped) | n/a | n/a | n/a | **Y** — "token-aware pack … ~4 chars/token" |
| Ranking | PageRank exists, never spent on a budget | personalised PageRank seeded by the chat | LSP symbol graph | cloud embeddings | library-doc retrieval | n/a | entrypoint + focus ranking |
| Local / private | **Y, fully** | Y | Y | P — embeddings leave the machine | N (SaaS) | N | varies |
| MCP tools | 19 | n/a | ~30 lang servers | n/a | 2 | ~80 / 23 toolsets, `GITHUB_READ_ONLY=1` | 3 / 10 |
| Published efficiency claim | none | 1k-token default map | "far better context efficiency" | 100k-file monorepo in "a few minutes" | 9,000+ libraries | n/a | repowise **96 % fewer tokens (2,391 vs 64,039)**; jCodeMunch 86–99 % |

Sources: https://aider.chat/docs/repomap.html · https://aider.chat/2023/10/22/repomap.html · https://github.com/oraios/serena · https://cursor.com/blog/secure-codebase-indexing · https://towardsdatascience.com/how-cursor-actually-indexes-your-codebase/ · https://context7mcp.com/ · https://github.com/github/github-mcp-server · https://github.com/nduc99911/repo-context-mcp · https://www.repowise.dev/guides/ai-context-mcp · https://github.com/jgravelle/jcodemunch-mcp · https://github.com/zhang-liz/mcp-token-benchmark

**Google Antigravity is not a competitor; it is a host pmat should be a tool inside.** A free
VS Code fork with an agent-first Manager surface, split into four surfaces sharing one harness
as of AntiGravity 2.0 (2026-05-19). pmat already has an `agy` subcommand and a
`work handoff --agy` path — and §10 records `pmat init --target agy` as the best-behaved
first-run surface measured anywhere in this audit.

### 6e. Honest placement

**Where pmat is genuinely alone** (verified present on HEAD; no competitor equivalent found in
any source above):

1. `analyze reachability` — tracked `.rs` files no compilation unit reaches, with a **third
   state** (quarantined behind a non-compiling cfg) so "reachable" cannot be claimed for code
   nothing builds.
2. `analyze unrun-tests` — tests no CI leg executes, keyed on full module path, with a ledger.
3. `analyze vacuous-tests` — `#[test]`s that cannot fail. Prior art exists in research and one
   Go linter (https://github.com/the4thamigo-uk/noassert; academia at https://testsmells.org/pages/testsmells.html),
   but no commercial platform in 6a ships it as a gate.
4. `comply coherence` (CB-2101) — classifies every declared threshold FIRING / VIOLATED /
   **VACUOUS** against a live measurement. **No competitor found that audits its own config for
   thresholds that cannot fire.** This is the most differentiated idea in the product.
5. `comply ratchet` (CB-2102) — baselines re-derived by RUNNING the recorded command, with
   "unmeasurable ⇒ FAIL" and a justification checked against the previous committed file.
   Sonar's new-code baseline is a date; this is a command whose output is the baseline.
   Strictly stronger.
6. `comply numeric-claims` (CB-2104), `popper-score`, `analyze hardcoded-paths`, `verify`.
7. Fully local, no account, MIT, one static binary — the only entry in 6a/6b with that row.

**Where pmat honestly loses.** No LSP (every serious competitor has one — 0 LSP protocol strings
in a 54.6 MB binary). No PR-diff gate, against a market whose default is "Clean as You Code" —
on a 1.1 M-line repo pmat gates the whole tree, so it goes red for reasons the PR did not cause.
SARIF exists on 17 `analyze` subcommands plus `pmat tdg` but is unwired and misses the
differentiators. The mutation column is empty while
the README sells it seven times — a credibility loss on top of a capability gap. The context
product has no budget against a 1k-token incumbent default. The MCP surface exposes 8 of
`pmat query`'s 49 flags, with no toolsets and no read-only mode. Footprint is an order of
magnitude above the Rust-tool norm.

**One win worth publishing rather than fixing.** pmat's tool definitions cost ~10,230 bytes for
19 tools (≈2,557 tokens, chars/4 ESTIMATE; ~135 tok/tool). The published benchmark
(https://github.com/zhang-liz/mcp-token-benchmark) puts Notion at 24 tools / 17,161 tokens and
Firecrawl at 26 / 16,565 — a 25× spread — with "97 % is inputSchema". pmat sits at the lean end.

**Positioning erosion, stated plainly.** CodeQL Rust is GA and free on public repos; Qlty's CLI
is free for commercial use with 40+ languages; both incumbents shipped MCP servers into pmat's
quadrant during 2026. "Local Rust quality tool with an MCP server" is no longer empty. What is
still empty is **"sovereign, offline, gates that audit themselves"** — items 4 and 5 above — and
that is the position §8 is ordered to defend.


### 6f. The CRUX scorecard, applied by hand

The audit that gives this document its name has seven checks. All seven were run against the HEAD
binary; three FAIL and one is degraded, and every PASS below has a live positive control, because a
scorecard whose passes cannot fail is the defect this document is about.

| check | verdict | evidence (one line) | disposition |
|---|---|---|---|
| C1 exit-nonzero-on-garbage | **PASS** | `pmat zzz-crux-notacommand` → rc=2, 0 B stdout; control `pmat --version` → rc=0 | — |
| C2 stdout-clean-on-failure | **FAIL** | `pmat` with no args and stdin not a tty → rc=2 with **8,998 B of help on STDOUT** (re-measured; the research run read 8,989 B) and 123 B on stderr; separately `pmat -q tdg <missing>` prints `0.0` on stdout at rc=5 | the `tdg` half is CRUX-17; the **no-args half is CRUX-27** |
| C3 ansi-off-when-piped | **PASS (armed)** | six commands emit 20–69 `ESC[` sequences under a pty and **0** when piped — the pty leg is the positive control that proves the check can fire; `--help` UNMEASURED because clap's `color` feature is off (CRUX-05) | — |
| C4 surface-removal | **degraded** | no `completion` subcommand, no `clap_complete`; discovery is a help-walk only | §9.2 item 4 |
| C5 peer-flag semantics vs `rg` | **FAIL** | `pmat query --literal <impossible>` exits **0**; `rg -q` and `grep -rq` exit 1, and CLAUDE.md maps pmat query onto them flag-for-flag. `--limit` is ignored in two of four modes: `--files-with-matches --limit {1,3,50}` → **1,611 lines** each and `--count --limit 3` → 1,611, against a working control (`--limit {1,3,10}` → 53 / 114 / 419 lines) | **CRUX-26** |
| C6 clean-cwd-on-success | **PASS** | five commands run from an empty directory leave it empty — but they write into the *analysed* directory, which is CRUX-02/§5.1's "`quality-gate` is a write command" | — |
| C7 mutating-verb preview | **FAIL** | Re-measured verb by verb: **7 of the 17 probed** advertise `--dry-run` — `refactor auto`, `refactor docs`, `work migrate`, `enforce extreme`, `maintain cleanup-resources`, `kaizen`, `split`. **Ten do not** — `hooks install`, `hooks uninstall`, `hooks refresh`, `agy sync`, `scaffold project`, `config set`, `work delete`, `refactor interactive`, `embed sync`, `init`. `pmat hooks install` writes into `.git/hooks` with no way to see what it would write first. **CORRECTION to the research census:** `pmat cache clear` **does not exist** (`pmat cache` offers only `stats` and `help`; `pmat cache clear` → `error: unrecognized subcommand`), and `embed`'s verbs are `sync`/`status`/`clear` — not "index / extract / localize / generate template" — with `embed clear` gated behind `--confirm`, a guard rather than a preview | **CRUX-27** |

`pmat init` is the one command that earns C7's *intent* without the flag — it is idempotent and
non-destructive with a measured md5-stable `1 kept` path (§10, #1031) — which is why CRUX-27 accepts
either a preview flag or that behavioural evidence.

### 6g. Other dimension findings, measured and not proposed

§4 and §5 are the only per-dimension findings sections, so results from the other eight dimensions
survive only where a §8 item happens to cite them. These did not become items and are recorded here
rather than lost — including the positives, because §12.1 forbids one-sided reporting.

| # | finding | evidence | why not a §8 item |
|---|---|---|---|
| 1 | **The sovereign replacement the doctrine names by example is finished and shipped disabled.** `src/utils/sovereign_compression.rs` (9,668 B) is a complete `trueno-zram-core` adapter behind `sovereign-compression = ["aprender-zram-core"]` (`Cargo.toml:640`), a feature in **neither `default` nor `full`** — while `lz4_flex` (`Cargo.toml:207`) ships inside `standard-deps`, which *is* in `default` (`:461,466`), with three non-adapter call sites (`function_index/build_persistence.rs`, `tdg/storage_impl.rs`, `tdg/storage_impl_tiered.rs`) | file read + `Cargo.toml` | governance, not product behaviour (§9.2 item 11) — but the drop reason there says "the actionable half should ride with CRUX-14" **without** saying the replacement is already written. It is. Flipping one feature flag is a day's work, not a build. |
| 2 | **docs.rs is genuinely fixed, and its gate is the house model.** `docs build (docs.rs environment)` is a required context (§5.6) that reproduces the docs.rs environment rather than asserting a proxy — the pattern every other gate in §8.14 lacks | branch-protection contexts | a positive; nothing to fix |
| 3 | **Feature-gated and unimplemented surfaces are disclosed unusually well.** `pmat --help` marks three top-level commands `[NOT AVAILABLE in the default build]` *with the enabling feature*, and the pattern repeats at depth (`analyze wasm`, `tdg dashboard`, `debug serve`, `report --format html\|pdf\|dashboard`, `serve --transport …`). Behaviour matches the label in every case probed | 4 live runs | a positive worth not regressing. **One inconsistency, folded into CRUX-17:** the same "not implemented" refusal exits 1 (`demo`, `report --format html`) and 2 (`debug serve`, `serve --transport web-socket`), and 2 is clap's per `src/cli_exit.rs:40` |
| 4 | **`pmat init` is the best-behaved first-run surface measured anywhere in this audit** — see §10 (#1031). Against it, **`pmat query` writes 172 KB into your cwd unasked** on first use | live runs | the write is CRUX-07/§9.2 item 6 territory; the disclosure gap alone is not worth an item |
| 5 | **Two self-falsified index claims, reported because they were looked for.** (a) A read-only `pmat query` does **not** bump `.pmat/context.db`'s mtime, so it does not reset CB-130's 24 h freshness clock — predicted and refuted by `touch -d` + `stat`. (b) "There is no state GC" is **false**: `pmat maintain cleanup-resources` exists, and cleans 62 GB — of which **zero bytes** are pmat's own 1.63 GiB | 2 fixtures | (a) is a non-finding; (b) is §9.2 item 6, and the corrected form is what makes that item strong |
| 6 | **231 async fns in non-test `src/` contain blocking calls** (`std::fs::*`, `Command::output()`, `WalkBuilder`, rayon, `std::sync::Mutex`) | `research/scan_async_sync_io.py` | harmless for one-shot CLI runs; **impact on `pmat serve` UNVERIFIED** — the server was never loaded. Do not file it until someone measures a stalled worker. |

---

---

## 7. Reconciliation with prior audits

### 7.1 fable-review.md §4 — all 24 backlog items at this tree

**Tally: 6 SHIPPED, 8 PARTIAL/mostly, 9 OPEN, 1 obsolete-in-part.**

| rank | id | verdict | evidence |
|---|---|---|---|
| 1 | unbypass-required-checks | PARTIAL | phantom contexts gone; `OrganizationAdmin bypass_mode: always` and `enforce_admins: false` unchanged |
| 2 | run-declared-test-targets-in-gate | PARTIAL | `ci / test` still `--lib`; two `--test all` single-test filters now run inside `feature-gate` |
| 3 | revive-orphaned-test-corpus | OPEN (now measured) | 407 orphans / 6,292 tests; no CI leg reads `analyze reachability` → **CRUX-12** |
| 4 | satd-gate-red-path | SHIPPED | the item's own falsifier now yields 1 violation, `--fail-on-violation` exits 1, verify reads the JSON. Residual is the *classifier* axis → **CRUX-01(b)** |
| 5 | mutation-kill-path | PARTIAL | CLI routed to `MutantExecutor`; 5 `passed:true` adapters remain; `pmat mutate` not in the default build → **CRUX-16** |
| 6 | mutation-meta-test | OPEN | no weak-suite meta-test found |
| 7 | comply-fresh-clone-parity | PARTIAL | `kani_harnesses` schema fixed; hermeticity UNVERIFIABLE (comply forbidden by the audit's own rules) |
| 8 | comply-becomes-required | OPEN | `quality-gate.yml:293 continue-on-error: true` |
| 9 | coverage-floor-ratchet | OPEN | upstream step is well built; pmat never sets `coverage_min` → **CRUX-14(1)** |
| 10 | kani-context-goes-real | OPEN | no kani job in any workflow |
| 11 | context-output-fixture-gate | OPEN | no production-formatter content assertion found → blocks/bundles with **CRUX-18** |
| 12 | mcp-inventory-parity | PARTIAL | 3 new compiled guards incl. description parity; **schemas still unpinned** → **CRUX-09** |
| 13 | work-complete-idempotence | PARTIAL | production `test_github_sync` + `is_pmat_owned_state` under test; 3 scenarios missing |
| 14 | dead-code-recall-corpus | MOSTLY DONE | seeded dead fn found, gate exits 1; **the item's own `__`-prefixed falsifier was itself unfalsifiable** — and CRUX-04 suggests re-running it with the cache deleted before building a corpus against a phantom |
| 15 | complexity-single-oracle | SHIPPED in effect | `--file` / `--path` / quality-gate all report 37 on the 36-branch fixture; no parity *test* found |
| 16 | msrv-pr-gate | OPEN | MSRV job is `on: release: published` only → **CRUX-14(2)** |
| 17 | mutation-ci-threshold | OPEN | `mutants` job double `continue-on-error`; ci.yml's own comment says it "has never executed a single mutant" |
| 18 | sovereign-ci-hardening | PARTIAL (3/4) | fmt, deny and the gate comparison fixed; clippy fallback still drops `--all-targets`; `bans` absent upstream |
| 19 | clean-room-publish-gate | SHIPPED in substance | `package-size` leg runs `cargo package` blocking, inside a required context |
| 20 | aprender-family-convergence | SHIPPED | family uniform at 0.64, arrow 59, `cargo tree -d` empty for that family |
| 21 | org-analyze-resolution | MOSTLY SHIPPED | E0004 fixed, ledger row in feature-matrix |
| 22 | self-metrics-honesty | PARTIAL | hardcoded 85.0 removed; binary size now enforced by a *different* mechanism than the one CLAUDE.md cites |
| 23 | docs-accuracy-required | OPEN | `validate-readme` is in the Makefile and pre-commit only, in no workflow |
| 24 | quality-gate-exit-semantics | SHIPPED | default exit 1 on blocking violations, `--report-only` opts out. Residual: `ExitCode::QualityGateFailure = 3` still has no raise site → **CRUX-17(2)** |

**§5 (do-not-do) compliance.** Held, with one standing violation (an eighth score command
family persists). Two §8 items sit adjacent to bans and are explicitly *not* banned:
CRUX-01 keeps exit 0 and changes only what the JSON asserts, so it does not "fix the verify
complexity gate's changed-files scoping to full-repo"; CRUX-18 adds a flag to an existing
command and a parameter to an existing tool, so it does not "expand AI-context scope (new
formats, more MCP tools)" — though its sequencing note (§8.18) records that
`context-output-fixture-gate` should land in the same PR, because a *truncating* formatter is
the single change most likely to silently drop content on a surface nothing asserts.

### 7.2 Issue #1114 — every checkable claim, re-measured

`ls -d server` → *No such file or directory*. Every `server/`-prefixed path in the issue is a
**STALE PATH**; the table says for each whether the *claim* survives the path move.

| # | claim | verdict | measurement |
|---|---|---|---|
| 1 | `server/src/cli/analysis_utilities.rs` is 10,478 lines | **FALSE (path) / TRUE and worse (claim)** | it is a directory: 70 files / 21,983 lines; the **include! closure** of `mod.rs` is 16,789 lines (§3 A2). Largest single file anywhere in `src/` is `src/mcp_pmcp/tool_functions/analysis_tools.rs` at 2,694 lines |
| 2 | `deep_context.rs` is 6,114 lines | **FALSE** | no such file; largest `deep_context*` file is `src/services/deep_context_tests_part4.rs`, 715 lines. Restated: two parallel implementations totalling 11,802 lines |
| 3 | `Commands` is an 895-line enum | **TRUE, and worse** | `src/cli/commands/commands_enum/definition.rs`, `pub enum Commands` → closing brace = **1,822 lines** (file 2,367), 71 variants |
| 4 | ~3,176 transitive dependencies | **FALSE as stated** | `grep -c '^\[\[package\]\]' Cargo.lock` → **847**; host-filtered default graph → 489. 3,176 is plausibly a `cargo tree` *line* count (edges) |
| 5 | four definitions of `McpRequest`/`McpResponse` | **TRUE, verbatim** | `src/mcp_integration/types.rs:66,82`; `src/demo/adapters/mcp.rs:14,23`; `src/agents_md/bridge.rs:97,153`; `src/models/mcp_types.rs:3,12` |
| 6 | sync IO inside `async fn discover_project_structure` | **TRUE, path moved** | `src/services/deep_context/analyzer_core/file_tree.rs:14` → `build_file_tree_recursive` (`:32`) whose first statement is `std::fs::metadata(path)?` |
| 7 | `cargo_dead_code_analyzer` does sync IO in async | **STALE-PATH / UNVERIFIED** | live references are test fixtures; no such async fn located |
| 8 | tools block on `crossbeam_channel::recv()` in async fns, "risking immediate deadlocks" | **FALSE as stated** | the recv is at `src/services/file_discovery_walker.rs:98`, inside the **sync** `fn walk` (`:30`), after `walker.run()` returns and `drop(tx)` (`:95`) — `pub fn discover_files` (`:7`) only calls `walk` — it drains a closed channel and cannot deadlock. The real, smaller hazard is a sync caller blocking a runtime thread |
| 9 | `O(E×V)` string allocations in `resolve_import_to_node` | **TRUE** | `src/graph/builder_analysis.rs:142-152` loops every `node_map` entry per import, allocating in `path_to_module` per candidate. (Compiled, but no production caller) |
| 10 | `O(N²)` **string cloning** in LCS duplicate detection | **HALF FALSE** | `src/tdg/scorers/duplication_analysis.rs:145-161` is a textbook DP LCS over an **integer** table comparing by reference. Quadratic time/memory: true. String cloning: **not present** |
| 11 | `estimate_complexity` calls `content.lines().collect::<Vec<&str>>()` per function | **NOT FOUND at the named shape** | 10 `estimate_complexity` definitions; the graph one (`src/graph/builder_analysis.rs:122`) iterates lazily with no `collect`. The issue names no file |
| 12 | 193 `.rs` in `server/tests/`, each its own integration binary | **file count TRUE and now larger / "each its own binary" FALSE** | `find tests -name '*.rs' \| wc -l` and `git ls-files 'tests/**/*.rs' 'tests/*.rs' \| wc -l` both → **258** (the `ls tests/*.rs \| wc -l` → 9 that an earlier draft published as the refutation is non-recursive and measures a narrower thing). The 193 is not invented either: `Cargo.toml:28` in this tree literally reads `# PERF: Disable autotest discovery - 193 .rs files = 193 binaries = slow coverage`, so the issue quoted pmat's own comment. Only the binary half is false: `grep -c '^\[\[test\]\]' Cargo.toml` → **11** declared targets under `autotests = false`, which has been in `Cargo.toml` since **`2aa5832a1` (2026-01-14, the single-crate refactor)** — `git log -S'autotests = false' -- Cargo.toml` returns two commits and `git show 2aa5832a1:Cargo.toml \| grep -n autotests` puts it at line 17 of that revision; `5249d4723` (2026-08-17) only added a prose mention elsewhere in the file. That is **seven months before #1114 was filed**, not two weeks |
| 13 | "zero pre-commit quality gates" | **FALSE** | `.git/hooks/pre-commit` is a 285-line generated PMAT gate with `set -e`; `scripts/install-git-hooks.sh` is 421 lines. Caveat: hooks are per-clone |
| 14 | many `docs/specifications/` files are empty stubs (<250 B) | **FALSE** | `find docs/specifications -name '*.md' -size -250c \| wc -l` → **0** of 41 |
| 15 | version drift: Cargo.toml 2.213.4 vs Docker 2.10.0 | **TRUE for the drift, stale for the number** | Cargo.toml is 3.35.0; `docker-compose.yml:6,24,42` and `docker-publish.yml:42` still hardcode `paiml/pmat:2.10.0` — a 3-major-version-stale tag republished on every push |
| 16 | 5+ conflicting workflows, 2400-line Makefile, 85+ ad-hoc scripts | **TRUE, numbers now** | 14 active `.yml` + 4 `.disabled`; `wc -l Makefile` → **2,789**; `ls scripts \| wc -l` → **90** |
| 17 | `PmatError` bypassed for `anyhow::Result` "over 650 times" | **TRUE, and 4× undercounted** | 889 `anyhow::Result`, 2,684 `anyhow` tokens; `PmatError` defined once at `src/models/error.rs:82` with **zero production callers** |
| 18 | "excellent elimination of panicking contexts in core logic" | **FALSE** (the issue's own positive finding) | `max_unwrap_calls = 100` is VIOLATED against a measured **20,390** — the one threshold `comply coherence` reports as firing-and-violated |
| 19 | Formal Verification scored 18.8 % "(missing provable-contracts)" | **UNVERIFIED** | needs `pmat rust-project-score`, which builds. The premise is wrong at HEAD: provable-contracts is a live path dep checked out by `quality-gate.yml:33-47` |

**Net: 7 TRUE, 5 FALSE, 3 half-true, 4 stale-path/unverified.** Two of the three headline
architectural findings (the 10 KLOC god module, the 6 KLOC `deep_context`) are FALSE as filed at
HEAD — though claim 1's *substance* is true and worse once `include!` is resolved — and the third
(193 test binaries) is half-true: the file count is right and now larger, the "each its own binary"
is wrong and has been for seven months. See §10 for the disposition.


### 7.3 The nine open issues the brief named, in one place

§8 disposes of these in scattered "Related issue" lines. This table is the index, so a reader can
see status without mining twenty items. Every verdict was re-measured on this tree.

| issue | subject | verdict at `512c60139` | evidence | where it lands |
|---|---|---|---|---|
| **#1090** | Epic: MCP hardening | **OPEN and reproducing.** HTTP still answers `{"jsonrpc":"1.0"}` with `200` + the full 19-tool result; `quality_proxy` still returns `accepted` for `operation:"write"` with no file created (7 calls, 0 files). Its own final comment says the guard "was written and withdrawn before merge" (`493f3d355` removed `http_frames.rs`, 578 lines) | §8.8, §8.10 | CRUX-08, CRUX-10 |
| **#1074** | cargo-deny blind to GHSA-only advisories | **OPEN, and the gate now boasts about it.** `gh secret list` → `CARGO_REGISTRY_TOKEN`, `NPM_TOKEN` only; the `dependabot-alerts` leg warns "the LIVE Dependabot arm did NOT run" and exits 0, while `feature-gate` (`feature-matrix.yml:807` needs it, `:818` echoes) asserts "no Dependabot advisory is open that cargo-deny cannot see" | §8.14 leg 3 | CRUX-14b |
| **#1035** | not-measured rendered as clean, 5 clusters | **OPEN, and this audit adds four fresh instances** — `quality-gate`'s dead-code arm, its duplicates count, `pmat tdg`'s directory walk (`files_ungraded: 0` after dropping 44 of 70 files), and `check_ladder_parses` reporting the population it did not inspect | §8.1, §8.2, §8.11, CRUX-25 | CRUX-01/02/11/25 |
| **#1034** | comply feature backlog | **OPEN; item (a) still false, and worse than stale.** `.github/workflows/mutation-diff.yml:50-66` declares itself a "REQUIRED CHECK (PMAT-630 B2)" and quotes, as "Verified against the live setting", a contexts list containing `mutation-diff`. Live protection is `["ci / gate","feature-gate","docs build (docs.rs environment)","pmat score","provable ladder"]` — no `mutation-diff`. And `.github/required-status-checks.txt:21-24` gives as its reason that the producing workflow "exists only on `feat/cb-2100-gate-effect` and is not on master" — the file **is** tracked on master (`git ls-files .github/workflows/mutation-diff.yml`). Two tracked files, mutually contradicting, both false against the live setting. Item (b), `quality-gate.yml:292-294 continue-on-error: true`, unchanged | this audit's own thesis, sitting in the enforcement metadata | **CRUX-14 leg 7** (new) |
| **#1019** | fleet epic: dead config | **OPEN.** `pmat.toml` carries `min_pattern_diversity`, `max_pattern_repetition`, `max_entropy_violations` under `[quality]`, none of them `QualityConfig` fields; `config --validate` prints "Configuration is valid … Custom Settings: 0" while `quality_gate_config.rs:155-161` actually reads `min_pattern_diversity` | §8.3 | CRUX-03 |
| **#1018** | fleet epic: tests that cannot fail | **OPEN.** 14 `contains("Usage:")` guards that a broken build satisfies (one of them product code); `CRITICAL_DUPLICATES` empty and iterated; `test_document_unavoidable_duplicates` is six `println!` and no assertion | §8.5, §8.14 legs 4-5 | CRUX-05, CRUX-14a |
| **#1017** | fleet epic: unreachable code | **OPEN; AC4 explicitly NOT MET** per its own triage comment ("no hook, no CI job, no verify stage"). 407 orphans / 6,292 tests; `grep -rn reachability .github/workflows/ Makefile` → nothing. `src/protocol/` and `src/state/` are a *second*, disjoint class this issue does not cover: reachable-because-compiled, referenced by nothing | §8.12, §4 A4 | CRUX-12, **CRUX-23** (new) |
| **#1014** | comply check RAM ~4 GB × CPU | **OPEN, and it is most of `pmat score`'s 9.8–12.7 GiB.** `COMPLY_BYTES_PER_WORKER = 4 GiB` (`check.rs:390`); `comply_concurrency()` sizes jobs from *available* RAM at start (`:449`), which is consistent with the 30 % run-to-run swing. Score's own defects are separable and are what CRUX-19 fixes | §5.4, §8.19 | CRUX-19 (context only) |
| **#999** | agent integration EV-0..EV-6 | **OPEN.** The discovery surface it depends on is the one `Cargo.toml:226` disabled: 0 of 71 subcommands print a usable `Usage:` line, no suggestions, no completions | §8.5, §9.2 item 4 | CRUX-05 |

---

---

## 8. The enhancement backlog

Ordering rule, inherited from fable-review §2: **a falsifiable gate that cannot currently fail
outranks a speedup, which outranks ergonomics, which outranks breadth.** Within that, rank by
(blast radius × confidence) ÷ effort. Effort: **S** ≤ 1 day, **M** ≤ 1 week, **L** > 1 week or
multi-PR.

**The numbering is not the ranking, and two items sit above the rule deliberately.** CRUX-01..20
are numbered in the order they were merged, and CRUX-21..32 were added after the first
verification round; a reader must not infer that CRUX-14 is worth less than CRUX-05 because it is
later. The two departures from the ordering rule are stated rather than hidden:

- **CRUX-06 (a build speedup) is ranked above every gate item** because it is a cost-of-work
  multiplier: one deleted line, and it makes every other item's edit→verify loop cheaper. §12.3
  lands it first regardless. Its justification is scoped in §5.5 — a *release* no-op build, not
  every cargo invocation.
- **CRUX-05 (help text — ergonomics by the taxonomy) is ranked above CRUX-11/12/14** because
  `src/docs_enforcement/cli_checker_validation.rs:74` makes it *also* a gate-integrity item: pmat's
  own product code certifies its own empty Usage sections as present. Without that line it would
  rank below them.

Every other item obeys the rule. Where two items are close, the tiebreak used was
(blast radius × confidence) ÷ effort in that order: a defect every *pmat user* can hit outranks one
only this repo hits (which is why CRUX-11 outranks CRUX-12), and a defect whose confidence is 1.0
from a reproduced fixture outranks one inferred from source.

**On the acceptance tests below — a correction to this document's own methodology claim.** An
earlier draft asserted that "every acceptance test below was RUN against the HEAD binary during
verification and fails today". That is **false as stated** and is withdrawn. Each item's *assertion*
was checked against the HEAD binary or the tree; the *snippet as written* was not always executed,
and three snippets were subsequently found to be **green on the unfixed binary** — which is the
"acceptance test that could not fail" class §3 counts three times and §12.2 rule 1 forbids. All
three are fixed in place and the transcripts are quoted at the item:

| snippet | what it did on the unfixed 3.34.0 binary | fixed at |
|---|---|---|
| CRUX-07 leg (c), `jq -S 'del(.generated_at)' \| sha256sum \| sort -u \| wc -l == 1` | **GREEN.** Six runs give six distinct raw hashes; the same six through `jq -S` give **one**. `jq -S` sorts object keys recursively, i.e. it normalises the exact variable under test | §8.7 |
| CRUX-05 leg 5, the `for c in …; do … \|\| echo "MISSING $c"; done` loop | **printed 71 `MISSING …` lines and exited 0** — a `… \|\| echo …; done` loop exits with the status of its last command, the identical defect §3 and §8.6 diagnose | §8.5 |
| CRUX-12's three-line snippet | ends in a bare `jq` with no `-e`, so it returns 0 whatever reachability reports | §8.12 |

**Shell discipline, now mandatory for every snippet here.** Each takes the CRUX-06 shape —
`set -euo pipefail` plus `fail(){ echo "FAIL: $*"; exit 1; }`, with each leg written as
`cmd || fail "leg N"` and every `jq` predicate given `-e`. Legs that merely printed a complaint have
been rewritten to accumulate into a variable and assert emptiness. This is not decoration: §8.1,
§8.2, §8.12, §8.15 and §8.19 all chain multiple legs, and without `set -e` only the last leg's
status escapes.

**One-shot evidence vs permanent gate.** Say which, per leg, before implementing. Legs that pin a
whole-tree census constant — CRUX-20's provenance objects, CRUX-11's flattened score, CRUX-12's
orphan counts — are **one-shot PR evidence**: committed as a CI gate they go red on the next
unrelated commit and get re-baselined by whoever is on call, which is the ratchet erosion this
document warns about. What may become a permanent gate is the **invariant**, never the count:
`unrecorded == 0`, `ast + heuristic_include_fragment + heuristic == files_analyzed`, provenance
equal before and after the change measured *in the same run*, ledger conservation. Every item below
that pins a literal total is marked accordingly.

The observed "today" value is quoted beside each assertion so a reviewer can tell a fix from a
re-interpretation. Every correction the three lenses returned is applied — where a correction
changed the *cause*, the *fix* changed with it.

**Path shorthand.** After a full path is given once, later citations in the same item use the
basename. Full paths, all verified to exist at `512c60139`:
`accurate_complexity_analyzer_core.rs`, `ast_rust_compat.rs`, `canonical_query.rs`,
`quality_proxy_analysis.rs`, `quality_proxy_operations.rs`, `tdg_calculator_core.rs` →
`src/services/…` · `build_helpers.rs`, `build_incremental.rs`, and the index `build.rs` →
`src/services/agent_context/function_index/…` (the crate's own `build.rs` is at the repo root) ·
`builder.rs`, `parallel_louvain.rs` → `src/graph/…` · `cache_operations.rs` →
`src/services/cargo_dead_code_analyzer/…` · `check.rs` →
`src/cli/handlers/comply_handlers/check_handlers/check.rs` · `classifier.rs`,
`detection_analysis.rs`, `detection_extraction.rs`, `detection_file_discovery.rs` →
`src/services/satd_detector/…` · `cli_mcp_surface_parity_tests.rs`, `http_server.rs`,
`quality_proxy_handler_impl.rs`, `simple_unified_server.rs`, `stdio_frames.rs`,
`tool_manifest.rs` → `src/mcp_pmcp/…` · `configuration_handlers_validation.rs`,
`quality_gates_handler_execution.rs`, `score_handler.rs`, `score_handler_display.rs`,
`work_tests_part1.rs` → `src/cli/handlers/…` · `definition.rs` →
`src/cli/commands/commands_enum/definition.rs` · `engine.rs`, `go_adapter.rs`, `rust_adapter.rs`,
`scoring.rs` → `src/services/mutation/…` · `quality_checks_part*.rs`, `quality_gate_config.rs`,
`quality_gate_part2a.rs`, `quality_gate_project.rs`, `churn.rs`, `tdg_formatting.rs` →
`src/cli/analysis_utilities/…` · `visitor.rs`, `context_impl/build.rs` →
`src/services/context_impl/…` · `binary_size_band.rs`, `dependency_duplicates_test.rs`,
`quality_gate_exit_status.rs` → `tests/modules/…` · `ci.yml`, `feature-matrix.yml`,
`post-release.yml`, `quality-gate.yml`, `docker-publish.yml`, `nightly.yml` →
`.github/workflows/…`; `sovereign-ci.yml` lives in `paiml/.github`, not this repo. Bare `mod.rs`
means `src/cli/analysis_utilities/mod.rs` in §4/§8.11 and `src/cli/language_analyzer/mod.rs` in
§5.3/§8.20.

---

### 8.1 CRUX-01 — `pmat verify` reports `ok: true` on a tree `quality-gate` fails with 35 blocking violations

**Problem.** CLAUDE.md makes `pmat verify` MANDATORY before every agent commit and promises "green
here ⇒ green in CI". On the clean tree it returns `{"ok": true, "stages_measured": 2}`, exit 0, with
complexity reporting `ok: null, not_applicable: "no Rust files changed vs HEAD…"` and **no
`not_measured` key**, while `quality-gate` exits 1 with 35 blocking violations. Two causes: (a) the
composite never consults a stage that declined; (b) the SATD stage passes `--strict`, which finds 0
where the default finds 3.

**Evidence.** `src/cli/verify.rs:194` `ok: !failed && measured > 0` (`:193` is the `VerifyReport {`
line itself); the guard `let not_measured = (measured == 0).then(...)` at `:189-192` fires only at
`measured == 0`, and `not_measured` (`:131`) is `Option<String>`, not a list.
`VerifyReport.ok` is `bool` (`:126`); only `StageReport.ok` is `Option<bool>`. `StageResult::
NotApplicable` shipped in v3.30.0 (`125a7b73e`) with the all-or-nothing rule only. Verify's SATD
call is `:456-463` (`analyze satd --strict --format json --fail-on-violation`); `satd_verdict`
(`:471-474`) passes on `Some(0)`. Repo: `analyze satd` → 3 (`quality_checks_part4.rs:117` `Bug:`,
`canonical_query.rs:52` `todo!()` in a `//!` example, `tdg_calculator_core.rs:110` `TODO(CB-128):`);
`--strict` → **0**. `quality-gate --format json` → `.results.blocking_violations 35`,
`.total_violations 37`, 14 `severity=error`.

**CORRECTION — the stated cause of (b) was wrong, so the fix changes.** Not "strict misses trailing
comments". Measured, one file per row: trailing `// TODO:` → strict **1**; trailing `// FIXME:` →
**1**; line-initial `// TODO(x):` → **0**; `// TODO[x]:` → **0**; `// TODO! ` → **0**; `// Bug:` →
**0**; `// BUG:` → **1**. Position is irrelevant. `MarkerMode::Strict`
(`src/services/satd_detector/classifier.rs:244-250`) requires `head == marker` (exact upper case,
`STRICT_MARKERS` `:120`) **and** `rest.strip_prefix(':')`; `Standard` (`:262-265`) accepts any of
`SEPARATORS = [':','(','[','!']` (`:139`) in any case. The blind spots are the **separator** and the
**case** — exactly why strict misses both markers quality-gate blocks on (`TODO(CB-128):` paren;
`Bug:` case, and that one is the `severity=error`). The proposed trailing-comment fix repairs
neither.

**Proposal.** (a) When any selected, non-skipped stage returns `NotApplicable`, set top-level `ok`
to `null` and emit `not_measured` as a **list of stage names** (`Vec<&'static str>` — a field-type
change from `Option<String>`). Keep exit 0: the command stops asserting safety, it does not start
failing. (b) Let `Strict` accept the same `SEPARATORS` as `Standard` while keeping the upper-case
marker and non-empty work-item requirement, so `TODO(CB-128): x`, `TODO[CB-128]: x` and `TODO: x`
match while `todo:`, `TODO x` and "this is a todo list" do not; decide explicitly whether lowercase
`Bug:` is debt (verify and quality-gate disagree today). Fix the `--strict` help string —
`STRICT_MARKERS` is five markers including XXX and imposes a `: <work item>` shape the help never
mentions.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
pmat verify --skip clippy,tests --format json | python3 -c '
import sys,json; d=json.load(sys.stdin)
assert d["ok"] is None and d["not_measured"]==["complexity"] and d["stages_measured"]==2' \
  || fail "leg 1: verify still asserts over a stage it declined"
# scratch crate: pub fn f() -> u8 { let x: u8 = 0; // TODO(CB-9): finish this
pmat analyze satd --path "$D" --strict --format json | jq -e '.total_violations == 1' \
  || fail "leg 2: strict still misses the paren separator"   # today 0
```
*Anti-vacuity (all four required).* **A1** blocks "always null": with one real Rust edit vs HEAD,
complexity RUNS → `isinstance(d['ok'], bool)` and `'not_measured' not in d`. **A2** blocks deleting
a stage: `[s['name'] for s in d['stages']] == ['format','complexity','satd','clippy','tests']`.
**A3** blocks `strict := default`, the lazy fix — **split into a mandatory half and a decided
half, because the merged single control forecloses the decision the proposal leaves open.**
*Mandatory (separator clause):* `TODO(CB-128): x` and `TODO[CB-128]: x` must match under Strict
while `todo:`, `TODO x` and `// this is a todo list` do not. *Decided (case clause):* lowercase
`Bug:` is the tree's **only** `severity=error` SATD finding and the one place verify and
quality-gate visibly disagree, so `Bug:`/`BUG:` is an explicit sub-deliverable with **both**
outcomes' fixture values written down before the code changes: if `Bug:` **is** debt, the fixture
(` ```/ //! todo!("…") / ``` `, `// this is a todo list`, `// Bug: Previously used walkdir
directly…`) must give `strict == 1` and `default == 2`; if it is **not**, `strict == 0` and
`default == 2`. Whichever is chosen, the fixture is updated deliberately — the merged control
demanded `strict == 0`, i.e. it silently answered "no" and left the divergence in place, so an
implementer who made verify and quality-gate agree would have failed the item's own test.
**A4** deletion: the same fixture minus the TODO → 0 under both. Repo consequence: with `tdg_calculator_core.rs:110` unchanged,
`pmat verify --skip clippy,tests --format json` must report `satd.ok == false`.

**Effort.** **M** — (a) is arguably S (~10 lines plus the field-type change); (b) is a match-arm
widening plus a fixture.

**Risk.** `ok: null` breaks scripts keying on `ok == true`; keep exit 0 and announce the field.
Widening strict SATD surfaces findings fleet-wide — a one-time ratchet rebase. **Out of scope:**
`.pmat-ratchet.toml:186`'s `satd_markers_src_comments`. Its `^[[:space:]]*(//|/\*|\*)` anchor is a
*different*, deliberate blind spot documented at `:188-192` ("the same markers occur 1,102 times
across `src/` once string literals, test fixtures and this project's own SATD ANALYSER are
counted"); widening it raises baseline 327 and needs its own `justification`.

**Framing (verification correction).** "35" is the true divergence but the fix closes **1** of it:
28 are entropy (not a verify stage), 5 of 6 complexity blockers are at or below the configured
limit, and of 3 SATD findings 1 is unambiguous debt. The defensible claim is narrow — *verify
asserts safety over a stage it did not measure.*

**Related issue.** #1035. Every verify defect in this family (#762–#765, #944) is CLOSED; not a
duplicate.

---

### 8.2 CRUX-02 — `quality-gate` renders three unmeasured dimensions as clean

**Problem.** The aggregate gate reports 0 violations for what it did not or could not measure, in
three independent places: **(a)** on an uncompilable crate — the state a pre-commit gate most often
meets — it prints `Checking dead code... 0 violations found` with the string `not_measured` absent
from the whole document, while `analyze dead-code` on the same tree returns
`{"not_measured": true, "reason": "Cargo check failed"}` at exit 5; **(b)** its coverage check
accepts **any** cache it finds, with no commit, mtime or breadth guard; **(c)** it reports
`duplicate_violations: 0` on a tree measuring 21.67 % duplication.

**Evidence.** (a) `src/cli/analysis_utilities/quality_checks_part1_dead_code.rs:65-68` —
`Err(_) => return Ok(violations), // No cargo project → no dead code violations`: the state is
discarded where it is produced, and "cargo check failed" and "not a cargo project" render
identically. (b) `read_coverage_from_detail_cache`
(`quality_checks_part2_coverage_sections.rs:38-44`) is a bare `read_to_string` + `from_str` + line
ratio. Measured: a fabricated cache (`git_hash "deadbeef…"`, timestamp 2019, one nonexistent file,
97/100 lines) → exit 0, `coverage_violations: 0`, PASSED. The repo's own gitignored cache (209
files, `git_hash` 583ea9ac2, mtime 2026-08-24) in a 4,036-file tree → PASSED;
`git rev-list --count 583ea9ac2..HEAD` → **114**. (c) `analyze duplicates` on `src/` →
`duplication_percentage 21.67, total_duplicates 33801, exact_duplicates 31929` over 240,671 of
1,110,797 lines / 4,036 files; `quality-gate --checks duplicates` → `duplicate_violations: 0,
passed: true`. Cause: `check_duplicates`
(`quality_checks_part2_security_duplicates.rs:123-176`) builds `HashMap<u64, Vec<PathBuf>>` from
**whole-file** hashes — no block-level detector behind the gate's word "duplicate". Disclosure:
`quality-gate --help | grep -icE "cargo|compil|build"` → **0**, on a command that shells out to
`cargo check`.

**CORRECTION.** Leg (b)'s absent-report half is **already shipped**: `run_coverage_check`
(`quality_gate_part2a.rs:20-46`) emits `[coverage] project - Code coverage was NOT measured (no
coverage report at …)` and exits 1. Restate the defect as: **the gate trusts any cache it finds.**

**Proposal.** (1) Propagate the analyzer's not-measured state into `results.not_measured[]` — but
distinguish the two `Err` cases, because the single `Err(_)` arm covers both "cargo check failed"
and "no Cargo.toml", and mapping both would leave every non-Rust repo permanently amber. (2) Widen
the **existing** coverage disclosure's trigger (absent OR `git_hash` not HEAD-or-ancestor OR mtime
older than the newest tracked source OR covering < N % of `files_examined`) rather than inventing a
second reporting path. (3) Rename the gate's finding to `identical_files`; wiring the real detector
is a **separate item**. (4) Say in `--help` that the gate runs `cargo check`.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
pmat quality-gate --format json -p fx/broken \
  | jq -e '.results.not_measured[] | select(.check=="dead_code" and (.reason|test("could not compile")))' \
  || fail "leg 1"
printf '{"git_hash":"deadbeef…","timestamp":"2019-01-01T00:00:00Z","files":{"src/deleted.rs":{"1":5}}}' \
  > fx/healthy/.pmat/coverage-cache.json
pmat quality-gate --checks coverage --format json -p fx/healthy \
  | jq -e '.results.coverage_violations > 0' || fail "leg 2"
# leg 3 (renaming, in scope): the misleading key is GONE and replaced by an honest one
pmat quality-gate --checks duplicates --format json \
  | jq -e '(.results | has("duplicate_violations") | not)
           and (.results.identical_files == 0)
           and (.results.not_measured[]? | select(.check=="duplicates"
                 and (.reason|test("block-level"))))' || fail "leg 3"
```
*Anti-vacuity.* **Leg 1 control A:** the same crate with the syntax error removed → no `dead_code`
entry. **Control B** (not-applicable ≠ not-measured): a dir with no `Cargo.toml` must report
`not_applicable`, never `not_measured` and never "0 violations, clean" — without B, "return
not_measured on every `Err`" passes and every non-Rust repo goes amber. **Leg 2 control:** a cache
stamped with HEAD's hash covering ≥ N % must stay silent, and the reason must name *which* guard
tripped so one hardcoded sentence cannot satisfy three. **CORRECTION — the merged leg 3 could not be satisfied by the work this item proposes.** It asserted
`.results.duplicate_violations > 0` on a tree whose whole-file duplicate count is measured at **0**,
so the only way to green it is the block-level detector that proposal (3) explicitly **splits out**,
and after the rename the key it reads no longer exists. An implementer landing the proposed scope
would have had a permanently red acceptance test. Leg 3 above is therefore the *rename* assertion,
which the proposed scope can actually pass and which makes the split honest rather than silently
deferring the user-visible half. **The two counter-tests move with the work:** a fresh single-file
crate → `identical_files == 0` (verified 0 today; without it, `identical_files = max(1, dupes)`
passes) and two files sharing **one identical function body** → the block-level count `> 0`
(verified 0 today — the gap a whole-file hash cannot fake) both belong to the split-out
**block-level duplicate detector** item, whose acceptance test they are.
**Do not use the merged global control** `(.results.not_measured//[]) | length == 0` on a healthy
crate: it would FAIL a correct fix, since a healthy crate legitimately has no coverage data. Scope
per-check.

**Effort.** **M** for (1)+(2)+(3). **Split out** "call the same detector": measured, `analyze
duplicates` costs 16.22 s / 323.7 CPU-s / 1.77 GB against the gate's 2.00 s / 1.82 CPU-s / 57 MB —
**8× wall, ~180× CPU, 31× RSS** on the command every agent runs before every commit. If it lands,
put it behind an opt-in `[quality] max_duplication_pct`.

**Risk.** Declared not-measured turns green gates amber on machines without a coverage cache —
the point, but it needs a release note. Ship the duplicates change as a new named finding type,
never folded into an existing threshold.

**Related issue.** #1035 (its Cluster-1 row "quality-gate reports satd_violations=0 while analyze
satd reports 55" is the same shape for another check).

---

### 8.3 CRUX-03 — `pmat config --validate` certifies a config it could not parse

**Problem.** The command validates the defaults it silently fell back to. Four inputs give
**byte-identical stdout and exit 0** — but for *two different reasons*, and conflating them
mis-states the fix. Three are the fallback defect: a `pmat.toml` with a section pmat cannot honour,
`not even toml ][`, and no config file at all, each certified after a silent fall-back to defaults.
The fourth — the repo's own fully valid `pmat.toml` copied into a scratch dir — **parses**, emits no
stderr warning, and still produces the same bytes, because the statistics block is hardcoded
(`configuration_handlers_validation.rs:118-122`: the literals `Sections: 7` and
`Total Settings: ~50`). That is a content-free report, not a fallback, and anti-vacuity control **C**
below is aimed at it specifically. Only stderr differs, and only for the corrupt case.

**Evidence.** All four print `Configuration is valid / All settings are within acceptable ranges /
No issues detected / Sections: 7 / Total Settings: ~50 / Custom Settings: 0`, exit 0. The warning
goes to **stderr** (`eprintln!`, `src/services/configuration_impl.rs:53`), so a CI job capturing
stdout and checking the exit code sees a clean pass with no warning at all. Cause is one level
below the observation: `ConfigurationService::new` (`:32`) is
`read_config_file(&default_path).unwrap_or_else(Self::default_config)`, and `read_config_file`
(`:45-68`) returns `None` for both "absent" and "present but unparsable" — the load status is
destroyed there; the handler (`src/cli/handlers/configuration_handlers_validation.rs:13`) then
validates the defaults. `PmatConfig` (`src/services/configuration_types.rs:6-34`) has nine sections
and `#[serde(default)]` on one (`semantic`), which is why any partial file fails strict deserialize.
The command has **zero tests**. **Same-tree contradiction as of 3.35.0:** `quality-gate` emits a
**blocking** `[error] config:` violation for the same bytes
(`quality_gate_project.rs:254-286`, #1105) while `config --validate` says valid and exits 0.

**Proposal.** (1) Carry the load status out of `ConfigurationService` and exit non-zero when a file
exists but did not load, naming file and key. (2) With no file, report "no configuration found —
validated nothing". (3) Print the source path of every setting validated, so `Custom Settings: 0`
cannot sit under a verdict. (4) **Reuse #1105's machinery** — `schema_pmat_toml_sections()`
(`quality_gate_project.rs:126`) and `nearest_known_section()` (`:145`) made `pub(crate)`; do **not**
build a second list (that commit's own doc comment warns two hand-maintained lists is how a
validator comes to disagree with the reader it polices). (5) Fix `scripts/setup-quality.sh:136-155`,
which still generates `[quality_gate]` / `max_cyclomatic_complexity` / `max_satd_comments` /
`min_test_coverage` and no `[system]` — pmat's own setup script writes a config that trips pmat's
own new blocking gate.

**CORRECTION — `PmatConfig`'s derive is the wrong oracle, both ways.** *Too strict:* partial files
pmat genuinely honours would start failing —
`quality_checks_part1_complexity.rs:176-215` deliberately parses a generic `toml::Table` with the
comment "`PmatConfig` has no serde defaults, so a strict deserialize of a partial file fails and
would silently drop the thresholds the user did write". *Too loose:* no `deny_unknown_fields`, so
unknown keys are invisible — the repo's own `pmat.toml` carries `min_pattern_diversity`,
`max_pattern_repetition`, `max_entropy_violations` under `[quality]`, none of them `QualityConfig`
fields, and `config --validate` prints "Configuration is valid … Custom Settings: 0" while
`quality_gate_config.rs:155-161` actually **reads** `min_pattern_diversity`. Validate against a key
registry unioning all readers, parsing generically.

**Acceptance test.** Four legs, all failing today.
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
printf 'not even toml ][\n' > pmat.toml
pmat config --validate && fail "leg 1: corrupt config still exits 0"
pmat config --validate 2>/dev/null | grep -q 'No issues detected' && fail "leg 1b: still certifies"
rm -f pmat.toml
pmat config --validate | grep -qi 'no configuration' || fail "leg 2: absent config not disclosed"
cp <repo>/pmat.toml . && printf '\n[quality_gate]\nmax_cyclomatic_complexity = 15\n' >> pmat.toml
pmat config --validate && fail "leg 3: unknown section still valid"
# leg 3 output must name `quality_gate` AND suggest `quality`
# leg 4: in the repo root, validate must NAME quality.min_pattern_diversity as honoured
pmat config --validate | grep -q 'min_pattern_diversity' || fail "leg 4"
```
Leg 3 is **hardened**: `[quality_gate]` *alone* fails to deserialize today (`missing field
'system'`), so the merged leg is satisfied by "exit non-zero on any parse failure" with zero
unknown-section detection and #1019's ask never gets built. Verified distinguishable: the full valid
config plus `[quality_gate]` parses, prints no warning, and still says valid at exit 0.
*Anti-vacuity, replacing the merged control.* `[quality] max_complexity = 30` **does not
deserialize** today (`missing field 'max_cognitive_complexity'`), so "must keep exiting 0" is
unsatisfiable within S. Use: **A** — a complete config from `pmat config --reset` exits 0 with no
stderr warning; **B** (works today, proves the validator is not always-pass) — that file with
`max_complexity = 0` → "Configuration validation failed / - Quality: max_complexity must be > 0",
exit 1; **C** — the printed section count must equal `schema_pmat_toml_sections().len()`, killing
the hardcoded `Sections: 7` / `Total Settings: ~50` at
`configuration_handlers_validation.rs:116-122` (PmatConfig has 9).

**Effort.** **S** — the failure path exists and is already wired to exit 1; the change is pushing
one more issue into the existing vec plus carrying load status out of the service. Note
`validate_all_sections` (`:22-28`) validates only 5 of 9 sections, so "No issues detected" is partly
vacuous even on a config that did load.

**Risk.** Users with a stale `pmat.toml` start getting non-zero exits from a command they may run in
CI; stage as warning-with-exit-1, and land with the `setup-quality.sh` fix.

**Related issue.** #1019. (#851 is CLOSED and covers `--config-path`, a different flag: today
`--config-path /does/not/exist.toml` correctly errors and exits 1 while the implicit `./pmat.toml`
silently defaults.)

---

### 8.4 CRUX-04 — the dead-code cache is keyed on the committed tree

**Problem.** `analyze dead-code` shells out to `cargo check` and memoises on `git rev-parse HEAD:`
— the **commit** tree, byte-identical before and after any uncommitted edit. Falsified in both
directions. Separately, a warm run that opens zero `.rs` files and execs zero compilers still emits
`compiler_scan.reason = "compiler-lint-ran"` with the present-tense detail "cargo check ran against
the existing lockfile", and carries no cache marker — a replay is byte-indistinguishable from a
50-second compiler pass.

**Evidence.** `get_tree_hash` = `git rev-parse HEAD:`
(`src/services/cargo_dead_code_analyzer/cache_operations.rs:35-46`); validity (`:57-70`) compares
only `report_schema`, `tree_hash`, `pmat_version`. Fixture (own `git init`, HEAD tree constant):

| run | tree | cache | wall | `dead_functions` |
|---|---|---|---|---|
| 1 | clean, cold | written | 0.12 s | 0 |
| 2 | dead fn appended, **not committed** | warm | 0.00 s | **0 — WRONG** |
| 3 | same dirty tree | **deleted** | 0.09 s | **1 — CONTROL** |
| 4 | reverted to clean | warm | 0.01 s | **1 — WRONG**, naming line 9 of an 8-line file |

`strace -f -e trace=execve` on a warm run: 20 execve, **two** successful (pmat, `git rev-parse
HEAD:`); zero cargo, zero rustc — and that run reported `compiler-lint-ran`. On this repo the
command swings **49.87 s / 4.96 GiB cold** vs **0.25 s / 20 MB warm** (200× / 260×). The cache file
already carries `report_schema`, `tree_hash`, `pmat_version`, `timestamp`. **Precedent: #748**
(CLOSED, blocker) was this exact class in the hooks cache, fixed by moving to `git write-tree`;
`src/tdg/hooks_cache/helpers.rs:37-68` now reads "A pre-commit run keyed that way is invariant to
exactly the change it is gating". Never carried across.

**Proposal.** Copy #748's shipped approach verbatim — `git write-tree` with a `HEAD^{tree}` fallback
for bare/unmerged states — rather than inventing `(path, mtime, len)`; that makes it S and makes the
two caches consistent. Add a `cache` object (`hit`, `tree_hash`, `written_at` ← the file's
`timestamp`, `pmat_version`), split the verdict into `compiler-lint-ran` vs `compiler-lint-cached`
in the past tense, and bump `DEAD_CODE_CACHE_SCHEMA`. Add the missing escape hatch: `--help` exposes
**no** `--no-cache`/`--force-refresh` (`without_cache()` (`cargo_dead_code_analyzer.rs:354`) and `force_refresh()` (`:362`) are
builder-only), so today the only workaround is deleting a file a developer
has no reason to know exists.

**Acceptance test — deterministic, replacing the unobservable "wall-time regression" clause.** Put a
shim on `PATH` logging argv and exec'ing the real cargo (resolve it with
`env -i PATH=$PATH sh -c 'command -v cargo'`; plain `which cargo` returns a shell function here and
the shim self-recurses). Assert exec count and cache flag move **together**:

| state | cargo execs | `cache.hit` | `reason` | `dead_functions` |
|---|---|---|---|---|
| A cold (cache deleted) | +1 | false | compiler-lint-ran | 0 |
| B rerun, no edit | **+0** | true | compiler-lint-cached | 0 |
| C append dead fn, uncommitted | +1 | false | compiler-lint-ran | **1, named** |
| D rerun, no further edit | **+0** | true | compiler-lint-cached | 1 |
| E `git checkout -- src/lib.rs` | +1 | false | compiler-lint-ran | 0 |

*Anti-vacuity.* Pairing `hit == true` with **zero** execs and `hit == false` with **exactly one**
`cargo check` makes "delete the cache" and "hardcode `hit=true`" both fail. **State D is what the
merged test lacked** — without it the cheapest passing fix is "bypass the cache whenever
`git status --porcelain` is non-empty", correct but turning every mid-edit run into the measured
49.87 s / 4.96 GiB cold path. Two more: **F** — `quality-gate --checks dead-code` on state C must
see the finding (a CLI-handler-only fix leaves the gate stale;
`quality_checks_part1_dead_code.rs:64` calls the same analyzer with `use_cache: true`); **G** — an
old-schema cache file must be a MISS, or every developer's existing cache keeps serving pre-fix
answers after upgrade. Keep the cache-deleted control for A/C/E (0/1/0) so a fix that changes the
analyzer's *answers* rather than the key is caught.

**Effort.** **M** (S for the key alone if #748 is reused; the rest is the `cache` object, four
output paths and the MCP mirror).

**Risk.** Keying on the working tree converts many hits into cold runs mid-edit; `cache.hit` keeps
the cost visible. **CORRECTION to the impact claim:** the installed pre-commit hook does **not**
consume this analyzer (it runs `analyze complexity`, `analyze satd`, `verify --stage clippy`). The
real consumers are `quality-gate --checks dead-code` and the MCP `analyze_dead_code` tool.

**Related issue.** **#748** (CLOSED, the precedent) and #1035. Also re-run fable-review §4 item 14's
falsifier **with the cache deleted** before building a recall corpus: that injection was into a
working tree, so "the detector misses it entirely" is plausibly this cache, not a recall gap.

---

### 8.5 CRUX-05 — one `Cargo.toml` line disables clap's usage, error-context and suggestions

**Problem.** `Cargo.toml:226` builds clap with `default-features = false` and only
`std, derive, help, env`. On the shipped binary: **0 of 71** subcommands print a usable `Usage:`
line, and every misuse error is context-free. For a CLI with 71 top-level and 35 `analyze`
subcommands and **no shell completions**, that is the entire discovery surface.

**Evidence.** `pmat --help | head -6 | cat -A` → `Usage: $`. Sweep over the root Commands block:
`total=71 usage_pmat=0 empty_usage=70 no_usage_line=1` (the outlier is clap's built-in `help`,
which has no help page). `analyze complexity --bogus-flag` → 34 B, exit 2,
`grep -c -- '--bogus-flag'` = **0**. `analyze complexit` → 32 B, no suggestion. Not a terminal
artefact: byte-identical at `COLUMNS=200`, under a pty (`script -qec`), and from
`~/.cargo/bin/pmat`. Control on the same box: `bashrs lnt` names `'lnt'`, tips `'init'`/`'lint'`,
prints `Usage: bashrs [OPTIONS] <COMMAND>`. Mechanism: `clap_builder-4.6.6/src/output/usage.rs`
`write_usage_no_title` is `#[cfg(not(feature = "usage"))] { false }` — writes nothing while the help
template still emits the heading; clap's `default` is
`["std","color","help","usage","error-context","suggestions"]`.
**How it survived:** `grep -rn 'contains("Usage:")' src/ tests/` → **14 sites**, not the five filed.
The load-bearing one is *product code*: `src/docs_enforcement/cli_checker_validation.rs:74`
`report.has_usage_section = help_text.contains("Usage:")`, one of five conjuncts of
`CliDocumentationReport::is_valid` (`src/docs_enforcement/cli_checker.rs:36-42`) — **pmat certifies
its own empty Usage sections as present.** Test-side: `src/tests/clap_command_structure_tests.rs:136,356`;
`tests/cli_acceptance/test_main_commands.rs:49,228`; `tests/cli_acceptance/test_analyze_commands.rs:466`;
`tests/modules/cli_docs_enforcement.rs:58,77,423,446`; `tests/bin/pmat_tests.rs:21`;
`tests/modules/quality_harness/mod.rs:2132`; `tests/modules/cli_functional_harness.rs:20`;
`tests/modules/cli_comprehensive_integration.rs:567`.
**Strongest single artefact:** commit `02292aaf8` (2026-08-20, on master, "fix: a bad flag was
reported as an unrecognized subcommand") quotes its own verified "Now:" output as
`error: unrecognized subcommand 'analyz'` / `tip: some similar subcommands exist` /
`Usage: pmat [OPTIONS] <COMMAND>`. Measured on a binary *containing* that commit: no token, no tip,
no usage line. A shipped fix was verified against a clap built with default features and its
evidence never held in the shipped build.

**Proposal.** Add `"usage", "error-context", "suggestions"` (consider `"color"`, `"wrap_help"`) at
`Cargo.toml:226`. Repair **all 14** sites to assert content, including the product checker, so the
regression cannot return the way it arrived.

**Acceptance test.** Five legs, all false today.
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
pmat --help | grep -qE '^Usage: +pmat .*<COMMAND>'        || fail "leg 1"
pmat quality-gate --help | grep -qE '^Usage: +pmat quality-gate' || fail "leg 2"
pmat analyze complexity --bogus-flag 2>&1 | grep -q -- '--bogus-flag' || fail "leg 3"
pmat analyze complexit 2>&1 | grep -qiE 'similar|did you mean'        || fail "leg 4"
# leg 5 — accumulate, then assert emptiness; the denominator guard is its own assertion
cmds=$(pmat --help | awk '/^Commands:/{f=1;next} /^Options:/{f=0} f && /^  [a-z]/ {print $1}' | sort -u)
[ "$(printf '%s\n' "$cmds" | wc -l)" -ge 70 ] || fail "leg 5 denominator: only $(printf '%s\n' "$cmds" | wc -l) subcommands"
missing=$(for c in $cmds; do
  pmat "$c" --help 2>&1 | grep -qE '^Usage: +pmat' || echo "$c"; done)
# `help` is clap's built-in and legitimately has no help page
missing=$(printf '%s\n' "$missing" | grep -v '^help$' | grep -v '^$' || true)
[ -z "$missing" ] || fail "leg 5: no usable Usage line for: $missing"
```
**CORRECTION — the merged leg 5 could not fail.** Its `for … do … || echo "MISSING $c"; done` loop
exits with the status of its last command. Run verbatim against the HEAD binary it **printed 71
`MISSING <subcommand>` lines and returned exit 0** — the identical shell defect §3 and §8.6
diagnose, shipped inside an item about gates that cannot fail. The version above accumulates and
asserts emptiness. **The sweep's baseline is 0 of 71, not "1 of 71"**, and the denominator guard is
now its own failing assertion rather than prose, so the leg cannot pass by shrinking the list.
*Anti-vacuity.* **(a)** unit test on the product code: `validate_sections("Usage: \n\nOptions:\n-h\n",
&mut r)` must leave `r.has_usage_section == false` and `r.is_valid() == false` — fails today.
**(b)** a source assertion that the bare predicate is gone from all 14 sites. **(c)** `Usage: pmat`
alone is gameable by one `override_usage` or a hardcoded footer (clap honours
`get_overridden_usage()` on the same path either way), so assert the line names the **full
subcommand path** plus a placeholder: `^Usage: +pmat analyze complexity` and a match on
`\[OPTIONS\]|<|\.\.\.` — a three-level path no single hardcode produces.

**Effort.** **S** for the dependency line; the 14 guard repairs are mechanical.

**Risk.** Binary size grows. The enforced band (`tests/modules/binary_size_band.rs`,
`EXPECTED_BYTES 55,000,000`, `FAIL_PCT 20`) gives 44–66 MB against 54,633,288 — ~11.4 MB of headroom
for what should be a few hundred KB (strsim + anstream). But the **quiet band is ±5 %**
(52.25–57.75 MB), ~3.1 MB before the test prints drift loudly; expect that output rather than
treating it as a regression. **Non-goal:** widening `QUIET_PCT` or `FAIL_PCT`. Deliberately raising
`EXPECTED_BYTES` is the documented process (`binary_size_band.rs:162`); loosening bands is not.

**Related issue.** #1018 (the 14 blind guards and the product checker), #999 (discovery). **No open
or closed issue names this defect**, and `fable-review.md` has zero hits for clap/usage/help-text.

---

### 8.6 CRUX-06 — `build.rs:21` declares a path outside the repo, so there is no incremental build

**Problem.** `cargo:rerun-if-changed=../assets/demo/` names a path that does not exist
(`ls -d ../assets` → *No such file*), carried into the root `build.rs` by `2aa5832a1` (the
single-crate refactor that deleted `server/build.rs`, where it resolved). Line 20 already declares
the real `assets/demo/`. Cargo treats a declared-but-missing rerun target as permanently stale, so
the build script re-runs and the lib and bin relink on **every** invocation.

**Evidence.** §5.5: 55.28 s wall / 263.46 s user / 499 % for a no-op release build, twice;
`build3.gitstat.before` == `.after`; cargo's fingerprint log names exactly one stale item. Static
replay at HEAD: 10 directives, `../assets/demo/` the only missing one. Mechanism reproduced in an
isolated 3-file control crate: declaring one present and one absent path reproduces pmat's exact
messages on every no-op build; deleting the missing-path line makes the next two builds print
`Finished dev profile … in 0.00s` with no `Compiling` line. No masked second cause: every other
declared path's max recursive mtime is weeks older than the build-script output fingerprint. Line 21
is dead in a second sense — its only consumer, `minify_demo_assets()` (`build.rs:489`), runs at
`:76` behind `if env::var("CARGO_FEATURE_DEMO").is_ok()`, and `demo` is not a default feature.

**Proposal.** Delete `build.rs:21`. Close the class with a `--lib` test extracting every literal
`cargo:rerun-if-changed=<path>`, asserting each resolves under `CARGO_MANIFEST_DIR`, and asserting
it extracted enough paths that a broken regex fails rather than certifies.

**Acceptance test — the merged version could not fail; this one does.**
```sh
set -uo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
n=$(grep -c 'rerun-if-changed=' build.rs) || fail "build.rs unreadable"
[ "$n" -ge 9 ] || fail "only $n directives (expected >=9)"
grep -q 'rerun-if-changed=\.git/HEAD'  build.rs || fail "provenance watch .git/HEAD removed"
grep -q 'rerun-if-changed=\.git/index' build.rs || fail "provenance watch .git/index removed"
dyn=$(grep -c 'rerun-if-changed={' build.rs); [ "$dyn" -eq 1 ] || fail "$dyn dynamic directives"
bad=$(grep -oP 'rerun-if-changed=\K[^"]+' build.rs | grep -v '[{$]' \
      | while read -r p; do [ -e "$p" ] || echo "$p"; done)
[ -z "$bad" ] || fail "declared but missing: $bad"
```
Measured: HEAD → `FAIL: declared but missing: ../assets/demo/`, exit 1. Line 21 deleted → PASS.
**Dodge 1** (line 21 rewritten as a `{}` format string, defect intact) → FAIL on `2 dynamic
directives`; the merged version **passed** this dodge, because the `[{$]` filter needed for the one
legitimate dynamic directive (`build.rs:1515`) also hides a bad path behind a format string.
**Dodge 2** (delete 21 *and* the two `.git/*` watches, silently weakening `PMAT_GIT_SHA` /
`PMAT_GIT_DIRTY`, consumed at `src/cli/commands/cli_struct.rs:27,29`) → FAIL on the count. The
merged script printed `MISSING:` and **exited 0** — a `while read … do [ -e ] || echo; done` loop
exits with the status of its last command.
*Behavioural half*, on the mechanism rather than wall-clock, with the git precondition the win
depends on: `git status --porcelain >/dev/null` before both builds, then a second
`CARGO_LOG=cargo::core::compiler::fingerprint=info cargo build --release` whose log must contain
neither `Compiling pmat` nor `stale: missing`. UNVERIFIED by this audit (no `cargo build` was run);
the control crate is the substitute.

**Effort.** **S** — one deleted line plus a ~15-line text test.

**Risk.** Essentially none; nothing can depend on a path that does not exist. **Claim correctly:**
cargo short-circuits on the *first* stale item, so `.git/index` (`build.rs:1678`) was never
evaluated and re-triggers the same relink after any git operation that rewrites the index. The win
is "a git-quiet tree now builds incrementally", not "no-op builds are free forever". CI blast radius
is limited (§5.5). Optional follow-up: `minify_demo_assets()` still reads the same non-existent
path behind the non-default `demo` feature — the other half of the same fossil.

**Related issue.** None filed; absent from every issue search and from fable-review §4/§5.

---

### 8.7 CRUX-07 — the index is not a faithful, reproducible view of the tree

**Problem.** Three defects in the artefact CLAUDE.md mandates over grep. **(a)** `check_mtime_reuse`
skips read+SHA256 for any file whose mtime predates `built_at` and copies the **old checksum forward
unverified**; nothing re-checks, so a content change whose mtime does not advance (`rsync -a`,
`tar -x`, `cp -p`, restore-from-backup, an older `git checkout`, an edit mid-build) is invisible
**forever**. **(b)** No walk is sorted, so result order follows `readdir` — a fresh CI clone and a
long-lived tree can disagree about a `--limit N` top-N, and no double-run gate can see this class.
**(c)** `analyze churn --format json` serialises a `HashMap` through serde.

**Evidence.** (a) `src/services/agent_context/function_index/build_helpers.rs:40-67` —
`if mtime >= *built_at { return None; }` then `existing.manifest.file_checksums.get(...)` with no
read; the stale checksum is republished under a fresh `built_at`. Reproduced: write `alpha_only`,
index; overwrite with `beta_only`; `touch -d 2020-01-01` → `Incremental update: 1 mtime-skipped,
0 checksum-reused, 0 re-parsed` / `No matching functions found for: beta_only`, and
`pmat query alpha_only` returns `src/lib.rs:2-2 | alpha_only | TDG: A+` **with the deleted source
line**; manifest `74bc8514…` vs `sha256sum` `b5ecfd1c…`. The fast path covers **4,477 of 4,477**
files here, and the metadata call is already made, so comparing `len()` and `ctime` costs zero extra
syscalls. (b) `build.rs:86` and `build_incremental.rs:148` are both `WalkBuilder::new(...).build()`
with no sort; 20 production sites, none sorted; `grep -rn 'sort_by_file_name' src/` has exactly one
hit and it is a **comment** at `src/services/facades/defect_prediction_facade.rs:602` — where this
repo already fixed the identical class ("Directory read order is filesystem-dependent … identical
input must produce identical output", `files.sort()`). `ignore` 0.4.33 exposes `sort_by_file_path`.
Measured: two byte-identical 6-file crates on ext4 and tmpfs (`diff -r` IDENTICAL CONTENT) have
different `os.listdir` order and return all six results reordered, each matching its own readdir.
(c) `src/models/churn.rs:38` `pub author_contributions: HashMap<String, usize>`; 10 runs → **5
distinct hashes**, the sole diff being key order.

**CORRECTION — leg (b)'s causal attribution.** Write order is **not** the variable: two ext4 dirs
written in opposite orders give *identical* `os.listdir` (ext4 htree orders by a filename hash) and
identical query order. The divergence is **filesystem type**. Vary that, drop "in opposite write
orders", keep the mandatory precondition assert.

**Two further legs folded in at merge, both same-surface and both S.** **(d) The index's two halves
are persisted with different care.** The `.db` write is atomic; `manifest.json` is not, and nothing
detects a manifest older than its DB — which is the same file leg (a)'s acceptance test reads
`built_at` from, so a torn write silently changes the meaning of that test. **(e) A save whose
failure is discarded, under a message that says it succeeded.**
`src/cli/handlers/query_handler/indexing.rs:227` is `let _ = index.save(index_path);`, printed under
an unconditional `Saving index (N changes)...`; on a read-only `.pmat/` the user gets the success
message and no index, and every later `pmat query` silently re-indexes at the measured 14.8–17.6 s
cold cost (§8.20c). Same shape at `src/cli/handlers/score_handler_compute.rs:530`
(`let _ = std::fs::write(filepath, json)`) and
`src/services/agent_context/document_index/sqlite_docs.rs:98` (a discarded FTS `DELETE` one line
above a `?`-propagated `INSERT`).

**Proposal.** (a) Keep the fast path but also compare `len()` and `ctime` from the metadata call
already being made — a content rewrite always advances ctime even when mtime is backdated — plus a
rotating verify slice hashing 1/64 of files per run (`hash(path) % 64 == run_counter % 64`) so a
laundered checksum is caught within 64 runs. **cfg-gate the ctime compare**: `ci.yml` and
`nightly.yml` both run a Windows job and Windows has no ctime (`Metadata::created()` is birth time
and does not advance on a rewrite), so on non-unix the `len()` compare plus the verify slice carry
the leg alone. (b) Sort the two walks that define persisted order, then the other 18; add a
`(file_path, start_line)` tie-break to the ranker. (d) Write `manifest.json` atomically (temp + `rename`, as the `.db` path already does) and add a
manifest-newer-than-DB detector. (e) `match index.save(...)`: on `Err`, `eprintln!` the path and the
error, and move the success message *after* the save; `persist_score` returns `Result` and its caller
warns; `sqlite_docs.rs:98` propagates with `?` like its neighbour. (c) `BTreeMap`, plus a tie-break on author name
in the markdown path — `src/cli/analysis_utilities/churn.rs:224` is
`sort_unstable_by(|a,b| b.1.cmp(a.1))` keyed on the count alone, a second latent instance.

**Acceptance test.** (a) Backdate **relative to `built_at`** (read it from
`.pmat/context.idx/manifest.json`, then `os.utime(f, (built_at-1, built_at-1))`) — not to 2020, or a
"treat ancient mtimes as changed" heuristic passes without fixing anything; verified the realistic
version reproduces identically. Three assertions together: `pmat query beta_only` matches;
`pmat query alpha_only` does **not**; and — the **fast-path-survival control**, so "delete the
optimization" cannot pass — a repeat query on the quiescent tree must still print
`Incremental update: [1-9][0-9]* mtime-skipped` with `0 re-parsed`. Non-vacuity: the same script
*without* the touch passes today (`0 mtime-skipped, 1 re-parsed`). Record the rebuild rate the risk
demands: after `chmod -R u+w .` (ctime advances, content does not) assert the tree re-hashed with 0
functions changed. (b) **Drop the two-filesystem dependency from the CI gate** (a runner without
`/dev/shm` makes the guard fail environmentally, and a flaky gate gets disabled); keep it as a
one-off proof and gate on the environment-independent invariant: one temp crate, six tied-score
files, assert `pmat query zz_ --limit 10 --format json` returns all six (guards truncation) **and**
that the emitted `(file_path, start_line)` sequence equals the sorted sequence. (c) **CORRECTION — the merged oracle destroyed the defect it was testing, and is GREEN today on the
unfixed binary.** The stated nondeterminism is *key order* ("the sole diff being key order"), and
`jq -S` sorts object keys recursively — so the canonicaliser normalised exactly the variable under
test. Reproduced against the HEAD binary: six runs of `analyze churn --format json` give **six
distinct raw sha256 hashes**, and the same six files through `jq -S 'del(.generated_at)' | sha256sum`
give **one**. The leg would have stayed green if `author_contributions` were never changed to a
`BTreeMap`. Replace it with two assertions, one of which preserves emission order:

```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
# content first — stability over an empty or wrong object proves nothing
pmat analyze churn --format json \
  | jq -e '(.author_contributions|keys|length) >= 3' || fail "content floor"
# leg c1 — RAW order-preserving canonicalisation: strips only time-valued keys, keeps key order.
#          json.load preserves document order in dicts, so this cannot normalise the defect.
raw=$(for i in $(seq 1 100); do
        pmat analyze churn --format json | python3 -c \
          'import sys,json; d=json.load(sys.stdin); d.pop("generated_at",None); print(json.dumps(d))' \
        | sha256sum; done | sort -u | wc -l)
[ "$raw" -eq 1 ] || fail "leg c1: $raw distinct raw outputs"        # today 6 in 6
# leg c2 — the -S form must ALSO be stable, which catches a newly ADDED timestamp field
srt=$(for i in $(seq 1 100); do
        pmat analyze churn --format json | jq -S 'del(.generated_at)' | sha256sum; done | sort -u | wc -l)
[ "$srt" -eq 1 ] || fail "leg c2: $srt distinct sorted outputs"     # today 1 — this is the control
```
The evidence this item needs is the **pair**, run against the shipped 3.34.0 binary: c1 RED (6 of 6
distinct) and c2 GREEN. c2 alone is not a gate; c1 alone misses added timestamps. Also assert the
key set and values equal `git shortlog -sn` over the same window, so an emptied object cannot pass
c1 trivially.

**Acceptance for (d) and (e).** (d) needs a **seeded torn-write fixture** — truncate
`manifest.json` mid-object after a successful `.db` write — and the detector must fire on it and
must NOT fire on a clean pair; without the seeded fixture the detector is unfalsifiable. (e) has its
anti-vacuity control built in: with `.pmat` writable the same command must be **silent** about save
failures, so an unconditional "could not save" warning cannot pass. The red half is
`chmod 500 .pmat` before a re-index — today it prints `Saving index (N changes)...` and exits 0 with
no index written.

**Effort.** **M** — (a)-predicate, (c), (d) and (e) are each S; the rotating slice needs a persisted
run counter (manifest schema touch) and the 20-site sweep plus the CI gate push toward M/L.

**Risk.** The verify slice adds ~1.5 % of a hash pass per run. Sorting changes persisted row order
once — a rebuild, not data loss. **The new determinism gate must be able to fail**: seed it with a
committed HashMap-serializing fixture and require it red on that first, and make it FAIL on empty
output or non-zero exit so `analyze dag` (0 B, exit 2) and `report` (0 B, exit 124) cannot score a
vacuous pass. Extend `scripts/dogfood/pmat-dogfood_surfaces.sh`'s existing `--twice` mode rather
than minting a harness — but note it is invoked by **no** CI job and no make target and byte-compares
*receipts*, not stdout, so it cannot see this class today.

**Related issue.** #1035 (nearest, for (c)). Legs (a) and (b) have no matching open issue. Prior
CHANGELOG work walked past all three: 3.32.0 fixed an "Index source-wipe" and a graph-metrics
`FxHashMap` ordering tie-break without touching these sites.

---

### 8.8 CRUX-08 — the two MCP transports have different JSON-RPC error contracts

**Problem.** stdio and streamable-HTTP share `build_server`, but response shaping does not:
`sort_tools_by_name` and `client_fault_code` live in the **stdio wrapper**, so over HTTP every
client fault returns `-32603 Internal error` — telling a mistaken caller the *server* broke, which a
conforming client retries. The guard that should catch it never binds a socket.

**Evidence.** Six probes through both transports (stdio via `pmat --mode mcp`; HTTP via
`pmat serve --transport http` with a 21-char `PMAT_MCP_HTTP_TOKEN` on loopback and
`Accept: application/json, text/event-stream`). **6 of 6 differ** — one more than the record's "≥4":

| probe | stdio | HTTP |
|---|---|---|
| unknown tool | `-32602` | `-32603` |
| `no/such/method` | `-32601`, id echoed | `-32700`, `"id":null`, real code buried in prose |
| `paths: 123` | `-32602` | `-32603` |
| `"jsonrpc":"1.0"` | `-32600` | **HTTP 200 + the full 19-tool result** |
| bad project path | `-32602` | `-32603` |
| missing `paths` | `-32602` | `-32603` |

Ordering: three fresh HTTP servers → three tool orders; three stdio runs → one, sorted. Schemas:
`additionalProperties` on **0 of 19**; `grep -rn deny_unknown_fields src/mcp_pmcp/` → 0;
`analyze_complexity {"paths":[…],"bogus_param":123}` → `status: completed` on **both**. Mechanism:
`repair_outbound` has exactly two references tree-wide, both in `src/mcp_pmcp/stdio_frames.rs`
(`:658` def, `:719` call); `sort_tools_by_name` `:675`; `client_fault_code` `:704` is a prefix match
on two Display strings (`:708-709`). `src/mcp_pmcp/http_server.rs` hands the shared server to
`StreamableHttpServer` and inherits neither. The guard,
`src/mcp_pmcp/cli_mcp_surface_parity_tests.rs:86-87`, is `include_str!("http_server.rs")` plus
`.contains()`. Transport faults borrow wrong codes too: no `Accept` → 406 with `-32700`; no
`Authorization` → 401 with `-32003`.

**Proposal.** One response post-processing layer inside `build_server` (or pmcp middleware it
applies) doing both the sort and the code correction, so every present and future transport inherits
both. Add `additionalProperties: false` to every advertised schema **and**
`#[serde(deny_unknown_fields)]` to every MCP args struct — either alone leaves one party silent.
**CORRECTION — one leg is not implementable in pmat.** "Replace prefix matching with a typed
client-fault error the handlers raise" cannot work at pmcp 2.17: `Server::create_response`
(`pmcp-2.17.0/src/server/mod.rs:1404-1422`) stamps `-32603` for **every** `Err` and keeps only
`e.to_string()`; `handle_list_tools` (`:1425`) iterates a `HashMap<String, ToolInfo>` (`:397`). pmcp is a sovereign batuta
crate — file the causal fix upstream, keep the pmat-side layer as a declared interim. Half the
divergence is also **inbound**: `-32600` comes from pmat's own stdio-only frame validator
(`stdio_frames.rs:318,341,367`), which a response layer cannot supply — share that too.

**Acceptance test.** Replay a **committed, id-pinned** probe fixture through both transports and
compare `(id, error.code)` pairs. Three implementation traps the merged version hits: **sort by id
before diffing** (stdio answers out of order — it emitted id=3 and id=5 *before* the id=1 initialize
result, while HTTP is necessarily in request order, so a naive `diff` reports concurrency as a
difference); **keep the `"id":null` rows** and sort them last (losing the id is part of the defect);
and pass an **absolute** path in the `bogus_param` probe, because `analyze_complexity` resolves
relative paths against the *server's* cwd and would otherwise return `-32603 path(s) not found` and
look like the failure under test.
*Anti-vacuity — five clauses, because equality alone is gameable.* **(1) Pin absolute codes:**
deleting `repair_outbound` from stdio makes both transports emit `-32603` for everything, the diff
empty and the gate green while the product is strictly worse. Assert bad path `-32602`, unknown tool
`-32601`-or-`-32602` (declare which), missing required field `-32602`, unknown method `-32601`
**with the id echoed, never null**, `"jsonrpc":"1.0"` `-32600`, garbage bytes `-32700` with id null.
**(2) Pin the fixture, not its size:** assert the exact id set is present and answered; counting
≥6/≥3 frames lets a happy-shape fixture certify parity. **(3) Determinism:** three fresh HTTP
processes must return byte-identical tool order, equal to stdio's. **(4) Drop the hardcoded 19:**
assert *every* live tool carries `additionalProperties: false` and every args struct derives
`deny_unknown_fields`, so tool #20 cannot pass by omission; keep the `bogus_param` probe so a
schema-only cosmetic fix fails. **(5) Prove it can fail and wire it:** run against the shipped
3.34.0 binary as a control — it must FAIL, and that run is committed evidence — and name its
execution site inside the required `ci / gate` or reachable from `cargo test --lib`.
`grep -rn 'transport-parity|mcp-surface|dogfood' .github/workflows/` returns one *comment* line;
`scripts/dogfood/pmat-gate-{transport-parity,mcp-surface}.sh` prove this repo accumulates gates
nothing invokes.

**Effort.** **M** for shared outbound repair + schema/serde tightening; **L** once the inbound
validator is shared and the upstream pmcp leg is included.

**Risk.** Changing HTTP codes breaks any client keying on `-32603` — the correct break, changelog it.
`deny_unknown_fields` rejects requests that today silently succeed; ship both halves in one release
so advertisement and behaviour never disagree. **Add, do not replace,**
`cli_mcp_surface_parity_tests.rs:86` — its "http_server.rs registers no tools of its own" assertion
is weak but real.

**Related issue.** **#1090 (OPEN)**, whose final comment says verbatim "Still open, deliberately —
the HTTP transport still answers `{"jsonrpc":"1.0"}` with 200 … A guard was written and withdrawn
before merge". Commit `493f3d355` removed `src/mcp_pmcp/http_frames.rs` (578 lines) and an e2e test
(178 lines). **Frame this as re-landing that guard**, fixing the three review defects #1090 records:
bound the parked-frame queue in **bytes** not entries, correlate on a server-minted id rather than
the client's `x-request-id`, and give its e2e test a CI leg that executes. A socket-binding target
does exist — `tests/e2e_http_serve_t.rs` (318 lines, `required-features = ["mcp-http"]`) — but it
asserts only auth, bind address and a `tools/list` round-trip, never speaks stdio, and **no
`.github/workflows/` file mentions `mcp-http`**. Also #1029.

---

### 8.9 CRUX-09 — the packaged `mcp.json` advertises a wrong inputSchema for 19 of 19 tools

**Problem.** `mcp.json` ships inside the crate for clients to read, and its schemas are a canned
shape chosen by tool **name**: 15 of 19 are advertised as taking `paths: string[]` whatever they
actually take. Calling every tool exactly as the manifest describes it fails for six.
`pmat_index_stats` is declared `additionalProperties: false` with no properties while the live tool
accepts `rebuild`, so a validating client rejects its own valid call.

**Evidence.** `tool_schema()` (`src/mcp_pmcp/tool_manifest.rs:131-167`) returns one of three canned
shapes by name, carrying the comment "minimal is not the same as wrong: advertising `paths` for a
tool whose handler rejects it would put an unusable argument name in a file that ships inside the
crate (#1029)" — while advertising `paths` for 15 of 19, including the six that reject it.
Field-by-field diff against a live `tools/list`: names 19/19 equal, descriptions 19/19 equal,
**inputSchema 19/19 different**. Behavioural probe on a git fixture: **6 of 19 fail** —
`quality_proxy` (missing `operation`), `pdmt_deterministic_todos` (`requirements`), `git_operation`
(`path`), `pmat_query_code` (`query`), `pmat_get_function` / `pmat_find_similar` (`function_id`). (A
7th, `analyze_vacuous_tests`, failed for a legitimate fixture reason — verified by rebuilding the
fixture with two `#[test]` fns, after which it returns ok.) The six guards at `tool_manifest.rs:239-599`
compare names, descriptions, the doc's tool count, two-run determinism and build-artifact paths —
**none compares inputSchema**. `mcp.json` is absent from `Cargo.toml`'s `exclude`, so the wrong file
is in the published tarball. The only schema-comparing test,
`src/mcp_pmcp/advertised_schema_parity_tests.rs`, pins each handler's args struct to its own
`metadata()` — the live side only. Related: `ls mcp_tool_schemas/ | wc -l` → **6** against 19 tools,
and `tool_metadata!`, the documented entry point, has **zero production call sites**.

**CORRECTION.** "19/19 differ" holds under canonical equality, but 3 (`analyze_reachability`,
`analyze_hardcoded_paths`, `analyze_vacuous_tests`) differ **only** in a property's human-readable
description; **16 of 19 differ in shape**. State the criterion as "19/19 byte-equal (today 0/19;
16/19 differ even ignoring description text)" so a fix cannot be scored partial by arguing prose.

**Proposal.** Delete `tool_schema()`. Render the manifest's inputSchema from each handler's
`metadata()` — the source `tools/list` serves — and add `manifest_schemas_match_handler_metadata`
beside `manifest_descriptions_match_handler_metadata`, which already instantiates all 19 handlers
synchronously. This completes a fix the file half-made one release ago: its module doc (`:16`) reads
"Only names and counts were ever compared, so the texts drifted for releases"; descriptions were
pinned in 3.33.0, schemas were left in exactly that state.

**Acceptance test.** Four legs, all red.
**A.** Per tool, `render_manifest(v)`'s `inputSchema` must equal that handler's
`metadata().input_schema` under canonical JSON; assert the handler-vec length equals
`LIVE_MCP_TOOLS.len()` equals the count of `.tool(` registrations parsed from
`simple_unified_server.rs` — **not** a hardcoded ≥19, so shrinking the surface cannot pass.
**B — richness floor, the clause that blocks the lazy fix.** Leg A is an equality between two
artefacts both under the author's control; the cheapest way to make 19/19 equal is to degrade the
handlers' `metadata()` down to the canned shape. Measured census, the floor: LIVE = 19 tools /
**51 properties** / **50 property descriptions** / **19 `required` entries across 18 tools**; the
manifest today has 18 / 0 / 3 across 3. Assert `properties ≥ 51`, `with-description ≥ 50`,
`required ≥ 19` over `≥ 18` tools, and that `pmat_index_stats` exposes `rebuild`.
**C — negative control, asserted not described.** Clone the handler vec, drop one property, assert
the comparator reports a mismatch; without it a comparator that compares nothing passes forever.
**D — behavioural, in-repo.** The research script hardcodes both the binary and repo paths, which
CLAUDE.md's rule forbids; port it to a `tests/` test spawning `env!("CARGO_BIN_EXE_pmat") --mode mcp`
against a `tempfile` git fixture containing a `#[test]`. Synthesize each call's arguments **from the
manifest**, honouring `required`/`type`/`enum` (pick `enum[0]`, skip non-required). Assert: every
tool produced a response (a truncated stream must FAIL, not pass); the enumerated count equals the
manifest's own `tool_count`; and no response error is in the argument-shape family (`missing field`,
`Missing required parameter`, `unknown field`, `Invalid arguments`). **Semantic errors must be
asserted as permitted** — once the manifest carries honest schemas, naive synthesis legitimately
produces `Invalid function_id format`, `Unsupported dag_type`, `Unsupported format`,
`Invalid min_grade` (measured: 7 such failures against live schemas), so A and D are in direct
tension unless D is scoped to the argument-shape family.

**Effort.** **M** for delete + share + regenerate + the tests. **Split out** the migration of 13
hand-rolled `json!` blocks onto `mcp_tool_schemas/`; a separate M that must not gate this.

**Risk.** Some handlers' metadata may be as thin as the canned shapes — a finding, not a regression.
Rendering from handlers makes `mcp.json` churn on every schema change; pin it with the new test.
CB-1656 already compares the committed manifest to a fresh render, so a hand-edit cannot fake the
fix — but note fable-review §4 `mcp-inventory-parity` leg (a)
(`include_str!(mcp.json) == render_manifest(...)`) would **not** catch this defect: it pins the file
to the renderer, and the renderer is what fabricates the schemas.

**Related issue.** #1029.

---

### 8.10 CRUX-10 — `quality_proxy`, the only MCP tool advertised as writing, never writes

**Problem.** In seven live `operation: "write"` invocations — several returning `status: accepted` —
**no file was created**; `find` over the target dir returns zero files. It returns `final_content`
for the caller to write, so every agent bypasses the gate using its own harness's writer. Three of
#1090's named bypasses reproduce verbatim, and two further tool **names** assert mutations their
handlers do not perform.

**Evidence.** `grep -n 'fs::write' src/services/quality_proxy_operations.rs` → exactly one hit,
`:268`, inside `#[cfg(test)]`. `proxy_operation` (`:36`) returns `(status, final_content, …)` and
never touches the target. Live: `{"operation":"write","file_path":".../OK.rs","content":"/// d\npub
fn ok(){}\n"}` → `status: accepted, passed: true`, `test -f OK.rs` → **False**. Bypasses:
`mode:"advisory"` → `accepted` while `passed:false` (unconditional at `:70`); client
`quality_config {"max_complexity":9999,"allow_satd":true}` → `passed` flips false→true **and the
SATD violation is deleted from `violations[]`** while `satd_count` stays 1. Mis-named:
`scaffold_project` returns "Context summary generated from file system analysis" with `ls -R`
byte-identical before and after; `git_operation`'s schema has one property `path`, no `operation`,
described as "Query git working-tree status". The repo asserts the opposite of its own behaviour
twice in comments written to justify a prior fix (`src/mcp_pmcp/tool_manifest.rs:39` "never learned
that the tool writes files"; `:324`), and `docs/mcp/TOOLS.md:105-107` now reads "**Writes files.**"
— a more emphatic false statement than the text it replaced. **#794** ("status 'accepted' for
operation=write/append is returned without ever writing the file") was filed against 3.29.0 and
**closed COMPLETED**; neither branch of its own suggested fix landed.

**Proposal.** **Prefer the rename**, against the merged record's preference, because
`CHANGELOG.md:545-550` (3.33.0, "Not done, deliberately") records the colliding decision: "No
`pmat_write_file` / `pmat_edit_file`. MCP has no primitive by which a server can gate a client's own
tools … pmat already owns the only layer that can intercept — the harness `PreToolUse` hook matching
`Write|Edit`." So: rename to `quality_check_content`, drop `write` from the enum, add
`written: false` to `ProxyResponse` (`src/models/proxy.rs:163-170` has no such field, so a client
cannot tell today). If the write is built instead, scope it under the project root, refuse on
blocking verdicts, never auto-fix silently. Independently close the two live bypasses — `advisory`
must never return `accepted` when `passed:false`, and client `quality_config` may only **tighten**
relative to the project's `pmat.toml` (the proxy reads no config source today, so "ignore it" leaves
the schema defaults as the only gate). Rename `scaffold_project` → `context_summary` and
`git_operation` → `git_status`, aliased for one release.

**Acceptance test.** Use only schema-declared fields, inspect the filesystem after each call.
**CORRECTIONS:** `"mode":"blocking"` is **not** in the enum (`quality_proxy_handler_impl.rs:12` →
strict|advisory|auto_fix|auto-fix) and returns `-32602 Invalid mode` before any gate runs, so the
merged leg 2 passes today *and* under the unconditional writer it was meant to exclude; use
`strict` (also the serde default). `"apply": true` is not a schema field and is silently ignored (no
`deny_unknown_fields` anywhere in `src/mcp_pmcp/`); drop it.
**L0 control:** record the pre-fix transcript (L1 returns `accepted` with the file absent).
**L1:** `{"operation":"write","file_path":"$T/OK.rs","content":"/// d\npub fn ok(){}\n",
"mode":"strict"}` → `status == "accepted"` AND `test -f $T/OK.rs` AND
`sha256($T/OK.rs) == sha256(response.final_content)` (the hash blocks a fix creating an empty file
to satisfy `test -f`). **L2 refusal, real mode:** same call, `content:"// TODO: x\npub fn f(){}\n"`,
`mode:"strict"` → `rejected` AND `test ! -e $T/BAD.rs`. **L3 advisory must not write:** same bad
content, `mode:"advisory"` → `passed == false` AND `test ! -e $T/ADV.rs` (today advisory returns
`accepted`, so a fix keying the write off `status` alone fails here). **L4 no-clobber:**
pre-populate `$T/EXIST.rs`; a rejected write must leave its sha256 unchanged. **L5 name honesty**,
satisfiable by *either* branch so the test does not force the capability expansion: a `--lib` test
asserting that for every `LIVE_MCP_TOOLS` entry the leading verb matches the handler's declared
effect (read | summarise | write); `scaffold_project` and `git_operation` fail until renamed,
`quality_proxy` passes only if it performs L1–L4 or carries no mutation verb and no `"write"` in its
enum.
**DELETE the polyglot leg** — it demanded a **regression** (§3). If a cross-language leg is wanted,
invert it into a guard for behaviour already correct: `# TODO: x` scores satd 1 as `.py`/`.sh` and 0
as `.rs`; `// TODO: x` scores 1 as `.rs` and 0 as `.py`/`.sh`; an unknown extension keeps the
conservative 1.

**Effort.** **M** for the rename branch plus the two bypass closures plus L5; **M/L** if the write is
built.

**Risk.** A real write is a capability expansion colliding with a recorded decision. Renaming breaks
clients with hardcoded names — one release, both names aliased.

**Related issue.** #1090 (OPEN; legs 1–4 verbatim). #794 is CLOSED-as-completed and still reproduces
— a ready-made regression-test target. Leg 5 appears nowhere in #1090.

---

### 8.11 CRUX-11 — `include!` defeats pmat's own headline metric: A (93.4) split, C (60.9) flattened

**Problem.** `pmat tdg` does not resolve `include!`, so byte-identical code scores two grade letters
apart depending only on how it is spliced. Any pmat **user** can buy a grade with a textual split —
the one finding that is a product defect for every user rather than a property of this tree. Worse
than filed: the *directory* walk does not merely mis-score fragments, it **drops** them and reports
full coverage.

**Evidence.** `pmat tdg src/cli/analysis_utilities/mod.rs` → `93.380` (A), breakdown
`{structural_complexity 25.0, semantic 20.0, duplication 17.72, coupling 15.0}`. An independently
written flattener over the 54-file closure → `60.934` (C),
`{structural_complexity 0.0, semantic 15.0, duplication 15.93, coupling 5.0}` — **32.446 points**.
Two different flatteners gave identical scores, and deleting rather than commenting the include
lines also yields 60.9. **Not a parse artefact:** `analyze complexity` on a dir holding only the
flat file reports `"analysis": "ast"` with `heuristic_include_fragment: 0` — both sides AST-scored.
**Not specific to this tree:** a synthetic 3-line host that `include!`s a 3,000-function fragment
scores 100.0 A+, byte-identical to the same host with the fragment absent, while the merged single
file scores 85.0 A-. **The cause is in the product, documented in its own source:**
`src/cli/language_analyzer/mod.rs:51 is_include_fragment()` is a **filename heuristic**; `:124-128`
routes matches to `Provenance::HeuristicIncludeFragment`; and
`src/cli/language_analyzer/ast_fallback.rs:66-67` states outright that a fragment is "Rust that pmat
has an AST analyzer for and deliberately does not apply" (the `HeuristicIncludeFragment` variant
itself is `:80-82`, worded differently). **The directory walk is worse:**
`src/tdg/analyzer_simple_core.rs:125` is `if is_include_fragment(file) { continue; }` — measured,
70 `.rs` on disk against `pmat tdg src/cli/analysis_utilities --format json` → `files_walked 26,
files_analyzed 26, files_ungraded 0, not_measured false, ungraded_files [],
duplication.covers_every_graded_file true` at score 94.70. **44 of 70 files (63 %) silently dropped,
with the payload asserting complete coverage** — #1035's class, in the headline metric. Repo scale
at HEAD: **2,295** `include!` sites (`-- src`); 2,061 fragment files / 528,656 lines (47.6 % of
`src/` lines);
`analysis_provenance {ast: 1802, heuristic_include_fragment: 2231}` over 4,036 files. **The resolver
already ships:** `collect_included_files` (`src/cli/handlers/complexity_handlers/analysis.rs:94`,
BFS, `MAX_INCLUDE_DEPTH = 8`) landed in 3.30.1 for #702, a private fn with exactly one caller —
`analyze complexity --file` prints "also analyzed via include!()" and reports 54 files. TDG cannot
reach it.

**Proposal.** **(a) Product first.** Lift the shipped resolver into a shared place and wire
`tdg`/file-health to it, attributing a fragment's items to the including module — attribution, not
new parsing. **(b) Repo.** Convert unguarded test `include!`s to
`#[cfg(test)] #[path="x_tests.rs"] mod x_tests;` — the form the tree already uses **174** times — and
ratchet the count down; that removes ~190,342 lines from every release build's parse and is the
prerequisite for excluding 11.79 MB of test source from the published `.crate`
(`src/mcp_integration/prompts.rs:80` is a bare `include!("prompts_tests.rs")` whose `#[cfg(test)]` is
on line 4 of the *included* file).

**Acceptance test — hardened; the merged version admitted two lazy fixes.**
**(a1) Direction pinned, expressed relatively so a scorer recalibration cannot break a correct fix:**
`pmat tdg src/cli/analysis_utilities/mod.rs` must land within 2.0 of the flattened closure's score
**AND the agreed score must be at least 25 points below today's split-only 93.4** (an absolute
`≤ 65.0` is derived from today's weights and would become unreachable if the fix changes how
duplication and coupling aggregate across attributed fragments — i.e. a correct fix could fail its
own test). **The flattener is a deliverable, not an assumption:** neither of the two used during this
audit is committed anywhere, so the PR must land one as a test fixture, or specify it precisely
enough to re-implement — BFS over `include!` targets with `MAX_INCLUDE_DEPTH = 8`, textual splice,
commenting-out and deleting the include lines both accepted (both were measured and both give
60.934). Without this, deleting the
size-sensitive `structural_complexity` term (25.0 host, 0.0 flat) makes both ~93 and the gate green
while the metric gets weaker. **(a2) Control — the size term must still bite:**
`pmat tdg src/services/mutation/scoring.rs` (403 lines, zero `include!`, today 92.5 A with
`structural_complexity 25.0`) must stay within 2.0 of 92.5 with that term still 25.0.
**(a3) Honest denominator, at the directory level where the bypass lands:** for any directory,
(files attributed to a host) + `files_analyzed` + `len(ungraded_files)` must equal the count of
`.rs` files on disk, **or** `not_measured` must be true — a single-file-path fix fails this.
**(a4) Detection completeness must itself be measured:** `is_include_fragment` matches on **names**
(`part*`, `*_tests*`, `html_*`, sibling-prefix), so diff it against ground truth — the targets
actually resolved from `include!("…")` — and ratchet false negatives and false positives toward 0,
or the bypass stays open to anyone naming a fragment `foo_impl.rs`. **(b1)** The unguarded count
becomes a `.pmat-ratchet.toml` metric whose predicate is the **resolved target set**, not a grep,
with `zero_is_reachable` set deliberately; falsify by adding one unguarded `include!` **inside the
pathspec the metric measures** (a scratch tree outside `src/` reproduces the UNMEASURABLE-vs-real-
zero trap CLAUDE.md records) and confirming `pmat comply ratchet` exits non-zero.

**Effort.** **L** overall; (a) alone is closer to M since the resolver exists. **Ship (a) and (b) as
separate PRs** — (b) is ~670 mechanical conversions with real `use super::*` name-resolution risk
(precisely the failure the `pmat_broken_tests` quarantine records for two families) and must not be
able to block (a), which carries the user-facing value.

**Risk.** Resolving `include!` makes this repo's own grades fall sharply and may break committed TDG
baselines and the ratchet — sequence the rebase with the change. Do (b) family by family. Note the
two parity criteria are in tension: even the include!-aware complexity path disagrees between split
and flat on the identical 839 functions (Max Cognitive 23 vs 25, Warnings 3 vs 5), so per-file
aggregation must be unified alongside attribution or the 2.0-point criterion fails for a second
reason.

**Re-measured counts (corrections).** **2,295 sites** — the "2,289, not 2,295" filed at merge is
**withdrawn** as a pathspec artefact, not a correction: `-- src` gives 2,295 and
`-- 'src/**/*.rs'` gives 2,289 at *both* commits, and the glob silently omits the 14 top-level
`src/*.rs` files (§3, §11). **2,061 fragment files** resolved over all of `src/`, which reproduces —
so the adjacent "not 2,066" needs the same pathspec note rather than a commit attribution;
`#[path = "` 174; unguarded test include!s **672 / 672 files / 212,856 lines** by one predicate and
386 / 123,901 by a stricter one — the merged 670 / 596 / 190,342 sits between two definitions, so
**the ratchet entry must carry the exact predicate as its reproduction command**.

**Related issue.** #1114 (its god-module claim — 16,789 lines by include! closure, not 10,478). Also
feeds #1035 via the directory walk's `files_ungraded: 0`.

---

### 8.12 CRUX-12 — 489 files hold 8,313 tests no build compiles; the instrument is wired to nothing

**Problem.** `analyze reachability` reports 407 orphans (126,933 lines, 6,292 `#[test]`) plus 82
quarantined behind `cfg(pmat_broken_tests)` (35,856 lines, 2,021 tests) and calls its own number a
**FLOOR** because 35 `mod` declarations could not be resolved. `src/tests/` alone is 188 files /
76,461 lines / 4,335 tests declared in no `mod` — 82 named `coverage_boost_*` (3,214 tests, 42,886
lines, added 2026-02-01..04-07), written to raise coverage and contributing none.
`Cargo.toml:30 autotests = false` makes the omission silent, and the set is growing:
`.pmat-ratchet.toml:77` records 166 files in `src/tests` where there are now 188.

**Evidence.** Live at HEAD, 0.32 s / 22,656 kB: `{"reachable":3944,"roots":137,"orphan_count":407,
"orphan_lines":126933,"orphan_tests":6292,"quarantined_count":82,"quarantined_lines":35856,
"quarantined_tests":2021,"unresolved_mods":35}`. `[o for o in orphans if
o.file.startswith("src/tests/")]` → **188**. `grep -nE '^(pub )?mod ' src/lib.rs | grep -w tests` →
nothing; no `src/main.rs`; `grep -n 'src/tests' tests/all.rs` → nothing.
`docs/status/unrun-tests-ledger.md` → "23659 of 26887 lib tests are executed; 3228 are compiled by no
leg", `<unsatisfiable>` = 2,199. **Gate absence:** `grep -rn reachability .github/workflows/ Makefile`
→ nothing; same on both hooks. The only caller is
`scripts/dogfood/pmat-dogfood-runner.sh:909-912`, whose `:911` is literally `mark reachability WARN`,
and nothing in `.github`/Makefile invokes that script. `--fail-on-orphan` → exit 1 in 0.34 s, so the
flag works. **The template exists and is gated:** `feature-matrix.yml:701` runs
`analyze unrun-tests --executed '' --check-ledger` and `:807` lists it in `feature-gate`'s `needs`.

**Proposal.** Mirror `unrun-tests`: `--write-ledger` / `--check-ledger` writing
`docs/status/orphan-files-ledger.md` with a per-file reason from a **closed enum**
(`registered-pending`, `deleted-<reason>`, `quarantined-<issue>`), a `.pmat-ratchet.toml` metric
whose command re-derives the count, and a `reachability` job in feature-matrix inside
`feature-gate`'s `needs`. Use a ledger rather than a bare `--fail-on-orphan`: 407 must ratchet down,
not block on day one. Then resolve `src/tests/` file by file — register (`#[cfg(test)] #[path]`) or
delete — expecting a compile-failure wave, which is the point. **Name the job
`reachability-ledger`:** feature-matrix already has an `orphan-ledger` job (`:326`, in
`feature-gate`'s needs) about orphan **features** in `Cargo.toml`; it shares the word and covers none
of this.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
grep -q '^\[metric.orphan_files\]' .pmat-ratchet.toml \
  || fail "no orphan_files ratchet metric"          # absent today (6 metrics, none this)
pmat comply ratchet -p . 2>&1 | grep -q orphan_files \
  || fail "ratchet does not evaluate it"            # absent today
pmat analyze reachability -f json \
  | jq -e '[.orphans[]|select(.file|startswith("src/tests/"))]|length == 0' \
  || fail "src/tests/ orphans remain"               # today 188
```
Every leg is `|| fail`-guarded and every `jq` carries `-e`: the merged snippet's first two legs were
bare `grep -q` that could not stop the script (no `set -e`), and its third ended in a bare `jq` with
no `-e`, so it returned 0 whatever reachability reported.
**CORRECTION:** the merged first leg, `pmat comply ratchet --format json | jq …`, **cannot be run** —
that subcommand has no `--format` (`error: unexpected argument found`; only `comply coherence` has
`-f/--format`). Either add `--format` to `comply ratchet` as part of the work (small, and it closes a
real machine-readability asymmetry) or assert against what exists, as above.
*Build-cost leg, labelled:* `pmat comply ratchet` exits 0 on unmodified HEAD and non-zero **naming
the file** after a `mod` line is deleted on a scratch branch — it shells to
`cargo clippy --features full` for one existing metric, so this leg costs a build.
*Instrument regression guard (0.3 s, passes today):* a two-file fixture where `pub mod b;` present →
`orphan_count 0` / exit 0, deleted → `orphan_count 1` naming `src/b.rs` / exit 1. Verified both ways.
*Anti-vacuity — the deletion hole the merged version left open.* `git rm -r src/tests/` drops
`orphan_count` 407 → 219, the ledger accounts for the rest, and **every leg goes green while 4,335
test functions and 76,461 lines leave the tree** — and "register or delete" makes mass deletion
on-policy. Add: (i) the ledger row carries `tests` and `lines` per file so a deletion is a diff, not
an absence; (ii) the CI leg fails if total declared test functions decreased while `orphan_count`
decreased. **CORRECTION — the merged floor does not close the hole it names.** The cited command's
own output at `512c60139` is **36,147** (`-- 'src/*.rs'` → 33,475 and `-- 'tests/*.rs'` → 2,672; note
that `src/*.rs` **is recursive** in git pathspec syntax and therefore already includes all of
`src/tests/`), while the floor quoted beside it was 4,340 — a number from a different scope, and not
even the right one for that scope (`-- 'src/tests/*.rs'` → **4,335**; 4,340 is the looser *occurrence*
count this item's own Risk paragraph warns against using). Wiring 4,340 gives a gate that stays green
after `git rm -r src/tests/`: 36,147 − 4,335 = 31,812 ≫ 4,340 — precisely the deletion hole the clause
claims to close. State the floor as the command's own current output with its exact pathspec and
regex (**36,147** at `512c60139`), and express the invariant as a **delta**: the leg fails if
`orphan_count` decreased by more than the number of files whose ledger rows changed to a non-deleted
reason. Better, follow clause (i) and assert **conservation**: Σ(ledger `tests`) + Σ(registered
tests) must not decrease; (iii) the quarantine class carries a ceiling, mirroring the
existing `pmat_broken_tests` ceiling of 47 enforced by a `--lib` test.

**Effort.** **M** for ledger + ratchet + CI leg. **Out of scope:** `#[serde(default)] reachable:
Option<bool>` on `BaselineEntry` and the TDG `grade_distribution` exclusion — real, but a separable
subsystem and what pushes an honest M toward L. (#1017's triage comment already specifies the
`BaselineEntry` change and names a second construction site at
`src/mcp_pmcp/tool_functions/quality_tools.rs:598` the merged record omits; omitting it would
reproduce #1029's CLI/MCP divergence class.)

**Risk.** Registering 4,335 tests surfaces a compile-failure wave and will tempt a second quarantine
— require a written reason per file and a ceiling. Quote the tool's numbers, not grep's: 4,335 tests
vs a naive grep's 3,872 lines / 4,340 occurrences; 76,479 lines vs `wc -l`'s 76,461.

**Related issue.** #1017 (OPEN; its triage comment states AC4 is NOT MET — "no hook, no CI job, no
verify stage"), #1018. **File it as "finish fable-review §4 EV-3"**: the analyzer half shipped in
3.32.0 and was corrected in 3.34.0, the gate half did not. EV-3's recorded blocker
(`run-declared-test-targets-in-gate`) does **not** apply to the `src/tests/` bulk — 188 of the 407 —
because registering those as `#[cfg(test)] mod` puts them in the **lib** target the existing
`cargo test --lib` leg already runs; only the 43 orphans under `tests/` need the `[[test]]` targets
first.

---

### 8.13 CRUX-13 — `analyze satd` does not walk shell scripts, and its census hides the omission

**Problem.** A directory whose debt lives in shell reports zero debt **and a balanced census**. Two
files with the same markers differing only in extension — `deploy.sh` (two) and `lib.rs` (one) —
yield `total_violations: 1, files_discovered: 1, files_analyzed: 1, census_balances: true` with every
`files_not_read` bucket zero. The census added to close #1035 discloses only files the walker
**admitted**, which makes the remaining blindness invisible rather than merely unmeasured.
`analyze complexity` on the same directory does it correctly:
`1 of 2 file(s) were not analyzed / no complexity analyzer for: .sh (1)`.

**Evidence.** Root cause is an extension allowlist: `SATDDetector::is_source_file`
(`src/services/satd_detector/detection_file_discovery.rs:180-206`) =
`rs|py|js|ts|jsx|tsx|java|cpp|c|h|hpp|cs|go|php|rb|swift|kt|scala|clj|hs|ml|elm`, applied inside
`find_source_files_partitioned` (`:59`) **during discovery**, and `discover_files`
(`detection_analysis.rs:46`) seeds the census from that already-filtered list — the census is
definitionally downstream of the drop. The capability exists: `CommentSyntax::for_path`
(`detection_extraction.rs:231-288`) handles
`sh|bash|zsh|fish|py|rb|r|jl|yaml|yml|toml|ini|cfg|conf|tf|mk|cmake|ps1|awk|gradle` with
`hash:true`. Scope: `git ls-files | wc -l` → 5,739; files matching the allowlist → 4,525 — exactly
`files_discovered`. **1,214 tracked files (21 %) never enter the denominator**: 73 `.sh`, 719 `.md`,
213 `.yaml|.yml`, 39 `.toml`, 4 `.ps1`, 2 extensionless (Makefile, Dockerfile). Python **is** already
walked (correction: drop it from the gap list). Compounding: default `analyze satd` → 3,
`--include-tests` → **64** across 31 files with `violations_listed: 10, violations_truncated: true`;
both `verify` and `quality-gate` run without `--include-tests`
(`quality_checks_part1_satd.rs:104` `let include_tests = false;`), so the two highest-signal
admissions in the tree (`BROKEN: coverage_tests_part1.rs truncated mid-expression`,
`BROKEN: complexity_tests_part1.rs truncated at line 500`) are reported by no gate. **A test pins the
defect:** `src/services/satd_detector_tests_extraction.rs:269` asserts
`!detector.is_source_file(&PathBuf::from("script.sh")) // .sh not in supported extensions` — invert
or delete it, and verify the new regression test fails on the old code before keeping it.

**Proposal.** Widen the walk at `detection_file_discovery.rs:180` (not at the census, not at the
formatter). For anything unclassifiable, count it into `files_discovered` and a new
`files_not_read` bucket `no_comment_syntax` so `census_balances` stays true and the summary names it.
**Design constraint:** `CommentSyntax::for_path` has a catch-all `_` arm returning a permissive
slash+hash union, so *nothing* is currently unclassifiable — `no_comment_syntax` is unreachable by
construction until `for_path` returns `Option<CommentSyntax>` (or a binary/vendored predicate is
added). Without that, "walk everything with the union" silently reads binaries as source.

**Acceptance test** (five independently red legs; every value measured).
```sh
printf '#!/bin/bash\n# shellcheck disable=SC2086\n# TODO: shell marker\n# FIXME: second\nT=$(mktemp -d /tmp/xXXXXXX)\n' > fx/deploy.sh
printf 'k: v  # HACK: yaml marker\n' > fx/ci.yaml; printf 'all:\n\t# XXX: make marker\n\techo hi\n' > fx/Makefile
printf '// TODO: rust marker\npub fn a(){}\n' > fx/lib.rs; printf '# TODO: python marker\n' > fx/s.py
printf 'Prose mentioning TODO: not a comment.\n' > fx/README.md; head -c 32 /dev/urandom > fx/blob.bin
pmat analyze satd --path fx --format json | jq -e '
  .total_violations==6 and .files_discovered==7 and .files_analyzed==6
  and .files_not_read.no_comment_syntax==1 and .files_not_read.total==1 and .census_balances==true
  and ([.violations[].file]|map(endswith("README.md"))|any|not)
  and ([.violations[].message]|map(test("shellcheck|bin/bash|XXXXXX"))|any|not)'
```
Plus: **(a)** three families must fire, one extensionless — the basenames must contain `ci.yaml`,
`Makefile` **and** `deploy.sh`, so an `.sh`-only fix dies. **(b)** truncation loud:
`--include-tests` must give `violations_truncated == false` or a listed count equal to the total.
**(c) denominator parity with the sibling that already gets this right** — not gameable by adding one
extension, since complexity's walk is independent: `satd.files_discovered` == complexity's "of N"
denominator (today 1 vs 7). **Say in the ticket what this leg does and does not assume:** §5.7 shows
five commands report five different denominators for one tree, and complexity's is the *broadest*,
which is why it is the right target here — but it is not a repo-wide oracle, and this leg must be
scoped to the fixture, never run over `src/`. **(d)** the summary must NAME what it could not read:
`grep -qE 'no comment syntax for: \.bin \(1\)'`. *The shebang and `# shellcheck` lines are the
false-positive falsifiers*; markers require the `TODO:` / `TODO(x)` shape (`// TODO fix` yields 0)
and matching is comment-scoped, so `mktemp -d /tmp/xXXXXXX` in shell **code** cannot fire.

**Effort.** **M**. **Scope split:** only the walk/census widening is one change; the
`violations_listed: 10` cap and "should the gates run `--include-tests`" are separate decisions with
separate blast radii (the latter changes what `pmat verify` fails on, fleet-wide).

**Risk.** Counts jump fleet-wide in one step — land with a ratchet rebase and a release note. **Be
honest about local scale:** on *this* repo the shell hole costs almost nothing —
`git ls-files '*.sh' | xargs grep -lE '#\s*(TODO|FIXME|HACK|XXX)'` returns **1 file / 2 lines**
(`tests/cli_integration_tests.sh`) and the Makefile none. **Do not pair that number with the
unanchored command:** dropping the `#\s*` prefix (i.e. counting markers in shell *code* and string
literals, not in comments) gives 9 files / 17 lines, which is a different measurement of a different
thing. Quote one command with its own output. The EV is the fleet-multiplied case plus the invisible-census defect, which is real
regardless of marker count.

**Related issue.** #1035. Note the maintainer has already triaged this row and dismissed the framing:
its final comment says the shell gap "is a scope question about `infra`, not a pmat defect; it needs a
shell analyzer, not a census fix". The counter-evidence is (i) no shell analyzer is required —
`CommentSyntax::for_path` already holds the `#` family, so this is walk scope — and (ii) the same
comment asserts "'0 violations' is no longer indistinguishable from '0 files examined'", which the
fixture falsifies directly. **That contradiction, not the shell coverage, is the load-bearing
finding.**

---

### 8.14 CRUX-14 — eight CI checks that compute a number and cannot fail a build

**Problem.** Enforcement, not the check graph, is where this repo is weak — and in eight places a
check exists, runs, and cannot fail. Two legs were added after the first merge (7 and 8); the item
is still split into three tickets below.

**Evidence** (each re-verified at HEAD). **(1) Coverage floor never armed.**
`grep -rn 'coverage_min|test_args|test_workspace' .github/workflows/*.yml` → nothing;
`ci.yml:14-17` passes only `repo:` to the org reusable workflow, whose floor step is
`if: ${{ inputs.coverage_min != '' }}` with correct ratchet logic. Stronger than the grep: on the
last four completed master runs (33313164568, 33311220200, 33305579323, 33238522607) the step
`Enforce coverage floor (OPT-IN ratchet …)` reports `"skipped"` in all four. **It has never
executed.** **(2) MSRV verified only after publish, and blind.** `post-release.yml:11-15` is
`on: release: types:[published]` + `workflow_dispatch`; `msrv-check` (`:66`) runs
`cargo check -p pmat --locked`, `--features full`, `cargo clippy` —
`grep -c -- '--tests\|--all-targets'` over that job → **0**. Probe:
`declared=1.91.0 graph_max=1.93.1 (serial_test_derive)`, exit 1; `serial_test` is a **direct
dev-dependency** (`Cargo.toml:396,407`). **(3) A required context asserts a supply-chain conclusion it
never measured.** `gh secret list` → `CARGO_REGISTRY_TOKEN`, `NPM_TOKEN` only.
`feature-matrix.yml:568-586`: `if [ -z "${GH_TOKEN:-}" ]` → `::warning::the LIVE Dependabot arm did
NOT run … This run verified the gate's logic, NOT this repository's advisories.` then `exit 0`;
`:807` puts `dependabot-alerts` in `feature-gate`'s `needs` and `:818`'s success echo ends "…and no
Dependabot advisory is open that cargo-deny cannot see". Confirmed live on run 33618635820. Open
alerts today: 0 — which is why it persists until it matters. **(4-5) Three inert
duplicate-dependency artifacts.** `bash scripts/check_dependency_duplicates.sh` →
`cd: scripts/../server: No such file or directory`, exit 1, and no caller anywhere.
`Makefile:1210-1211` ends both legs in `|| echo`. `tests/modules/dependency_duplicates_test.rs:18`
`MAX_DUPLICATE_PACKAGES = 70`; `:21-24` `CRITICAL_DUPLICATES` is **empty** and `:123` iterates it
(mathematically unfailable); `test_document_unavoidable_duplicates` (`:139-185`) is six `println!`
and no assertion. The only CI use of that binary is `feature-matrix.yml:652`, name-filtered to
`the_shipped_binary_size_stays_in_band`. Measured: **35** duplicate crate names (getrandom, rand,
rand_core, hashbrown each at three versions); `KNOWN_UNAVOIDABLE` = 28 of which **13 are stale**, and
**20 real duplicates undocumented**; `grep -c '^\[metric\.duplicate' .pmat-ratchet.toml` → 0.
**(6) semver-checks absent.** Not in any workflow; `Makefile:1582` ends `|| echo`; `:1609` pipes into
`grep` so the exit status is grep's; `scripts/release-checklist.sh:165-172` is `if command -v … else
WARN`. Branch protection contexts: `["ci / gate","feature-gate","docs build (docs.rs
environment)","pmat score","provable ladder"]`.
**(7) The enforcement metadata itself claims a gate that does not exist — twice, in opposite
directions.** `.github/workflows/mutation-diff.yml:50-66` declares its job a "REQUIRED CHECK
(PMAT-630 B2)" and quotes, under the heading "Verified against the live setting", a contexts list
ending `"pmat score","mutation-diff"`. The live list, above, does not contain `mutation-diff`.
Meanwhile `.github/required-status-checks.txt:21-24` — the manifest CB-2100 reads when it has no
admin API access — explains that `mutation-diff` is deliberately *not* listed because "the workflow
that produces that context … exists only on `feat/cb-2100-gate-effect` and is not on master". That
workflow **is** tracked on master (`git ls-files .github/workflows/mutation-diff.yml`). Two tracked
files contradict each other and both contradict live protection, inside the one artefact whose whole
job is to record what enforces what — this audit's own thesis, in the enforcement metadata. #1034.
**(8) Seven of fourteen `deny.toml` advisory ignores match nothing, and cargo-deny reports that as a
warning inside a green `advisories ok`.** Five of the seven name crates absent from `Cargo.lock`
entirely, with reason strings describing a tree (indicatif, ratatui, nalgebra, simba, apr-cli) that
no longer exists. An ignore matching nothing is indistinguishable, at the exit code, from an ignore
doing its job. A prior manual prune was tried and reverted because local and CI cargo-deny disagree
about `paste` — which is why the fix below is a **conjunction**, not a prune.

**Proposal — split into three tickets;** the merged item bundles work with three owners and three
blast radii. **CRUX-14a (L, code-only): legs 1, 2, 4-5.** Pass `coverage_min:` from `ci.yml` at the
measured baseline; move `msrv-check` to `pull_request`; delete the dead script, give the two
unfailable tests assertions, prune the 13 stale entries, move the real number into
`.pmat-ratchet.toml` as `duplicate_crate_names` baseline 35. **Legs 7 and 8 join 14a**: leg 7 is S and
code-only — make one artefact authoritative by deleting the contexts list from `mutation-diff.yml`'s
comment, pointing it at `.github/required-status-checks.txt`, and correcting that file's stated
reason (the workflow is on master; the honest reason is that the context was never added to
protection). Leg 8 is M — add a check that fails when an `ignore` entry's advisory matches nothing
**AND** the crate its `reason` names is absent from `Cargo.lock`; the conjunction is what makes it
environment-safe, since the `paste` disagreement is about *matching*, not presence (`paste 1.0.15`
IS in the lockfile). Mirror `.github/dependabot-acknowledged.txt`'s rule and give every surviving
entry an expiry. **CRUX-14b (S): leg 3, restated as a
code-testable invariant** so it does not depend on an owner action no PR can land —
`feature-gate`'s success echo may not contain a Dependabot-advisory clause unless the same run's
`dependabot alerts` job log contains neither `did NOT run` nor `NOT this repository's advisories`. A
code-only PR (deleting the clause) satisfies it today; provisioning `DEPENDABOT_TOKEN` satisfies it
the other way. The missing-secret behaviour was argued on purpose in 3.34.0 ("failing on the missing
secret would make a required gate permanently red") and #1074 tracks the PAT — so the genuinely new
finding is the **contradicting echo**, not the dark arm.
**CORRECTION — the echo invariant alone sits at the observation, not the cause, and it is the one
item in §8 whose cheapest satisfying fix makes the record quieter without making anything able to
fail.** Deleting a sentence from an echo leaves a *required* context (`feature-gate`) still exiting 0
over an unmeasured supply-chain dimension — which §12.2 rule 3 (a fix goes where the defect is
caused) and rule 4 (a fix may make a gate able to fail more, never less) both forbid. Keep the echo
invariant as the S-effort **floor**, and add the causal half the merged item omitted: have the
`dependabot-alerts` leg emit an explicit `not_measured` verdict when `GH_TOKEN` is absent, and have
`feature-gate` surface that as a **distinct, visible state** — a workflow annotation plus a
`not_measured` row in its job summary — rather than as `success`. The deliverable is making "we did
not measure this" visible, not making the boast smaller. Until `DEPENDABOT_TOKEN` exists (#1074) the
dimension is unmeasured; say so where a reviewer sees it.
**CRUX-14c (separate EV): leg 6.** Do **not**
make semver-checks required: pmat has **zero reverse dependencies** on crates.io (measured) and
~9,000 public items with no declared API boundary; a required gate there fails on ordinary refactors
and gets disabled — the exact outcome `feature-matrix.yml:564-565` warns about. Declare the supported
surface first (an `api` module plus `#[doc(hidden)]` on the rest) and require it only over that, or
keep it advisory.

**Acceptance test.** **(1)** `grep -q 'coverage_min' .github/workflows/ci.yml` — **0 hits today**,
`ci.yml:14-17` passes only `repo:` — and the floor step's conclusion must be `"success"`, not
`"skipped"`, in the next master run. **Plus the clause the merged version lacked, without which the
leg is gameable in the cheapest possible way:** nothing above ties the committed floor to the
measured coverage, so `coverage_min: 1.0` makes the step execute, report `success` forever, and
survive any synthetic-drop demonstration (a drop to 0 % still trips a 1 % floor, so even the redness
leg passes), while "commit `floor(measured*100)/100`" is prose in the Proposal and not an assertion
here. Add an assertion **in this repository** — the upstream floor step lives in `paiml/.github` and
could not be read — that parses `coverage_min` out of `ci.yml` in the same job that produces the
coverage number and fails when `measured - coverage_min > 0.5`: a floor far below reality is
indistinguishable from no floor. Redness is then shown with a synthetic drop sized **relative to the
committed floor**, not to jitter; the upstream compares 2-decimal floats, so measure jitter first and
commit `floor(measured*100)/100`. **(2) SPLIT:** (i) *normal + build* deps ≤ declared `rust-version`,
on `pull_request`, both `default` and `full`, `--locked` — the real `cargo install` contract, **green
today**, must stay green; (ii) dev-deps in a separately named step
`cargo +$MSRV check --locked --all-targets`, whose **only permitted fix is pinning `serial_test`,
never raising `rust-version`** — plus a guard that `rust-version` did not increase vs the merge-base
without a CHANGELOG entry. The merged single assertion ("`msrv_probe.py` must exit 0") is satisfied
most cheaply by raising `rust-version` to 1.93.1, which **reintroduces the v3.24.0 `cargo install`
break the item cites as motivation**. Note `serial_test` being a dev-dep means this cannot break
`cargo install` today — it breaks `cargo +1.91.0 test` for a contributor, and the point is that no
job can see either. **(3)** the two-grep invariant above, with the five-arm `--self-test` still
passing in the same run so the gate cannot go quiet by deletion. **(4-5)**
`test ! -e scripts/check_dependency_duplicates.sh` AND `grep -c '^\[metric\.duplicate'
.pmat-ratchet.toml` ≥ 1 AND the ratchet must fail on a scratch tree with one extra duplicate.
**(6)** on a scratch branch changing a re-exported `pub fn` signature the job must exit non-zero —
that half is what stops a job running semver-checks with `|| true` from counting as enforcement.
**(7)** `gh api repos/paiml/paiml-mcp-agent-toolkit/branches/master/protection --jq
'.required_status_checks.contexts'` must equal **every** in-repo claim of required status — the
manifest `.github/required-status-checks.txt` and any contexts list quoted in a workflow comment.
Anti-vacuity: editing either list to disagree must turn the check RED. Today the comparison fails
against two files at once, in opposite directions. **(8)** `cargo deny check advisories 2>&1 | grep -c
advisory-not-detected` is **7** today and must fall to ≤ 2 (the environment-disputed `paste` plus at
most one other, each with a recorded reason) **while the command still exits 0 and still prints
`advisories ok`** — so deleting the whole `[advisories]` block cannot satisfy it.

**Effort.** **L** for 14a; **S** for 14b; 14c is not S and should be ranked on its own.

**Risk.** Arming several gates at once turns master red in several places on the first run — one per
PR, baselines measured never aspirational. The coherence binding for the duplicate metric must be
re-classified when it lands, or `comply coherence` keeps reporting the key VACUOUS.

**Related issue.** #1074 (the PAT), #1018 (the unfailable tests), fable-review §4 items 9 and 16 —
both still unshipped. **Honour EV-9's recorded precondition**: it is
`blocked_by: run-declared-test-targets-in-gate`, and §5 bans chasing a coverage number "before the
test corpus actually executes", because today's measurable baseline is `--lib`-only. Say so, or land
it after CRUX-12.

---

### 8.15 CRUX-15 — `--format` is not a contract on `analyze`

**Problem.** **(a)** `analyze reachability`, `vacuous-tests`, `unrun-tests` and `hardcoded-paths`
declare `--format` with no `[possible values:]`, so `--format sarif` silently yields human text and
`--format banana` produces **byte-identical** output — while `analyze complexity --format banana`
correctly exits 2. **(b)** `analyze dag` accepts no `--format` at all, so pmat's own dependency graph
is Mermaid-only. **(c)** SARIF is structurally valid from many subcommands but **no workflow uploads
any of it**, the checks unique to pmat cannot emit it, `artifactLocation.uri` mirrors `-p` verbatim,
and one binary emits several `tool.driver.name` values.

**Evidence.** (a) On a git fixture, `{banana, sarif, summary}` are `cmp`-identical for all four
subcommands while `--format json` differs (json IS honoured); the exit code is the *analysis's*
verdict, not a usage error. Measured with a clap-aware parser over the 35 `analyze` subcommands: 33
accept `--format`, **29 enumerate their values in 22 distinct sets**, and exactly **4** are
free-string — precisely the four named. (The merged "only 12 enumerate, in 8 sets" counted clap's
inline rendering and missed the multi-line `Possible values:` block.) (b) `analyze dag --format json`
→ `error: unexpected argument found`, exit 2; `analyze clippy` also has no `--format`, so it is **two
of 35**, not one. (c) `grep -rniE 'sarif|codeql|upload-sarif|code-scanning' .github/workflows/ | wc -l`
→ **0**. Six emitters produce valid 2.1.0 with **five** distinct `tool.driver.name` (`pmat`,
`pmat-satd-detector`, `pmat-tdg`, `paiml-provability-analyzer`, `paiml-proof-annotator`), **two**
`$schema` URIs, `uriBaseId` absent everywhere; `analyze tdg -p "$PWD/fx" --format sarif` emits an
**absolute** uri, and one result's uri is a **directory** (`fx`), which code scanning cannot map.
`src/cli/defect_helpers/format_sarif.rs` exists while `analyze defects --help` enumerates
`[text, json, junit]` and rejects sarif at exit 2 — a SARIF writer already written and unreachable.
Repair scope: `grep -rnE 'analyze (reachability|vacuous-tests|unrun-tests|hardcoded-paths)' Makefile
scripts/ .github/workflows/` → 5 hits, **none passing a `--format` value enumeration would newly
reject**, so leg (a) breaks nothing in-tree.

**Proposal.** **Split.** *CRUX-15a (S)*: four clap enums + `--format <mermaid|json|dot>` on
`analyze dag`. *CRUX-15b (L, not M)*: one SARIF writer, `tool.driver.name = "pmat"`, a stable
`automationDetails.id` per family, repo-relative URIs plus `uriBaseId: "%SRCROOT%"`, `--format sarif`
on the differentiators, and an upload step. The L is measured:
`git grep -lE '^\s*(pub )?fn [a-z_]*sarif[a-z_]*\(' -- src | grep -v tests | wc -l` → **44** non-test
files define a SARIF formatter (**93** including tests, 100 over the whole tree), including two
byte-identical duplicates
(`src/cli/analysis_utilities/tdg_formatting.rs:56` and
`src/cli/handlers/tdg_handler_formatting.rs:54` both hardcode the same empty `pmat-tdg` envelope).
Forty-four independent writers, over the corrected emitting set of **17 `analyze` subcommands plus
`pmat tdg`** (§6a), is what makes this L: calling it M invites a partial landing that renames six
string literals and declares victory. **CORRECTION:** an earlier draft of this item carried **60**
for that command; re-run verbatim at `512c60139` it returns 44, and no variant of it returns 60
(`[[:space:]]` for `\s`: 44; `git grep -lP`: 44; case-insensitive: 44; without `grep -v tests`: 93;
whole tree: 100; without the `^\s*` anchor: 48). Use 44.
**Reuse the existing precedent:** `pmat comply check --format sarif` already emits
`"uriBaseId": "%SRCROOT%"` (`src/cli/handlers/comply_handlers/check_handlers/check.rs:724`, landed in
`518436dd6` because comply's SARIF leaked the author's home directory).

**Acceptance test.** **(0)** for each of the four: `--format banana` → exit **2** AND
`--help | grep -q '\[possible values:'` AND `--format json | jq -e 'type=="object"'` AND
`--format summary` exits ≤ 1 — the four together block "delete `--format`", which otherwise makes
`--format banana` an unexpected-argument error and the test green with the feature gone.
**(1)** `analyze dag --format json | jq -e '.nodes|length > 0'`.
**(2)** `analyze vacuous-tests --format sarif | jq -e '.runs[0].tool.driver.name=="pmat"'`, plus one
`$schema` across the emitting set (today two), `automationDetails.id` present and distinct per
family, and every `artifactLocation.uri` relative, not `./`-prefixed, and `test -f`-resolvable under
the repo root — today `analyze tdg` emits a bare **directory** as a result uri, which an empty result
set would never expose. Keep `results|length >= 1`.
**(3) — REPLACED.** `grep -rq 'upload-sarif' .github/workflows/` passes on a job with `if: false`,
`continue-on-error: true`, or `workflow_dispatch`-only — and the merged mitigation ("gate it behind a
label or a scheduled run initially") produces exactly that. Assert instead that the uploading step has
no `continue-on-error` and its job triggers on `push` to master (parse the YAML, do not grep), **and**
`gh api …/code-scanning/analyses --jq '[.[]|select(.tool.name=="pmat")]|length'` ≥ 1 with `created_at`
newer than the merge commit. If the alert backlog forces staging, stage as master-push with
`category:` set and severity filtered — never `if:`-disabled.
**Removed leg.** `analyze dag --help | grep -A1 '^Usage:' | tail -1` non-empty **cannot pass and is
mis-attributed**: that pipeline returns the blank line after `Usage:` even on a correct binary, and
the bare `Usage:` is universal (all 35 analyze subcommands, plus `pmat`, `query`, `quality-gate`,
`verify`) — it is CRUX-05's defect, and fixing it inside dag would be a symptom patch.

**Effort.** **S** (15a) + **L** (15b).

**Risk.** Enumerating values starts rejecting invocations that silently succeed today (checked: none
in-tree). Uploading SARIF creates a first-run alert backlog. **Non-goal:** satisfying leg (2) by
editing six string literals without unifying the writer.

**Related issue.** #1029 (sibling class: new differentiator subcommands are second-class). No issue
covers `--format` enumeration, `analyze dag --format`, or SARIF upload; `fable-review.md` has zero
hits for sarif/format/dag.

---

### 8.16 CRUX-16 — `pmat mutate` does not exist, yet README sells it seven times

**Problem.** `pmat mutate --target src/` exits 2 with `error: unrecognized subcommand` on **both** the
HEAD build and the installed crates.io 3.34.0 binary. README documents it **seven times**
(`README.md:270,336,337,338,497,547,561`), including a copy-pasteable CI step
(`- run: pmat mutate --target src/ --threshold 80`) and the Mutation-Score row of the "Falsifiable
Quality Commitments" table, whose Verification Method is `pmat mutate --threshold 80` — a table of
falsifiers naming a falsifier that cannot be run, under "Failure = Regression: Any commitment
violation blocks CI merge". Underneath, five adapters return `TestRunResult { passed: true }`
unconditionally, so `engine.rs:154-158` records every mutant as `Survived` and reports a 0 % kill
rate as a *measurement*.

**Evidence.** `pmat --help | grep -ci mutate` → 0; `strings -a pmat | grep -icE '^mutate$'` → 0.
Gate: `Mutate(MutateArgs)` is `#[cfg(feature = "mutation-testing")]` at
`src/cli/commands/commands_enum/definition.rs:1349-1350` and `AnalyzeCommands::Mutate` at
`src/cli/commands/analyze_commands/mod.rs:1820-1821`; `Cargo.toml:461` `default` excludes it.
`grep -rn 'passed: true' src/services/mutation/*adapter*.rs` → **5** (`lua:64`, `go:77`, `cpp:77`,
`python:78`, `wasm:78`); `go_adapter.rs:75` still says `// Minimal implementation for now` (`:74` is the `run_tests`
signature; the stub body is `:74-77`); the
control `rust_adapter.rs:78` is `passed: output.status.success()`. **The tree's own disclosure
mechanism deliberately exempts it:** `src/cli/commands/feature_gate_disclosure_tests.rs:20-22`
excludes `mutate` on the rationale that it is "`#[cfg]`-ed out of the enum entirely, so [it] never
appear[s] in `--help` and **cannot mislead**". `README.md:497` falsifies that directly.
**CORRECTION:** `pmat --help | grep -c 'NOT AVAILABLE'` → **3** (demo, org, agent), not seven (6 total
in `src/`, three of them nested). Sharper as three: the placeholder mechanism demonstrably exists and
`mutate` was never given one. No gate could catch it —
`grep -rn 'BAD COMMAND' .github/workflows/ scripts/ Makefile` → nothing; the only hit is prose at
`CLAUDE.md:308`.

**Proposal.** (a) Decide `mutate`: a thin wrapper shelling to `cargo mutants --in-diff` (consistent
with how pmat already drives bashrs, pv and cargo-mutants as subprocesses — no new crate dependency),
**or** delete all seven README claims. (b) Regardless: replace each stub `run_tests` with
`bail!("mutation backend <lang> is not implemented")` or delete the adapter. (c) Promote CLAUDE.md's
prose extractor into a `--lib` test asserting every `pmat <subcommand>` cited in README.md and
CLAUDE.md parses. **Sequencing:** (a)+(b) leave `engine.rs`'s temp-dir never-kill path intact
(`write_temp_mutant` → `run_tests(&temp_file)`, so a mutant can only be CompileError or Survived), so
the keep-the-command route must depend on fable-review §4 item 5 (`mutation-kill-path`) — otherwise
the credibility fix ships a command that still reports a number it did not measure.

**Acceptance test.** The merged extractor `grep -ohE 'pmat [a-z][a-z-]+'` prints **six** BAD lines,
five of them English prose (`pmat itself as`, `pmat as a subprocess`, `--transport http pmat
http://…`, `[pmat book](…)`, `every pmat invocation cited`) — red for the wrong reason, and the
cheapest way to green it is tightening the regex, i.e. the gaming path. Corrected extractor:
restrict to fenced ` ```bash `/` ```sh `/` ```yaml ` blocks and inline backticks, require `pmat` in
command position, reject a following `:` or `/`:
```python
CMD_START = re.compile(r'(?:^|[|;&]|\$\(|&&|\|\||^\s*-\s*run:\s*)\s*pmat\s+([a-z][a-z0-9-]*)(?![:/])')
```
Measured with that regex: **22 distinct subcommands**, BAD list = exactly `BAD: mutate`, exit 1.
*Anti-vacuity.* **(i) Positive control**, run before the loop, erroring if the extracted set lacks
both a known-good command (`analyze`) and — until resolved — `mutate`; narrowing the regex to drop
`mutate` then FAILS instead of passing. **(ii) Floor:** ≥ 20 distinct subcommands against this
extractor (22 today), or set equality against a committed list. **(iii) A disclosure stub is not
enough:** every invocation lifted from a fenced block must parse in the **default** build; a
non-default command is allowed only if the same block also contains `--features <feat>`. Assert the
Commitments table specifically — each row's Verification Method must parse in the default build.
**(iv) Replace the merged control for (b)**, which pins the bug: "the Rust adapter must still return
Ok on a passing fixture" is satisfied by exactly the broken behaviour (`rust_adapter.rs` runs
`cargo test` at the *cargo root*, ignoring the mutant it was handed). Use a behavioural control — on
`examples/rust-mutation-testing/` a run must report `killed > 0 AND survived > 0` (structurally
impossible today), and a backend that cannot measure must emit `not_measured`, never `0 %` and never
`Ok`. Drop `grep -c 'passed: true' == 0` (gameable by `let passed = true;`); keep the assertion that
`GoAdapter::run_tests` returns `Err` naming the backend.

**Effort.** **S** for delete-README + `bail!` + the parse test; **M** if the wrapper is built.

**Risk.** A wrapper introduces a runtime tool dependency — it must report `not_measured` when
cargo-mutants is absent, or it recreates the class this audit is about. fable-review §5 bans an Nth
*language* backend; a cargo-mutants wrapper is not that.

**Related issue.** #1018 (its acceptance criterion is unsatisfiable on 3.34.0 as written; its body
warns the "can never Kill" claim needs re-measuring), #999 (EV-4), #1035; fable-review §4 items 5 and
17. **File (b) as executing `mutation-kill-path`**, whose definition_of_done already says "delete or
honestly-error the Go/Python/C++/WASM stub adapters (bail 'backend not implemented', never
`passed:true`)". The genuinely new half is (a)+(c): that `mutate` does not parse **at all** in every
shipped binary, which fable-review assumed it did.

---

### 8.17 CRUX-17 — no machine-readable failure contract

**Problem.** Success paths emit clean JSON; failure paths emit **nothing** on stdout and English
prose on stderr, so a JSON consumer gets an empty-input parse error and must scrape prose. One
condition — "the path you gave does not exist" — produces three exit codes and five sentences, one a
flatly wrong `Permission denied (os error 13)` for a path that does not exist. The exit-code layer
was correctly rebuilt and then barely adopted, and `not_measured` — the one good convention — has
four incompatible JSON types.

**Evidence.** Sweep over every `analyze` subcommand with `--format json -p /nonexistent-dir-xyz-777`:
**n=35, fail=35** (the merged 36/≥30 counted `help`, which is skipped before `n` increments). 33 of
35 emit **0 bytes** on stdout; the exceptions are `dead-code` (472 B of `{"not_measured":true,…}`,
not an error envelope) and `build-tdg` (24 B of progress prose, worse than 0 for a JSON consumer).
Exit spread on that one condition: **1 ×25, 2 ×8, 5 ×2**. `pmat -q tdg /nonexistent-dir-xyz-777` →
`Error: Permission denied (os error 13)`, exit 1; `pmat -q tdg ./definitely-not-here` → prints `0.0`
on **stdout**, exit 5. Adoption, outside `src/cli_exit.rs` and excluding tests: `with_code(` = 0,
`quality_gate_failure(` = **0**, `ExitCoded` = 0, `configuration_error(` = 2, `analysis_error(` = 7
(the record's 13 was a grep-boundary artefact; 2+7 = the 9 raise sites its prose claims);
`process::exit(` = **70**, of which **52** are `exit(1)`. `ExitCode::QualityGateFailure = 3`
(`src/cli_exit.rs:40`) has **no raise site** — `quality_gates_handler_execution.rs:68` is
`std::process::exit(1)`, bypassing `real_main`, and `quality-gate --help` documents "Without it,
blocking violations exit 1". Stale classifier lore in **five** places, one a live test forbidding a
user-facing message from containing the word "violation"
(`src/cli/handlers/satd_handler_analysis.rs:64,298-303`; also `src/cli/mod.rs:408`,
`src/services/big_o_analyzer_analysis.rs:24`,
`src/cli/handlers/analyze_defects_handler/handler.rs:22`) — while `src/bin/pmat.rs:181` now delegates
to `cli_exit::code_for`, whose own test asserts the opposite. `--format json` does not imply quiet:
`analyze complexity --format json -p fx > merged.txt 2>&1` → exit 0 and `json.tool` fails at char 0
(four lines of emoji progress precede the `{`); split streams are 1,531 B of valid JSON and 264 B of
prose. `not_measured` types measured live: **str** (verify — 895 B of clean JSON on stdout with 0 B
stderr, the model shape), **bool** (tdg), **bool** (dead-code), **array-of-objects** (MCP
`quality_gate`).

**Proposal.** One `cli::json_error(code, message, detail) -> Value` emitting
`{"error":{"code":…,"exit":…,"message":…,"path":…}}`, called from `real_main`, which already owns
`write_fatal_error` and knows the resolved `--format`. Route
`quality_gates_handler_execution.rs:68,109` through `cli_exit::quality_gate_failure`; add a clippy
`disallowed_methods` entry for `std::process::exit` outside `src/bin` and `cli_exit`; delete or give
a raise site to `PermissionDenied`; make a machine `--format` imply quiet unless `--verbose`; unify
`not_measured` to one type. **Do not merely delete** the stale wording assertion — replace it with
the assertion it should always have been (the message MAY contain "violation"; the exit code is the
one declared at the raise site).

**Acceptance test.** **Leg 1 is unsatisfiable as merged and must be split:** 8 of 35 subcommands
(`dag, makefile, name-similarity, clippy, wasm, cluster, topics, models`) exit 2 with
`error: unexpected argument found` **inside `Cli::parse()`, before `real_main` runs**, so the
proposal physically cannot make them pass. Drive each subcommand with *its own* valid path syntax so
every probe reaches a runtime failure, and add a separate assertion that a genuine usage error
(`analyze complexity --no-such-flag`) still exits 2 with **no** envelope, so misuse is not swallowed
into the error object. Harden against a hardcoded constant: per probe assert
`d["error"]["exit"] == <observed process exit status>`, a non-empty `message` containing the
offending path, and stderr still non-empty and still containing that path (the risk section promises
the human line survives — assert it, or the fix buys JSON purity by deleting prose). Then the
discrimination check: path-not-found must yield **one** code across all probes (today three), **and**
a different refusal class — an existing but empty directory, today `tdg` exit 5 "no gradable source
files" — must yield a **different** code. A single hardcoded string fails the second half; a
per-command hardcode fails the first.
**Leg 2:** `pmat quality-gate -p fx_bad --checks complexity --checks satd; test $? -eq 3` (today 1)
with the control `-p fx_good` → 0 (measured: 1 and 0). **`--checks` is repeatable, not
comma-separated**, and must be scoped: with the default check set even a one-line clean crate exits 1
because the coverage gate correctly refuses — so the merged control would have passed vacuously for
the wrong reason. **Leg 3:** `analyze complexity --format json -p fx > merged.txt 2>&1 && python3 -m
json.tool merged.txt` (today `Expecting value: line 1 column 1`), control: human mode still emits
> 100 B on stderr (measured 264 B). **Leg 4 — replace; the merged version has a lazy-deletion pass.**
"At most one non-absent type … with ≥ 2 of 3 emitting" is satisfied by **removing** `not_measured`
from `pmat verify`, leaving tdg + dead-code (one type, two emitters, green) while deleting the
895-byte/0-stderr refusal envelope the item nominates as its own model. Require **all three** to emit
a non-absent `not_measured` of the same type, and require `verify --format json` in a Cargo-less dir
to still carry a non-empty human reason plus per-stage `not_applicable` prose. Add the same clause
for `quality_gate` over MCP, the fourth type.

**Effort.** **XL, not L.** Four independent changes; the merged item is larger than its largest part.
**Sequence:** leg 4 (S) and leg 2 (M) first — both independently valuable and gateable — then 1 and 3.

**Risk.** Changing exit codes breaks scripts keying on 1 — ship behind a documented table and a
changelog entry, and keep the human stderr line byte-identical so prose scrapers survive. Two unnamed
blockers for leg 2: `tests/modules/quality_gate_exit_status.rs:138` asserts `run.code == Some(1)` (a
hard-won regression test whose stated intent is "non-zero" — **update it to `Some(3)`, never delete
it**, and record why in its docstring), and `docs/SPECIFICATION.md:75` states "fail-fast semantics
(exit code 1 on violation)" and must change with it.

**Related issue.** #1114 (its claim 17 — `anyhow` over `PmatError` — is the same surface; `PmatError`
has zero production callers, so "wire it or delete it" belongs here), #1035.

---

### 8.18 CRUX-18 — the flagship feature has no token budget

**Problem.** README's first line sells "Zero-configuration AI context generation for any codebase",
and `pmat context --format llm-optimized` emits **8,670,750 bytes** — ~2.17 M tokens (ESTIMATE,
chars/4), larger than any model's context window — with no size control anywhere. The full `--help`
has no `--max-tokens`/`--map-tokens`/`--budget`/`--top-k`; the only related flag,
`--include-large-files`, *increases* size and is a documented no-op that prints "Flag ignored" while
`--help` describes it as working; and the MCP `generate_context` schema is exactly
`[paths, format, max_depth, include_dependencies]`.

**Evidence.** `wc -c` → 8,670,750; 5.53 s / 287,708 kB (re-measured at HEAD; the research run was
5.99 s / 284.8 MB — run-to-run noise). The `Context` clap variant
(`src/cli/commands/commands_enum/definition.rs:87-120`) declares eight args and no size control.
`src/cli/handlers/utility_handlers/context_generation.rs:51-53` prints
`Warning: --include-large-files is not yet implemented. Flag ignored.` **CORRECTION:** a binary-wide
`strings` sweep finds `max_tokens` on **4** lines, not 1 — none a `pmat context` flag (the
`analyze duplicates` clap arg, its "required argument" message, a secret-scanner keyword list where
`max_tokens` is listed as a NON-secret token name, and an embedded YAML comment); all 61 `max_tokens`
hits under `src/` are duplicates-extraction. The gap is also a live docs contradiction:
`docs/specifications/components/infrastructure.md:91` lists "Token budget optimization" as an
existing capability, and none of the nine `pmat prompt` subcommands exposes any budget flag.
Competitors, accessed 2026-09-02: Aider `--map-tokens`, default **1k**, personalised PageRank
(https://aider.chat/docs/repomap.html); repowise 2,391 vs 64,039 tokens
(https://www.repowise.dev/guides/ai-context-mcp); jCodeMunch 86–99 %
(https://github.com/jgravelle/jcodemunch-mcp).

**Proposal.** Add `--max-tokens N` to `pmat context` and `max_tokens` to `generate_context`.
**CORRECTION — the named ranking substrate provably scores nothing.** The merged proposal says "spend
the budget by descending PageRank over the `ProjectContextGraph` that `analyze_project_with_cache()`
already builds". `src/services/context_impl/build.rs:375-397` (the doc comment on
`build_context_graph`, called from `visitor.rs:483`) states: "No edges are extracted, so the graph has
no relationships and PageRank produces nothing: `num_edges()` is 0 and `hot_symbols()` is empty for
every project", and `:433-434` "PageRank over an edgeless CSR scores nothing, so this is a no-op in
practice". `pmat query --rank-by pagerank` is likewise inert here — measured,
`--limit 30 --files-with-matches` diffs **IDENTICAL** with and without the flag. Rank instead by
signals the deep-context path already computes and prints per file at no extra parse:
`**File Complexity**`, `**Functions**`, and per function `[complexity] [cognitive] [big-o] [satd]
[churn]`. If a genuine call-graph ranker is wanted the only working one is
`src/services/dag_call_edges.rs:24 add_call_edges` (Rust-only) — the second parse `build.rs:391-393`
warns about ("re-parsing every source file a second time, on a path whose cost is already dominated
by parsing") — so **raise the effort to L** on that route. Fix or remove `--include-large-files` in
the same change.

**Acceptance test.** Seven legs; all red today at L1.
```sh
set -euo pipefail
pmat context --format llm-optimized --max-tokens 20000  -o /tmp/c20k.md   # L1: today exit 2
pmat context --format llm-optimized --max-tokens 200000 -o /tmp/c200k.md
pmat context --format llm-optimized                     -o /tmp/cfull.md
b20=$(wc -c </tmp/c20k.md); b200=$(wc -c </tmp/c200k.md); bf=$(wc -c </tmp/cfull.md)
test "$b20" -lt 90000; test "$b200" -gt "$b20"; test "$b200" -lt 900000   # L2 monotone
test "$bf" -gt "$b200"                                                    # L3 default unchanged
grep -qE 'tokens (used|realised)[^0-9]*[0-9]+' /tmp/c20k.md               # L4 declared
grep -qE '[0-9]+ files? (dropped|omitted|elided)'  /tmp/c20k.md
```
**L4b reconciliation:** parse `### ` headers from both outputs and assert
`len(kept) + declared_dropped == len(full)` and `set(kept) ⊆ set(full)`, so the declaration cannot be
a decorative constant. **L5 content survives, not just paths:** ≥ 75 % of kept file sections must
carry a non-blank line after the header — the output is 3,633 `### ` sections of which 305 are
already content-free, and emitting all 3,633 bare headers is only 173,156 bytes, so a paths-only
"fix" passes the merged legs 2 and 3 outright. **L6 ranking is real:** the kept set must not be a
prefix of emission order (`|kept ∩ full[:len(kept)]| / |kept| < 0.80`). **L7 depth:** something from
the last quartile of the full output must survive. L6/L7 replace the merged anti-vacuity leg, which
was `grep -qF "$(pmat query …)"` — measured, `grep -qF "$(true)"` **exits 0**, so an empty command
substitution (missing index, query error) makes the leg silently pass, and its oracle does not
measure what it claims since `--rank-by pagerank` changes nothing here. Separately:
`--include-large-files` must either change the byte count on a fixture containing a >500 KB file, or
be gone from `--help`.

**Effort.** **M** with in-tree per-file signals; **L** if a real call-graph ranker is built.

**Risk.** Truncation that is not clearly declared creates a **new** instance of the
not-measured-reads-as-complete class — L4/L4b are not optional. Ship opt-in; do not change the
default. **Sequencing:** fable-review §5 bans expanding AI-context scope before
`context-output-fixture-gate` (§4 EV-11) exists, because the production formatter has zero content
gates. CRUX-18 adds no format and no MCP tool so it is not banned — but a *truncating* formatter is
the single change most likely to silently drop content on a surface nothing asserts, and L5/L6/L7
**are** the first content assertions on that formatter. **Bundle the two, land them in one PR**, and
revive `src/tests/extreme_tdd_context_fix.rs` as EV-11 specifies.

**Related issue.** The `--include-large-files` no-op is cited in-code as GH-46, but issue #46 is
"Sprint 1 #003: Migrate Dead Code Command to Uniform Contracts" (CLOSED, unrelated); the real tracker
is **#269** (CLOSED, "silent CLI flag discards … rename `_param` → `param`, add warning to stderr") —
a closed issue standing over a flag still unimplemented and still described as working. Fix that
reference in the same change. No open issue mentions a context token budget.

---

### 8.19 CRUX-19 — `pmat score` grades the repo with whichever `pmat` is first on `$PATH`, discards its child's cost disclosure, and has no timeout

**Problem.** Three score-owned defects: a wrong-binary measurement, an undisclosed invocation of the
one command this project forbids, and no timeout on it. **The 9.8–12.7 GiB peak is context, not the
subject:** none of the four proposed fixes reduces it — most of it is #1014's known-open
`pmat comply check` cost (`COMPLY_BYTES_PER_WORKER = 4 GiB`), which is separately tracked and
separately owned, and which is why the band test below has to be split into two numbers. The item is
titled for what it fixes.

**Evidence.** §5.4. Two runs of the identical command in the identical tree: **9.76 GiB** and
**12.71 GiB** (a 30 % swing) at load 7.18 with 110 GB free, 11.3 M minor faults ≈ 44 GiB of page-in
traffic. `src/cli/handlers/score_handler.rs:579-587` —
`Command::new("pmat").args(["comply","check","--format","json"]).current_dir(path).stdout(piped)
.stderr(Stdio::null()).output()`, with the catch-all `_ => (Err(COMPLY_UNMEASURED…))` at `:610`. No
`current_exe()`, no timeout anywhere in the file. `strace -f -e trace=execve` proves the child is
`/home/noah/.cargo/bin/pmat` (`commit: unknown`, three days older) while the parent is the HEAD
build. `grep -n compute_comply src/` → declared `:579`, called once at `:337`. Budget:
`.pmat-metrics.toml:59 max_memory_usage_mb = 512  # ≤512MB for typical projects` (the record cited
`:63`); `.pmat-ratchet.toml`'s `[coherence.binding."performance.max_memory_usage_mb"]` is
`kind = "budget"` with the justification "…`pmat comply check` itself is documented at 4.1 GB per
worker in check.rs (`COMPLY_BYTES_PER_WORKER`) … nothing has ever compared the two"
(`COMPLY_BYTES_PER_WORKER = 4 * 1024 * 1024 * 1024` at `check.rs:390`). Template:
`grep -rl PMAT_REQUIRE_PEAK_RSS tests/ src/` → **0 files**; `tests/modules/binary_size_band.rs` has
exactly the wanted shape (EXPECTED/QUIET/FAIL bands, `PMAT_REQUIRE_BINARY_SIZE=1` making absence a
FAILURE at `:106-119`, a two-direction counter-test at `:168-175`, wired at `feature-matrix.yml:651`).

**Proposal.** `std::env::current_exe()` (or run the compliance step in-process); **capture the child's
stderr and echo it** rather than adding a new print — `check.rs:483-487` already emits
`comply: N group(s), M at a time (~X GB peak; PMAT_COMPLY_JOBS overrides)` to stderr precisely so its
cost is attributable, and `.stderr(Stdio::null())` destroys it, which also fixes the swallowed error;
add a timeout. Then a peak-RSS band test built like `binary_size_band.rs`. **Do not merely re-number
the budget:** raising `max_memory_usage_mb` while nothing reads it reproduces the failure CLAUDE.md
documents for binary size (50,000,000 in the metrics file vs a hardcoded 52,428,800 in a **deleted**
test, "aligned" per comment, 2.43 MB apart, neither reading the other). Require the band test to
**parse** the key out of `.pmat-metrics.toml` rather than duplicating a constant, and require
`pmat comply coherence --format json` to stop reporting it VACUOUS. If it cannot be made FIRING,
**delete the key** — a third un-agreeing number is worse than none.

**Acceptance test.** **(1)** Source pin, portable, no strace and no personal paths — extend the pin
that already exists for this exact class at `src/services/spec_falsification_tests.rs:564-575`: in
`score_handler.rs`'s tests, `assert!(!src.contains(concat!("Command::new(", "\"pmat\")")))` and
`assert!(src.contains("current_exe"))`. **(2) Replace the merged leg**, which is **vacuous today**:
`pmat score 2>&1 | grep -qi comply` already passes because `score_handler_display.rs:173-186` prints
the literal label `Comply:` unconditionally (`research/m/repo_score.out` line 9:
`Comply:      5.0  (5 errors, 15 warnings)`). Assert instead a **deletion-proof** JSON invariant —
`.comply_errors` non-null AND `.sub_scores.comply` non-null AND `"comply"` absent from
`.not_measured` AND `.dimensions_measured` equal to a pinned constant — plus a **disclosure** leg
naming the resolved path and the child's wall time, asserted as a string equality against
`current_exe()` in a new `children: [{exe, secs, peak_rss_kb}]` field, not a grep. Keep the existing
counter-test `an_unmeasured_comply_reports_no_counts_at_all` (`score_handler.rs:1036`).
**(3)** Band **two** numbers separately — score's own `ru_maxrss`/`VmHWM` and each child's — because
`/usr/bin/time` reports max(self, children) and after the fix "score's own peak" and "the
invocation's peak" are different numbers; pinning the wrong one makes the band unfalsifiable, and
removing the child would otherwise read as score getting lighter. Keep `PMAT_REQUIRE_PEAK_RSS=1`
making absence a FAILURE, wire it as `feature-matrix.yml:651` wires its sibling, and keep the
two-direction counter-test (a seeded 3× **and** a seeded 0.3× must both be REJECTED, so widening
`FAIL_PCT` until the band means nothing also fails).

**Effort.** **S**, and honest: `compute_comply` is already `async fn`, so `tokio::time::timeout` needs
no dependency change, and the band test is a clone of an existing file.

**Risk.** `current_exe()` changes what score measures where the binaries differ — that is the fix, but
published score numbers move once. A peak-RSS band is machine-sensitive; set it wide enough for a
48-core CI box and a laptop, and require the env var so absence cannot read as green. **Numbers to
quote correctly:** run 1 is 10,237,248 kB = **9.76 GiB** (not the 9.53 GiB two artefacts carry — a
kB→MiB slip), run 2 is 12.71 GiB, i.e. **20×–26×** the 512 MB budget. Most of the peak is #1014's
known-open comply cost (re-measured post-fix at 6.3–8.5 GB), and the 30 % variance is consistent with
`comply_concurrency()` sizing jobs from *available* RAM at start (`check.rs:449`). The score-owned
defects are the wrong binary, the discarded stderr and the missing timeout.

**Related issue.** #1014 (OPEN, the memory half). #956 and #897 are CLOSED and are the same
`Command::new("pmat")`-vs-`current_exe()` class, already fixed at
`src/services/spec_falsification_engine.rs:122` and `src/cli/verify.rs:304` — `score_handler.rs` is
the surviving third site.

---

### 8.20 CRUX-20 — the analysis walks are single-threaded and read every file 2–7 times

**Problem.** Three measured hot paths where the parallel machinery exists in-tree and is not used:
**(a)** `analyze complexity` carries the comment "PERFORMANCE OPTIMIZATION: Process files in parallel
batches" over a bare `futures::try_join_all`, which runs a chunk on **one** task — the `tokio::fs`
reads overlap, every parse/annotate does not (see §5.2); **(b)** the same path reads each file 2–7 times;
**(c)** the cold index build is 14.86 s at 99 % of one core with pmat's own profiler stamping `ANDON`.

**Evidence.** (a) `src/cli/analysis_utilities/quality_checks_part4.rs:153` (comment) over `:168`
(`try_join_all` on `chunks(20)`), runtime multi-thread by default (`src/bin/pmat.rs:157`). Three runs:
`cpu=113 % / 110 % / 112 %`, wall 1.55/1.88/1.60, `files_analyzed` 4,521. Comparators from the same
sweep on the same box: `analyze duplicates` 1995 %, `tdg` 1768 %. rayon: 33 sites in 19 production
files, none in these walks. A docs-only walk costs 0.03 s, so the 1.72 CPU-s is analysis, not
discovery. (b) `strace … openat`, `-p src/graph`, stderr "Successfully analyzed 42 file(s)": **113
openat on `.rs`, 87 successful, 26 ENOENT, 49 distinct files**; `parallel_louvain.rs` and
`builder.rs` 7× each. Two causes: `quality_checks_part4.rs:464` reads the content then hands a **path**
to `src/cli/language_analyzer/mod.rs:34` → `src/services/ast_rust_compat.rs:22` →
`src/services/accurate_complexity_analyzer_core.rs:7` (`tokio::fs::read_to_string(path)` again); and
`is_included_by_sibling` (`src/cli/language_analyzer/mod.rs:78-107`) probes two sibling candidates per
`_` via `file_contains` (`:105`), a **blocking** `std::fs::read_to_string` inside an async fn.
(c) `Index built: 23563 functions in 12.8s`, `query profile: 14825ms total / load_index: 14791ms
ANDON / query: 15ms`, 99 % CPU; warm 0.15–0.18 s. Re-measured cold at HEAD under heavier load:
17.65 s / `load_index: 17560ms` / 99 %. **Quote it as a band — 14.8–17.6 s at 99 % CPU** — the CPU%
is the stable half and is what the gate should assert.

**Proposal.** (a) Replace `try_join_all` with rayon `par_iter` over a synchronous
`analyze_complexity_file`, or `spawn_blocking` over chunks. The nearest in-tree precedent for per-file task dispatch is
`tokio::spawn` at `src/services/context_impl/build.rs:112` and `:192` — note that file contains no
`spawn_blocking`, so the CPU-bound variant has to be written, not copied; leave the census/denominator logic untouched.
(b) Thread `&content` down into `analyze_rust_file_with_complexity` /
`AccurateComplexityAnalyzer::analyze_file` (add a `_from_str` variant) and replace
`is_included_by_sibling` with a per-directory memo built by reading each candidate parent once.
(c) Parallelise the per-file parse/annotate phase of the index build with the same rayon machinery
`analyze duplicates` uses, keeping the SQLite write serial. **CORRECTION — drop the AST-cache
clause.** "The live AST cache cannot compensate: `AstCacheStrategy::max_size()` is 100" is true of
`pmat context`, not the measured path: there are exactly two **production** callers, both
`src/services/context_impl/visitor.rs:352,411`. (`grep -rn get_or_compute_ast src/` returns 18
lines: those 2, three definitions — `cache/manager.rs:57`, `cache/cache_trait.rs:10`,
`cache/persistent_manager_core.rs:39` — 6 test calls in `cache/persistent_manager_tests.rs` and 3
commented-out lines in `cache/cache_property_tests.rs`. Quote the caller count, not that command.) Raising
`src/services/cache/strategies_ast.rs:105` would not move leg (b) by one open. Move it to its own
`pmat context` item (776 openat for 42 distinct files, 119 % CPU, 5.99 s) with its own RSS band, or
drop it — as written it is a fix where the defect was observed, in a different command.

**Acceptance test.** Each leg pins the work done, so a speedup by measuring less cannot pass.
**(a)** three runs must each report `cpu > 400%` **and** `wall < 0.80 s` (today 1.55–1.88 s; CPU alone
passes a parallelisation that thrashes) **and** the whole provenance object must be unchanged.
**One-shot PR evidence, not a committed gate** (§8 preamble): the literals below are a whole-tree
census that drifts on any commit touching `src/`, so committed as CI they go red on the next
unrelated change and get re-baselined. What may be committed is the **invariant** —
`unrecorded == 0`, `ast + heuristic + heuristic_include_fragment == files_analyzed`, and the object
equal before and after the change *measured in the same run*. Scope qualifier, because §4 quotes a
second and different provenance census: this one is `--path src` at `--top-files 0`; §4's
`{ast: 1802, heuristic_include_fragment: 2231, heuristic: 3}` over 4,036 files is the repo-scope run.
Neither is wrong; they measure different sets.
`jq -S '{p:.analysis_provenance, f:.summary.total_functions, v:(.violations|length)}'` ==
`{"f":58549,"p":{"ast":2110,"files_analyzed":4521,"heuristic":88,"heuristic_fallback":0,
"heuristic_include_fragment":2323,"unrecorded":0},"v":1719}`. The `unrecorded == 0` term doubles as
the regression pin for the ledger interaction in Risk.
**(b)** `grep '\.rs"' t.txt | grep -vc ENOENT` ≤ 50 (today 87) **and** — replacing the merged
`files_analyzed == 42`, which the lazy fix satisfies — pin the whole provenance object on `src/graph`
(again one-shot evidence, not a gate):
`{"ast":15,"files_analyzed":42,"heuristic":0,"heuristic_fallback":0,"heuristic_include_fragment":27,
"unrecorded":0}`. **27 of the 42 files are fragments**, so simply deleting `is_include_fragment`
removes the 26 ENOENT probes and the 7× re-reads, drops opens far below 50, and leaves
`files_analyzed` at 42 — the merged test goes green with the classifier gone.
**(c)** name the fixture (a pinned copy of `src/` with no `.pmat/`), require CPU > 400 % **and**
`load_index:` < 5000 ms, **and** require stderr to still contain verbatim
`Index built: 23563 functions`, `Clones: 1448 functions with duplicates`,
`Faults: 3050 functions with patterns` and `SQLite index saved: 23563 functions, 1104303 call edges`,
plus a warm `pmat query "error handling" --limit 5` whose result set is byte-identical to the pre-fix
run — otherwise "index less" passes.

**Effort.** **L.** **Split:** (a) is ~20 lines at `quality_checks_part4.rs:160-174` and can land
alone; (c) carries the EV (a 12 s cliff on every fresh clone, every CI job and each of this repo's 54
agent worktrees, on the command CLAUDE.md mandates over grep). **If effort is cut, cut (b)'s cache
clause first, then (a); never (c).**

**Risk.** Parallelising a walk that shares a process-global provenance ledger
(`src/cli/language_analyzer/ast_fallback.rs`) needs an audit first — but the interaction is milder
than feared and mis-stated in the merged record: `record` (`:120-131`) writes to a
`std::sync::Mutex<BTreeMap>` and the module doc (`:48-51`) says a task-local run id was **deliberately
rejected** so off-task records still land, so rayon will not silently lose provenance and one
uncontended mutex per file for 4,521 files is noise. The real hazard is narrower: `ARMED` is cleared
only inside `take()` (`:165`) and `RunGuard` (`:141`) has **no `Drop`**, so a panic in a parallel
worker unwinding past `take` leaves the ledger armed process-wide. Raising any cache cap raises peak
RSS — measure it against the band CRUX-19 installs.

**Related issue.** None filed. The ARMED-leak sub-risk touches #1068 (analysis provenance ledger);
the cold-index cost touches the `pmat query` mandate in CLAUDE.md.


### 8.21 CRUX-21 — every crates.io install reports `commit: unknown`, although the SHA ships inside the tarball

**Problem.** The provenance line exists precisely to stop "measured against the wrong binary" — the
class CRUX-19 is about, and the class `build.rs:1636-1647` cites v3.28.2 for. It is armed only for
someone building from a git checkout, i.e. **never** for a user who ran `cargo install pmat`.

**Evidence.** Measured, two binaries side by side:

```
$ /mnt/nvme-raid0/.../release/pmat --version   → pmat 3.34.0 / commit: 01fba4f6554742ae690fa00131444ddf722a5334 / worktree: clean
$ ~/.cargo/bin/pmat --version                  → pmat 3.34.0 / commit: unknown / worktree: unknown
```

`emit_build_provenance` (`build.rs:1648-1679`) shells `git rev-parse HEAD` and
`git status --porcelain`, and on failure emits the literal string `unknown`. In an extracted `.crate`
there is no `.git`, so both always fail. **The answer is a sibling of `build.rs` in the same
tarball:** `~/.cargo/registry/src/index.crates.io-*/pmat-3.34.0/.cargo_vcs_info.json` carries
`{"git":{"sha1":"71f7d37d1b20233e98cf58b24d52ea636f55a4da"}}`, and
`ls …/pmat-3.3*/.cargo_vcs_info.json | wc -l` → **6**, i.e. present for every 3.3x release. So
3.34.0's SHA is already printable and is being thrown away.

**The structural half is worse than `unknown`, and is UNVERIFIED because it needs a build.** The
`git rev-parse` runs with the build script's cwd (the package root). For a vendored or
git-dependency build the package root sits inside the **consumer's** repository, so the command
*succeeds* and the binary reports the consumer's HEAD and the consumer's dirty flag as pmat's
provenance — a confidently wrong answer. Falsifier looked for: an alternate provenance source
pinning the SHA another way. There is none (`grep -n PMAT_GIT_SHA src/ build.rs` → 3 hits, all in
this path), and `pmat diagnose --format json`'s `build_info.git_commit` is `null`.

**This is load-bearing for the rest of the document.** §1's whole method is knowing which binary
produced a number; §5.4 and §8.19 use "`~/.cargo/bin/pmat` reports `commit: unknown` and is three
days older" as evidence for CRUX-19. An audit that leans on the symptom must dispose of the defect.

**Proposal.** `emit_build_provenance` falls back to `$CARGO_MANIFEST_DIR/.cargo_vcs_info.json`'s
`.git.sha1` before emitting `unknown`, and **refuses the ambient `git rev-parse` when
`CARGO_MANIFEST_DIR` is not itself the repo root** (that clause is the fix for the vendored-build
wrong-SHA case, and is the half that makes this more than cosmetic). Wire the same value into
`diagnose`'s `build_info.git_commit`, `rust_version` and `build_date`, which are null today. ~10
lines.

**Acceptance test** — run against a binary built from the **extracted tarball**, not the checkout.
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
cd "$(mktemp -d)" && tar xzf ~/.cargo/registry/cache/*/pmat-<v>.crate && cd "pmat-<v>"
cargo build --release
./target/release/pmat --version | grep -qE '^commit: [0-9a-f]{40}$' || fail "still unknown"
./target/release/pmat diagnose --format json \
  | jq -e '.build_info.git_commit|test("^[0-9a-f]{40}$")' || fail "diagnose still null"
# anti-vacuity: the printed sha must EQUAL the tarball's own record, so a hardcoded
# 40-hex constant, or the ambient repo's HEAD, cannot pass
test "$(./target/release/pmat --version | sed -n 's/^commit: //p')" \
   = "$(jq -r .git.sha1 .cargo_vcs_info.json)" || fail "sha does not match .cargo_vcs_info.json"
```
Today: `commit: unknown` and `git_commit: null`. The sha-equality leg is the control that stops the
lazy fix (printing *something* 40 characters long). A second control for the structural half:
building the crate as a path dependency of a scratch consumer repo with its own dirty `.git` must
still print the tarball's sha, never the consumer's HEAD.

**Effort.** **S.** The build requires `cargo build`, so this item's acceptance run is **UNVERIFIED**
by this audit; the defect itself is measured above with no build at all.

**Risk.** None material. A checkout build keeps `git rev-parse`; only the fallback and the
manifest-root guard are new.

**Related issue.** None filed.

---

### 8.22 CRUX-22 — `analyze duplicates` defaults to the 10× mode for 6.9 % more findings

**Problem.** `--detection-type` defaults to `all`, which costs **16.67 s wall / ~218 CPU-s /
1.69 GiB** against `exact`'s **1.61 s / 596 MiB** — a 10× wall and ~3× RSS penalty for **+6.9 %**
findings — on the single most expensive command in the tool and the only breach of the 512 MB budget
outside `score`. `--min-lines`, the flag a user would reach for, **cannot bound it**.

**Evidence.** Same tree, same binary, back to back (`/usr/bin/time`, `research/dup_probe.txt`; the
`--detection-type` flag is on the HEAD binary's `--help`: "Detection type: exact, renamed, gapped,
semantic, or all"):

| invocation | wall | %CPU | max RSS | `blocks_analyzed` |
|---|---|---|---|---|
| `--detection-type exact` | **1.61 s** | 102 % | **596 MiB** | 35,342 |
| default (`all`) | **16.67 s** | 1988 % | **1.69 GiB** | 37,785 |
| `--min-lines 20` (still `all`) | 17.28 s | 1948 % | 1.68 GiB | 9,085 |

Raising `--min-lines` from 5 to 20 cuts `blocks_analyzed` **4.2×** and changes wall and RSS **not at
all**, because the near-miss MinHash pass builds its fragments before `min_lines` is consulted — the
code says so at `src/cli/analysis/duplicates_detection.rs:637-644` ("A near-miss fragment is a whole
function"). Mechanism of the 1.69 GiB, located: `:413-433` reads every file **sequentially**
(consistent with the 102 % of the `exact` run) and `:424` retains the full text of every source file
for the whole run; `find_structural_similarities` (`:510-560`) then does `content.clone()` per file
at `:522-527` — **a second complete copy of the corpus live at the same time**; and
`src/cli/analysis/duplicates_extraction.rs:117-141` slides `windows(min_lines)` one line at a time
allocating three `String`s per window (a joined content string, a `format!("{:x}")` hash, and a full
copy of the path), all retained, with three extractors running over the same lines at
`--detection-type all` (`:44-47`), then moved into a `HashMap<String, …>` keyed by the hex string at
`:649-660`.

**Proposal.** **(a) S — flip the default to `exact`**, and say in `--help` what `all` costs. A user
who wants near-miss detection asks for it; a user running the pre-commit path does not.
**(b) M — intern the block tuple** (`duplicates_extraction.rs:117-141` and `:649-660`): replace
`file_str.to_string()` with an `Arc<str>` or a `u32` file id and `format!("{:x}", …)` with the raw
`u64`, removing two of the three allocations per window. **(c) L — bound the near-miss engine** so
`--min-lines` is consulted before fragments are built. (a) and (b) are independent; (c) is the real
fix and must not gate them.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
# (a)+(c): the default invocation fits the budget, WITHOUT measuring less
/usr/bin/time -f '%M' -o rss.txt pmat analyze duplicates --min-lines 20 --format json > d.json
[ "$(cat rss.txt)" -lt 1000000 ] || fail "still $(cat rss.txt) kB"      # today 1,758,024
jq -e '.metrics.blocks_analyzed == 9085' d.json || fail "work changed"  # the pin
# (b): interning, measured where it is cheapest to see
/usr/bin/time -f '%M' -o e.txt pmat analyze duplicates --detection-type exact --format json > x.json
[ "$(cat e.txt)" -lt 400000 ] || fail "exact still $(cat e.txt) kB"     # today 610,288
jq -e '.metrics.blocks_analyzed == 35342' x.json || fail "exact work changed"
```
*Anti-vacuity.* Both legs pair an RSS ceiling with an **unchanged `blocks_analyzed`**, so the win
cannot come from analysing fewer blocks — which is exactly what a naive "raise the default
`--min-lines`" would do. For (a), add the second direction: `--detection-type all` must still be
*reachable* and must still find the 37,785 blocks, so flipping the default cannot be implemented by
deleting the near-miss engine.

**Effort.** **S** (a) / **M** (b) / **L** (c).

**Risk.** (a) changes what the default reports — 2,443 fewer blocks — so it is a release-note change
and must ship with the ratchet rebase for any duplication baseline. It does **not** interact with
CRUX-02: the `quality-gate` duplicates finding is a whole-file hash and never called this engine.

**Related issue.** None filed. CRUX-20 is the adjacent perf item and does not touch this path.

---

### 8.23 CRUX-23 — two root `pub mod`s nothing calls, invisible to dead-code, reachability and `unreachable_pub` alike

**Problem.** `src/protocol/` (13 files, **2,047 lines**) has zero references outside its own subtree;
`src/state/` (**3,896 lines**) is referenced only from `tests/integration_test.rs.disabled`. Both are
`pub mod` at `src/lib.rs:168` and `:185`, in the **default** build, and both compile into every
shipped binary. ~5,900 lines of dead code that **no pmat check can see**.

**Evidence.** `git grep -lE 'crate::protocol\b|pmat::protocol\b' -- src tests | grep -v
'^src/protocol/'` → **0**; the same for `state` returns only the `.disabled` file.
`analyze dead-code` on this repo reports **5 dead functions / 0 dead modules** across 3,228 files
(0.003 %) — it works at function granularity inside compiled modules. `analyze reachability`
correctly calls both modules reachable, because the build does compile them. `unreachable_pub` is
structurally near-vacuous here for the reason §4 A4 gives: every internal module is `pub mod` at the
root, so almost every `pub` item is reachable by construction. Three checks, one of which is the
product's headline differentiator, and the class falls between all of them.

**Proposal.** Add the check that can see it — a `--lib` test first, a `pmat` analyzer second:
**every root `pub mod` must be referenced from at least one non-test file outside its own subtree,
OR be behind a non-default feature, OR carry a written exemption** on the `analyze_mcp_exposure`
ledger pattern (`src/cli/analyze_mcp_exposure.rs:109` — a total match with no catch-all, so a new
row is a compile error, with a `BACKLOG_CEILING` ratchet). Then delete `src/protocol/`.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
n=$(git grep -lE 'crate::protocol\b|pmat::protocol\b' -- src tests | grep -cv '^src/protocol/' || true)
[ "$n" -gt 0 ] || [ ! -d src/protocol ] || fail "src/protocol still present and unreferenced"
```
*Non-vacuity proof, and it is the point of the item:* **run the new gate on HEAD BEFORE the
deletion.** It must name **both** `protocol` and `state` and exit non-zero. A gate that ships green
on the tree that motivated it has not been shown to fire — that is the failure §12.2 rule 1 names,
and this item is the cheapest place in the document to demonstrate compliance with it. Second
control: adding an exemption row for `state` must make the gate green *for `state` only*, and
deleting the exemption must make it red again.

**Effort.** **S** — one `--lib` test plus one `git rm -r`.

**Risk.** `src/state/` has an event-sourcing subsystem someone may intend to revive; the exemption
row exists for exactly that, and forces the intent to be written down with an issue number rather
than inferred from the file's continued existence.

**Related issue.** #1017 (unreachable code) — but this is a **disjoint class** from the 407 orphans
that issue tracks: those are files no compilation unit reaches, these are files every build compiles
and nothing calls. File it as a second acceptance criterion there, or as its own issue.

---

### 8.24 CRUX-24 — the complexity gate blocks two below the limit it announces

**Problem.** `pmat.toml` sets `max_complexity = 30`; the gate's banner prints
`cyclomatic 30, cognitive 100 (from ./pmat.toml)`; and it then blocks at **28**, because the warning
band is derived as `limit - 2` and warnings are counted in `blocking_violations`. **Five of pmat's
six complexity blockers comply with the repo's own configured limit.** A user who raises the limit to
permit a 30-complexity function still fails.

**Evidence.** `src/services/complexity/aggregation.rs:122-129`:
`thresholds.cyclomatic_warn = max_cyc.saturating_sub(2).max(1);` and the same shape for cognitive.
Nothing in the banner, the help or `pmat.toml`'s schema mentions the derived band. Consequence,
measured: `quality-gate --format json` → `blocking_violations 35`, of which the complexity component
is 6 and five of those six are at or below 30.

**Why this is filed separately, and why it changes how "35" reads.** §2 item 1, §5.1's exit-1 row and
§11's breakdown all quote 35. That number is the size of the verify-vs-quality-gate divergence, not a
count of genuine violations, and the largest single reason is this item — an **over-reporting**
defect pointing the opposite way from everything else in §8. It was observed during CRUX-01's
verification, recorded in that item's Framing paragraph as a bare parenthesis, and never disposed of.
Disposing of it is the point of this entry.

**Proposal.** (a) Print the band in the banner: `cyclomatic 30 (warn at 28), cognitive 100 (warn at
98)`. (b) Either make warnings non-blocking, or give `[quality]` an explicit `warn_complexity` so the
band is a choice rather than a surprise. Do not simply delete the band — a narrow warning band is a
defensible design; an *undisclosed* one is not.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
cd <repo>   # pmat.toml: max_complexity = 30
pmat quality-gate 2>&1 | grep -q 'cyclomatic 30 (warn at 28)' || fail "band still undisclosed"
pmat quality-gate --format json | python3 -c "
import sys,json; d=json.load(sys.stdin)['results']
cx=[v for v in d['violations'] if v['check_type']=='complexity']
assert all('threshold: 30' in v['message'] or v['severity']!='error' for v in cx), 'error at <30'
assert d['blocking_violations'] < d['total_violations'] - 2, 'warnings still block'
" || fail "warnings still counted as blocking"
```
*Anti-vacuity.* The two legs must both hold: (a) alone is satisfiable by editing a string while the
behaviour stands; (b) alone is satisfiable by widening the band to zero, which weakens the gate and
is forbidden by §12.2 rule 4. Add the counter-test that a function at cyclomatic **31** still
produces a `severity=error` blocking violation, so "make warnings non-blocking" cannot be
implemented as "make everything non-blocking".

**Effort.** **S.**

**Risk.** Trees whose only complexity findings were warnings go green in one step; that is the
correction, and it needs a release note plus a ratchet rebase for any complexity baseline.

**Related issue.** #1035 (its class, inverted: a gate reporting more than it measured rather than
less).

---

### 8.25 CRUX-25 — a comply check that reports the population it did not inspect

**Problem.** `check_ladder_parses` iterates the active contracts, `continue`s past every one whose
`contract.json` is unreadable, unparseable, or missing `verification_level`, and then returns
`CheckStatus::Pass` with `format!("All {} ticket level(s) parse to L0..L5", contracts.len())` — the
**population**, not the inspected set. With N contracts and zero readable files it reports "All N
parse" having read none.

**Evidence.** `src/cli/handlers/comply_handlers/check_handlers/check_work_ladder_declaration.rs:13-63`
— three successive `let … else { continue; }` arms at `:27`, `:30` and `:33`, then the `Pass` at
`:57-62` whose message interpolates `contracts.len()`. This is a **static source finding and needed
no run**, which is why it is in §8 despite §3's standing exclusion of comply results (`pmat comply
check` saturates this machine and was never executed).

**Context, measured statically, that makes it worth a ticket rather than a one-line fix.** 58 of 165
`ComplianceCheck`-producing functions read gitignored state directly, 49 of them `.pmat-work/` — a
945 MB directory that exists here and on no fresh clone. Those checks correctly return `Skip`, and
`Skip` is counted; but nothing distinguishes "not applicable" from "the evidence lives in a directory
git does not carry", so **two machines' pass counts are not comparable**. The good half is real and
is reported here too: the checks Skip rather than Pass, which is the right behaviour and is why this
item is narrow.

**Proposal.** Two statically-verifiable halves. **(a)** Any check that reports a count must report
the count it **inspected**, never the population — fix `check_ladder_parses` and sweep the check
tree for the same shape (`format!` interpolating a collection length that a `continue` can bypass).
**(b)** Give `CheckStatus::Skip` a typed reason — `NotApplicable | RequiresLocalState { path } |
Disabled` — and emit `state_dependent_skips` in the JSON, with a footer naming the count.

**Acceptance test.** The source-level assertion is the gate, because the runtime leg cannot be run
here:
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
# (a): the message must carry the inspected count, and a `continue` must decrement it
grep -q 'contracts.len()' src/cli/handlers/comply_handlers/check_handlers/check_work_ladder_declaration.rs \
  && fail "still reports the population"
# plus a --lib test: N contracts, N-1 unreadable  =>  message says "1 of N", status is not Pass
```
**UNVERIFIED leg, marked as such:** `git clone --depth 1 file://$PWD /tmp/fresh && cd /tmp/fresh &&
pmat comply check --format json | jq -e '.state_dependent_skips > 0'`. It needs a comply run, which
§3 forbids, so it is **not** this item's gate. Its two-directional control is recorded for whoever
runs it: the field must be `> 0` on a fresh clone **and substantially smaller on this machine**,
where `.pmat-work/` exists — a hardcoded constant fails one of the two.

**Effort.** **M** — (a) is S; (b) touches the `CheckStatus` type and 165 call sites' construction.

**Risk.** Typing `Skip` is a JSON schema change for every comply consumer; ship both halves together
and changelog the field.

**Related issue.** #1034 (comply feature backlog), #1035 (its exact class — "not measured" rendered
as clean — inside the one command family §8 otherwise never examines).

---

### 8.26 CRUX-26 — `pmat query` inverts the exit-code contract of the tools it is documented as replacing

**Problem.** CLAUDE.md's mandatory search policy maps `pmat query` onto `rg`/`grep` flag for flag
(`--literal` = `rg -F`, `--regex` = `rg -e`, `--files-with-matches` = `rg -l`, `--count` = `rg -c`,
`--exclude` = `grep -v`). Both incumbents exit **1** on no match. `pmat query` exits **0**. And
`--limit` is silently ignored in two of its four modes.

**Evidence.** Measured against the HEAD binary in this repo:

```
$ pmat query --literal "zzqqxx_never_appears_anywhere_9182" --limit 3   → rc 0, 0 B stdout
$ rg -q / grep -rq  (same pattern)                                      → rc 1
$ pmat query --literal "unwrap()" --files-with-matches --limit 1|3|50   → 1611 lines each
$ pmat query --literal "unwrap()" --count --limit 3                     → 1611 lines
--- control: the flag works in the default mode ---
$ pmat query --literal "unwrap()" --limit 1 | 3 | 10                    → 53 / 114 / 419 lines
```

`--help` says `-l, --limit <LIMIT>  Maximum number of results  [default: 10]` with no caveat, so the
**default** `--limit 10` also silently does nothing in those two modes: `pmat query --literal x
--count` dumps 1,611 lines where the documented default is 10.

**Cost, concretely.** Every `rg`-shaped idiom inverts. `pmat query --literal "$sym" >/dev/null ||
echo "not found"` never prints, and a CI guard written `pmat query --regex "$forbidden" && exit 1`
fires on **every** run whether or not the pattern exists. This is a peer-semantics defect: the
behaviour is defined by the incumbent's users, not by us — CRUX scorecard C5 (§6f).

**Proposal.** Exit **1** on no match in all four modes, keep 0 on a hit, keep 2 for usage errors.
Honour `--limit` in `--files-with-matches` and `--count`. Document both in `--help`.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
pmat query --literal "zzqqxx_never_appears_9182" >/dev/null 2>&1 && fail "no-match still exits 0"
pmat query --literal "unwrap()" >/dev/null 2>&1 || fail "a hit must still exit 0"
[ "$(pmat query --literal 'unwrap()' --files-with-matches --limit 3 | wc -l)" -le 3 ] || fail "-l ignored"
[ "$(pmat query --literal 'unwrap()' --count --limit 3 | wc -l)" -le 3 ] || fail "-c ignored"
```
*Anti-vacuity.* The second leg is the control that blocks "always exit 1"; the `-le 3` legs are
paired with a floor — `--limit 50` must return **more** than `--limit 3` — so "always return 3" and
"return nothing" both fail. Today: 0, 0, 1611, 1611.

**Effort.** **S.**

**Risk.** Exit-code change breaks any script keying on 0 — the correct break, and the whole point.
It must land with CRUX-17's exit-code table and changelog entry, not before it.

**Related issue.** #999 (agent integration). No issue names it.

---

### 8.27 CRUX-27 — the two remaining CRUX scorecard failures: help on stdout, and mutating verbs with no preview

**Problem.** Two independent C-check failures (§6f) that no other item reaches. **(a) C2:** `pmat`
with no arguments and stdin not a tty exits 2 with **8,998 bytes of help on STDOUT** and 123 on
stderr — a machine consumer piping stdout gets a help page where it expected data or nothing.
CRUX-17 covers only the *other* half of C2 (`pmat -q tdg <missing>` printing `0.0` on stdout at
exit 5) and its proposal does not reach this one, because the no-args path is inside `Cli::parse()`.
**(b) C7:** only **7 of the 17 mutating verbs probed** advertise a preview flag, and the ten that do not include everything that writes into `.git/`.

**Evidence.** (a) measured above. (b) the census, re-run verb by verb against the HEAD binary:
`refactor auto`, `refactor docs`, `work migrate`, `enforce extreme`, `maintain cleanup-resources`,
`kaizen` and `split` have `--dry-run`; `hooks install`, `hooks uninstall`, `hooks refresh`,
`agy sync`, `scaffold project`, `config set`, `work delete`, `refactor interactive`, `embed sync` and
`init` have none. `pmat hooks install` writes into `.git/hooks` with no way to see what it would
write first. The check is **armed** — the positive control fires, seven commands do advertise one.
**Two corrections to the research census, both found by re-running it:** `pmat cache clear` does not
exist at all (`pmat cache` has only `stats` and `help`), and `embed`'s verbs are `sync`/`status`/
`clear`, with `embed clear` gated behind `--confirm` — a guard, which should be argued as satisfying
C7's intent or not, never silently counted in either column. **Method note for whoever re-runs
this:** in zsh an unquoted `$c` holding `"hooks install"` is **not** word-split, so a naive
`for c in "hooks install" …; do pmat $c --help; done` reports every two-word verb as
`unrecognized subcommand`. Split explicitly, or the census measures the shell.

**Proposal.** (a) Send the no-args help to **stderr** and keep exit 2 (clap's
`Command::print_long_help` to stderr on `ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand`), so
`pmat > out` produces an empty `out`. Explicit `pmat --help` keeps writing to stdout — that is a
request, not a failure. (b) Add `--dry-run` to the mutating verbs that lack one, printing the exact
paths and bytes it would write, **or** demonstrate the `pmat init` property instead: idempotent and
non-destructive with a measured unchanged checksum on re-run (§10, #1031). Either satisfies C7's
intent; `pmat init` already earns it without the flag.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
out=$(pmat </dev/null 2>/dev/null | wc -c); [ "$out" -eq 0 ] || fail "$out B of help on stdout"
pmat </dev/null 2>&1 >/dev/null | grep -q 'Usage' || fail "help vanished instead of moving"
pmat --help | wc -c | grep -qv '^0$'  || fail "explicit --help must still use stdout"
while read -r a b; do            # explicit split: zsh does NOT word-split an unquoted "$c"
  pmat $a $b --help 2>&1 | grep -q -- '--dry-run' || echo "$a $b"
done > nopreview.txt <<'EOF'
hooks install
hooks uninstall
agy sync
scaffold project
work delete
EOF
[ ! -s nopreview.txt ] || fail "no preview for: $(tr '\n' ' ' < nopreview.txt)"
```
*Anti-vacuity.* Leg 2 blocks the lazy fix for (a) — deleting the help output rather than moving it.
Leg 3 blocks moving *all* help to stderr, which would break `pmat --help | less`. For (b), a
`--dry-run` that accepts the flag and does nothing must fail: assert that on a fixture,
`hooks install --dry-run` names `.git/hooks/pre-commit` on stdout **and** leaves
`test ! -e .git/hooks/pre-commit` true.

**Effort.** **S** for (a); **M** for (b) across five verbs.

**Risk.** (a) changes what a script capturing stdout sees on misuse — from a help page to nothing,
which is the point; changelog it beside CRUX-17's exit-code table.

**Related issue.** None filed.

---

### 8.28 CRUX-28 — two written, ratcheted MCP gates that nothing can run, one not even declared

**Problem.** CRUX-08 records in passing that
`grep -rn 'transport-parity|mcp-surface|dogfood' .github/workflows/` returns one *comment* line, and
that `scripts/dogfood/pmat-gate-{transport-parity,mcp-surface}.sh` "prove this repo accumulates
gates" — and then proposes nothing about them. This is CRUX-14's category exactly (a check exists
and cannot fail a build), and CRUX-14's eight legs do not include it.

**Evidence.** Both scripts exist (`ls scripts/dogfood/pmat-gate-*.sh`).
`pmat-gate-transport-parity.sh` **is** declared in `Cargo.toml`'s
`[package.metadata.dogfood] gates` (`:435`, beside `pmat-gate-fleet.sh`);
**`pmat-gate-mcp-surface.sh` is not declared at all.** Neither is invoked by any workflow or by any
make target — the Makefile's only reference is `dogfood-install` (`:152-159`), which *symlinks*
`scripts/dogfood/pmat-*` into `$HOME/.claude/skills/dogfood/` and runs nothing. And the harness
would not have found CRUX-08 even if it ran: `scripts/dogfood/pmat-transport-parity.sh:85-95`
classifies a JSON-RPC error response as a **harness fault** (`MCP_ERROR*|*"-32602"*`), with a comment
explaining that a validation error "is almost always the HARNESS calling the tool wrongly" — which is
precisely the signal CRUX-08's six divergences live in.

**Proposal.** (1) Declare `pmat-gate-mcp-surface.sh` in `[package.metadata.dogfood] gates`
(`Cargo.toml:432-436`). (2) Add a job to `.github/workflows/` that runs the **declared** gates —
whatever `scripts/dogfood/pmat-dogfood_gates.py` parses out of that table, so adding a gate to the
table is sufficient to run it — and put that job in the `needs:` chain of a required check
(`feature-gate` is the established place). (3) Fix the harness's error-as-fault handling so a
transport's error **code** is data, not an aborted comparison; that is the prerequisite for CRUX-08's
committed probe fixture.

**Acceptance test.** Two clauses, and the second is the one that matters:
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
grep -q 'pmat-gate-mcp-surface.sh' Cargo.toml || fail "still undeclared"
grep -rq 'pmat-dogfood_gates' .github/workflows/ || fail "no workflow runs the declared gates"
```
*Anti-vacuity, carried from the research record verbatim:* **a deliberate one-line deletion of a
`.tool(...)` registration in `simple_unified_server.rs` must turn a required check RED**, and the
run's log must contain the gate's own receipt line
(`mcp-surface: stdio=19 http=19 tools (ratchet 19), 0 failed invocation(s)`). Without the receipt
clause, a job that runs the script and ignores its exit status passes; without the deletion clause,
a job that runs it on a tree where it cannot fail passes. Today no workflow mentions either script,
so neither can fail anything.

**Effort.** **M** — (1) is one line, (2) is a job, (3) is the harness change and is where the week
goes.

**Risk.** Arming a ratcheted gate that has never run in CI will fail on its first master run for
reasons nobody has seen; land it on a branch, read the receipt, then wire the `needs:`.

**Related issue.** #1090, #1029. Fold into CRUX-08's PR if the transport work lands first — but it
must not be *absorbed* into it, because the wiring is what makes CRUX-08's fix enforceable.

---

### 8.29 CRUX-29 — recover the package headroom: 11.79 MB of test source ships, against a 9.0 MiB hard limit

**Problem.** 1,195 files / 413,004 lines / **11.79 MB of 38.06 MB** of `src/` is test-named source,
and all of it goes into the published `.crate` (`Cargo.toml`'s `exclude` drops `/docs/`, `/scripts/`
and `/rust-docs/` but **nothing under `src/`**). The `package-size` CI job hard-fails at
`mib >= 9.0` (`feature-matrix.yml:506-507`, "at or past the 9.0 MiB budget"), and `Cargo.toml:22-25`
records that the package once measured 9.4 MiB against crates.io's 10 MiB ceiling, "so one more asset
would have made the crate unpublishable". This is the audit's only near-term **release blocker**, and
§4 A3 states it while no item owns it.

**Evidence.** As above, plus the reason it cannot be fixed today: **596 of those files are pulled by
unguarded `include!` and are hard build inputs** — excluding them from the tarball would ship a crate
that does not compile. That is what makes this item *sequenced after* CRUX-11(b), not independent of
it.

**Proposal.** After CRUX-11(b) converts the unguarded test `include!`s to
`#[cfg(test)] #[path="…"] mod`, add `"**/*_tests.rs"` to `Cargo.toml`'s `exclude` and let the
existing `package-size` job — which builds the tarball, **not** `--no-verify` — prove it still
compiles.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
n=$(cargo package --locked --allow-dirty --list | grep -c '_tests\.rs' || true)
[ "$n" -eq 0 ] || fail "$n test files still in the tarball"    # today >1000
cargo package --locked --allow-dirty || fail "the packaged crate no longer builds"
```
*Anti-vacuity — the second leg is the whole control.* Excluding files is trivially satisfiable and
trivially wrong; the job's own comment records a 7.4 MiB tarball that did not compile. `--no-verify`
is forbidden here for that reason. Record the packed MiB before and after in the PR, and add the
9.0 MiB threshold to the ledger so a future reader does not re-derive "94 % of 10 MiB" from a comment
(§4 A3).

**Effort.** **M**, sequenced after CRUX-11(b).

**Risk.** Anyone consuming the published crate's test sources (nobody: zero reverse dependencies,
measured) would lose them. The real risk is landing it *before* CRUX-11(b) and shipping an
uncompilable crate — hence the ordering and the build leg.

**Related issue.** None filed. Blocks nothing; blocked by CRUX-11(b).

---

### 8.30 CRUX-30 — CLAUDE.md documents a dead-path checker, and nothing runs it

**Problem.** §9.2 item 10 disposes of two stale citations of the deleted `src/tests/binary_size.rs`
by "folding the doc repair into the next ratchet PR" — i.e. it fixes the two observed instances and
leaves unrun the checker that should have caught them. That is the symptom-not-cause pattern §12.2
rule 3 forbids, inside this document.

**Evidence.** CLAUDE.md itself publishes the check (a `grep -oE` over backtick-quoted repo paths,
piped through `test -e`) and says to run it after editing the file. Measured: it **FAILS at HEAD**
with value 1 (`src/tests/binary_size.rs`, deleted at `acf72dd77`) — and that same dead file is the
cited evidence for `.pmat-ratchet.toml:334-343`'s CB-2101 classification of
`thresholds.binary_max_bytes`, i.e. a live coherence binding justified by a file that no longer
exists. It is run by **no** workflow. And it `sed`s the `:NNN` off every citation, so it is
structurally blind to a second class this document is full of: all three
`scripts/install-git-hooks.sh:NNN` anchors in CLAUDE.md point at the wrong lines, because the
PMAT-630 clippy block was prepended above them.

**Proposal.** Extend the checker to (a) fail on any dead path — already possible, just not run
anywhere — and (b) for a `path:NNN` citation, require `sed -n 'NNNp' path` to be non-empty and, where
the prose names an identifier, that the identifier appears within ±5 lines. Run it in
`quality-gate.yml` beside `pmat score` (it costs milliseconds). Extend the same scan to
`.pmat-ratchet.toml`'s `justification` strings, which is where the load-bearing citation rotted.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
paths=$(grep -oE '`[A-Za-z0-9_./()-]+\.(md|rs|sh|toml)(:[0-9]+)?`' CLAUDE.md \
        | tr -d '`' | sed 's/:[0-9]*$//' | sort -u)
[ "$(printf '%s\n' "$paths" | wc -l)" -ge 30 ] || fail "extracted only $(printf '%s\n' "$paths" | wc -l) paths"
dead=$(printf '%s\n' "$paths" | while read -r p; do [ -e "$p" ] || echo "$p"; done)
[ -z "$dead" ] || fail "dead: $dead"          # today: 1
sed -n '238p' scripts/install-git-hooks.sh | grep -q 'pmat-book' \
  || fail "line anchor wrong (correct it to :352, or fix the doc)"    # exits 1 today
```
*Anti-vacuity.* The `>= 30` floor is the control: a broken regex extracts nothing, `dead` is empty,
and a silent pass certifies the document — which is exactly how this class survives. The anchor leg
is the second control: without it, a checker that strips `:NNN` reports green on three wrong line
numbers.

**Effort.** **S.**

**Risk.** None; it is a text check. Expect it red on its first run and fix the citations in the same
PR — including this document's own, which §12.1 already requires.

**Related issue.** #1018 (tests that cannot fail — a checker nothing runs is the same class).
**This item supersedes §9.2 item 10**, which now reads as its trigger rather than its disposition.

---

### 8.31 CRUX-31 — nothing asserts which TLS provider is linked, and #1113 is about to change it

**Problem.** The whole of the #1053 analysis ("`ring` is not linked") is a comment in `Cargo.toml`.
Open PR **#1113** bumps `pmcp` — the one crate that decides the answer — and a wrong outcome is not a
compile error but a **runtime `panic!("No provider set")`** the first time a TLS connection is made.

**Evidence.** `grep -rn 'aws_lc\|aws-lc' src/ tests/ | wc -l` → **0**: no test, anywhere, observes
which provider resolves. The pattern to copy already exists and is good —
`src/dependency_policy_tests.rs` reads `include_str!("../Cargo.lock")` under `--lib` registration and
carries its own anti-vacuity control (`dependency_policy_absence_scan_can_actually_find_crates`).

**Proposal.** A test beside `src/dependency_policy_tests.rs`, same registration and same technique,
asserting that `aws-lc-rs` resolves exactly once and that `ring`'s presence is unchanged — plus an
**over-correction guard** that fails if the scan finds *neither*, so a lockfile-format change cannot
read as "provider absent, all fine".

**Acceptance test.** `cargo test --lib dependency_policy` must contain a test that fails when the
lockfile's provider set changes, and `grep -rn 'aws_lc\|aws-lc' src/ tests/ | wc -l` must be > 0
(0 today). The absence-scan control that ships with the existing file is the non-vacuity proof;
reuse it rather than writing a new one.

**Effort.** **S**, with a **dated trigger**: it is worth landing before #1113 merges, because after
the merge the test is a regression test for a change nobody observed.

**Risk.** None. It asserts the status quo and fails when the status quo moves, which is the whole
value.

**Related issue.** #1053 (the claim), **#1113** (the trigger).

---

### 8.32 CRUX-32 — `pmat context` cannot parse 28 files, and the capability to fix 2,231 of that class already ships

**Problem.** §4 A2 records that `pmat context` — the artefact agents consume, and the one CRUX-18
proposes to budget — cannot parse 28 files because a fragment is deliberately brace-unbalanced, and
§2 item 9 leans on the same `include!`-vs-analyzer split. No item repairs the parse path. CRUX-11
fixes TDG *attribution*; CRUX-18 budgets the context output while never noting that the artefact
being budgeted is silently short by 28 real source files, 8 of them `include!` fragments and 4 of
them the complexity analyser's own tests.

**Evidence.** `analyze complexity` already covers **2,231** such files via
`heuristic_include_fragment`, so the capability exists in the binary and is not shared with the
unified Rust analyzer. `pmat context` emitted 11 parse failures into its 8,670,750-byte output on the
measured run (9 × "Unified Rust analysis failed: cannot parse string into token stream"). Three of
the 28 are not fragments at all: `src/demo/templates/html_{body,scripts,styles}.rs` are HTML, JS and
CSS wearing a `.rs` extension.

**Proposal.** **Cheap half (M):** rename those three to `.html/.js/.css` and pull them with
`include_str!` — that fixes 3 of 28 for free and removes a category error. **Full half (L):** before
declaring a parse failure, look for an `include!("<basename>")` site and parse the *includer* (which
is balanced), attributing spans back to the fragment; or, minimally, reuse the existing fragment
heuristic instead of dropping the file.

**Acceptance test.**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
pmat analyze vacuous-tests --format json \
  | jq -e '(.skipped.unparseable|length) == 0' || fail "still skipping unparseable files"
pmat context --output /tmp/c.md 2>&1 | grep -c 'failed to parse .*\.rs' | grep -q '^0$' \
  || fail "context still drops files"
```
*Counter-test, mandatory:* a genuinely malformed `.rs` — `fn f( {` — must **still** be reported
unparseable. Without it the cheapest fix is to stop reporting parse failures, which converts a
disclosed gap into an undisclosed one, i.e. manufactures a fresh #1035.
**Add a leg to CRUX-18's L4 reconciliation** asserting the budgeted context's file set is not
silently short by 28: `len(kept) + declared_dropped == len(full)` is only honest if `len(full)` is
the real file count.

**Effort.** **M** for the rename alone; **L** for reassembly.

**Risk.** Reassembly re-parses the includer per fragment — the second-parse cost
`src/services/context_impl/build.rs:391-393` warns about, on a path already dominated by parsing.
Measure it against CRUX-20's numbers before committing to the L half.

**Related issue.** #1035 (28 files absent with no disclosure in the artefact). Bundle the rename with
CRUX-18; do not let the L half gate it.

---

---

## 9. Refuted and out of scope

### 9.1 Killed in verification: none

**All 20 merged candidates survived all three lenses**, and 12 more (CRUX-21..32) were added
afterwards from findings the merge had dropped without a written reason — which is its own result:
the merge's failure mode was silent omission, not over-inclusion. No candidate was refuted — but that is a weaker
statement than it sounds, and the honest reading is in §3: 47 corrections were returned, 4 of them
overturning a stated **root cause** while the defect stood, 3 finding an acceptance test that
**could not fail**, 9 finding one a lazy fix would pass, and 1 demanding an outright regression.
The verification's value was not in killing items; it was in the fact that **half the proposed
fixes and a third of the proposed falsifiers were wrong in a way that reading the report could not
reveal.** Every survivor's entry in §8 is the corrected version, not the merged one.

Where a *sub-claim* was refuted, the parent survived on the remainder and the refutation is
recorded inline: CRUX-01(b)'s trailing-comment cause (refuted — it is the separator and the case);
CRUX-02's "coverage is silent when absent" (refuted — already fixed; the live defect is staleness);
CRUX-10's polyglot leg (refuted — language-aware SATD is correct in both directions);
CRUX-18's PageRank substrate (refuted — the graph is edgeless by construction, documented in its
own source); CRUX-20's AST-cache clause (refuted for the measured path — two callers, neither in
it).

### 9.2 Dropped at merge, with the reason (do not re-propose without new evidence)

1. **Workspace / multi-crate split.** Only 6.0 % of `src/` is cleanly extractable; `cli` and
   `services` are 64 % of the tree and mutually cyclic. The cheap prerequisite (move
   `src/cli/colors.rs`, 47 of 92 cycle edges, a pure rename) fixes no measurable failure. Kept as a
   documented prerequisite chain so the next audit stops proposing the split as step 1.
2. **Extend the MCP exposure registry to the 71-variant `Commands` enum.** Real (70 of 71 have no
   exposure declaration) and the template exists at `src/cli/command_wire_names.rs`, but #1029's
   literal defect is solved for the family it named and the residual is prevention, not a live wrong
   answer.
3. **CLI/MCP flag parity for `pmat_query_code`** (8 MCP properties vs 49 CLI flags). An MCP-only agent
   cannot run the workflow CLAUDE.md declares MANDATORY, and the token budget is affordable
   (~2,557 tokens for the whole list). A capability gap, not a wrong answer — and CRUX-08/09 must land
   first so new properties are advertised correctly on both transports.
4. **Shell completions** (`clap_complete` + `pmat completion <shell>`). Cheap, and largely subsumed by
   CRUX-05: once clap's suggestions and usage are restored the discovery cliff is far less steep, and
   completions can ride the same PR.
5. **`pmat quality-gate --base <ref>` / diff-scoped gates.** Every commercial competitor's core pitch,
   and the machinery half-exists — but it is a *feature*: nothing reports a wrong answer for lack of
   it, and it interacts badly with CRUX-02's not-measured work.
6. **`pmat cache prune` and 1.63 GiB of unbounded local state.** `.pmat` 728 MB + `.pmat-work` 945 MB;
   `workspace.db` 582 MB that can never satisfy its own freshness predicate and is never deleted;
   `context.db.bak` 52 MB with no writer left in `src/`; `maintain cleanup-resources` finds 62 GB of
   candidates and **zero bytes of pmat's own** while advertising a `caches` target that exits 0 doing
   nothing. **Strongest of the dropped items — promote it if a slot frees.**
7. **Retire the dead `PmatError`** (36 variants, zero production callers, 317 of 355 occurrences its
   own tests) and its unreachable ~30-entry JSON-RPC code map: ~1,500 lines and one of two
   contradictory `to_mcp_code` maps. Fold the decision into CRUX-17.
8. **Dependabot #1112** (`getrandom` 0.2→0.4): one-line `E0425` at
   `src/cli/handlers/mcp_onboarding.rs:80` (0.3 renamed the free function to `fill`); 0.4.3 wants
   rustc 1.85 so MSRV is no obstacle. A PR review comment, not a spec item.
9. **Makefile and `scripts/` dead surface** — 101 of 208 targets orphaned, 3 calling deleted scripts
   (`make test-curl-install` runs a script deleted in PR #582), five `server-*` targets addressing a
   directory that does not exist, 57.5 % of `scripts/` dead to automation, and CI invoking 2 of 208
   targets while `Makefile:12-14` calls itself the single source of truth. Cosmetics under the ranking
   rule; worth a cleanup PR with an `audit-targets` ratchet.
10. **Stale citations of the deleted `src/tests/binary_size.rs`** in `CLAUDE.md:209` and
    `.pmat-ratchet.toml:334-343` — the latter is CB-2101's load-bearing justification for classifying
    `thresholds.binary_max_bytes`, i.e. a coherence binding justified by evidence that no longer
    exists. **PROMOTED, not dropped.** The verdict is still correct and the doc repair is still a
    one-liner, but repairing two observed instances while leaving the checker unrun is the
    symptom-not-cause pattern §12.2 rule 3 forbids — in this document, about this document's own
    class of defect. It is now **CRUX-30**; this entry records the trigger.
11. **The sovereignty doctrine: 80 % claimed vs 11.0 % measured** (12 of 108 direct normal deps;
    3.4 % by resolved package; 41 % under the most charitable reading) — and the dependency the policy
    names by example is on the wrong side: `src/utils/sovereign_compression.rs` is a finished
    `trueno-zram-core` adapter behind an orphan feature in neither `default` nor `full`, while
    `lz4_flex` ships in the default bundle with four call sites. Governance rather than product
    behaviour; the actionable half (ratchet the count of external deps with a shipped sovereign
    replacement, starting at 1) should ride with CRUX-14.
12. **`ci / gate` waiting 15.7–18.8 min in the self-hosted queue for a 6-second aggregator** — 32 % of
    the 49-minute critical path, fixed by one `runs-on:` line at `sovereign-ci.yml:1309`. **Dropped
    because the fix is in `paiml/.github`**; file it there, where it benefits ~19 consumers.
13. **MCP toolsets and a read-only posture** (`PMAT_MCP_TOOLSETS`, `PMAT_MCP_READONLY=1`, naming the
    toolsets that already exist implicitly — analysis, query, gates, scaffold, git). §6a and §6e both
    score pmat down for "19 flat tools, no toolsets, no read-only mode" against GitHub MCP's 23
    toolsets + `GITHUB_READ_ONLY=1` and SonarQube MCP's 40+ across named toolsets, so the row is real
    and the procurement-checkbox argument stands. **Dropped because its safety motive is
    self-refuted:** the two tools it would lock down do not mutate. `quality_proxy` writes nothing
    (CRUX-10, 7 `operation:"write"` calls → 0 files), and `scaffold_project` leaves `ls -R`
    byte-identical. There is nothing to lock down today, so the remaining value is toolset
    *selection* (context economy) — worth doing after CRUX-08/09 make the schemas honest, not before.
    If it is revived, its acceptance test is two legs and the second is the control: with
    `PMAT_MCP_READONLY=1` the tool list must not contain `quality_proxy`, **and without the env var
    it must**, so deleting the write tools cannot satisfy it. Note the leg-1 index in the research
    record is stale: measured against the HEAD binary, `tools/list` returns 19 tools with
    `quality_proxy` at index **17**, not 18.
14. **Unifying the two file-discovery layers** (§5.7): `analyze complexity` and `analyze dead-code`
    walk the tree with code that shares nothing, which is why five commands report five denominators.
    Real, and it is the reason no two throughput figures in §5 are comparable — but nothing reports a
    *wrong* answer for it, both walks are defensible for their own command, and merging them touches
    every analyzer at once. Recorded so CRUX-13's parity leg can say what it assumes rather than
    assuming it silently.
15. **`pmat lsp`** (diagnostics-only). Zero LSP protocol strings in a 54.6 MB binary and every
    competitor has in-editor feedback, so the gap is real — but it is the largest build on the list
    (L→XL) for a capability no issue asks for, and CRUX-15's SARIF work delivers part of the same
    value through a channel that already exists.

### 9.3 Explicitly not proposed (fable-review §5 still binds)

No item below entered the backlog: an Nth language mutation backend; a new analyzer or query mode
without a labelled fixture corpus in the same PR; a ninth score command; re-implementing aprender
inside pmat; rebuilding `pmat org analyze` by vendoring removed upstream code; advertising
`l3_kani_proved` or mutation-kill numbers before the runner executes in CI; chasing the 95 %
coverage number before the corpus executes; re-enabling auto-release; treating `ROADMAP.md` as a
live planning input; re-scoping `pmat verify`'s complexity gate to full-repo; flipping
`quality-gate --fail-on-violation` on against the existing baseline.

---

## 10. Issues that can be closed

### #1114 — "Epic: Comprehensive Code Quality & Architecture Audit" → **close and re-file the seven survivors**

The tree it audits does not exist (`ls -d server` → *No such file or directory*), and **5 of its 19
checkable claims are FALSE at HEAD** — including two of the three headline architectural findings.
The full claim-by-claim table is §7.2. The decisive items: the "10,478-line god module" is a 70-file
directory (the real defect is worse and different — 16,789 lines by `include!` closure, §8.11);
`deep_context.rs (6,114 lines)` does not exist; **"193 test binaries" is half-true and must be quoted
carefully** — there are now **258** tracked `.rs` files under `tests/` (`find tests -name '*.rs' |
wc -l`), and the 193 is transcribed from this repository's own `Cargo.toml:28` comment, so only the
"each its own integration binary" half is false: 11 declared `[[test]]` targets under
`autotests = false`, which has been in `Cargo.toml` since **`2aa5832a1` (2026-01-14)** — `git log -S`
returns two commits and `git show 2aa5832a1:Cargo.toml | grep -n autotests` puts it at line 17 of
that revision — i.e. **seven months before the issue was filed**, not two weeks. Publishing
"`ls tests/*.rs | wc -l` → 9" as *the* refutation of 193 invites a one-line rebuttal that would
discredit the whole table, because `ls` is non-recursive and measures a narrower thing. Then: "zero
pre-commit quality gates" against a 285-line hook; "many empty stub specs" against
`find docs/specifications -name '*.md' -size -250c | wc -l` → **0 of 41**; "~3,176 transitive
dependencies" against a measured 847 packages (489 host-filtered). Its coverage/mutation/benchmark
claim is also refuted (`ci/coverage` runs 32.9 min and is required through `ci / gate`;
nightly-bench ran 2026-09-01).
**Re-file these seven, each with its own issue:** the `Commands` enum (1,822 lines, worse than the
895 filed); four verbatim `McpRequest`/`McpResponse` definitions; 889 `anyhow::Result` against one
`PmatError` with zero production callers; sync IO inside `async fn discover_project_structure`
(`src/services/deep_context/analyzer_core/file_tree.rs:14→:32`); `resolve_import_to_node` O(imports
× nodes) with a per-candidate allocation (`src/graph/builder_analysis.rs:142-152`); every docker
artifact publishing tag `2.10.0` against a 3.35.0 crate; and toolchain sprawl.
**Why closing matters:** leaving the epic open with a majority-false body keeps sending agents to a
directory layout removed in Jan 2026.

### #1031 — "Complete AGY (Antigravity 2.0) Support & Comply Integration" → **close, delivered**

Verified at HEAD: `pmat init --target agy` exits 0 having written `.agents/hooks.json`,
`.agents/mcp_config.json`, `.agents/skills/pmat-quality/SKILL.md`,
`.agents/hooks/pmat-quality-feedback.sh` and `AGENTS.md`, and **REFUSES** `.agents/plugins.json`
with a written reason (no published schema) plus a tracking-issue reference — printing
`5 written, 0 already current, 0 kept, 1 refused` and a `next:` step. Re-running reports
`0 written, 5 already current`; with a pre-existing user `AGENTS.md` it reports `1 kept` with the
file's md5 unchanged (`5502266989cfbdbaa4905d8a3051fd1d` before and after) — idempotent and
non-destructive. The comply side is wired: CB-1664 validates `hooks.json` at
`src/cli/handlers/comply_handlers/check_handlers/check_macs_agy_hooks.rs:11`, alongside
`check_macs_agy_mcp.rs` and `check_macs_agy_skills.rs`. **This is the best-behaved first-run
surface measured anywhere in this audit and should be the house model for the other
`init --target` variants.**

### #1029 — "MCP tool list is hand-curated" → **close for the `analyze` family; re-file two residuals**

The literal ask — that the list be derived so a new subcommand cannot be CLI-only by omission — is
delivered for the family it names, with enforcement stronger than requested.
`src/cli/analyze_mcp_exposure.rs:109` `macro_rules! analyze_mcp_registry` emits a **total match with
no catch-all** (`:129-137`), so adding an `AnalyzeCommands` variant without declaring
`Tool` / `CliOnly(reason)` / `Backlog(issue)` is a **compile error**; the 37 rows are 9 Tool /
6 CliOnly / 22 Backlog, and `BACKLOG_CEILING = 22` (`:236`) is ratcheted by two tests
(`src/cli/analyze_mcp_exposure_tests.rs:203,224`). The three analyzers the issue names by hand are
live in the built binary's `tools/list` (19 tools; its headline "16" is stale), and
`pmat mcp manifest` prints the honest projection.
**Re-file separately, because they are different defects:** (1) the other **70** top-level
`Commands` variants have no exposure declaration of any kind, while the totality machinery already
exists in `src/cli/command_wire_names.rs` and is currently spent on a wire name; (2) two dead tool
inventories still name analyzers the registry marks `Backlog` —
`src/handlers/initialize_tools_*.rs` (21 tools, reachable only from
`src/tests/e2e_full_coverage.rs`) and `src/contracts/mcp_simple/handler.rs:89` (8 tools, reachable
only from its own tests) — which is the exact reason #1029 deleted
`src/contracts/mcp_impl_server.rs`.

---

## 11. Verification ledger

Every load-bearing number in this document, with the command or `path:line` that produced it.
`MEASURED` = observed this session against the HEAD binary or the tree. `ARTEFACT` = read from a
prior instrumented run in `research/` — **a directory that is not in this repository** and will not
survive the session (§1, Artefact custody); every ARTEFACT row must be re-derived or its file
preserved before it is relied on. `ESTIMATE` and `UNVERIFIED` are marked as such. **Every `git grep`
count below carries its pathspec**, because two pathspecs over one tree produce two numbers and
neither is a correction of the other.

| claim | value | source | status |
|---|---|---|---|
| Audited commit / written-against commit | 01fba4f65 / 512c60139 | `git log --oneline -1` | MEASURED |
| Delta between them | 5 files: CHANGELOG, Cargo.lock, Cargo.toml, README, mcp.json; **0 lines under `src/`** | `git diff --stat 01fba4f65 512c60139` | MEASURED |
| Binary under test | 54,633,288 B, `3.34.0 / commit: 01fba4f65… / worktree: clean` | `stat -c%s`; `pmat --version` | MEASURED |
| verify on a clean tree | `{"ok": true, "stages_measured": 2}`, exit 0, no `not_measured` key | `pmat verify --skip clippy,tests --format json` | MEASURED |
| verify composite logic | `ok: !failed && measured > 0`; guard fires only at `measured == 0` | `src/cli/verify.rs:194`, guard `:189-192`; contract at `:66-76` | MEASURED |
| quality-gate on the same tree | exit 1, `blocking_violations 35`, `total_violations 37`, 14 `severity=error` | `pmat quality-gate --format json` | MEASURED |
| quality-gate breakdown | complexity 6, satd 3, entropy 28, others 0; `files_examined 5374`. **Five of the six complexity rows are at or below the configured `max_complexity = 30`** — they are warnings from the undisclosed `limit - 2` band counted as blocking (§8.24) | `research/m/repo_qualitygate.*` | ARTEFACT |
| SATD default vs strict | 3 vs **0** | `pmat analyze satd --format json [--strict]` | MEASURED |
| strict blind spots | separator + case, not position: `TODO(x):`→0, trailing `TODO:`→1, `Bug:`→0, `BUG:`→1 | `src/services/satd_detector/classifier.rs:244-250` vs `:262-265`, `SEPARATORS` `:139` | MEASURED |
| SATD `--include-tests` | 64 violations / 31 files, `violations_listed 10, violations_truncated true` | `pmat analyze satd --include-tests --format json` | MEASURED |
| gates run SATD without `--include-tests` | `let include_tests = false;` | `src/cli/analysis_utilities/quality_checks_part1_satd.rs:104` | MEASURED |
| dead-code discarded not-measured | `Err(_) => return Ok(violations)` | `src/cli/analysis_utilities/quality_checks_part1_dead_code.rs:65-68` | MEASURED |
| gate on an uncompilable crate | `dead_code_violations: 0`, string `not_measured` absent from the document; analyzer alone → exit 5 `{"not_measured": true}` | both commands, same fixture | MEASURED |
| coverage cache guard | bare `read_to_string` + `from_str`, no git_hash/mtime/breadth check | `src/cli/analysis_utilities/quality_checks_part2_coverage_sections.rs:38-44` | MEASURED |
| the repo's own cache | 209 files, `git_hash` 583ea9ac2, mtime 2026-08-24, 97.3 %; **114 commits behind** | `git rev-list --count 583ea9ac2..HEAD` | MEASURED |
| absent-report case already handled | `coverage_violations: 1`, exit 1, "was NOT measured" | `src/cli/analysis_utilities/quality_gate_part2a.rs:20-46` | MEASURED |
| duplicates: analyzer vs gate | 21.67 % / 33,801 blocks / 31,929 exact over 240,671 of 1,110,797 lines, 4,036 files **vs** `duplicate_violations: 0` | `analyze duplicates` / `quality-gate --checks duplicates` on `src/` | MEASURED |
| gate's duplicate detector | whole-file `HashMap<u64, Vec<PathBuf>>`, no block-level pass | `src/cli/analysis_utilities/quality_checks_part2_security_duplicates.rs:123-176` | MEASURED |
| duplicates cost vs gate cost | 16.22 s / 323.7 CPU-s / 1.77 GB **vs** 2.00 s / 1.82 CPU-s / 57 MB | `research/m/repo_{duplicates,qualitygate}.time` | ARTEFACT |
| `config --validate` on 4 inputs | byte-identical stdout, exit 0 (valid config, corrupt, absent, wrong section) | 4 runs in a scratch dir | MEASURED |
| its cause | `read_config_file(...).unwrap_or_else(Self::default_config)`; `None` for both absent and unparsable | `src/services/configuration_impl.rs:32`, `:45-68`, `eprintln!` at `:53` | MEASURED |
| `PmatConfig` serde defaults | 9 sections, `#[serde(default)]` on 1 (`semantic`) | `src/services/configuration_types.rs:6-34` | MEASURED |
| the validator can fail | `max_complexity = 0` → "Configuration validation failed", exit 1 | full config + one edit | MEASURED |
| dead-code cache key | `git rev-parse HEAD:`; validity = schema + tree_hash + version | `src/services/cargo_dead_code_analyzer/cache_operations.rs:35-46`, `:57-70` | MEASURED |
| cache falsified both ways | dirty→0 (control 1); reverted→1 (control 0) | 4-run fixture, own `git init` | MEASURED |
| warm run runs no compiler | 20 execve, 2 successful (pmat + git); still `reason: compiler-lint-ran` | `strace -f -e trace=execve` | MEASURED |
| cold vs warm dead-code on this repo | 49.87 s / 4.96 GiB vs 0.25 s / 20.6 MB (200× / 260×) | `/usr/bin/time` | ARTEFACT |
| precedent for the fix | hooks cache moved to `git write-tree` | `src/tdg/hooks_cache/helpers.rs:37-68` (issue #748, CLOSED) | MEASURED |
| clap features | `default-features = false, features = ["std","derive","help","env"]` | `Cargo.toml:226`; `cargo metadata --no-deps` | MEASURED |
| Usage sweep | `total=71 usage_pmat=0 empty_usage=70 no_usage_line=1` | loop over the root Commands block | MEASURED |
| error text | `error: unexpected argument found` 34 B, exit 2, `grep -c -- '--bogus-flag'` = 0 | `pmat analyze complexity --bogus-flag` | MEASURED |
| control (clap with defaults) | `bashrs lnt` names the token, tips, prints Usage | same box | MEASURED |
| blind guards | **14** `contains("Usage:")` sites, incl. product code | `grep -rn 'contains("Usage:")' src/ tests/`; `src/docs_enforcement/cli_checker_validation.rs:74` | MEASURED |
| a shipped fix whose evidence never held | `02292aaf8` quotes clap output this build cannot produce | `git show 02292aaf8` | MEASURED |
| binary-size band | EXPECTED 55,000,000, QUIET ±5 %, FAIL ±20 % → 44–66 MB; wired at `feature-matrix.yml:651-652` into `feature-gate` | `tests/modules/binary_size_band.rs` | MEASURED |
| build.rs bad path | `cargo:rerun-if-changed=../assets/demo/`; `ls -d ../assets` → No such file | `build.rs:21` | MEASURED |
| no-op build cost | 55.28 s wall / 263.46 s user / 499 %; 56.12 s / 265.82 s / 496 % | `research/build{2,3}.time` | ARTEFACT |
| cargo's own reason | one `stale: missing ".../../assets/demo/"`, then `StaleDepFingerprint` | `research/build3.stderr` (`CARGO_LOG=…fingerprint=info`) | ARTEFACT |
| directives declared / missing | 10 / exactly 1 | `grep -oP 'rerun-if-changed=\K[^"]+' build.rs` | MEASURED |
| fossil provenance | introduced by `2aa5832a1` (single-crate refactor), which deleted `server/build.rs` | `git log -S… -- build.rs` | MEASURED |
| index mtime fast path | `if mtime >= *built_at { return None; }` then reuse the stored checksum | `src/services/agent_context/function_index/build_helpers.rs:40-67` | MEASURED |
| backdated-mtime reproduction | `1 mtime-skipped`; new fn missed; **deleted fn returned with its source**; manifest `74bc8514…` vs disk `b5ecfd1c…` | scratch crate | MEASURED |
| fast-path coverage on this repo | `4477 mtime-skipped, 0 checksum-reused, 0 re-parsed` | `research/m/repo_query1.err` | ARTEFACT |
| walks unsorted | 20 production `WalkBuilder` sites, none sorted; the only `sort_by_file_name` is a **comment** | `src/services/facades/defect_prediction_facade.rs:602` | MEASURED |
| churn nondeterminism | **5 distinct hashes in 10 runs**; sole diff is `author_contributions` key order | `analyze churn --format json` ×10 | MEASURED |
| churn field type | `pub author_contributions: HashMap<String, usize>` | `src/models/churn.rs:38` | MEASURED |
| MCP transport divergence | **6 of 6** probe shapes differ (see §8.8 table) | both transports, one fixture | MEASURED |
| tools/list ordering | 3 fresh HTTP servers → 3 orders; stdio 1 order, sorted | 6 process starts | MEASURED |
| repair layer is stdio-only | `repair_outbound` has 2 references, both in `stdio_frames.rs` (`:658` def, `:719` call) | `grep -rn` | MEASURED |
| the parity guard | `include_str!("http_server.rs")` + `.contains()`; binds no socket | `src/mcp_pmcp/cli_mcp_surface_parity_tests.rs:86-87` | MEASURED |
| open schemas | `additionalProperties` on **0 of 19**; `deny_unknown_fields` 0 hits; `bogus_param` → `status: completed` on both | live `tools/list` + 2 calls | MEASURED |
| manifest schema fidelity | names 19/19 equal, descriptions 19/19 equal, **inputSchema 19/19 differ** (16/19 differ in shape) | canonical-JSON diff vs live | MEASURED |
| manifest callability | **6 of 19** fail on argument shape | fidelity probe on a git fixture | MEASURED |
| schema source | canned by name at `tool_schema()`; rendered at `:174`; no schema guard among 6 tests | `src/mcp_pmcp/tool_manifest.rs:131-167`, `:239-599` | MEASURED |
| `quality_proxy` writes nothing | 7 `operation: write` calls → 0 files created | `find` after each | MEASURED |
| its only `fs::write` | `:268`, inside `#[cfg(test)]` | `src/services/quality_proxy_operations.rs` | MEASURED |
| bypasses | advisory→`accepted` while `passed:false` (`:70`); client `quality_config` flips `passed` and deletes the SATD row | live calls | MEASURED |
| mis-named tools | `scaffold_project` leaves `ls -R` identical; `git_operation` = git status | live calls | MEASURED |
| TDG vs `include!` | 93.380 (A) split, 60.934 (C) flat, **Δ 32.446**; both AST-scored | `pmat tdg` ×2 + `analysis_provenance` | MEASURED |
| synthetic reproduction | 100.0 A+ split vs 85.0 A- merged on unrelated code | 3-file probe | MEASURED |
| directory walk drops fragments | 70 `.rs` on disk → `files_walked 26, files_analyzed 26, files_ungraded 0, not_measured false` | `pmat tdg src/cli/analysis_utilities --format json`; `src/tdg/analyzer_simple_core.rs:125` | MEASURED |
| include! scale at HEAD | **2,295 sites** (`git grep -c 'include!(' -- src`; the narrower `-- 'src/**/*.rs'` gives 2,289 because the glob drops the 14 top-level `src/*.rs` files — 4,019 of 4,033 — and both numbers are identical at `01fba4f65` and `512c60139`); 2,061 fragment files / 528,656 lines (47.6 % of `src/` lines) by resolving every target over all of `src/`; `#[path = "` 174 | `git grep -c 'include!(' -- src` + target resolution | MEASURED |
| SARIF emitters | **17** `analyze` subcommands advertise `--format sarif` (assembly-script, build-tdg, complexity, comprehensive, dead-code, deep-context, defect-prediction, duplicates, incremental-coverage, lint-hotspot, makefile, proof-annotations, provability, satd, tdg, wasm, web-assembly) of 36, **plus `pmat tdg`** = 18 surfaces | loop over `analyze --help`'s Commands block, `grep -ci sarif` per subcommand | MEASURED |
| SARIF formatter census | **44** non-test files define one (93 including tests, 100 tree-wide) | `git grep -lE '^\s*(pub )?fn [a-z_]*sarif[a-z_]*\(' -- src \| grep -v tests \| wc -l` | MEASURED |
| crates.io build provenance | `~/.cargo/bin/pmat --version` → `commit: unknown / worktree: unknown`, while `pmat-3.34.0/.cargo_vcs_info.json` carries `sha1 71f7d37d1b20233e98cf58b24d52ea636f55a4da`; present for all 6 of 3.30.0–3.34.0 | `--version`; `cat`/`ls` in the registry src tree; `build.rs:1648-1679` | MEASURED |
| duplicates: `exact` vs the default | 1.61 s / 102 % / 596 MiB / 35,342 blocks **vs** 16.67 s / 1988 % / 1.69 GiB / 37,785; `--min-lines 20` → 9,085 blocks at 17.28 s / 1.68 GiB | `research/dup_probe.txt` (paired back-to-back) | ARTEFACT |
| unreferenced root modules | `src/protocol/` 13 files / **2,047 lines**, 0 refs outside its subtree; `src/state/` **3,896 lines**, referenced only by `tests/integration_test.rs.disabled`; both `pub mod` at `src/lib.rs:168` / `:185` | `git ls-files \| xargs wc -l`; `git grep -lE 'crate::protocol\b\|pmat::protocol\b'` | MEASURED |
| complexity warn band | `cyclomatic_warn = max_cyc.saturating_sub(2).max(1)` — banner says 30, gate blocks at 28 | `src/services/complexity/aggregation.rs:122-129` | MEASURED |
| comply reports the population | `Pass` with `format!("All {} ticket level(s) parse", contracts.len())` after three `let … else { continue; }` arms | `check_work_ladder_declaration.rs:13-63` (`:27`, `:30`, `:33`, `:57-62`) | MEASURED (source) |
| `pmat query` peer semantics | no match → exit **0** (`rg -q`/`grep -rq` → 1); `--files-with-matches --limit {1,3,50}` → 1,611 lines each; `--count --limit 3` → 1,611; control `--limit {1,3,10}` → 53 / 114 / 419 | 8 runs against the HEAD binary in this repo | MEASURED |
| no-args help on stdout | rc=2 with **8,998 B on stdout**, 123 B on stderr | `pmat </dev/null >o 2>e` | MEASURED |
| mutating-verb preview | **7 of 17 probed** advertise `--dry-run`; `hooks install`, `hooks uninstall`, `hooks refresh`, `agy sync`, `scaffold project`, `config set`, `work delete`, `refactor interactive`, `embed sync`, `init` do not. `pmat cache clear` **does not exist** (`pmat cache` = `stats`, `help`) | verb-by-verb `--help` census, words split explicitly | MEASURED |
| package-size hard limit | job fails at `mib >= 9.0`; `Cargo.toml:22-25` records the pre-exclusion 9.4 MiB / 94 % figure as a **comment**, and the current packed size is not measured | `.github/workflows/feature-matrix.yml:506-507`; `Cargo.toml:22-25` | MEASURED (threshold) / **UNVERIFIED** (current size) |
| dogfood gates nothing runs | `pmat-gate-transport-parity.sh` declared at `Cargo.toml:435`; `pmat-gate-mcp-surface.sh` **not declared**; no workflow or make target invokes either (`Makefile:152-159` only symlinks them) | `Cargo.toml`; `grep -rn` over `.github/workflows/` and `Makefile` | MEASURED |
| harness blinds itself to the defect | a JSON-RPC error / `-32602` is classified as a harness fault | `scripts/dogfood/pmat-transport-parity.sh:85-95` | MEASURED |
| required-status contradiction | `mutation-diff.yml:50-66` claims the context is required and quotes a live list containing it; `required-status-checks.txt:21-24` says its workflow is not on master; the workflow **is** tracked; live protection contains neither claim | `git ls-files`; both files; `gh api …/protection` | MEASURED |
| link surface | 137 cargo targets (1 lib / 2 bin / 113 example / 11 test / 9 bench / 1 build script); 114 link units under `--all-targets` on default features; rlib 294,094,466 B and 299,361,140 B | `cargo metadata --no-deps`; `ls -la target/release/deps/libpmat-*.rlib` | MEASURED |
| share of `ci / lint`'s 17.9 min attributable to those links | — | not timed in isolation | **UNVERIFIED** |
| five denominators | complexity 4,521/5,374 · dead-code 3,228/4,433 · dag 4,487/5,481 · query index 4,477 · cold index 4,030 | five commands, one tree | MEASURED |
| two discovery layers | Path A `file_discovery_walker.rs:7`; Path B hand-rolls `cargo_dead_code_analyzer/analysis.rs:112-118` + `parsing.rs:293-299`, excluding tests by default at `analysis.rs:175-193`, `:570-580` | source + a 10-file polyglot fixture (9-of-10 vs 4) | MEASURED |
| tests/ file count | `find tests -name '*.rs' \| wc -l` → **258**; `ls tests/*.rs \| wc -l` → 9 (non-recursive); `grep -c '^\[\[test\]\]' Cargo.toml` → 11 | 3 commands | MEASURED |
| `autotests = false` provenance | added by `2aa5832a1` (2026-01-14), line 17 of that revision; `5249d4723` only added prose | `git log -S'autotests = false' -- Cargo.toml`; `git show` | MEASURED |
| clap regression provenance | introduced by `a36329bc6` (2025-06-01); `git show v0.1.9:server/Cargo.toml` → `clap = { version = "4.5", features = ["derive", "env"] }`; v0.1.5 and v0.1.9 both dated 2025-05-28 | `git log -S`; `git show`; `git log -1 --format=%ad` | MEASURED |
| declared test functions | `-- 'src/*.rs'` 33,475 + `-- 'tests/*.rs'` 2,672 = **36,147**; `-- 'src/tests/*.rs'` **4,335**. Note `src/*.rs` is recursive in git pathspec syntax | `git grep -cE '^\s*#\[(tokio::)?test\]'` | MEASURED |
| shell SATD, comment-anchored | **1 file / 2 lines** (`tests/cli_integration_tests.sh`); the unanchored form gives 9 / 17 and measures a different thing | `git ls-files '*.sh' \| xargs grep -lE '#\s*(TODO\|FIXME\|HACK\|XXX)'` | MEASURED |
| churn determinism oracle | 6 raw runs → **6** distinct sha256; the same 6 through `jq -S 'del(.generated_at)'` → **1** | 6 execs + 12 hashes | MEASURED |
| CRUX-05 leg 5 as merged | printed **71** `MISSING …` lines and **exited 0** | the loop, verbatim, against the HEAD binary | MEASURED |
| sovereign compression | `sovereign-compression = ["aprender-zram-core"]` (`Cargo.toml:640`) is in neither `default` nor `full`; `lz4_flex` (`:207`) is inside `standard-deps`, which is in `default` (`:461`, `:466`) | `Cargo.toml`; `git grep -c lz4_flex -- src` | MEASURED |
| complexity provenance | `{ast: 1802, heuristic_include_fragment: 2231, heuristic: 3}` of 4,036 | `analyze complexity --path src --format json --top-files 0` | MEASURED |
| the resolver already ships | `collect_included_files`, BFS, `MAX_INCLUDE_DEPTH = 8`, one caller | `src/cli/handlers/complexity_handlers/analysis.rs:94` | MEASURED |
| reachability | orphans 407 / 126,933 lines / 6,292 tests; quarantined 82 / 35,856 / 2,021; `unresolved_mods 35`; 0.32 s / 22,656 kB | `pmat analyze reachability -f json` | MEASURED |
| `src/tests/` | 188 files / 76,461 lines (`wc -l`) / 4,335 tests (tool) / 82 `coverage_boost_*` | `git ls-files` + the tool | MEASURED |
| nothing runs it | `grep -rn reachability .github/workflows/ Makefile` → nothing; only caller's next line is `mark reachability WARN` | `scripts/dogfood/pmat-dogfood-runner.sh:909-912` | MEASURED |
| the gated template | `analyze unrun-tests --check-ledger` at `feature-matrix.yml:701`, in `feature-gate`'s needs at `:807` | file read | MEASURED |
| unrun lib tests | 3,228 of 26,887 compiled by no leg; 2,199 `<unsatisfiable>` | `docs/status/unrun-tests-ledger.md` | MEASURED |
| ratchet metrics | 6, none about orphans or duplicates | `grep -n '^\[metric\.' .pmat-ratchet.toml` | MEASURED |
| SATD walk allowlist | 22 extensions; no `sh`/`make`/`yaml`/`dockerfile`; applied during discovery | `src/services/satd_detector/detection_file_discovery.rs:180-206`, `:59` | MEASURED |
| tracked files outside it | 5,739 tracked vs 4,525 admitted → **1,214 (21 %)** never in the denominator | `git ls-files` + the allowlist | MEASURED |
| the test that pins it | `assert!(!detector.is_source_file(&PathBuf::from("script.sh")))` | `src/services/satd_detector_tests_extraction.rs:269` | MEASURED |
| coverage floor never ran | step `Enforce coverage floor…` = `"skipped"` on 4/4 recent green master runs | `gh run view --json jobs` | MEASURED |
| MSRV | declared 1.91.0, graph max **1.93.1** (`serial_test_derive`, a direct dev-dep); job is `on: release: published`, no `--tests`/`--all-targets` | `cargo metadata --filter-platform`; `post-release.yml:11-15,66,135-141` | MEASURED |
| Dependabot arm | no `DEPENDABOT_TOKEN` secret; leg warns "did NOT run" and exits 0; `feature-gate` echo asserts the opposite | `gh secret list`; `feature-matrix.yml:568-586,807,818`; run 33618635820 | MEASURED |
| duplicate-dependency artifacts | script exits 1 on `cd scripts/../server`, no caller; `Makefile:1210-1211` `\|\| echo`; `CRITICAL_DUPLICATES` empty, one test with no assertion | 3 files | MEASURED |
| real duplicates | **35** crate names at >1 version; `KNOWN_UNAVOIDABLE` 28, 13 stale, 20 undocumented | `cargo tree -d --target x86_64-unknown-linux-gnu` | MEASURED |
| semver-checks | absent from all workflows; `Makefile:1582` `\|\| echo`; `:1609` piped into grep | grep + file read | MEASURED |
| branch protection contexts | `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder` | `gh api …/branches/master/protection` | MEASURED |
| `--format` enumeration | 35 analyze subcommands; 33 accept `--format`; **29 enumerate in 22 sets**; **4** free-string; **2** have no `--format` (`dag`, `clippy`) | clap-aware parse of `--help` | MEASURED |
| free-string proof | `{banana, sarif, summary}` `cmp`-identical for all four | 12 runs | MEASURED |
| SARIF | 0 upload steps; 6 emitters, **5** driver names, **2** `$schema`, `uriBaseId` absent, one uri is a directory | grep + 6 runs | MEASURED |
| an unreachable SARIF writer | `src/cli/defect_helpers/format_sarif.rs` exists; `analyze defects` enumerates `[text, json, junit]` | file + `--help` | MEASURED |
| `pmat mutate` | `error: unrecognized subcommand`, exit 2, on both binaries; `--help \| grep -ci mutate` → 0 | 4 runs | MEASURED |
| README claims | 7 (`README.md:270,336,337,338,497,547,561`), incl. a CI step and the Commitments-table falsifier | `grep -c "pmat mutate" README.md` | MEASURED |
| stub adapters | 5 × `passed: true`; control `rust_adapter.rs:78` is `output.status.success()` | `grep -rn` | MEASURED |
| disclosure exemption | `mutate` excluded because it "cannot mislead"; `--help \| grep -c 'NOT AVAILABLE'` → 3 | `src/cli/commands/feature_gate_disclosure_tests.rs:20-22` | MEASURED |
| JSON failure sweep | **n=35, fail=35**; 33 emit 0 stdout bytes; exits 1×25, 2×8, 5×2 | loop over `analyze` subcommands | MEASURED |
| wrong errno | `tdg /nonexistent…` → "Permission denied (os error 13)", exit 1; `tdg ./definitely-not-here` → prints `0.0`, exit 5 | 2 runs | MEASURED |
| exit-code adoption | `quality_gate_failure(` 0 raise sites; `analysis_error(` 7; `configuration_error(` 2; `process::exit(` 70 (52 × `exit(1)`) | `git grep` excl. tests | MEASURED |
| `not_measured` types | str (verify) / bool (tdg) / bool (dead-code) / array (MCP quality_gate) | 3 live runs + `src/cli/handlers/score_handler.rs:70` | MEASURED |
| `pmat context` size | **8,670,750 B**, 5.53 s, 287,708 kB; ≈2.17 M tokens | `wc -c`; `/usr/bin/time -v` | MEASURED / ESTIMATE (tokens) |
| no budget flag | none in `--help`; `Context` variant has 8 args; MCP schema `[paths, format, max_depth, include_dependencies]` | `--help`; `definition.rs:87-120`; live `tools/list` | MEASURED |
| `--include-large-files` | prints "not yet implemented. Flag ignored." while `--help` describes it as working | `src/cli/handlers/utility_handlers/context_generation.rs:51-53` | MEASURED |
| the PageRank substrate is inert | "`num_edges()` is 0 and `hot_symbols()` is empty for every project"; `--rank-by pagerank` diffs IDENTICAL | `src/services/context_impl/build.rs:375-397,433-434`; 2 query runs | MEASURED |
| `score` peak RSS | **10,237,248 kB (9.76 GiB)** and **13,329,792 kB (12.71 GiB)**; 11.3 M minor faults | `research/m/repo_score{,2}.time` | ARTEFACT |
| the budget | `max_memory_usage_mb = 512`, classified `kind = "budget"` | `.pmat-metrics.toml:59`; `.pmat-ratchet.toml` binding | MEASURED |
| the hidden child | `Command::new("pmat")… .stderr(Stdio::null())`, no timeout; strace shows `~/.cargo/bin/pmat` | `src/cli/handlers/score_handler.rs:579-587`; `strace -f -e trace=execve` | MEASURED |
| comply's own disclosure, discarded | `comply: N group(s) … ~X GB peak` to stderr; `COMPLY_BYTES_PER_WORKER = 4 GiB` | `check.rs:483-487`, `:390` | MEASURED |
| the existing band template | `PMAT_REQUIRE_PEAK_RSS` in **0** files; `binary_size_band.rs` has the exact shape | grep + file read | MEASURED |
| complexity CPU | 113 / 110 / 112 % over three runs, `files_analyzed 4521`; comparators 1995 % / 1768 % | `/usr/bin/time -f 'cpu=%P'` ×3 | MEASURED |
| the sequential batch | comment at `:153` over `try_join_all` at `:168`, no spawn/rayon | `src/cli/analysis_utilities/quality_checks_part4.rs` | MEASURED |
| read amplification | 113 openat on `.rs`, 87 successful, 26 ENOENT, 49 distinct for 42 analysed files | `strace -f -e trace=openat`, `-p src/graph` | MEASURED |
| the double read | `:464` reads content → `mod.rs:34` → `ast_rust_compat.rs:22` → `accurate_complexity_analyzer_core.rs:7` re-reads by path | source chain | MEASURED |
| cold index | 14.86 s / 99 % / 208,520 kB, `load_index: 14791ms ANDON`; re-measured under load 17.65 s / 17,560 ms / 99 %; warm 0.15–0.18 s | `research/m/cold_query*.{time,err}` + a re-run | ARTEFACT + MEASURED |
| CI cost | 43 jobs / 7.67 runner-hours / 49.0 min critical path; `ci / gate` queued 15.7–18.8 min in 2 of 4 runs for a 6-s job | `gh api …/check-runs` | MEASURED |
| cancelled master runs | 26 of the last 60 (43 %) | `gh run list --workflow ci.yml --limit 60` | MEASURED |
| `#[ignore]` | 352 in `src/`, 772 in `tests/` → 1.05 % and **28.0 %** of compiled tests | `git grep -cE '^\s*#\[ignore'` | MEASURED |
| MCP tool-definition cost | 10,230 B for 19 tools ≈ 2,557 tokens (~135/tool) | live `tools/list \| jq -c .result.tools \| wc -c` | MEASURED / ESTIMATE (tokens) |
| reverse dependencies on crates.io | `{"meta":{"total":0}}` | crates.io API | MEASURED |
| `.pmat` derived state | 728 MB (workspace.db 582 MB, context.db 68 MB, context.db.bak 52 MB) for a 48 MB source tree | `du -sh .pmat`; `du -ah \| sort -rh` | MEASURED |
| post-fix build timings; `--features mutation-testing` build; `rust-project-score`; comply hermeticity | — | require `cargo build/test/clippy` or `comply check`, both outside the read-only budget | **UNVERIFIED** |

---

## 12. Definition of done

### 12.1 For this audit

A reviewer takes any §8 item and, from *title + Problem + Evidence + Proposal + Acceptance test*,
either opens the pmat ticket or rejects the item — **with zero follow-up questions**. Every item
carries its own falsifier with the exact assertion that must turn it RED, the observed "today"
value beside it, and at least one named anti-vacuity control identifying the lazy fix it blocks.
Every repo path cited resolves (`test -e`), every number traces to §11, and anything that could not
be measured under the read-only budget is marked UNVERIFIED rather than estimated. Two standing
caveats on that promise, stated rather than assumed: **§11's ARTEFACT rows point outside the
repository** until someone preserves them (§1), and **a `git grep` number is only reproducible with
its pathspec**, which is why every such row now carries one. No item is
UNDERSPECIFIED. Findings without a fetched artefact do not appear in §8 — they appear in §9 with
the reason.

### 12.2 The wording every enhancement must satisfy

Four rules, in order of how often this audit found them broken:

1. **A gate must be able to fail.** Before a fix is accepted, its acceptance test must be run
   against the **pre-fix** binary and observed to FAIL, and that transcript is the evidence. A test
   that prints a complaint and exits 0 is not a gate (three of the twenty merged tests did exactly
   that). A leg that logs SKIPPED and exits 0 is not a gate. A metric that measures 0 against a
   baseline above 0 is UNMEASURABLE, not passed.
2. **Every gate carries a named anti-vacuity control.** State, in the ticket, the cheapest fix that
   would satisfy the test without fixing the defect — delete the feature, hardcode a constant,
   truncate the output, measure less, make both sides equally wrong — and add the assertion that
   blocks it. Nine of the twenty merged tests were passable by a lazy fix; each is now paired with
   a control naming it.
3. **A fix goes where the defect is caused, not where it was observed.** `check_dead_code`'s
   `Err(_)` arm, not the gate's printer. `Cargo.toml:226`, not the six tests that could not see it.
   The cache **key**, not the answer it returns. The `include!` **resolver**, not the TDG score.
   `ConfigurationService`, not the validation handler. `build_server`, not the stdio wrapper. Where
   the cause is upstream (pmcp's `create_response`), say so and file it there, keeping the local
   change as a declared interim.
4. **A fix may make a gate able to fail more; never less.** No item in §8 weakens a threshold,
   widens a band, deletes an assertion, or adds a `|| true`. Where a change would make a gate
   permanently red (CRUX-14's coverage floor, CRUX-15's SARIF upload, CRUX-14c's semver), the
   ticket says so and specifies a **measured** baseline or an explicit staging that is still able
   to fail — never `if: false`, never `continue-on-error`.

Plus two standing constraints inherited from doctrine: **no new external dependency** where a
batuta-stack crate or an already-vendored one will do (no item in §8 adds one); and **the ratchet
rule** — a baseline may only improve, and raising one requires a written `justification` checked
against the previous committed version of `.pmat-ratchet.toml`.

### 12.3 Landing order

Ship in EV order (§8.1 → §8.32, reading §8's preamble for the two places where the numbering and the ranking deliberately part company), with three exceptions the verification established:

- **CRUX-06 first regardless** — one deleted line, which removes the only permanently-stale item in
  the fingerprint. Measured at **55 s wall / 263 CPU-s for a no-op *release* build**, at 1-minute
  load 26.4–26.8; the **debug cost is UNVERIFIED** and `pmat verify` is **not** a beneficiary (its
  clippy stage short-circuits on a receipt, `src/cli/verify.rs:553-558`, and its test stage is a
  debug `cargo test --lib`, `:510-511`). It still ranks first because it makes every other item's
  release-build loop cheaper at a cost of one line, not because it speeds up `verify`.
- **CRUX-12 before CRUX-14's coverage leg** — a coverage floor over a `--lib`-only suite ratchets a
  fiction, which fable-review §5 already bans.
- **CRUX-18 bundled with `context-output-fixture-gate`** (fable-review §4 EV-11) — its L5/L6/L7
  legs are the first content assertions the production context formatter has ever had, and a
  truncating formatter must not ship without them.

Split before starting, where §8 says so: CRUX-14 → a/b/c, CRUX-15 → a/b, CRUX-11 → (a) product and
(b) repo, CRUX-02 → gate honesty vs the block-level detector, CRUX-17 → four independently gateable
legs, CRUX-22 → (a) default flip / (b) interning / (c) bound the engine. Each split half must carry
its own falsifier; a half that cannot fail on its own is not a ticket.

Three ordering constraints among the added items: **CRUX-29 after CRUX-11(b)** (excluding test
sources before the `include!`s are converted ships a crate that does not compile); **CRUX-26 with
CRUX-17**, not before it, because both change exit codes and they must land under one table and one
changelog entry; and **CRUX-31 before PR #1113 merges**, which is the only dated trigger in the
backlog. **CRUX-23 and CRUX-30 are the two cheapest gate-integrity items in the document** — a
`--lib` test and a text check, each demonstrably red on HEAD — and are the right first PRs for
anyone establishing that §12.2 rule 1 is being followed.

*Generated by a 12-dimension research fleet, a 3-lens adversarial verification, and a second pass
that audited the merge itself and recovered twelve candidates it had dropped in silence.
2026-09-02, against `origin/master` `01fba4f65`, written against `512c60139`.*
