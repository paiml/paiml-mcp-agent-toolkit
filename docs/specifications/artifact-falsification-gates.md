# Artifact Falsification Gates

**Status**: active
**Location**: `tests/modules/quality_harness/`
**Entry points**: `make gate-artifact`, `make gate-flag-efficacy`, `make gate-differential`

## Why these exist

Dogfooding the released 3.29.0 artifact produced **243 confirmed defects**. At
the time the repository had ~19,000 passing library tests, a clean `clippy
--all-targets`, and a self-reported TDG score of 95.3 / A+.

The tests could not see the defects because they were written *from* the
implementation rather than from the requirement. Roughly twenty of them
asserted the defect outright — `assert_eq!(score.total, 0.0)`, a proptest
encoding a stub's empty return as an invariant, five vacuous
`assert!(result.is_ok() || result.is_err())`.

The three largest classes were:

| Class | Count | Example |
|---|---|---|
| Fabricated output | 51 | `let coverage = 65.0; // Simulated coverage` |
| Flags that parse but change nothing | 49 | `--top-files` accepted, never read |
| Cross-surface contradiction | 24 | MCP said 10/18, CLI said 6/9, same function |

These gates target the first two. They share a design constraint that makes
them work where conventional tests did not:

> **They never need to know the correct answer.**

For a code-quality tool nobody knows what the right TDG score is, so a wrong
number is indistinguishable from a right one and survives indefinitely. Both
gates instead assert *properties that hold for any honest measurement*,
whatever its value.

## Gate B — flag efficacy

```
for every flag F reachable from the binary's own --help:
    observable(cmd F) ≠ observable(cmd)
```

`observable` is the triple (exit code, stdout, stderr), normalised for
durations, timestamps, temp paths and git object ids. Exit code is included
deliberately: a flag whose only effect is to change exit status is effective,
and a check that prints findings while exiting 0 is a defect — nine such
defects were in the 3.29.0 sweep.

Enumerated options are checked by pitting two legal values against each other
(`--format json` vs `--format summary`) rather than by presence.

Verdicts are `Effective`, `Refuses` (unimplemented and says so — a pass),
`NoOp`, `Errors` (the flag breaks a working command), and `Skipped` with a
reason. **Skips are printed, never counted as passes.**

## Gate A — differential corpus

```
for every numeric leaf L in a command's JSON output:
    L(empty) , L(tiny) , L(large)  must not all be equal
```

Three corpora are generated: an empty-but-valid project, a one-function
project, and a 114-file project carrying every defect family pmat claims to
detect (complexity, SATD, fault patterns, duplication, dead code, superlinear
algorithms, long functions, one deliberately pathological file), with repo
hygiene graduated alongside the code. All three are real git repositories,
committed with hooks disarmed and with dates computed *relative to now* so
every commit falls inside the default analysis windows.

Array *lengths* count as leaves. A `files[]` of identical length for an empty
and a large project is the same defect wearing a different hat. Array
*elements* are not compared, because index 0 of two different projects is not
the same thing.

Configuration echoed back into the output (thresholds, targets, schema
versions, elapsed time) is excused by key name. That exclusion list is
deliberately narrow — every fragment added to it shrinks what the gate proves.

## The escape hatches

Each gate has exactly one: `ALLOWED_NOOPS` and `ALLOWED_CONSTANTS`, plus
`NON_MEASURING` for commands that do not measure the project at all. Every
entry is a `(command, item, reason)` triple, and the reason is a claim someone
must defend in review. Adding an entry is the only way for a violation to
pass, and it is visible in the diff.

`ALLOWED_NOOPS` is empty. `ALLOWED_CONSTANTS` holds only leaves the corpus
provably cannot vary — a fixed checklist's length, buckets no fixture input
belongs in — each naming the corpus limitation rather than excusing pmat.

## Running them

```bash
make gate-artifact                          # both, against this workspace's build
make gate-flag-efficacy-full                # the entire command tree, not the core subset
PMAT_BIN=$(which pmat) make gate-artifact   # against the installed artifact
```

`PMAT_BIN` is the point. A green `cargo test --lib` coexisted with all 243
defects, so the working-tree build is not the thing being certified — the
binary users install is. The same variable lets a release candidate be A/B'd
against the previous published version.

