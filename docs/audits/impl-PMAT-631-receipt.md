# Implementation receipt — PMAT-631 (CRUX-09, #1150)

## Identity

| field | value |
|---|---|
| ticket | PMAT-631 |
| spec | docs/specifications/pmat-architecture-crux-audit.md §8.9 (CRUX-09) |
| branch | PMAT-631-mcp-json-schema-fidelity |
| HEAD at receipt | 82c97e83f |
| discover.json sha256 (16) | e6da70485e91900c |
| gate_cmd | make gate-artifact (discovered after PMAT-632; `gate_cmd_fallback=false`) |
| phase gate | pmat verify |
| required checks | ci / gate, docs build (docs.rs environment), feature-gate, pmat score, provable ladder |

## Plan and routing

| phase | acceptance command | mode | trigger |
|---|---|---|---|
| P1 render inputSchema from handler metadata; delete `tool_schema()` | `cargo test --lib manifest_schemas_match_handler_metadata` | direct | - |
| P2 regenerate mcp.json; pin byte-for-byte | `cargo test --lib committed_mcp_json_is_pinned_to_the_renderer` | direct | - |
| P3 pv contract same PR; DoD gate | `make gate-artifact`; spec §8.9 script | direct | - |

Quorum: never (`--quorum never`). Routing direct because `|M|=1` (`src/mcp_pmcp/tool_manifest.rs`) and the
change was fully specified; no subagent was dispatched for this ticket.

## Dispatch ledger

| phase | mode | agent | turns | maxTurns hit | resumed |
|---|---|---|---|---|---|
| P1–P3 | direct | orchestrator | see estimates | - | - |

## Verification — claimed vs re-run (all runs are the orchestrator's own)

| check | before (master, 47d643272) | after (this branch) |
|---|---|---|
| spec §8.9 script, 13 legs | `FAIL: A1: 19/19 shipped inputSchemas differ from tools/list` | `PASS: A1 A2 B1 B3 D1 D1b green; controls P A3 B2 C D2 D3 green` |
| `manifest_schemas_match_handler_metadata` | n/a (new) | ok |
| `committed_mcp_json_is_pinned_to_the_renderer` | RED before regenerate | ok after regenerate |
| named mutation: canned `paths` schema restored for `pmat_index_stats` in mcp.json | — | pin test FAILED; restored → ok |
| `cargo test --lib -- tool_manifest` | — | 12 passed, 0 failed, 1 ignored |
| `cargo fmt --all -- --check` | — | clean |
| `cargo clippy --lib --bins` | — | no warnings |
| `pmat verify --skip clippy,tests` | — | ok, 3 stages measured |
| unwrap ratchet (`git grep -oF '.unwrap()' -- 'src/*.rs' \| wc -l`) | 20343 | 20343 |
| unrun-tests ledger | — | regenerated for the 3 new lib tests |
| `pmat verify` (full: clippy + tests) | — | PENDING |
| `make gate-artifact` (DoD) | — | PENDING |

## pv contract

`contracts/macs-artifacts-v1.yaml` 1.0.0 → 1.1.0: equation `manifest_schema_faithful`, falsifiers
`manifest_schema_faithful` → `manifest_schemas_match_handler_metadata` and `manifest_pinned` →
`committed_mcp_json_is_pinned_to_the_renderer`. pv_lane = contract present, same PR.

## Jidoka log (.pmat/jidoka.jsonl rows for this ticket)

- PMAT-632: discover.sh probed only `make gate` and fell back to `cargo test --workspace` on a repo
  with `gate-artifact`; patched (enumerate gate-like targets; `--gate <target>` on ambiguity).
- orchestrator: `git reset --hard origin/master` discarded the uncommitted roadmap.yaml edit holding
  the freshly created ticket; re-added. `git reset --hard`, `git checkout -- .`, `git clean -f*`,
  `git stash drop` are forbidden for worker and orchestrator for the rest of this engagement.
- PMAT-632 (scope): the skill doc describes an edit/deny hook that is not installed; enforcement is
  orchestrator discipline plus the transcript gate.
