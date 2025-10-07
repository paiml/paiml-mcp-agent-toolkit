# Sprint 24 Plan: Learning System & Sub-Agent Scaffolding

**Sprint Duration**: 2-3 weeks
**Target Release**: v2.144.0
**Start Date**: 2025-10-07
**Focus**: Priority 0 (Sub-Agents) + Priority 1 (Workflows & Pattern Learning)

---

## Sprint Goals

### Primary Objectives
1. **PMAT-7007**: Implement 5 core Claude Code sub-agents (MVP)
2. **PMAT-7008**: Create declarative workflow API
3. **PMAT-7009**: Build pattern learning system

### Success Criteria
- ✅ 5 production-ready sub-agents tested with Claude Code
- ✅ Declarative workflow API with 85%+ test coverage
- ✅ Pattern learning integrated with ML mutation predictor
- ✅ All documentation complete
- ✅ v2.144.0 released to crates.io

---

## Ticket Breakdown

### PMAT-7007: Claude Code Sub-Agent Scaffolding
**Priority**: P0 - Critical
**Estimated**: 1-2 weeks (10-14 days)
**Complexity**: High

**MVP: 5 Core Sub-Agents**
1. ComplexityAnalyst (3 days)
2. MutationTester (3 days)
3. SATDDetector (2 days)
4. DeadCodeEliminator (2 days)
5. DocumentationEnforcer (2 days)
6. Infrastructure + CLI (2 days)
7. Testing + Documentation (2 days)

**Work Breakdown**:
- Days 1-3: Infrastructure (SubAgentGenerator, templates, CLI)
- Days 4-8: 5 core sub-agents
- Days 9-10: Testing & documentation
- Days 11-14: Validation with Claude Code, polish

**Deliverables**:
- `server/src/scaffold/agent/subagents.rs` (350 lines)
- 5 sub-agent templates (800 lines total)
- CLI handlers (400 lines)
- Documentation (1,300 lines)
- Tests (500 lines)

---

### PMAT-7008: Declarative Workflow API
**Priority**: P1 - High
**Estimated**: 3-5 days
**Complexity**: Medium

**Work Breakdown**:
- Day 1-2: Core builder (WorkflowBuilder, agent())
- Day 3: Conditional branching (and_when, and_unless)
- Day 4: Advanced features (and_race, retry policies)
- Day 5: Testing & documentation

**Deliverables**:
- `server/src/workflow/builder.rs` (600 lines)
- `server/src/workflow/conditions.rs` (200 lines)
- `server/src/workflow/retry_policies.rs` (150 lines)
- Tests (400 lines)
- Documentation (600 lines)
- Examples (6 workflows)

---

### PMAT-7009: Pattern Learning System
**Priority**: P1 - High
**Estimated**: 5-7 days
**Complexity**: Medium-High

**Work Breakdown**:
- Day 1-2: Storage layer (SQLite schema, PatternStorage)
- Day 3-4: Feature extraction (complexity, mutations, SATD)
- Day 5-6: Integration (ML predictor, similarity matching)
- Day 7: CLI & testing

**Deliverables**:
- `server/src/services/learning/` module (2,000 lines)
- ML predictor integration (100 lines modified)
- CLI handlers (350 lines)
- Tests (400 lines)
- Documentation (700 lines)

---

## Sprint Schedule (3 Weeks)

### Week 1: Sub-Agent Infrastructure + Workflows

**Days 1-2** (PMAT-7007 + PMAT-7008)
- [ ] Design SubAgentGenerator API
- [ ] Create sub-agent template structure
- [ ] Implement WorkflowBuilder core

**Days 3-4** (PMAT-7007 + PMAT-7008)
- [ ] CLI commands for sub-agents
- [ ] MCP tool mapping validation
- [ ] Conditional branching in workflows

**Day 5** (PMAT-7007 + PMAT-7008)
- [ ] Complete workflow advanced features
- [ ] Sub-agent template rendering

**Weekend**: Buffer for unforeseen issues

### Week 2: Core Sub-Agents + Pattern Learning Foundation