Reports land in `$TMPDIR/pmat-flag-efficacy-report.txt` and
`$TMPDIR/pmat-differential-corpus-report.txt`.

## The gates guard themselves

A falsification harness that cannot fail is the defect it hunts. This is not
hypothetical: **the first run of Gate A discovered zero commands and exited
0**. The help parser could not read clap's expanded layout, so every
`--format` looked unsynthesisable, no JSON-capable command was found, and the
sweep reported a clean bill of health over an empty set.

The countermeasures, all of which run in the normal (non-ignored) suite:

- Both sweeps **fail when discovery returns fewer than ten subjects**.
- Gate B fails when fewer than twenty flags were actually exercised, so a run
  that degenerates into skips cannot pass.
- `noop_detection_is_load_bearing` proves the comparator can return `NoOp` —
  a comparator that always reports a difference would pass everything.
- `expanded_help_layout_yields_enumerated_values` pins the exact clap layout
  that caused the vacuous first run.
- `large_corpus_contains_every_defect_family` and
  `corpora_differ_in_the_ways_metrics_should_notice` prove the fixtures
  contain what the verdicts are drawn from.
- `constant_detection_catches_a_fabricated_score` reproduces the shipped
  four-literal 0.79 score and asserts the detector flags it.

## Fixture artifacts: the failure mode to expect

A differential gate reports "this number did not respond to the input". That
conclusion is only as good as the input. During bring-up, **eight of Gate A's
first findings were faults in the fixture, not in pmat** — each looked exactly
like a real defect until it was reproduced by hand.

| Symptom | Actual cause |
|---|---|
| `analyze churn` reports `total_commits: 0` | `init.templateDir` copied the user's pre-commit hook into every `git init`; the hook ran, failed, and aborted every corpus commit |
| `analyze churn` *still* zero after commits exist | commit dates were pinned to a fixed calendar date seven months in the past, outside churn's 30-day default window |
| `analyze dead-code` reports zero on 15 dead files | dead functions were shorter than `--min-dead-lines` (default 10) and filtered out |
| `analyze dead-code` reports zero on *long* dead files | the harness ran under `cargo test`, so the nested `cargo check` that dead-code shells out to inherited the parent jobserver and produced no diagnostics |
| `repo-score` constant across all corpora | all three corpora shipped an identical README and Cargo.toml, so repo-hygiene scoring had no axis to vary along |
| `analyze defects` reports `high/medium/low = 0` | truthful — every defect rule emits `critical`, and `critical` did vary |
| `analyze big-o` reports an empty `O(n^2)` bucket | truthful — the "branch-heavy" fixture functions have *sequential* loops in separate branches, which really is O(n) |
| `analyze tdg` reports `f_grade_count: 0` and the F-grade gate passes | truthful — the worst file in the corpus graded C+, so there was no F to count. The overall gate did fail, via `MinimumGradeGate` |

The last two share a root cause worth stating separately: **a differential
check can only interrogate the range its corpus spans.** A bucket that is
empty because nothing in the fixture belongs in it is not a defect. The corpus
now carries nested-loop functions and one deliberately pathological file so
those tails are populated.

Gate B produced its own five:

| Symptom | Actual cause |
|---|---|
| ~40 `--color` flags reported as no-ops | the harness set `NO_COLOR=1` for determinism, under which `--color always` and `--color auto` correctly produce identical output. Determinism already came from stdout being a pipe |
| `--min-dead-lines`, `--top-files` reported as no-ops | the probe values (1 and 5) did not *straddle* the corpus — every dead region is ~13 lines, so both admitted everything. Probes are now 1 and 50 |
| 20 flags reported as "breaks a working command" | the command was already broken. `pmat split` exits 1 with "FILE argument is required" before any flag is added; the verdict never consulted the baseline's exit code, so every flag inherited the blame |
| all 28 `pmat query --*` flags reported as no-ops | the baseline panics without a search term, emitting identical text whichever flag is added. With a real query, `--faults` plainly works (7396B vs 6996B) |
| 3 flags reported as "breaks a working command" | they *refuse honestly* — `--ml is not implemented: ... this flag would relabel them without changing them`. That is the desired behaviour for an unimplemented feature and now scores as a pass |

The general rule behind all three: **a differential verdict is only as good as
its control.** Suppressing the thing under test in the environment, probing
with values that cannot discriminate, and omitting the baseline from the
comparison each produce confident, wrong findings.

