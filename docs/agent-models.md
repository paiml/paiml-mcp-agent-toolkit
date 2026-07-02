# Agent Model Registry

> **Single source of truth for model ids referenced anywhere in `docs/`.**
> MACS F6 (Component 32). CB-1657 lints a deny-list of superseded model ids
> (`claude-3-*`, `claude-2*`, `gpt-4-turbo`) out of every doc *except* the
> History table below — stale ids in agent-facing docs are executable
> misinformation, so they are treated like code and linted.

## Current models (2026-07)

| Role | Model id | Notes |
|------|----------|-------|
| Primary (autonomous coding) | `claude-fable-5` | Mythos-class; default effort `high`; xhigh via ultracode |
| Primary (approved orgs) | `claude-mythos-5` | Same base model as Fable 5, without dual-use measures |
| High-capability | `claude-opus-4-8` | Opus 4.8 |
| Balanced | `claude-sonnet-5` | Sonnet 5 |
| Fast | `claude-haiku-4-5-20251001` | Haiku 4.5 |

When a doc needs to name a model, reference this table (or the role, e.g.
"the primary autonomous model") rather than pinning an id inline. That keeps a
model migration to a single edit here.

## History (allow-listed for CB-1657)

Superseded ids are preserved here for archaeology; CB-1657 permits the
deny-listed ids **only** inside this section.

| Retired id | Superseded by | Retired |
|------------|---------------|---------|
| `claude-3-opus` | `claude-opus-4-8` | pre-2026 |
| `claude-2` | `claude-3-opus` | pre-2026 |
| `gpt-4-turbo` | `claude-opus-4-8` (fallback) | 2026-07 (MACS-015) |
