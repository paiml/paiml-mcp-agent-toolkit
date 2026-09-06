# release receipt — 3.38.0 (cut on master 5a9a82bf4; STOPPED before merge/tag/publish)

| field | value |
|---|---|
| base | `master` 5a9a82bf4 = #1196 merge (PMAT-674) on top of 97e826ae8 = #1195 merge (PMAT-673) on top of 7aff1179d = #1194 (roadmap dedupe + tickets) |
| branch / PR | `release/3.38.0` · PR #1197 · **draft, no auto-merge, no tag** |
| bump | Cargo.toml:4, Cargo.lock (pmat entry), README.md:628, mcp.json:3, CHANGELOG.md (section from the three PR bodies); PMAT-673 and PMAT-674 marked `completed` |
| tickets merged | #1195 head 1ce01bc1a: check-runs success=42 skipped=5 failure=0, merged 2026-09-05T20:58:47Z; #1196 head d9180f92e: success=43 skipped=5 failure=0, merged 2026-09-05T21:51:58Z; no reruns on either |
| gate_cmd_fallback | `true` in discover.json (`cargo test --workspace`) — recorded, not fixed, not run; `pmat verify` was the gate on both tickets |
| infra pin bump 3.37.0 → 3.38.0 | named follow-up, not done here |

## Pre-merge gates run on this branch (b233e7911)

| gate | result |
|---|---|
| `make validate-book` | PASS — 4 critical chapters (05, 07, 13, 14), Ch13 control "1 of 1 script(s) fail without a working pmat" OK; wall 0.7 s, which means it exercised an already-built pmat, not a fresh build of this tree — the book at `/home/noah/src/pmat-book` was present |
| fixture dogfood of the built tree (`cargo metadata` target dir, `pmat 3.38.0`) | `work add first` exit 0 → `PMAT-011`, lock file `11`; `work add second` exit 0 → `PMAT-012`, lock file `12` (distinct, sequential, high-water mark advanced); append a second `- id: PMAT-011` row → `work validate` exit 1: `error: duplicate id PMAT-011 at …/roadmap.yaml:21, …/roadmap.yaml:53`; an unparseable roadmap: `work validate` exit 1 and `work add` exit 1 (nothing written) |
| trap on the way | `./target/debug/pmat` inside the repo is a stale Sep-2 binary; the real target directory is off-site (`cargo metadata --format-version 1 | jq -r .target_directory`). The first fixture pass used the stale one and read "Validation passed" on a duplicated id — the evidence above is from the real one |

## Why this stops here

The instruction: the release workflow publishes; if it lacks a clean-room hard gate or has `continue-on-error` on publish, STOP and report; never `cargo publish` by hand, no `--allow-dirty`, no `--skip`.

- The only workflow that runs `cargo publish` is `.github/workflows/release.yml.disabled` (publish behind `gate` and `verify` on `[self-hosted, clean-room]`) — disabled since #382, "manual publish only". `automated-release.yml.disabled` and `auto-tag-release.yml.disabled` likewise. Nothing enabled publishes the crate.
- A `v3.38.0` tag today would trigger `docker-publish.yml` (an image of a version that is not on crates.io) and nothing else; `post-release.yml` runs only on a published GitHub release.
- Every earlier release (3.34.0 → 3.37.0) was published by hand after the tag (`env -u CARGO_REGISTRY_TOKEN cargo publish --locked`), which this run was told not to do.

Not done, deliberately: merge of #1197, tag `v3.38.0`, `gh release create`, `cargo publish`, and therefore the post-publish `cargo install pmat --version 3.38.0 --locked` dogfood (the fixture run above is on the tree's own build, not on the published crate).

## Follow-ups named

1. Re-enable `release.yml` (it carries the clean-room gates) or decide the manual path; then merge #1197, tag on the merge commit, publish, dog-food the installed crate.
2. Whole-file re-serialisation of `roadmap.yaml` on every `work add` / `work edit` (#1193 / #1169, second half).
3. Cross-checkout id collisions: the lock file is per checkout; a union-across-refs mint is not in PMAT-673.
4. One raw-text id scanner instead of two (`roadmap_service_operations.rs::id_key_value` vs `ticket_validate_migrate.rs::collect_id_lines`).
5. Infra pin bump 3.37.0 → 3.38.0.

## Verdict

**STOPPED(publish-path-absent)** at the first pass — release cut prepared on PR #1197 as a draft; nothing published, nothing tagged.

**Then SHIPPED by the operator's decision** ("merge and publish by hand"): #1197 merged 2026-09-06 (9e494f95f, required checks green on its head), tag `v3.38.0`, GitHub release, manual `cargo publish --locked` exit 0, crates.io 3.38.0, docs.rs `doc_status: true`, binary-release and post-release green, installed-crate dog-food green — see `docs/audits/release-3.38.0-dogfood.md`. — RELEASE-3.38.0-RECEIPT-END