**Days 6-7** (PMAT-7007)
- [ ] ComplexityAnalyst sub-agent
- [ ] MutationTester sub-agent

**Days 8-9** (PMAT-7007 + PMAT-7009)
- [ ] SATDDetector sub-agent
- [ ] Pattern storage schema + PatternStorage

**Days 10-11** (PMAT-7007 + PMAT-7009)
- [ ] DeadCodeEliminator sub-agent
- [ ] DocumentationEnforcer sub-agent
- [ ] Feature extraction for pattern learning

**Day 12** (PMAT-7009)
- [ ] Similarity matching algorithm
- [ ] ML predictor integration

**Weekend**: Testing & validation

### Week 3: Integration, Testing & Release

**Days 13-14** (All Tickets)
- [ ] Integration tests for all features
- [ ] Property tests for workflows & pattern learning
- [ ] Claude Code validation (manual)

**Days 15-16** (All Tickets)
- [ ] Documentation completion
- [ ] Examples for all features
- [ ] README updates

**Day 17** (Release)
- [ ] Version bump to v2.144.0
- [ ] Release notes
- [ ] Publish to crates.io
- [ ] GitHub release
- [ ] Roadmap update

---

## Parallel Workstreams

### Stream A: Sub-Agent Development (Days 1-14)
- Owner: Primary developer
- Focus: PMAT-7007
- Deliverable: 5 production-ready sub-agents

### Stream B: API Development (Days 1-7, then Days 10-14)
- Owner: Secondary developer (or same developer in parallel phases)
- Focus: PMAT-7008 (Days 1-5), PMAT-7009 (Days 6-12)
- Deliverable: Workflow API + Pattern learning

### Stream C: Testing & Documentation (Days 13-17)
- Owner: All contributors
- Focus: Integration, validation, release
- Deliverable: v2.144.0 release

**Note**: If single developer, follow sequential schedule. If team, parallelize streams.

---

## Dependencies & Blockers

### Internal Dependencies
- ✅ Agent scaffolding system exists (`server/src/scaffold/agent/`)
- ✅ Workflow executor exists (`server/src/workflow/executor.rs`)
- ✅ ML mutation predictor exists (`server/src/services/mutation/ml_predictor.rs`)
- ✅ MCP tools all implemented and documented

### External Dependencies
- Claude Code (for sub-agent testing) - User must have access
- rusqlite (already in Cargo.toml)
- Handlebars (already in deps)

### Potential Blockers
1. **Claude Code API Changes**: Mitigation - Follow VoltAgent patterns
2. **Performance Issues**: Mitigation - Early benchmarking
3. **Feature Creep**: Mitigation - Strict scope adherence (MVP only)

---

## Testing Strategy

### RED → GREEN → REFACTOR Phases

**Phase 1: RED (Write Failing Tests)**
- Days 1-2 of each ticket
- Define expected behavior
- Write unit tests that fail
- Document test coverage gaps

**Phase 2: GREEN (Make Tests Pass)**
- Days 3-5 of each ticket
- Minimal implementation to pass tests
- Focus on correctness, not optimization

**Phase 3: REFACTOR (Improve Code Quality)**
- Days 6-7 of each ticket
- Optimize performance
- Improve readability
- Ensure CC <8

### Test Coverage Requirements
- Unit tests: ≥85% coverage
- Property tests: All core algorithms
- Integration tests: End-to-end workflows
- Manual testing: Claude Code sub-agents

---

## Quality Gates

### Pre-Commit Checks
- ✅ All tests passing (cargo test)
- ✅ No compiler warnings
- ✅ Complexity <8 (PMAT quality gate)
- ✅ No generic descriptions in docs
- ✅ Formatting (cargo fmt)
- ✅ Linting (cargo clippy)

### Pre-Release Checks
- ✅ Integration tests pass
- ✅ Property tests pass
- ✅ Manual validation with Claude Code (sub-agents)
- ✅ Documentation complete
- ✅ Examples functional
- ✅ Release notes written
- ✅ Roadmap updated

---

## Risk Management

