# impl receipt — PMAT-679 (roadmap re-serialisation: work add appends, work edit patches one block)

| field | value |
|---|---|
| ticket | PMAT-679 · kind=code · landed on the urgent #1200 branch `PMAT-685-cot-derive-hollow` / PR #1201 (operator: "same PR") |
| worker | `paiml-impl-worker` opus `abfc9dff933413c84` — 37 tool uses, no resume |
| RED commit / GREEN commit | ef4753cfc / b14fbc6e1 (+ f21fd9956 ledger) |
| acceptance (worker claimed → orchestrator rerun) | exit 0 → exit 0 (`env -u RUST_MIN_STACK cargo test --lib -- work_add_append_only work_add_refuses_invalid work_add_allocator work_validate_duplicate roadmap_text roadmap_service`; 85 tests across every suite the branch touches, plus `the_committed_ratchet_holds_at_head`) |
| mutation (discrimination) | worker: add routed back through write_roadmap_unlocked → work_add_append_only_add_appends_the_row_and_rewrites_nothing RED; the other three append-only tests, work_add_refuses_invalid_* and work_add_allocator_* GREEN. Orchestrator re-run: see the release receipt's mutation table. |
| pv contract | `contracts/work/PMAT-679.yaml` — `pv validate` valid, `pv lint` PASS |
| quorum | folded into the PR-level quorum on #1201's final diff (see release receipt); no per-ticket quorum |

## What changed
- `roadmap_text::{render_item_block, row_indent, append_item, replace_item_block}` — pure text operations.
- `add_item_with_next_id` writes `raw + block` (the strict parse and the validator stay as pre-checks); `RoadmapService::replace_item_raw` + `handle_work_edit` replace exactly one row's block.
- Test A1: a fixture with a comment, an unknown key, a flow-style row, a block scalar and a trailing comment — `new == old + block`, changed lines == the block's lines.

## Gaps / findings
- `work start` / `work complete` / `work delete` / `work sync` still round-trip the whole file (`upsert_item`, `remove_item`, `save`) — follow-up ticket; the 2,532-line class is closed for `add` and `edit` only.
- `work edit` re-renders the edited row from the model: an unknown key ON THAT ROW is dropped (only untouched rows are byte-identical) — by design, recorded.
- A2 (`roadmap: []` opens into a block sequence) passed before and after: a control, not a RED test.

verdict: DONE pending the green train's required checks on PR #1201.

IMPL-PMAT-679-RECEIPT-END
