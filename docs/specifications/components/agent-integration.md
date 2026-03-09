# Agent Integration

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 10

## Claude Agent SDK

### Integration Points

- `pmat` as MCP tool provider for Claude Code
- Claude Code skills integration (`claude-skills-spec-v1`)
- Sub-agent workflows with Actix actor framework

### MCP Tools Exposed

| Tool | Description |
|------|-------------|
| `pmat_query_code` | Semantic code search |
| `pmat_get_function` | Get function with full metrics |
| `pmat_find_similar` | Find similar functions |
| `pmat_index_stats` | Index health and statistics |
| `pmat_comply_check` | Run compliance checks |
| `pmat_tdg_score` | Get TDG score for function |

## AGENTS.md Protocol

Standard agent protocol bridging:
- Declares available tools and capabilities
- Defines input/output schemas
- Supports multi-agent orchestration

## Sub-Agent Architecture

### Actix Actor Framework

Multi-agent workflows using Actix actors:
- Each analysis phase can run as an actor
- Message-passing for inter-phase communication
- Supervision trees for fault tolerance

### Architect-Programmer Protocol (SWE-CI)

Dual-agent workflow for requirement-driven development:
1. Architect agent: analyzes test gaps, produces requirements
2. Programmer agent: implements requirements, validates

See [swe-ci-evolution.md](swe-ci-evolution.md) for full specification.

## Claude Code Skills

### Registered Skills

| Skill | Trigger | Action |
|-------|---------|--------|
| `/quality` | User asks about code quality | Run TDG analysis |
| `/comply` | User asks about compliance | Run pmat comply |
| `/search` | User asks to find code | Run pmat query |
| `/evolve` | User asks about evolution | Compute EvoScore |

## Key Files

| File | Purpose |
|------|---------|
| `src/mcp_server/` | MCP server implementation |
| `src/mcp_pmcp/` | PMCP protocol handler |
| `AGENTS.md` | Agent protocol declaration |

## References

- Consolidated from: agents, claude-agent-integration, claude-code-agent-mode-spec,
  claude-skills-spec-v1, claude-sub-agents-spec-actix
