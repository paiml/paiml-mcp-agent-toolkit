# Sprint 21: Scaffolding System Refinements

**Sprint:** Sprint 21
**Duration:** 2-3 days
**Status:** 📋 Planning
**Focus:** Address dogfooding findings from v2.139.0 and add high-value enhancements

## Executive Summary

Sprint 20 successfully delivered v2.139.0 with complete scaffolding and maintenance system. Dogfooding revealed several high-value improvements and minor issues that would significantly enhance the developer experience. Sprint 21 focuses on refinements based on real-world usage.

## Dogfooding Findings

From `docs/dogfooding/v2.139.0-INTEGRATION-SHOWCASE.md`:

### What Works Exceptionally Well ✅
1. Performance - Health checks are incredibly fast (9.8s)
2. Validation - Roadmap validation catches all inconsistencies
3. Hooks - Pre-commit hooks successfully block bad commits
4. UX - Progress indicators and error messages are clear
5. Documentation - Comprehensive guides make features discoverable

### Issues Identified 🔧
1. **Hook Verification** - Minor timestamp check issue (verification shows "outdated" after refresh)
2. **Coverage Performance** - Coverage check not in default health due to ~100s runtime
3. **Sequential Checks** - Health checks run sequentially, could be parallelized
4. **Manual Ticket Creation** - Had to manually create Sprint 20 ticket files
5. **MCP Not Exposed** - Scaffolding features not available via MCP protocol

### Opportunities 🚀
1. Parallel health check execution for 3-5x speed improvement
2. Auto-generate ticket files from roadmap entries
3. Expose scaffolding via MCP for agent integration
4. Smart coverage (only changed files)
5. Enhanced hook diagnostics

## Sprint 21 Proposed Tickets

### Priority 0 (Critical) - Performance & UX

#### TICKET-PMAT-6010: Parallel Health Check Execution
**Estimated Effort:** 3-4 hours
**Priority:** P0 - Critical

**Problem:**
Health checks run sequentially, wasting time when multiple checks are requested.

**Current Behavior:**
```bash
pmat maintain health --check-build --check-tests
# Build: 10s (sequential)
# Tests: 60s (sequential)
# Total: 70s
```

**Proposed Behavior:**
```bash
pmat maintain health --check-build --check-tests
# Build + Tests in parallel
# Total: 60s (limited by slowest)
# Improvement: 14% faster
```

**Implementation:**
- Use tokio spawn for concurrent execution
- Aggregate results with timeout handling
- Maintain progress indicators for each check
- CC target: <8

**Success Criteria:**
- Multiple checks run in parallel
- Progress shows all active checks
- Total time = max(check_times) not sum(check_times)
- All tests passing

---

#### TICKET-PMAT-6011: Fix Hook Verification Timestamp Issue
**Estimated Effort:** 1-2 hours
**Priority:** P0 - Critical

**Problem:**
Hook verification shows "content outdated" immediately after refresh.

**Current Behavior:**
```bash
$ pmat hooks refresh
✅ Hook refreshed

$ pmat hooks verify
⚠️ Hook content outdated
```

**Root Cause:**
Timestamp comparison using strict equality instead of content hash.

**Solution:**
- Use content hash for verification
- Compare against expected template
- Update timestamp only when content actually changes
- CC target: <5

**Success Criteria:**
- Verification passes after refresh
- Only shows outdated when content differs
- Timestamp updates correctly

---

### Priority 1 (High) - Developer Experience

#### TICKET-PMAT-6012: Auto-generate Ticket Files from Roadmap
**Estimated Effort:** 3-4 hours
**Priority:** P1 - High

**Problem:**
Sprint 20 required manually creating 5 ticket files. This is tedious and error-prone.

**Proposed Feature:**
```bash
# Generate ticket files for uncreated tickets
pmat maintain roadmap --generate-tickets

# Output:
# 📝 Generating ticket files...
#   Created TICKET-PMAT-6002.md
#   Created TICKET-PMAT-6003.md
#   Created TICKET-PMAT-6004.md
#   Created TICKET-PMAT-6005.md
#   Created TICKET-PMAT-6006.md
# ✅ Generated 5 ticket files
```

**Implementation:**
- Parse roadmap for ticket IDs
- Check if ticket file exists
- Generate template with:
  - Ticket ID as title
  - Sprint from roadmap context
  - Priority placeholder
  - Status from roadmap checkbox
  - Template structure
- CC target: <7

