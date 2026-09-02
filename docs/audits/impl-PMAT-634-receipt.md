# Implementation receipt — PMAT-634 (CRUX-05, #1148)

## Identity

| field | value |
|---|---|
| ticket | PMAT-634 (allocated on master via #1158; PMAT-635 records why it is not re-added here) |
| spec | docs/specifications/pmat-architecture-crux-audit.md §8.5 (CRUX-05) |
| branch | fix/crux-05-clap-usage-error-context |
| PR | #1160 |
| gate_cmd | make gate-artifact |
| phase gate | spec §8.5 acceptance script (`scripts/cli-usage-audit.sh`), `cargo test --lib -- docs_enforcement clap_command_structure` |

## Plan and routing

| phase | acceptance command | mode | trigger |
|---|---|---|---|
| P1 add clap `usage`, `error-context`, `suggestions`; harden 14 `contains("Usage:")` guards | `bash scripts/cli-usage-audit.sh` against the release binary | worker: agy (single, in a worktree; `--quorum never`) | - |
| P2 checker regression test + contract | `cargo test --lib an_empty_usage_heading_is_not_a_usage_section` | direct | - |
| P3 DoD gate | `make gate-artifact` | direct | - |

## Dispatch ledger

| phase | mode | agent | outcome |
|---|---|---|---|
| P1 | agy worker (worktree, own target dir) | agy | edited 11 files, then **crashed** before verifying (two crash logs under ~/.gemini/antigravity-cli/crashes, empty stdout/stderr) — no receipt returned; treated as `partial=true` |
| P1 finish, P2, P3 | direct | orchestrator | regex corrected (`\s+` spanned the newline; `[ \t]+\S` on one line), 3 `unwrap()` → `expect()`, contract written, gates run |

## Verification — claimed vs re-run (every row is the orchestrator's own run)

| check | worker claim | orchestrator |
|---|---|---|
| spec §8.5 script, unfixed binary | — | `FAIL: leg 1: root Usage line` |
| spec §8.5 script, fixed binary | — | `ALL LEGS PASS` (controls 3b 4c 4d 6b 7c green) |
| `an_empty_usage_heading_is_not_a_usage_section` | — | FAILED with agy's `\s+` regex (matched `Options` across the newline); **ok** after the one-line fix; checker module 19/19 |
| bare `contains("Usage:")` predicates in src/ + tests/ | — | 0 (was 14) |
| `cargo fmt --all -- --check` | — | clean |
| `cargo clippy --lib --bins` | — | no warnings |
| unwrap count (`git grep -oF '.unwrap()' -- 'src/*.rs' \| wc -l`) | — | 20346 after agy → **20343** = baseline after `expect()` |
| binary size | — | 55,411,848 B, inside the ±5 % band (band untouched; five-whys below) |
| `make gate-artifact` (DoD) | — | **PASS** — "artifact falsification gates passed" at 78e796d3d |
| unrun-tests ledger | — | regenerated for the new checker test |

## pv contract

`contracts/cli-usage-lines-v1.yaml` (new): `usage_line_present`, `error_names_token`, `no_blind_usage_guard`; falsifiers name the checker test and `scripts/cli-usage-audit.sh`.

## Named mutation, observed RED

Reverting the three Cargo.toml features IS the unfixed binary: `FAIL: leg 1`. The checker's negative fixture (`Usage: \n\nOptions:`) is the mutation for the guard rewrite and was RED on agy's first regex.

## Binary growth — five whys (band not adjusted)

`cargo bloat --release --crates` (unstripped): clap_builder 437.8 → 523.0 KiB — **+85 KiB is this fix**; pmcp 1.2 → 1.9 MiB is #1113 (pmcp 2.17→2.19, merged the same day); since 2026-08-15 `.text` grew 3.5 MiB, 1.9 MiB of it pmat's own code (3.32–3.35 gates and analyzers). The "42 MB" in `.pmat-metrics.toml` is a 2025-11-23 comment nothing reads; the first real CI measurement was 54,284,232 B (#1079). Root cause: size is gated as one scalar with no attribution — filed PMAT-633 (per-crate ratchet).

## Jidoka log

- agy crashed mid-ticket with no receipt; the orchestrator re-ran every claim rather than assuming any.
- agy's regex used `\s+`, which spans newlines, so an empty heading followed by `Options:` matched; its own negative test caught it (kept).
- three `unwrap()` on a static regex moved the unwrap ratchet +3; replaced with `expect()`.
- ci / provenance and ci / security RED at actions/checkout (HTTP 400) — runner-side, paiml/.github#57; not rerun.

## Gaps

- `pmat verify` full was not run on this branch; the same CB-200 lib-test drift (PMAT-636) would report on it as on #1158. Recorded, not hidden.
- Shell completions (`clap_complete`) are out of scope per the spec's dropped list.

## Decisions taken conservatively [A]

- [A] The worker's edits were kept and corrected in place rather than redone; every claim was re-executed.
- [A] The ticket id was allocated on master (PMAT-634 via #1158) and only referenced here, to avoid a duplicate roadmap entry at merge (PMAT-635).

## Transcript gate

One subagent (agy) ran for this ticket, in a worktree, alone. No overlapping intervals.

## Verdict

PENDING → DONE when #1160 merges green on the required checks without a rerun.
