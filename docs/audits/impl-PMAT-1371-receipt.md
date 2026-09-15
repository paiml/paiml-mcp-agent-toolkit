# impl receipt — PMAT-1371 (`roadmap sync` fails closed)

ticket PMAT-1371 · issue paiml/paiml-mcp-agent-toolkit#1371 · PR #1372 · branch PMAT-1371-roadmap-sync-fail-closed · author session infra-0b.

## The fix
`read_work_store_rows` (`src/roadmap/sync.rs`) was a line-scan, and `RoadmapSources::new` de-duplicated rows by id. Now the read goes through the strict model and `check_roadmap_text`, as `work add` (PMAT-676) and `work edit` (PMAT-679) already do. The table from #1371, re-run with this branch's binary against paiml/infra's 280-row roadmap:

| mutation | 3.40.1 | this branch |
|---|---|---|
| `not: [yaml` | rc 0, 0 items | rc 1, parse error with line and column |
| duplicate id | rc 0, 280 items (de-duplicated) | rc 1 `duplicate id PMAT-604 at …:4984, …:5055` |
| `status: shipped` | rc 0 | rc 1 `unknown status 'shipped'` |
| status deleted | rc 0, renders `""` | rc 1 `missing field 'status' at line 4984` |
| tab before a key | rc 0 | rc 1 |
| clean | 280 items | 280 items; render byte-identical except 17 titles whose `''` escape is now decoded |

## Claims a reviewer can check against the diff
| claim | where it is in the diff |
|---|---|
| unrun-tests ledger re-rendered for the 9 new tests | `docs/status/unrun-tests-ledger.md`: `24212 of 27339` → `24221 of 27348` (two changed lines). `the_committed_ledger_matches_the_tree` passes on HEAD. |
| `.unwrap()` ratchet unchanged | the new tests use `expect`; `git grep -oF '.unwrap()' -- 'src/*.rs' \| wc -l` = 20325 = the baseline in `.pmat-ratchet.toml`; `the_committed_ratchet_holds_at_head` passes |
| `parse_rows` kept deliberately | public API of the published 3.40.1 (`pub mod roadmap` → `pub mod sync` → `pub fn parse_rows`); removing it is a semver break a patch release must not make. Its doc says it is no longer the read path. Its two tests remain. |
| roadmap | this row's acceptance criteria only; the rest of the file is master's copy |

## Deliberately not in this diff
- PMAT-1359 and PMAT-1361 were left `planned` when #1360 closed their issues. They are completed in their own PR, since that is separate work. Until that PR lands, `traceability` (CB-2115, not a required check) reports them as ORPHAN-ROADMAP here too.
- The DRIFT between this row's title and #1371's title was fixed on GitHub by renaming the issue. The roadmap title is immutable (pmat#1240), and there is no repo change.
