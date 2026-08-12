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
project, and a ~120-file project carrying every defect family pmat claims to
detect (complexity, SATD in canonical *and* non-canonical phrasing, fault
patterns, duplication, dead code, superlinear algorithms, a multi-hop
`use crate::` dependency chain, long functions, one deliberately pathological
file, and one uncommitted Critical-risk file), plus the non-Rust inputs several
commands exist to read: WebAssembly binaries and WAT sources, AssemblyScript
files, and GGUF/APR/safetensors models including one deliberately unreadable
header. Repo hygiene is graduated alongside the code. All three are real git
repositories, committed with hooks disarmed and with dates computed *relative
to now* so every commit falls inside the default analysis windows; the large
corpus's second commit lands on a branch so `main..HEAD` is non-empty.

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

`ALLOWED_CONSTANTS` holds only leaves the corpus provably cannot vary — a fixed
checklist's length, buckets no fixture input belongs in — each naming the corpus
limitation rather than excusing pmat.

### The `ALLOWED_NOOPS` policy

**Every entry must name a demonstrated real effect**: the code path that reads
the flag, and the input or `--format` under which the flag was observed
changing something. "Legitimately inert" is not a reason. These are:

> `("analyze provability", "--include-evidence", "gates the per-property
> evidence blocks (provability_helpers_json.rs:32,
> provability_helpers_detailed.rs:61,201): `-f json --include-evidence` adds a
> \"properties\": [...] array to each function; the summary renderer has no
> per-function section")`

> `("score", "--stack", "appends the 'Stack Quality (CB-150)' block listing
> sovereign dependencies found in Cargo.toml
> (score_handler_display.rs:5-31); adding `aprender`/`trueno` to the fixture's
> Cargo.toml makes it appear. The corpus has an empty [dependencies] section,
> and the function early-returns when none are found")`

An entry that cannot name an effect is a defect being *documented* rather than
fixed — which is precisely how 49 no-op flags shipped in 3.29.0. The gate
enforces the shape of the claim (`allowed_noops_name_a_real_effect`); only
review can enforce its truth, which is why the reason is long enough to check.

Two rules follow from it:

- A flag on a `DENY_PATHS` command may not be allow-listed. Allow-listing a
  flag the sweep never runs hides it twice over.
- Preferring a harness fix to an allow-list entry is the default. Of the 51
  no-op verdicts that were not defects, 25 were corrected in the harness and
  only 26 became entries.

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
- `normalisation_keeps_decimal_digits_and_still_erases_object_ids` pins the
  rule that used to turn two different PageRank values into the same
  `0.<SHA>`. A normaliser that erases the measurement passes everything.
- `agreeing_failed_probes_are_skipped_not_booked_as_noop` pins both halves of
  the skip condition, including that a failing command which still printed a
  report remains a control.
- `allowed_noops_name_a_real_effect` enforces the shape of every escape-hatch
  entry, and the report lists allow-list entries that suppressed nothing —
  a stale exception is how an exception outlives its reason.

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

## Triage of the first full sweep (3.30.0)

The sweep's first complete run against the 3.30.0 artifact returned **140 no-op
verdicts**. Every one was reproduced by hand against a dumped corpus before
being classified. Nothing was left unexamined:

| Verdict | Count | Disposition |
|---|---|---|
| Real defect | 89 | fixed in the source; the gate must keep failing until they are |
| Harness artifact | 25 | corrected in the corpus or the probe — the flag was never given a chance |
| Legitimately inert | 26 | `ALLOWED_NOOPS`, each naming a demonstrated real effect |
| Not reached | 0 | — |

Against one and the same binary, before and after the twenty-five harness
corrections and the twenty-six allow-list entries:

```
before: 401 effective, 3 refuses-honestly, 116 no-op, 1 error-out, 267 skipped
after:  400 effective, 3 refuses-honestly,  52 no-op, 1 error-out, 237 skipped
```

Every one of those 52 is a triaged defect awaiting its source fix — none is an
artifact and none is unexplained. The gate stays red until they land, which is
the point: an `ALLOWED_NOOPS` entry for a real defect converts a caught defect
into a documented one, and that is exactly how the 49 shipped.

### Where the gate stands after the source fixes

The fixes for those 52 landed in the same release. Re-run against a binary
built from the fixed tree, on the same corpus and the same core roots:

```
after fixes: 424 effective, 3 refuses-honestly, 29 no-op, 1 error-out, 237 skipped
```

