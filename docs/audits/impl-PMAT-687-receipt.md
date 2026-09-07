# impl receipt — PMAT-687 [train/FG]: fleet gate admissibility for a pmat tag

**Ticket:** PMAT-687 (kind:code, carried from 3.39.0) · **Branch:** `PMAT-687-fleet-gate` · **Orchestrator:** Fable (direct; workflow edits are orchestrator-only). Paired change in the fleet: paiml/.github#65.

## Mechanism (five-whys terminal)
`release.yml` could never go green on a pmat tag: `gate / lint-gate`'s "Banned path scan" greps every tracked `*.rs *.toml *.sh` for four workstation prefixes, and pmat's own banned-path analyzer (`src/services/hardcoded_paths.rs`) names those strings in its recognisers and test fixtures (14 lines), plus one comment in `check.rs` quoted an absolute checkout path. PMAT-686 scrubbed 17 other files at 3.39.0 and pinned these two as debt because editing the analyzer trips `pmat verify`'s complexity gate on its pre-existing `classify` (cognitive 33 > 25). So 3.39.0 shipped `clean_room=local-modeA` with a prerelease created by hand — a producer gating itself.

## Fix
- **Fleet side (paiml/.github#65):** the per-repo `EXCLUDE` for `paiml-mcp-agent-toolkit` gains exactly `src/services/hardcoded_paths\.rs` — the file whose purpose is recognising those strings. Every other shipped file is still scanned.
- **Repo side (this branch):** `check.rs:693`'s comment no longer quotes a workstation path; the in-repo mirror of the scan (`src/tests/fleet_banned_paths_tests.rs`, PMAT-686) models the same one-file exclusion (`FLEET_EXCLUDED`) and pins **no** debt — the tree must be clean and the excluded file must still exist.

## RED → GREEN
- RED 039217d74: mirror test with the exclusion modelled and the pins removed fails on exactly one line: `src/cli/handlers/comply_handlers/check_handlers/check.rs:693: /home/<user>` (the literal, redacted here).
- GREEN 75e2e128e: `cargo test --lib -- fleet_banned_path_scan_is_clean` → 1 passed. `TMPDIR=/tmp/tmpx pmat verify --skip tests --format json` → ok=true (format, complexity, satd, clippy all measured true; tests ran separately above).
- Discrimination: the RED named the comment line only, not the analyzer file — the exclusion is doing exactly its one job.
- Anti-vacuity (the exclusion must not blind the scan): PO-G1's mirror still scans every other tracked file; plant a banned prefix in any `src/*.rs` outside the excluded file and the test names it (the RED commit is that observation on check.rs).

## Workflow probe (PO-G3)
Runs after both PRs merge: `gh workflow run release.yml -f tag=v3.39.0 -f probe_fail_verify=true` → expected `gate / lint-gate` green, `verify` red, no release for the probe. Recorded in the 3.40.0 release receipt (gate table, `clean_room=`).

Orchestrator turns: 5

verdict: GREEN

IMPL-PMAT-687-RECEIPT-END
