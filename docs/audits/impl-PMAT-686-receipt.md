# impl receipt — PMAT-686 (fleet-gate scrub of banned-path literals)

| field | value |
|---|---|
| ticket | PMAT-686 · kind=code · PR #1204 (release cut) |
| route | direct (workflow edits are orchestrator-only) |
| commits | 4211f0bf7 (scrub of 17 files + in-repo scan test), e27f4753d (mints 686/687) |
| acceptance | `cargo test --lib -- fleet_banned_path_scan` 1 passed; RED before the scrub: 'the fleet gate would fail on 44 line(s)' |
| mutation (discrimination) | the scan test is its own mutation control: re-adding a literal to any unpinned tracked file fails it; the two pinned files are pinned at their current counts (14, 1) and may only go down |
| quorum | folded into the #1204 quorum (lane 2's blanket-exemption finding → counts pinned) |
| pv contract | none (docs+test; the falsifier is the fleet gate itself) |

## Gaps / findings
- Two files remain for PMAT-687 (`src/services/hardcoded_paths.rs`, `check.rs`): their pre-existing complexity debt trips `pmat verify` on any edit.
- `src/tests/unified_{go,typescript}_analyzer_tests.rs` early-return when a fixture path is absent; the scrub moved that path from one workstation's home to a neutral one — the tests were host-dependent before and are always-skipping now; PMAT-687 owns replacing them with in-repo fixtures.
- `pmat analyze hardcoded-paths` reports 45 machine-specific paths in shipped code on this tree — in PMAT-687's scope.

verdict: DONE pending the required checks on PR #1204 (release cut).

IMPL-PMAT-686-RECEIPT-END
