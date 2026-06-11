# Autonomous Verify Loop (`pmat verify`)

**Audience**: autonomous coding agents (e.g. Fable 5 in autonomous mode).
**Rule**: never commit without a green `pmat verify`.

## The loop

In autonomous mode there is no human to catch a clippy/test failure before a
push. The pre-commit hook and `pmat quality-gate` both miss **clippy** and
**tests** — exactly the checks CI blocks on — so passing them is *not* enough.
`pmat verify` runs the CI-faithful set and gives a **"green here ⇒ green in CI"**
guarantee.

```
edit → pmat verify --format json → (red? read violations, fix) → repeat → commit on green
```

## How to call it

| Situation | Command |
|-----------|---------|
| Default agent check (machine-readable) | `pmat verify --format json` |
| Auto-fix the trivial stuff first | `pmat verify --fix` then `pmat verify` |
| Doc-only / non-Rust change | `pmat verify --skip clippy,tests` |
| Iterating on one failing gate | `pmat verify --stage clippy` |
| Full report (don't stop at first failure) | `pmat verify --no-fail-fast` |

## Reading the JSON

```json
{ "ok": false, "stages": [
  { "name": "clippy", "ok": false,
    "violations": [ {"file":"src/x.rs","line":230,"rule":"clippy::nonminimal_bool","message":"..."} ] } ] }
```

- Top-level `ok: true` ⇒ safe to commit. `ok: false` ⇒ do **not** commit.
- For `clippy`, each `violations[]` entry is `file:line:rule` — go fix that
  exact site. For `format`/`complexity`/`satd`/`tests`, read the stage `detail`
  (tail of the tool output).

## Stages (fail-fast order)

1. **format** — `cargo fmt --check` (sub-second). `--fix` re-formats.
2. **complexity** — pmat's analyzer on *changed* files (cyclomatic ≤ 30,
   cognitive ≤ 25), matching the incremental pre-commit gate.
3. **satd** — `pmat analyze satd --strict` (no self-admitted technical debt).
4. **clippy** — `cargo clippy --lib --bins -- -D warnings` (CI-faithful;
   whole-crate; the dominant cost — but you only pay it once stages 1–3 pass).
5. **tests** — `cargo test --lib`.

## Why this is the pmat paradigm for autonomy

The bottleneck in autonomous operation is the **verify loop**, not the model.
One command, CI-faithful, machine-readable, fail-fast — so an agent self-verifies
in seconds for the common cases and never wastes an ~11-minute CI cycle on a
trivial `clippy`/format failure. See
`docs/specifications/pmat-verify-autonomous-preflight.md`.
