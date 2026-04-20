# KAIZEN-0008: Multi-persona Five Whys (MAR-style)

**Source paper:** arxiv:2512.20845 — "MAR: Multi-Agent Reflexion Improves Reasoning Abilities in LLMs" + arxiv:2602.09937 — "Why Do AI Agents Systematically Fail at Cloud Root Cause Analysis?"
**Category:** sub-agent-behavior
**Priority:** medium
**Effort:** M

## Problem
The AI-RCA-failure paper (2602.09937) shows single-agent Reflexion loops (one model acting + judging + reflecting) produce *repeated reasoning errors and confirmation bias*. Today `pmat five-whys` issues a single LLM prompt with an optional `--auto-analyze` pass — it is structurally a single-agent loop and shares this weakness. Questions like "why is memory growing" tend to converge too early on the first plausible chain.

## Proposed improvement
Add `pmat five-whys --multi-persona` that runs the MAR pattern:
- **Analyst** persona: collects evidence (complexity, TDG, churn — the existing weighted signals).
- **Skeptic** persona: challenges the leading hypothesis with counter-evidence.
- **Architect** persona: proposes structural root cause.
- **Judge** synthesizes and outputs a final Five-Whys chain with disagreement markers.

Each persona gets *scoped evidence* (not the full conversation) — analogue to paper's "separated acting, diagnosing, critiquing, aggregating".

## Impact
- MAR paper reports substantial reasoning-quality lift on multi-step diagnosis.
- Converts Five-Whys from one-shot into an adversarial dialogue, which matches Toyota Way's team-based genba practice.

## Implementation sketch
1. Extend `five_whys` module with `Persona` enum + per-persona prompt template.
2. Evidence gating: each persona sees only task-relevant evidence slices.
3. Final output format adds `disagreements[]` alongside the Five-Whys chain.
4. Feature-flag behind `--multi-persona` to keep single-agent mode as default.

## Acceptance criteria
- `pmat five-whys --multi-persona "stack overflow"` runs 4 personas and emits a synthesized report + disagreements section.
- Integration test: on a known red-herring input, multi-persona produces a different and better root cause than single-persona.
- Book chapter updated.
- **No MCP schema change** — CLI flag only; existing MCP tool returns richer report when flag is set in args.
