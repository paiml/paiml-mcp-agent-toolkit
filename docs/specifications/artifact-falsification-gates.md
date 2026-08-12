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

Verdicts are `Effective`, `NoOp`, `Errors` (the flag breaks a working command),
and `Skipped` with a reason. **Skips are printed, never counted as passes.**

## Gate A — differential corpus

```
for every numeric leaf L in a command's JSON output:
    L(empty) , L(tiny) , L(large)  must not all be equal
```

Three corpora are generated: an empty-but-valid project, a one-function
project, and a 107-file project carrying every defect family pmat claims to
detect (complexity, SATD, fault patterns, duplication, dead code, long
functions). All three are real git repositories with pinned author and
committer dates — several defects only appear outside a repository, and
unpinned dates make churn metrics irreproducible.

Array *lengths* count as leaves. A `files[]` of identical length for an empty
and a large project is the same defect wearing a different hat. Array
*elements* are not compared, because index 0 of two different projects is not
the same thing.

Configuration echoed back into the output (thresholds, targets, schema
versions, elapsed time) is excused by key name. That exclusion list is
deliberately narrow — every fragment added to it shrinks what the gate proves.

## The escape hatches

Each gate has exactly one: `ALLOWED_NOOPS` and `ALLOWED_CONSTANTS`. Both are
empty at introduction. Every entry is a `(command, item, reason)` triple, and
the reason is a claim someone must defend in review. Adding an entry is the
only way for a violation to pass, and it is visible in the diff.

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
conclusion is only as good as the input. During bring-up, **six of the first
findings were faults in the fixture, not in pmat** — each looked exactly like a
real defect until it was reproduced by hand.

| Symptom | Actual cause |
|---|---|
| `analyze churn` reports `total_commits: 0` | `init.templateDir` copied the user's pre-commit hook into every `git init`; the hook ran, failed, and aborted every corpus commit |
| `analyze churn` *still* zero after commits exist | commit dates were pinned to a fixed calendar date seven months in the past, outside churn's 30-day default window |
| `analyze dead-code` reports zero on 15 dead files | dead functions were shorter than `--min-dead-lines` (default 10) and filtered out |
| `analyze dead-code` reports zero on *long* dead files | the harness ran under `cargo test`, so the nested `cargo check` that dead-code shells out to inherited the parent jobserver and produced no diagnostics |
| `repo-score` constant across all corpora | all three corpora shipped an identical README and Cargo.toml, so repo-hygiene scoring had no axis to vary along |
| `analyze defects` reports `high/medium/low = 0` | truthful — every defect rule emits `critical`, and `critical` did vary |

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
