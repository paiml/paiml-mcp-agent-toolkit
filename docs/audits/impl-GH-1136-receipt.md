# Implementation receipt: GH-1136

Ruling: option (b), won't do, relayed by pmat-cop (Opus escalation). Issue #1136 allows "decide it or write the decision down".

## Changes
- `src/cli/command_wire_names.rs`: module-doc section "Why there is no MCP-exposure column (#1136, decided: won't do)". It quotes crux-audit §9.2 item 2 ("prevention, not a live wrong answer"), names the revisit triggers (`pmat tdg` first), and points at #1136 and #1029.
- Test `mcp_exposure_decision_tests::mcp_exposure_decision_is_written_down` pins the decision text. It only reads the source above its own module, so its needles cannot match themselves. It fails on the old file (no section).
- `docs/roadmaps/roadmap.yaml`: GH-1136 `planned` -> `completed`, with a notes line.
- `docs/status/unrun-tests-ledger.md`: regenerated with `pmat analyze unrun-tests --executed '' --write-ledger` (one lib test added; the line drifts as sibling PRs merge, so no figure is quoted here; the committed line is the measurement).

## Not done
- No `McpExposure` column for the 71 `Commands` variants (option a).
- Issue comment is posted after master traceability is green (CB-2115).
