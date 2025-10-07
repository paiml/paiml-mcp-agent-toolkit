# PMAT-7007: Claude Code Sub-Agent Scaffolding

**Status**: 🚀 TODO
**Priority**: P0 - Critical
**Complexity**: High
**Estimated Duration**: 1-2 weeks (MVP: 5 core sub-agents)
**Sprint**: 24
**Created**: 2025-10-07

---

## Objective

Create a library of specialized PMAT sub-agents compatible with Claude Code's sub-agent system, enabling domain-specific AI expertise for code quality workflows.

**Inspiration**: [VoltAgent Awesome Claude Code Sub-Agents](https://github.com/VoltAgent/awesome-claude-code-subagents)

---

## Background

### Current State
- PMAT has agent scaffolding (`server/src/scaffold/agent/`) with 4 templates
- Agent templates generate full Rust projects with tests and quality gates
- Feature-based composition (state machines, monitoring, async handlers)
- **Gap**: No domain-specific sub-agents for Claude Code integration

### VoltAgent Sub-Agent Pattern
- 80+ specialized sub-agents across 10 categories
- Markdown-based definitions with standardized structure
- MCP tool integration for capabilities
- Automatic delegation by Claude Code based on context

### Value Proposition
- **For Users**: Get specialized expertise (ComplexityAnalyst, MutationTester, etc.)
- **For PMAT**: Showcase MCP tools in real workflows
- **Marketing**: "PMAT + Claude Code = 20+ AI quality experts"

---

## Scope

### MVP: 5 Core Sub-Agents (Week 1-2)

1. **ComplexityAnalyst**
   - Focuses on cyclomatic and cognitive complexity analysis
   - MCP tools: `analyze_complexity`, `analyze_cognitive_complexity`
   - Suggests refactorings when CC > threshold

2. **MutationTester**
   - Mutation testing specialist with ML prediction
   - MCP tools: `mutation_test`, `mutation_predict`, `equivalent_detector`
   - Prioritizes high-value mutants, suggests test improvements

3. **SATDDetector**
   - Technical debt identifier (TODO, FIXME, HACK comments)
   - MCP tools: `analyze_satd`, `analyze_context`
   - Tracks debt evolution and resolution patterns

4. **DeadCodeEliminator**
   - Unused code removal specialist
   - MCP tools: `analyze_dead_code`, `analyze_imports`
   - Identifies safe-to-remove code

5. **DocumentationEnforcer**
   - Generic description detector
   - MCP tools: `check_generic_docs`, `analyze_context`
   - Enforces PMAT documentation standards

### Future Phases (Deferred)
- Language Specialists (Rust, Python, TypeScript, WASM)
- Architecture & Design (RefactoringAdvisor, BorrowCheckerTutor)
- Testing & Verification (TestCoverageAnalyst, PropertyTestGenerator)
- Orchestration (QualityGateOrchestrator, MetricsAggregator)

---

## Sub-Agent Template Structure

Each sub-agent is a markdown file with this format:

```markdown
# [Agent Name]

## Description
One-line description of agent's purpose

## Capabilities
- Capability 1
- Capability 2
- Capability 3

## Tools Used
- MCP Tool 1 (from PMAT server)
- MCP Tool 2

## Role Definition
Detailed prompt defining:
- Expertise areas
- Constraints and limits
- Behavioral guidelines
- Quality standards

## Communication Protocol
**With Main Claude Code**:
- Receive: [inputs]
- Return: [outputs]

**With Other Sub-Agents**:
- Coordinates with: [agent names]
- Exchanges: [data types]

**With PMAT MCP Server**:
- Calls: [tool list]

## Implementation Workflow
1. Step 1
2. Step 2
3. ...

## Example Invocations
**Automatic**:
[Trigger] → [Response]

**Manual**:
@AgentName [command]

## Quality Gates
- Metric 1: threshold
- Metric 2: threshold
```

---

## Implementation Plan

### Phase 1: Infrastructure (Days 1-3)

**1.1 Sub-Agent Generator Module**
- New file: `server/src/scaffold/agent/subagents.rs`
- Enum: `PmatSubAgent` with all agent types
- Struct: `SubAgentGenerator` with template rendering
- Method: `generate_subagent(agent: PmatSubAgent) -> Result<String>`
- Method: `export_for_claude_code(agent, output_dir) -> Result<PathBuf>`

**1.2 Template Storage**
- Directory: `server/src/scaffold/agent/subagent_templates/`
- Files: `complexity_analyst.md.tmpl`, `mutation_tester.md.tmpl`, etc.
- Use Handlebars for variable substitution (already in deps)

**1.3 MCP Tool Mapping**
- Struct: `SubAgentToolMap` mapping agents → MCP tools
- Validation: Ensure all referenced tools exist in MCP schema
- Auto-generation from MCP server capabilities

### Phase 2: Core Sub-Agents (Days 4-8)

**2.1 ComplexityAnalyst**
- Template: Complete role definition
- MCP tools: `analyze_complexity`, `analyze_cognitive_complexity`
- Example invocations: 3 scenarios
- Quality gates: CC thresholds

**2.2 MutationTester**
- Template: ML-guided mutation testing workflow
- MCP tools: `mutation_test`, `mutation_predict`, `equivalent_detector`
- Example: Multi-step mutation analysis with test suggestions
- Quality gates: ML accuracy ≥75%

**2.3 SATDDetector**
- Template: Technical debt tracking
- MCP tools: `analyze_satd`, `analyze_context`
- Example: SATD evolution report
- Quality gates: Detection accuracy

**2.4 DeadCodeEliminator**
- Template: Safe code removal
- MCP tools: `analyze_dead_code`, `analyze_imports`
- Example: Unused function identification
- Quality gates: False positive rate <5%

**2.5 DocumentationEnforcer**
- Template: Generic description detection
- MCP tools: `check_generic_docs`, `analyze_context`
- Example: Generic description replacement suggestions
- Quality gates: Detection rate >95%

### Phase 3: CLI Integration (Days 9-10)

**3.1 CLI Commands**
```bash
pmat scaffold subagent list
pmat scaffold subagent create <agent-name> [--output DIR]
pmat scaffold subagent create-all [--category CATEGORY]
pmat scaffold subagent validate <file>
pmat scaffold subagent export-mcp-mapping
```

**3.2 Command Handlers**
- New file: `server/src/cli/handlers/subagent_handlers.rs`
- Handlers: `list_subagents`, `create_subagent`, `validate_subagent`
- Output: Markdown files to `.claude/subagents/`

### Phase 4: Testing & Documentation (Days 11-14)

**4.1 Unit Tests**
- Test sub-agent generation for all 5 core agents
- Validate markdown format
- Verify MCP tool references
- Test template variable substitution

**4.2 Integration Tests**
- Generate sub-agents and validate with Claude Code (manual)
- Test MCP tool invocation from sub-agent context
- Verify multi-agent coordination

**4.3 Documentation**
- `docs/features/CLAUDE_CODE_SUBAGENTS.md` - User guide
- `docs/development/SUBAGENT_DEVELOPMENT.md` - Adding new sub-agents
- Examples: 5 complete sub-agent files in `examples/subagents/`
- README update: Add sub-agent section

**4.4 Property Tests**
- All generated markdown is valid
- All referenced MCP tools exist
- Template variables are substituted correctly

---

## Files to Create

### New Files
```
server/src/scaffold/agent/subagents.rs              (350 lines)
server/src/scaffold/agent/subagent_templates/
  ├── complexity_analyst.md.tmpl                    (150 lines)
  ├── mutation_tester.md.tmpl                       (200 lines)
  ├── satd_detector.md.tmpl                         (150 lines)
  ├── dead_code_eliminator.md.tmpl                  (150 lines)
  └── documentation_enforcer.md.tmpl                (150 lines)
server/src/cli/handlers/subagent_handlers.rs        (400 lines)
docs/features/CLAUDE_CODE_SUBAGENTS.md              (800 lines)
docs/development/SUBAGENT_DEVELOPMENT.md            (500 lines)
examples/subagents/
  ├── complexity-analyst.md
  ├── mutation-tester.md
  ├── satd-detector.md
  ├── dead-code-eliminator.md
  └── documentation-enforcer.md
```

### Files to Modify
```
server/src/scaffold/agent/mod.rs                    (export subagents module)
server/src/cli/handlers/mod.rs                      (export subagent_handlers)
server/src/cli/commands.rs                          (add subagent subcommand)
README.md                                            (add Claude Code section)
```

**Estimated Total**: ~2,200 new lines + 50 modified lines

---

## Testing Strategy

### Unit Tests (RED → GREEN)
1. **Template Rendering**
   ```rust
   #[test]
   fn test_generate_complexity_analyst() {
       let gen = SubAgentGenerator::new();
       let result = gen.generate_subagent(PmatSubAgent::ComplexityAnalyst);
       assert!(result.is_ok());
       assert!(result.unwrap().contains("# Complexity Analyst"));
   }
   ```

2. **MCP Tool Validation**
   ```rust
   #[test]
   fn test_all_tools_exist_in_mcp_schema() {
       let mapping = SubAgentToolMap::new();
       for (agent, tools) in mapping.iter() {
           for tool in tools {
               assert!(mcp_schema_has_tool(tool));
           }
       }
   }
   ```

3. **CLI Command Tests**
   ```rust
   #[test]
   fn test_subagent_list_command() {
       let output = run_cli(&["scaffold", "subagent", "list"]);
       assert!(output.contains("ComplexityAnalyst"));
       assert!(output.contains("MutationTester"));
   }
   ```

### Integration Tests
1. Generate all 5 sub-agents
2. Validate markdown format
3. Test with Claude Code (manual validation)

### Property Tests
1. All generated markdown is parsable
2. All tool references are valid
3. All templates have required sections

---

## Success Criteria

### Functional Requirements ✅
- ✅ 5 core sub-agents implemented
- ✅ CLI commands functional
- ✅ Markdown output compatible with Claude Code
- ✅ MCP tool mapping accurate
- ✅ All unit tests passing

### Quality Requirements ✅
- ✅ Test coverage ≥85%
- ✅ All sub-agents validated with Claude Code
- ✅ Documentation complete with examples
- ✅ Zero generic descriptions in templates

### Performance Requirements ✅
- ✅ Sub-agent generation <1s per agent
- ✅ Bulk generation (5 agents) <5s

---

## Risks & Mitigation

### Risk 1: Claude Code API Changes
**Impact**: High - Sub-agents may not work if format changes
**Mitigation**:
- Follow VoltAgent's established patterns (community-validated)
- Test with actual Claude Code before declaring complete
- Version sub-agent format in templates

### Risk 2: MCP Tool Mismatch
**Impact**: Medium - Sub-agents reference non-existent tools
**Mitigation**:
- Auto-generate tool mapping from MCP schema
- Validation step checks tool existence
- CI test ensures schema consistency

### Risk 3: Prompt Engineering Quality
**Impact**: Medium - Sub-agents may not behave as intended
**Mitigation**:
- Iterate on prompts with real Claude Code testing
- Gather feedback from early users
- Maintain prompt library with proven patterns

### Risk 4: Maintenance Burden
**Impact**: Low-Medium - 20+ sub-agents to maintain
**Mitigation**:
- Generate from templates (single source of truth)
- Automate updates when MCP tools change
- Community contributions via PR process

---

## Dependencies

### Internal
- Existing agent scaffold system (`server/src/scaffold/agent/`)
- MCP server and tool schema
- Handlebars templates (already in deps)
- CLI command infrastructure

### External
- Claude Code (for testing sub-agents)
- VoltAgent patterns (reference implementation)

---

## Deliverables

1. **Code**
   - SubAgentGenerator module
   - 5 sub-agent templates
   - CLI handlers
   - Tests (unit + integration)

2. **Documentation**
   - User guide for Claude Code sub-agents
   - Developer guide for adding new sub-agents
   - 5 example sub-agent files

3. **Validation**
   - All tests passing
   - Sub-agents tested with Claude Code
   - MCP tool mapping verified

---

## Next Steps After MVP

### Phase 2: Language Specialists (Deferred)
- RustQualityExpert
- PythonQualityExpert
- TypeScriptQualityExpert
- WasmDeepInspector

### Phase 3: Advanced Agents (Deferred)
- RefactoringAdvisor (uses pattern learning from PMAT-7009)
- TestCoverageAnalyst
- PerformanceProfiler

### Phase 4: Orchestration (Deferred)
- QualityGateOrchestrator (coordinates multiple sub-agents)
- WorkflowOptimizer
- MetricsAggregator

---

## Related Tickets

- PMAT-7008: Declarative Workflow API (sub-agents can use fluent workflows)
- PMAT-7009: Pattern Learning (RefactoringAdvisor will use this)
- PMAT-7002: WASM Deep Inspection (WasmDeepInspector sub-agent uses these tools)

---

## References

- [VoltAgent Sub-Agents](https://github.com/VoltAgent/awesome-claude-code-subagents)
- [Learning System Ideas](../specifications/learning-system-ideas.md#priority-0)
- [Existing Agent Scaffold](../../server/src/scaffold/agent/)

---

**Created**: 2025-10-07
**Last Updated**: 2025-10-07
**Status**: Ready for implementation
