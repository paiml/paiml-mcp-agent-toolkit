# impl receipt — PMAT-1371 (`roadmap sync` fails closed)

ticket PMAT-1371 · issue paiml/paiml-mcp-agent-toolkit#1371 · PR #1372 · branch PMAT-1371-roadmap-sync-fail-closed · author session infra-0b (was infra-80 before the 2026-09-15 17:33 CEST reboot).

## The fix (what the title says)
`read_work_store_rows` in `src/roadmap/sync.rs` was a line-scan, and `RoadmapSources::new` de-duplicated rows by id. Now the read goes through the strict model and `check_roadmap_text`, as `work add` (PMAT-676) and `work edit` (PMAT-679) already do. The table in #1371, re-run with this branch's binary against paiml/infra's 280-row roadmap:

| mutation | 3.40.1 | this branch |
|---|---|---|
| `not: [yaml` | rc 0, 0 items | rc 1 parse error with line/column |
| duplicate id | rc 0, 280 items (deduped) | rc 1 `duplicate id PMAT-604 at …:4984, …:5055` |
| `status: shipped` | rc 0 | rc 1 `unknown status 'shipped'` |
| status deleted | rc 0, `""` | rc 1 `missing field 'status' at line 4984` |
| tab before a key | rc 0 | rc 1 |
| clean | 280 items | 280 items; render byte-identical except the 17 titles whose `''` escape the line-scan had carried literally |

## What the diff also does, and why it is in this PR
Merging master in after #1360 turned CB-2115 red for this branch, and for master too once the 60-minute grace window runs out:
- **ORPHAN-ROADMAP PMAT-1359, PMAT-1361.** #1360 closed #1359 and #1361 at 17:17Z and left both rows `planned`. The fix is `pmat work sync --direction github-to-yaml`: pmat is the only writer, the dry-run planned exactly two `close-item` actions, and the diff is those rows' `status` and `updated` lines, nothing more. Opening a separate PR would re-queue behind the same red check.
- **DRIFT PMAT-1371.** I minted this row with a shortened title, and the roadmap title is immutable (pmat#1240). The GitHub issue was renamed to match it; the repo diff has no change for this.

After both: `pmat work sync --check-only` reads coherent, and `pmat work validate --check-base origin/master` passes.

## Verification
| check | result |
|---|---|
| `cargo test --lib roadmap::sync` | 15 passed; 7 of the new tests RED before the fix |
| `pmat verify` (on 3.40.1 master) | only red: the unwrap ratchet and the unrun-tests ledger, both from this change; both settled in the fix commit |
| two ratchet tests after the master merge | ledger re-render: no change; `.unwrap()` in src = 20325 = baseline |
| CB-2113 / CB-2115 | ✓ / coherent after the sync |