**Template Generated:**
```markdown
# TICKET-PMAT-XXXX: [Title from roadmap]

**Sprint:** [Detected from roadmap]
**Priority:** [TBD - To be determined]
**Estimated Effort:** [TBD]
**Status**: [GREEN if checked, PLANNED if unchecked]
**Created:** [Current date]

## Problem Statement

[TODO: Describe the problem this ticket solves]

## Solution

[TODO: Describe the proposed solution]

## Implementation Plan

[TODO: Add implementation details]

## Acceptance Criteria

- [ ] [TODO: Add acceptance criteria]

## Quality Metrics

- **CC:** [Target]
- **Coverage:** >80%
- **Tests:** [Number and type]
```

**Success Criteria:**
- Generates ticket files for missing tickets
- Detects sprint from roadmap context
- Uses roadmap checkbox for status
- Dry-run mode available
- All tests passing

---

#### TICKET-PMAT-6013: MCP Server for Scaffolding
**Estimated Effort:** 4-6 hours
**Priority:** P1 - High

**Problem:**
Scaffolding features only available via CLI. Agents can't use scaffolding capabilities.

**Proposed Feature:**
Expose scaffolding via MCP protocol with tools:
- `scaffold_agent` - Create new agent project
- `scaffold_wasm` - Create new WASM project
- `validate_roadmap` - Check roadmap structure
- `health_check` - Run project health checks
- `generate_tickets` - Auto-create ticket files

**Implementation:**
- Add MCP tools in `server/src/mcp/tools.rs`
- Reuse existing scaffolding handlers
- Add proper error handling
- Support all template types
- CC target: <8 per tool

**MCP Tool Example:**
```json
{
  "name": "scaffold_agent",
  "description": "Generate a new MCP agent project with quality gates",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string", "description": "Project name (snake_case)"},
      "template": {"type": "string", "enum": ["basic", "stateful", "hybrid"]},
      "features": {"type": "array", "items": {"type": "string"}},
      "quality_level": {"type": "string", "enum": ["extreme", "high", "standard"]}
    },
    "required": ["name", "template"]
  }
}
```

**Success Criteria:**
- 5 MCP tools exposed
- All tools tested via MCP protocol
- Documentation updated with MCP examples
- Integration tests for each tool
- CC <8 for all tools

---

### Priority 2 (Medium) - Optimization

#### TICKET-PMAT-6014: Smart Coverage (Changed Files Only)
**Estimated Effort:** 4-5 hours
**Priority:** P2 - Medium

**Problem:**
Full coverage check takes 100+ seconds. Most of the time only a few files changed.

**Proposed Feature:**
```bash
# Smart coverage (git-aware, only changed files)
pmat maintain health --check-coverage --smart

# Output:
# 📊 Smart Coverage (changed files only)
#   Analyzing: 3 files changed since last commit
#   Running coverage on affected modules...
#   ✓ Coverage: 85.2% (3 files, 2.5s)
```

**Implementation:**
- Detect changed files via git
- Map files to test modules
- Run targeted coverage
- Cache results for unchanged files
- Fall back to full coverage if needed
- CC target: <9

**Success Criteria:**
- Runs coverage on changed files only
- 10x faster for small changes
- Falls back to full on first run
- Cache invalidation works correctly
- All tests passing

---

#### TICKET-PMAT-6015: Enhanced Hook Diagnostics
**Estimated Effort:** 2-3 hours
**Priority:** P2 - Medium

**Problem:**
When hooks fail, diagnostics are minimal. Hard to debug configuration issues.

**Proposed Feature:**
```bash
pmat hooks diagnose

# Output:
# 🔍 Hook Diagnostics
#
# ✅ Hook Installation
#   Location: .git/hooks/pre-commit
#   Permissions: executable (755)
#   Size: 2.3 KB
#   Last modified: 2025-10-06 12:04:56
#
# ✅ Configuration
#   Config file: .pmat-gates.toml
#   Valid: Yes
#   Max complexity: 10
#   Min coverage: 80%
#
# ✅ Dependencies
#   pmat binary: /usr/local/bin/pmat (v2.139.0)
#   cargo: Available
#   git: Available
#
# ⚠️ Warnings
#   - Hook execution time: 45s (consider --quick flag)
#
# 💡 Suggestions
#   - Add PMAT_HOOK_QUICK=1 to .git/hooks/pre-commit for faster checks
```

**Implementation:**
- Check hook installation
- Validate configuration
- Test dependencies
- Measure execution time
- Provide actionable suggestions
- CC target: <7

**Success Criteria:**
- Comprehensive diagnostics
- Actionable suggestions
- Performance insights
- Dependency checking
- All tests passing