### High-Risk Items
1. **Sub-Agent Prompt Quality** (PMAT-7007)
   - **Risk**: Sub-agents may not behave as intended
   - **Mitigation**: Test with real Claude Code, iterate on prompts
   - **Contingency**: Provide manual mode (users can edit prompts)

2. **Workflow API Ergonomics** (PMAT-7008)
   - **Risk**: API may be confusing or verbose
   - **Mitigation**: Study VoltAgent patterns, gather early feedback
   - **Contingency**: Iterate based on user feedback in v2.145.0

3. **Pattern Learning Accuracy** (PMAT-7009)
   - **Risk**: Pattern matching may not improve predictions
   - **Mitigation**: Validate with synthetic data, benchmark accuracy
   - **Contingency**: Make pattern learning optional (opt-in)

### Medium-Risk Items
1. **Performance** - Mitigation: Early benchmarking
2. **Storage Growth** (PMAT-7009) - Mitigation: Retention policies
3. **Cross-Platform Testing** - Mitigation: CI for Linux/Mac/Windows

---

## Success Metrics

### Functional Metrics
- [ ] 5 sub-agents generated and validated
- [ ] 10+ workflow examples working
- [ ] Pattern learning improves ML accuracy by 5-10%

### Quality Metrics
- [ ] Test coverage ≥85%
- [ ] All property tests passing
- [ ] Zero critical bugs in MVP

### Community Metrics
- [ ] Release notes published
- [ ] Documentation on claude.com/docs (if applicable)
- [ ] Reddit/HN announcement (optional)

---

## Post-Sprint Review

### What Went Well
- (To be filled after sprint)

### What Could Be Improved
- (To be filled after sprint)

### Lessons Learned
- (To be filled after sprint)

### Carryover to Sprint 25
- Language-specific sub-agents (deferred)
- Advanced workflow features (deferred)
- RAG-based memory (deferred per specification)

---

## Sprint Artifacts

### Documentation
- [ ] `docs/features/CLAUDE_CODE_SUBAGENTS.md`
- [ ] `docs/features/DECLARATIVE_WORKFLOWS.md`
- [ ] `docs/features/PATTERN_LEARNING.md`
- [ ] `docs/tickets/TICKET-PMAT-7007.md` (already created)
- [ ] `docs/tickets/TICKET-PMAT-7008.md` (already created)
- [ ] `docs/tickets/TICKET-PMAT-7009.md` (already created)
- [ ] `docs/tickets/SPRINT-24-STATUS-UPDATE.md` (post-sprint)
- [ ] `docs/release_notes/v2.144.0.md` (end of sprint)

### Code Artifacts
- [ ] 3 new modules (subagents, workflow/builder, learning)
- [ ] 5 sub-agent templates
- [ ] 6 workflow examples
- [ ] ~5,000 lines of production code
- [ ] ~1,300 lines of tests

---

## Communication Plan

### Daily Standups (If Team)
- What did I complete yesterday?
- What am I working on today?
- Any blockers?

### Weekly Check-ins
- End of Week 1: Infrastructure review
- End of Week 2: MVP feature completeness review
- End of Week 3: Release readiness review

### Stakeholder Updates
- Week 1: "Sub-agent infrastructure ready"
- Week 2: "5 core sub-agents + workflow API complete"
- Week 3: "v2.144.0 released with learning system"

---

## Rollout Plan

### v2.144.0 Release Checklist
1. [ ] All tickets marked complete
2. [ ] Version bumped in Cargo.toml
3. [ ] Release notes written
4. [ ] Git tag created (v2.144.0)
5. [ ] Published to crates.io
6. [ ] GitHub release created
7. [ ] Roadmap updated
8. [ ] README updated (add sub-agent section)
9. [ ] Announcement prepared (optional)

### Post-Release
- [ ] Monitor GitHub issues for bugs
- [ ] Gather user feedback on sub-agents
- [ ] Plan Sprint 25 based on feedback

---

**Created**: 2025-10-07
**Status**: Active Sprint
**Next Review**: End of Week 1 (2025-10-14)