The countermeasures are in the harness: hooks disarmed via `core.hooksPath`
plus `--no-verify`, commit dates computed relative to now, dead regions sized
past the tool's own threshold, repo hygiene graduated across corpora, and
`scrub_cargo_env` stripping `CARGO*`/`RUSTC*`/`RUSTFLAGS`/`LD_LIBRARY_PATH`
from every spawned process.

Two structural guards make a recurrence visible rather than silent:

- Every report prints a **corpus fingerprint** (file count, byte count, commit
  count) so a finding can be tied to the fixture that produced it.
- `sweep_readings_match_a_hand_run` asserts the harness's own reading of
  `analyze dead-code` matches what the command returns from a shell. A gate
  whose numbers disagree with the tool it audits does not merely miss defects
  — it manufactures them.

**Triage rule: reproduce every finding by hand against a dumped corpus before
filing it.** Use `PMAT_CORPUS_OUT=/tmp/corpus PMAT_CORPUS_SIZE=large cargo test
--test all -- --ignored dump_corpus --nocapture`.

## Verified findings at introduction

Reproduced by hand against a dumped corpus. Everything else the sweeps
reported is untriaged and is not claimed as a defect.

1. **`comply report` output is byte-identical** for an empty project and a
   114-file defect-rich one — only the timestamp differs. It also reports
   `project_version: 3.30.0` (pmat's version) for a project whose Cargo.toml
   says `0.1.0`, and returns `is_compliant: true` with three checks in `Warn`.
2. **`analyze entropy` extracts zero patterns** from 107 files including 20
   byte-identical pairs, so every entropy metric is permanently `null`. Note
   the #650 fix is working correctly here: it reports "not measured" rather
   than claiming zero diversity. The defect is upstream, in the extractor.
3. **`analyze duplicates.structural_similarities` is always 0**, while the
   hash-based detector in the same invocation finds 162 clones and 85%
   duplication.
4. **`score sub_scores.{coverage,dbc,evoscore,pv_lint} = 50`** — hardcoded
   fallbacks at `score_handler_compute.rs:18,37,54` for dimensions that were
   never measured, folded into a composite as if they had been. This is the
   same class as the 51 fabricated values in 3.29.0.
5. **`--quiet` is inert** on `analyze complexity` — 1644 bytes with and
   without.
6. **TDG sensitivity**: `src/awful.rs` in the corpus — 399 lines, ~300
   branches, four levels of nesting, three SATD markers — grades **76.6/B**.
   For a technical-debt grader that is worth investigating; it is also why the
   corpus cannot produce an F-grade input.
7. **`pmat query` with no search term panics** rather than reporting a usage
   error: `thread 'main' panicked at query_handler/query_execution.rs:76` with
   exit 101. A missing required argument should be a clean clap error.

Three things pmat does *right* that the gate initially mistook for defects,
recorded because they are the behaviour to preserve:

- `analyze complexity --ml`, `analyze satd --evolution` and
  `analyze deep-context --full` **refuse and say why** ("this flag would
  relabel them without changing them", "the flag(s) would be accepted and
  ignored"). That is the fix for this exact defect class, and the gate now
  scores it as a pass via the `Refuses` verdict.
- `analyze entropy` reports `null` for unmeasurable metrics instead of
  claiming zero diversity (#650).
- `tdg check-quality` fails correctly on the defect-rich corpus, via
  `MinimumGradeGate`.

## What these gates do not cover

Stated explicitly, because a bounded gate that does not say what it bounds
reads as full coverage:

- **Cross-surface equivalence** (CLI vs MCP vs HTTP) — the 24-defect class.
  Not covered here; the durable fix is deriving the MCP tool registry from the
  CLI command enum so drift is impossible by construction.
- **Mutating and interactive commands** — `oracle`, `kaizen`, `refactor`,
  `hooks`, `agent`, `serve` and others are denied by name in `DENY_ROOTS`,
  each with a stated reason. They are excluded because the harness must be
  safe to run anywhere, not because they are exempt from the invariant.
- **Options taking unenumerated values** (paths, integers) — skipped, and
  reported as skipped.
- **Correctness of any value.** These gates prove a number responds to its
  input. They cannot prove it responds *correctly*.