- PMAT-633: binary-size growth has no per-crate attribution (five-whys attached below).
- PMAT-635: `pmat work add` allocates ids from the checkout's tracked roadmap.yaml, so a worktree at
  the branch base minted PMAT-631 a second time. [A] Tickets PMAT-634 (CRUX-05) and PMAT-635 are
  committed on THIS branch so master carries them first; the CRUX-05 PR references PMAT-634 without
  re-adding it, which avoids a duplicate entry at merge without hand-editing the YAML.

## Estimates

| K̂ | basis | K (budget) | actual turns (this ticket) |
|---|---|---|---|
| 3 | first-run[U] (`estimate.sh`, 0 rows) | 120 | recorded in .pmat/estimates.jsonl |

## Gaps

- `pmat verify` full and `make gate-artifact` were still running when this receipt was first
  written; the row above is updated when they conclude. A NotRun lane is a gap, not a pass.
- The migration of the 13 hand-rolled `json!` schema blocks onto `mcp_tool_schemas/` is split out
  per the spec (separate M); not part of this ticket.
- Known and cited: `refactor.*` is simulated (EV-0), nothing here depends on it; the protocol served
  is 2024-11-05 (EV-1), unchanged.

## Decisions taken conservatively [A]

- [A] The manifest keeps `LIVE_MCP_TOOLS` as the ordering/description source and takes only the
  inputSchema from handler metadata, rather than deriving names too — the smaller change; the
  positional test already pins order.

## Transcript gate

`transcript-gate.sh`: PASS, vacuous — it found 0 subagents to check because none were dispatched for this ticket (all phases direct). Stated here rather than counted as evidence.

## Binary growth — five whys (CRUX-05 build 55,411,848 B vs band centre 55,000,000)

Measured with `cargo bloat --release --crates` (unstripped, so absolute sizes differ from the
stripped artefact; deltas are what matter):

| tree | .text | pmat | pmcp | clap_builder | std | reqwest | h2 |
|---|---|---|---|---|---|---|---|
| ece3baf11 (2026-08-15) | 34.0 MiB | 17.2 MiB | 1.2 MiB | 437.8 KiB | 2.0 MiB | 401.5 KiB | 366.2 KiB |
| CRUX-05 tree (master + clap features) | 37.5 MiB | 19.1 MiB | 1.9 MiB | 523.0 KiB | 2.3 MiB | 475.2 KiB | 483.8 KiB |
| delta | **+3.5 MiB** | +1.9 MiB | +0.7 MiB | +85 KiB | +0.3 MiB | +74 KiB | +118 KiB |

1. **Why is the CRUX-05 binary above the band centre?** It is 778 KB above the audit's 54,633,288 B
   (01fba4f65): ~85 KiB is clap's `usage`/`error-context`/`suggestions` (the fix itself), the rest is
   the pmcp 2.17→2.19 bump merged the same day (#1113, +0.7 MiB .text). Still inside the ±5 % band
   (52.25–57.75 MB); the band is not adjusted.
2. **Why did .text grow 3.5 MiB since 08-15?** pmat's own code is +1.9 MiB: the 3.32–3.35 gates and
   analyzers (reachability's third state, comply ratchet/coherence, falsification gates, MCP
   hardening). Dependencies account for the remaining ~1.6 MiB (pmcp, reqwest 0.13, h2, tokio, std).
3. **Why was "42 MB" believed?** `.pmat-metrics.toml:45`'s `binary_max_bytes = 50_000_000 # (current:
   42 MB)` was written 2025-11-23 (b8a97ca6f) and is read by nothing (CLAUDE.md: recorded, not
   enforced). The first REAL CI measurement was 54,284,232 B on 2026-08-25 (#1079), which is where
   the 55,000,000 band centre came from. There was no 13 MB jump; there was a comment.
4. **Why does nothing show per-crate growth?** No gate runs `cargo bloat`; the band sees one number.
5. **Root cause:** size is gated as a scalar with no attribution, so growth is neither prevented nor
   explained until someone runs bloat by hand. Filed as PMAT-633: `cargo bloat --crates` in nightly
   with a per-crate baseline that may only fall, beside the band.

## Verdict

PENDING — becomes DONE when the two pending rows read PASS and #1158 is merged green on the
required checks without a rerun.