**23 of the 52 are fixed; 29 remain, and every one of the 29 is a triaged
defect.** None was allow-listed to get there — the allow-list *shrank* by one,
because `quality-gate --color` became effective and its entry was deleted
rather than left to rot (the reason it stated, "all fifteen `quality_gate_*.rs`
printers reference `crate::cli::colors` zero times", is now false: `--color
always` emits 189 ANSI-carrying lines against 0 for `--color never`). The
allow-list is therefore 25 entries, of which 2 are unexercised because their
command is skipped, not because they are stale (`comply cross-crate --color`,
`tdg history --color`).

The 29 split into two shapes, and the difference decides what "fixed" means for
each:

| Shape | Count | Flags |
|---|---|---|
| Chatter still survives `--quiet` | 6 | `analyze bottleneck`, `analyze comprehensive`, `analyze defect-prediction`, `analyze provability`, `context`, `score` |
| `--quiet` offered on a command that emits no chatter at all | 12 | `analyze big-o`, `analyze defects`, `analyze deep-context`, `analyze entropy`, `analyze models`, `comply asset-validate`, `comply audit`, `comply init`, `comply report`, `deps-audit`, `explain`, `repo-score` |
| Flag unrelated to `--quiet`, never wired to anything | 11 | `analyze dead-code --include-unreachable`, `analyze deep-context --dag-type`, `analyze deep-context --cache-strategy`, `analyze tdg --ml`, `analyze provability --analysis-depth`, `analyze incremental-coverage --detailed`, `analyze big-o --high-complexity-only`, `analyze assembly-script --wasm-complexity`, `analyze web-assembly --top-files`, `tdg check-quality --fail-on-violation`, `comply report --include-history` |

The first six are unambiguous: `analyze provability --quiet` still prints
`🔬 Analyzing function provability... / 📂 Discovering functions in project... /
📊 Found 124 source files`, byte-identical to the run without the flag. Those
lines are noise by any reading, and the fix is to route them through
`status_eprintln!`.

The middle twelve are a different claim and must not be quietly reclassified.
Their commands print a result and nothing else, so a flag that suppresses noise
correctly changes nothing — but "it prints only a result" is an assertion about
the binary, not a licence, and it had never been measured when the twelve were
first counted. Each was then measured, one command at a time, and only two
answers were accepted: **stderr is empty on the swept invocation** (so the flag
has nothing to take away, and the only stderr the command can produce is an
error, which must survive `--quiet`), or **the flag's effect exists one format
or one branch over** and was reproduced there. `comply report --output F` prints
`✓ Compliance report written to F` (43 B) and `--quiet` takes it to 0 B with the
file still written; `comply init` on an uninitialised directory goes 378 B → 101 B;
`analyze deep-context --format sarif` goes 62 B → 0 B. Those four carry a
demonstrated effect. The other eight carry a demonstrated *absence*, measured in
bytes on the corpus and named in the entry.

That refinement is the policy's boundary, and it is narrow on purpose: an entry
may state an absence only where the absence itself was measured on the corpus
the gate runs. "Legitimately inert" with nothing behind it remains the exact
move this policy exists to forbid, and the four `--quiet` flags that really did
suppress chatter — `analyze bottleneck` (40 B → 0 B), `analyze provability`
(125 B → 0 B), `context` (60 B → 0 B), `score` (55 B → 0 B) — were fixed in the
source, not allow-listed. `score` is the one to remember: it was first triaged
as "no stderr at all" on a small fixture, and only the ~120-file corpus exposed
its 55 B. **A fixture too small to make the noise is not evidence that there is
none.**

`comply audit --quiet` was the weakest of the twelve, and its entry says so
rather than hiding it: `comply audit` refuses on the corpus with "Audit requires
clean git state" and exits 1 (179 B of stderr) before reading any flag, which is
the same declined-precondition shape `baseline_unusable` already skips for
`comply cross-crate` and `tdg history`. Rather than convert the finding into a
skip — indistinguishable, on a closing pass, from making the gate green by hand
— the flag was exercised on a clean-git copy of the same corpus, where
`-f markdown` prints its banner through `status_println!` and `--quiet` drops it.
The entry records both the refusal the sweep sees and the effect it hides.

`analyze graph-metrics --export-graphml` is the single `Errors` verdict: the
flag makes a working command exit 1. It is a triaged defect too, and it does
not block the gate — only no-ops do — which is itself worth fixing, since a
flag that breaks its command is not better than one that does nothing.

**The gate is red on purpose and stays red.** It cannot be turned on as a
required check until those 29 land, and the way to turn it on is to fix them,
not to widen the escape hatch.

### Where the gate stands at the close of 3.30.0

Same binary tree, same corpus, same core roots, after the closing wave of source
fixes, twelve measured allow-list entries and two corpus corrections:

```
first full sweep: 401 effective, 3 refuses-honestly, 116 no-op, 1 error-out, 267 skipped
after harness fix: 400 effective, 3 refuses-honestly,  52 no-op, 1 error-out, 237 skipped
after source fix:  424 effective, 3 refuses-honestly,  29 no-op, 1 error-out, 237 skipped
final:             436 effective, 4 refuses-honestly,   2 no-op, 0 error-out, 240 skipped
```

`ALLOWED_NOOPS` is 37 entries, of which 2 remain unexercised because their
command is skipped rather than because they are stale (`comply cross-crate
--color`, `tdg history --color`). The `Errors` verdict is gone: `analyze
graph-metrics --export-graphml` no longer breaks its command and now scores
Effective (stdout 164 B → 12 412 B of GraphML, exit 0 in both runs).

Two of the closing round's verdicts were **the corpus's fault, not the
binary's**, and were fixed in `build_corpus` rather than allow-listed — which is
the difference between a gate that proves a flag works and a gate that files the
proof away:

| Flag | Why it read as inert | Corpus correction |
|---|---|---|
| `analyze dead-code --include-unreachable` | every dead item in the fixture was an item nothing *referenced*; unused and unreachable are different findings, and the corpus supplied only the first | three of the fifteen `dead_*` modules now trail their `return` with statements rustc reports as `unreachable_code`. `Unreachable blocks: 0 → 3` with the flag; the default run stays byte-identical, because the flag is the only thing that admits such a finding |
| `analyze web-assembly --top-files` | of four wasm inputs, two `.wat` are deliberately not reported and `broken.wasm` fails validation, so the report had exactly one row and a ranking limit had no ranking to cut | a second hand-assembled `small.wasm` (two functions, no import, one memory page). `--top-files 1` now drops a row and prints "Showing the top 1 of 2 files" |

The second one is the general lesson restated: **a limit flag needs a list
longer than the limit**, and a fixture that yields one row makes every ranking
flag in the tool look inert.

The two remaining no-ops are the last two of the six "chatter survives
`--quiet`" defects, and they are defects in the binary, not in the gate:
`analyze defect-prediction` (144 B of stderr, unchanged) and `analyze
comprehensive` (207 B, unchanged). Both write through bare `eprintln!`
(`defect_prediction_handler.rs:73,89`;
`comprehensive_analysis_handler/handler.rs:14`, `.../helpers.rs:114,147,154`)
while the `analysis_utilities` copies of the same banners already route through
`status_eprintln!` — the wave reached one surface and not its twin, which is the
failure mode "fix the rule, then check every other caller of the rule" exists to
catch. `make gate-artifact` is wired into `pre-release-checks` (step 7, hard
failure) and stays red until they land.

Gate A is red for reasons of its own, unchanged by any of the above and verified
identical against the pre-change corpus generator:

```
summary: 2 inert command(s), 54 constant leaf/leaves, 19 skipped
```

`comply report` and `comply review` emit no leaf that responds to any corpus,
and 54 numeric leaves are identical for an empty project and a defect-rich one.
Each is either an unmeasured value or configuration, and none has been triaged
yet; `ALLOWED_CONSTANTS` stands at 25 entries. Until that triage happens,
`make gate-artifact` cannot go green on Gate A alone.

### One cause behind forty flags: `--quiet`

**40 of the 89 defects are the same defect.** `--quiet` was reported inert on
`analyze big-o`, `bottleneck`, `churn`, `clippy`, `complexity`,
`comprehensive`, `coverage-improve`, `dag`, `dead-code`, `deep-context`,
`defect-prediction`, `defects`, `duplicates`, `entropy`, `graph-metrics`,
`incremental-coverage`, `lint-hotspot`, `proof-annotations`, `provability`,
`satd`, `symbol-table`, `tdg`, on all eleven `comply` leaves, and on `context`,
`deps-audit`, `enforce extreme`, `explain`, `quality-gate`, `repo-score` and
`score`.

There are not forty bugs. `apply_ux_settings` sets `PMAT_QUIET=1`
(`src/cli/cli_run_command.rs:61`) and exactly one place in the tree reads it:
`should_show_progress()` (`src/cli/progress.rs:124`). Every handler that writes
its chatter with a bare `eprintln!` rather than through `ProgressIndicator` is
therefore unaffected by the flag, and nearly all of them do. One unread
environment variable, forty flags that parse and change nothing.

This is the shape to look for first in any batch of no-op verdicts, and the
reason the triage groups verdicts by cause before counting them: forty
individually-filed tickets would have produced forty local patches around a
single missing consumer. **Fix the rule, then check every other caller of the
rule** — a contradiction fixed on one surface is a contradiction recreated.

### The 25 harness artifacts, by class

| Class | Flags | Correction |
|---|---|---|
| The corpus lacks the input the flag acts on | `analyze models --check`; every `web-assembly` / `assembly-script` flag | `build_corpus` now writes `mod.wasm`/`broken.wasm`/two `.wat` sources, three AssemblyScript files, and four model files (valid GGUF/APR/safetensors plus one unparseable `.gguf`, with no model card or tokenizer so all three `--check` findings fire) |
| The corpus lacks the *structure* the flag measures | `analyze dag --max-depth`, `analyze satd --strict`, `analyze comprehensive --confidence-threshold` / `--min-lines`, `analyze incremental-coverage --top-files` | a six-module `use crate::` chain (the corpus graph was a star, so there was no second hop for a depth limit to cut); non-canonical debt phrasing (strict mode had nothing to narrow); one uncommitted ~985-line Critical-risk file (no High/Critical file meant no "Focus on N high-risk files" line to filter); the second commit now lands on a branch so `main..HEAD` is non-empty |
| The probe values do not straddle the corpus | `analyze dead-code --max-percentage`, `analyze graph-metrics --convergence-threshold`, `analyze comprehensive --min-lines` | `PROBE_VALUES` names the pair per flag (1.0 vs 50 against a 3.8% density; 1e-12 vs 0.5 for a tolerance defaulting to 0.001); line counts probe 1 vs 1000 |
| The flag only exists alongside another flag | `analyze dead-code --fail-on-violation`, `project-diag --quiet`, `comply report --color` | `PROBE_CONTEXT` carries the companion (`--max-percentage 1.0`, `--output FILE`, `--format text`) into **both** the probe and its control |
| The baseline was already broken | `analyze coverage-improve --color` / `--fast` / `--format`, `comply enforce --color` / `--disable` / `--format`, `comply cross-crate` ×4, `tdg dashboard --color` / `--open`, `tdg history --format`, `analyze clippy --dry-run` | the fixture Makefile gained a `coverage` target and `git init --template=` no longer leaves `.git/hooks` missing; `baseline_unusable` now recognises feature-gated subcommands, declined preconditions that exit 0, and failures inside an external tool |
| The harness's own normalisation ate the effect | `analyze graph-metrics --convergence-threshold` | `\b[0-9a-f]{7,40}\b` matched the fractional digits of a float, so two different PageRank values both normalised to `0.<SHA>`. Object-id replacement now requires an `a`-`f` character |
| The sweep would have mutated its own fixture | `analyze clippy`, `comply enforce` | `DENY_PATHS`: `analyze clippy` runs `clippy --fix` and rewrites source; `comply enforce` installs hooks and auto-proceeds when stdin is not a tty, so `--yes` being denied does not stop it |

A twenty-sixth was found by the corrected sweep itself: `analyze satd
--include-tests` changed nothing because the corpus's only test file was
clean — a flag whose job is to *include test files* cannot be observed until
they contain something to include. The test file now carries its own debt.

Two of these deserve restating, because they are the two ways a gate lies:

- **An already-broken baseline blames the flags.** Eleven of the twenty-five
  were a command that died in `make`, in git, or in a missing cargo feature
  before it read any flag of its own. A control that never ran is not a
  control, and the correct verdict is `Skipped` — printed, never a pass.
- **A fixture that cannot express the flag's domain manufactures no-ops.** A
  depth limit on a graph with no depth, a strict mode over a corpus with only
  canonical markers, and a limit over an empty result set all produce
  byte-identical output for the same reason: the fixture, not the flag.

And one correction that had to be *narrowed* immediately after it was made: the
new "both probe values failed, so this proves nothing" skip originally keyed on
exit code alone. A non-zero exit is the normal outcome for the commands this
gate cares most about — `analyze lint-hotspot`, `quality-gate` and `enforce
extreme` all exit 1 on the defect-rich corpus while printing a full report — so
that rule silently excused every no-op flag on every gate that was doing its
job, and `analyze lint-hotspot --color` went from a reported no-op to a skip in
one run. The skip now also requires that both runs rendered **no stdout at
all**. Every widening of a skip condition is a narrowing of what the gate
proves.

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
   without. *Fixed*: `analyze complexity --quiet` now scores `Effective`, along
   with 22 of the other 39 commands that carried the same defect. The 18 that
   remain are itemised above.
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
  `hooks`, `agent`, `serve` and others are denied by name in `DENY_ROOTS`, and
  two individual leaves (`analyze clippy`, `comply enforce`) in `DENY_PATHS`,
  each with a stated reason. They are excluded because the harness must be
  safe to run anywhere and must not edit the fixture the rest of the sweep
  reads, not because they are exempt from the invariant.
- **Options taking unenumerated values** (paths, integers) — skipped, and
  reported as skipped.
- **Correctness of any value.** These gates prove a number responds to its
  input. They cannot prove it responds *correctly*.
