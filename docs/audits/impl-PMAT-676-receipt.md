# impl receipt — PMAT-676 (work add and work edit refuse what work validate rejects)

| field | value |
|---|---|
| ticket | PMAT-676 · kind=code · landed on the urgent #1200 branch `PMAT-685-cot-derive-hollow` / PR #1201 (operator: "same PR") |
| worker | `paiml-impl-worker` opus `a60a9cae3f55d9971` — 40 turns + one resume |
| RED commit / GREEN commit | 6db203fd3 / 4e71c7862 (+ 90bc21747 ledger, be7745abf contract) |
| acceptance (worker claimed → orchestrator rerun) | exit 0 → exit 0 (`env -u RUST_MIN_STACK cargo test --lib -- work_add_refuses_invalid work_add_allocator work_validate_duplicate roadmap_text`; 85 tests across every suite the branch touches, plus `the_committed_ratchet_holds_at_head`) |
| mutation (discrimination) | worker: check_roadmap_text removed from add_item_with_next_id → work_add_refuses_invalid_add_refuses_a_duplicated_id_and_writes_nothing RED, all 11 work_validate_duplicate_* + the edit test GREEN. Orchestrator (worktree, first call site = the edit path): work_add_refuses_invalid_edit_refuses_a_duplicated_id_and_writes_nothing RED, 28 GREEN — the two write paths are pinned independently. |
| pv contract | `contracts/work/PMAT-676.yaml` — `pv validate` valid, `pv lint` PASS |
| quorum | folded into the PR-level quorum on #1201's final diff (see release receipt); no per-ticket quorum |

## What changed
- `src/services/roadmap_text.rs` — the ONE scanner (`id_lines`, `duplicate_ids`, `next_id_number`) and ONE validator (`check_roadmap_text`), renders validate's wording verbatim.
- `add_item_with_next_id` and the edit save path call the validator under the write lock before any write; `id_key_value` and the allocator's private scanner deleted.
- Two behaviour changes pinned by `work_add_refuses_invalid_one_scanner_settles_both_readings`: a flow-style row now counts for the allocator (was a false low); an id quoted inside a block-scalar body no longer counts.

## Gaps / findings
- `work start` / `work complete` / `work sync` still save through `upsert_item`/`save` without the text check (named by the worker; follow-up).
- `next_id_number` is re-exported under its old path for one `#[cfg(test)]` module outside the worker's scope.

verdict: DONE pending the green train's required checks on PR #1201.

IMPL-PMAT-676-RECEIPT-END