---

### Priority 3 (Low) - Nice to Have

#### TICKET-PMAT-6016: Roadmap Health Trends
**Estimated Effort:** 3-4 hours
**Priority:** P3 - Low

**Problem:**
Can see current roadmap health but no historical trends.

**Proposed Feature:**
```bash
pmat maintain roadmap --health --trends

# Output:
# 📊 Roadmap Health Trends (Last 30 days)
#
# Sprint Completion Rate:
#   Oct 1-5:   Sprint 19 (7 tickets) - 100% ✅
#   Oct 6-10:  Sprint 20 (6 tickets) - 100% ✅
#
# Velocity:
#   Average: 5.5 tickets/sprint
#   Trend: Stable
#
# Ticket Age:
#   Oldest open: None
#   Average completion: 2.1 days
```

**Implementation:**
- Track roadmap state in `.pmat-history/`
- Store daily snapshots
- Calculate trends
- Visualize completion rate
- CC target: <8

**Success Criteria:**
- Tracks historical data
- Shows completion trends
- Calculates velocity
- Identifies bottlenecks

---

## Sprint 21 Metrics

### Estimated Effort

| Priority | Tickets | Total Effort |
|----------|---------|--------------|
| P0 | 2 | 4-6 hours |
| P1 | 2 | 7-10 hours |
| P2 | 2 | 6-8 hours |
| P3 | 1 | 3-4 hours |
| **Total** | **7** | **20-28 hours** |

### Recommended Scope

**Minimum (P0 only):**
- TICKET-PMAT-6010: Parallel health checks
- TICKET-PMAT-6011: Fix hook verification

**Recommended (P0 + P1):**
- Above +
- TICKET-PMAT-6012: Auto-generate tickets
- TICKET-PMAT-6013: MCP scaffolding

**Full (All tickets):**
- Above +
- TICKET-PMAT-6014: Smart coverage
- TICKET-PMAT-6015: Hook diagnostics
- TICKET-PMAT-6016: Health trends

## Success Criteria

### Must Have (P0)
- [ ] Parallel health checks working
- [ ] Hook verification issue fixed
- [ ] All existing tests still passing
- [ ] Documentation updated

### Should Have (P1)
- [ ] Ticket auto-generation working
- [ ] MCP scaffolding exposed
- [ ] 5 new MCP tools tested
- [ ] Integration showcase updated

### Nice to Have (P2-P3)
- [ ] Smart coverage implemented
- [ ] Hook diagnostics available
- [ ] Health trends tracking

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Parallel execution race conditions | Medium | High | Extensive testing, proper synchronization |
| MCP integration complexity | Medium | Medium | Reuse existing handlers, incremental approach |
| Smart coverage edge cases | High | Low | Fallback to full coverage, clear warnings |
| Scope creep | Medium | Medium | Focus on P0+P1 only, defer P2-P3 |

## Dependencies

- v2.139.0 features (all implemented)
- tokio for async/parallel execution
- MCP protocol (already integrated)
- Git for smart coverage

## Related Work

- Sprint 20: UX Improvements (completed)
- v2.139.0: Scaffolding system (completed)
- Dogfooding findings (documented)

## Next Steps

1. **Review & Approve** - Team review of Sprint 21 plan
2. **Prioritize** - Confirm P0+P1 scope for Sprint 21
3. **Create Tickets** - Generate ticket files (or use new auto-gen feature!)
4. **Implementation** - Execute Sprint 21 (2-3 days)
5. **Dogfood** - Validate improvements in PMAT development
6. **Release** - v2.140.0 with refinements

## Notes

### Why These Tickets?

All tickets derived from real dogfooding findings:
- **Parallel checks**: Observed sequential execution waste
- **Hook verification**: Encountered false "outdated" warnings
- **Auto-gen tickets**: Manually created 5 ticket files in Sprint 20
- **MCP scaffolding**: Agents can't use scaffolding features yet
- **Smart coverage**: 100s coverage is too slow for rapid iteration

### Sprint 21 vs Future Sprints

**Sprint 21 (This sprint):**
- Refinements and bug fixes
- High-value quick wins
- Based on v2.139.0 dogfooding

**Future Sprints (Post-21):**
- P2 backlog items (data validation, transformation, etc.)
- Advanced features (ML-based refactoring, etc.)
- New capabilities (not refinements)

---

**Status:** Ready for review
**Estimated Duration:** 2-3 days (P0+P1 scope)
**Target Release:** v2.140.0
