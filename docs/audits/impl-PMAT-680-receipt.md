# impl receipt — PMAT-680 (single-authority id mint across checkouts)

| field | value |
|---|---|
| ticket | PMAT-680 · kind=code · landed on the urgent #1200 branch `PMAT-685-cot-derive-hollow` / PR #1201 (operator: "same PR") |
| worker | `paiml-impl-worker` opus `ae3faa842f35b986f` — 45 tool uses, no resume |
| RED commit / GREEN commit | ec54168d1 / e79f0014e (+ 384dd1a00 ledger) |
| acceptance (worker claimed → orchestrator rerun) | exit 0 → exit 0 (`env -u RUST_MIN_STACK cargo test --lib -- work_add_single_authority work_add_append_only work_add_refuses_invalid work_add_allocator roadmap_text roadmap_service roadmap_id_authority`; 85 tests across every suite the branch touches, plus `the_committed_ratchet_holds_at_head`) |
| mutation (discrimination) | worker: the every-ref term dropped from the mint → work_add_single_authority_counts_ids_on_other_refs RED; the two-checkout, common-dir, fallback, allocator (13) and append-only (4) tests GREEN. RED before the fix: two checkouts × 3 rounds minted [PMAT-011, 012, 013, 011, 012, 013] — 3 distinct of 6; ids on other refs ignored; no shared lock file. |
| pv contract | `contracts/work/PMAT-680.yaml` — `pv validate` valid, `pv lint` PASS |
| quorum | folded into the PR-level quorum on #1201's final diff (see release receipt); no per-ticket quorum |

## What changed
- `src/services/roadmap_id_authority.rs` — `IdAuthority::discover` (git common dir → `<common>/pmat/roadmap-id.lock`, shared by every worktree; sibling lock outside git), `max_id_across_refs` (every `refs/heads` + `refs/remotes` roadmap blob, deduplicated by blob id, plus HEAD).
- Readers and writers of every checkout take the same lock; `next = max(raw text, shared high-water mark, every ref) + 1`.
- Test S2: two worktrees, three concurrent rounds → six distinct ids, each checkout's file == old + its own appended blocks.

## Gaps / findings
- `IdAuthority::discover` runs one `git rev-parse` per lock acquisition (every `load()` too); no cache — follow-up if `work list` latency on large repos matters.
- Two separate CLONES still race between fetches (the authority is the union of refs this clone knows); a server-side counter is not in scope.

verdict: DONE pending the green train's required checks on PR #1201.

IMPL-PMAT-680-RECEIPT-END
