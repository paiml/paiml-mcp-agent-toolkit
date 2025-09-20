# Uniform Contracts System - Implementation Roadmap

## Overview
This roadmap tracks the implementation of the uniform contracts system across all PMAT interfaces (CLI, MCP, HTTP) to ensure identical parameter names and behaviors.

## Status: Phase 1 Complete ✅
**Foundation implemented and working** - Contract system operational with 9/9 tests passing.

---

## Sprint Planning

### Sprint 1: CLI Migration & Integration (Current Sprint)
**Duration**: 2 weeks  
**Status**: 🚀 Ready to Start  
**Goal**: Migrate existing CLI commands to use uniform contracts

#### Sprint 1 Tickets

##### Ticket #001: Migrate Complexity Command
- **Type**: Feature
- **Priority**: High
- **Estimate**: 3 story points
- **Assignee**: Claude Agent
- **Description**: Replace existing complexity command with uniform contract version
- **Acceptance Criteria**:
  - [ ] `pmat analyze complexity` uses uniform parameters (`--path`, not `--project-path`)
  - [ ] All existing functionality preserved
  - [ ] Backward compatibility warnings implemented
  - [ ] Tests pass for both old and new parameter names
- **Definition of Done**: Command uses `AnalyzeComplexityContract`, tests pass, documentation updated

##### Ticket #002: Migrate SATD Command  
- **Type**: Feature
- **Priority**: High
- **Estimate**: 2 story points
- **Assignee**: Claude Agent
- **Description**: Ensure SATD command uses uniform contract parameters
- **Acceptance Criteria**:
  - [ ] `pmat analyze satd` uses `AnalyzeSatdContract`
  - [ ] Parameter names consistent across interfaces
  - [ ] Severity handling standardized
  - [ ] Integration tests pass
- **Definition of Done**: SATD command fully uniform, tests pass

##### Ticket #003: Migrate Dead Code Command
- **Type**: Feature  
- **Priority**: Medium
- **Estimate**: 2 story points
- **Assignee**: Claude Agent
- **Description**: Update dead code analysis to use uniform contracts
- **Acceptance Criteria**:
  - [ ] `pmat analyze dead-code` uses `AnalyzeDeadCodeContract`
  - [ ] All parameters standardized
  - [ ] Response format consistent
  - [ ] Performance maintained
- **Definition of Done**: Command migrated, tests pass, performance verified

##### Ticket #004: CLI Integration Testing
- **Type**: Testing
- **Priority**: High  
- **Estimate**: 2 story points
- **Assignee**: Claude Agent
- **Description**: Comprehensive testing of migrated CLI commands
- **Acceptance Criteria**:
  - [ ] All CLI commands use uniform contracts
  - [ ] Parameter validation working
  - [ ] Error messages consistent
  - [ ] Help text updated
- **Definition of Done**: Full CLI test suite passes, documentation complete

---

### Sprint 2: HTTP & MCP Standardization (Next Sprint)
**Duration**: 2 weeks
**Status**: 📋 Planned
**Goal**: Ensure HTTP and MCP interfaces fully conform to contracts

#### Sprint 2 Tickets

##### Ticket #005: HTTP Endpoint Validation
- **Type**: Feature
- **Priority**: Medium
- **Estimate**: 3 story points
- **Description**: Validate all HTTP endpoints use contracts correctly
- **Acceptance Criteria**:
  - [ ] All endpoints use contract validation
  - [ ] OpenAPI spec generated from contracts
  - [ ] Error handling standardized
  - [ ] CORS properly configured

##### Ticket #006: MCP Tool Schema Generation
- **Type**: Feature
- **Priority**: Medium  
- **Estimate**: 2 story points
- **Description**: Auto-generate MCP tool schemas from contracts
- **Acceptance Criteria**:
  - [ ] Schema generation automated
  - [ ] All MCP tools have correct schemas  
  - [ ] Parameter validation working
  - [ ] Tool discovery functioning

##### Ticket #007: Cross-Interface Testing
- **Type**: Testing
- **Priority**: High
- **Estimate**: 3 story points
- **Description**: Test identical behavior across all interfaces
- **Acceptance Criteria**:
  - [ ] Same parameters produce same results
  - [ ] Error handling identical
  - [ ] Response formats consistent
  - [ ] Performance comparable

---

### Sprint 3: Quality Gate & Refactoring (Future Sprint)
**Duration**: 2 weeks
**Status**: 🔮 Future
**Goal**: Complete remaining commands and quality gates

#### Sprint 3 Tickets

##### Ticket #008: Quality Gate Integration
- **Type**: Feature
- **Priority**: Medium
- **Estimate**: 4 story points
- **Description**: Implement quality gate with uniform contracts

##### Ticket #009: Refactor Command Integration  
- **Type**: Feature
- **Priority**: Low
- **Estimate**: 3 story points
- **Description**: Update refactor commands to use contracts

##### Ticket #010: Legacy Cleanup
- **Type**: Refactoring
- **Priority**: Low
- **Estimate**: 2 story points  
- **Description**: Remove backward compatibility layer

---

## Sprint Metrics & Tracking

### Sprint 1 Velocity Target
- **Total Story Points**: 9
- **Completion Target**: 100%
- **Quality Gate**: All tests pass, no regressions

### Definition of Ready (DoR)
- [ ] Requirements clearly defined
- [ ] Acceptance criteria specified
- [ ] Dependencies identified
- [ ] Estimate provided

### Definition of Done (DoD)
- [ ] Code implemented and tested
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Code reviewed and merged
- [ ] Sprint demo completed

## Risk Management

### High Risks
1. **CLI Parameter Breaking Changes** - Mitigation: Backward compatibility layer
2. **Performance Regression** - Mitigation: Benchmark tests
3. **Integration Complexity** - Mitigation: Phased rollout

### Medium Risks  
1. **Test Coverage Gaps** - Mitigation: Comprehensive test suite
2. **Documentation Drift** - Mitigation: Automated doc generation

## Success Metrics

### Sprint Success Criteria
- [ ] All planned tickets completed
- [ ] No critical bugs introduced
- [ ] Performance maintained or improved
- [ ] Documentation current and accurate

### Overall Project Success
- [ ] 100% parameter consistency across interfaces
- [ ] Zero legacy parameter inconsistencies
- [ ] Full contract validation coverage
- [ ] Comprehensive test coverage (>95%)

---

### Sprint 4: Ruchy Language Integration (Priority Sprint) ✅ COMPLETED
**Duration**: 4 weeks
**Status**: ✅ Completed
**Goal**: Full Ruchy language support with AST extraction and quality gates

#### Sprint 4 Tickets

##### Ticket #011: Ruchy AST Parser Implementation
- **Type**: Feature
- **Priority**: Critical
- **Estimate**: 8 story points
- **Assignee**: Claude Agent
- **Description**: Implement Ruchy language AST extraction using Logos lexer
- **Acceptance Criteria**:
  - [x] Parse all Ruchy syntax (functions, types, actors, patterns)
  - [x] Extract AST items with line numbers
  - [x] Handle ML-style syntax correctly
  - [x] Support .ruchy and .rh extensions
- **Definition of Done**: Parser handles 95%+ of Ruchy syntax, tests pass

##### Ticket #012: Ruchy Complexity Analysis
- **Type**: Feature
- **Priority**: High
- **Estimate**: 5 story points
- **Assignee**: Claude Agent
- **Description**: Calculate complexity metrics for Ruchy code
- **Acceptance Criteria**:
  - [x] Cyclomatic complexity for functions
  - [x] Pattern matching complexity (+1 per arm)
  - [x] Actor complexity (+3 per handler)
  - [x] Proof complexity (+5 per tactic)
- **Definition of Done**: Accurate complexity metrics, validated against manual analysis

##### Ticket #013: Ruchy Quality Gates
- **Type**: Feature
- **Priority**: High
- **Estimate**: 3 story points
- **Assignee**: Claude Agent
- **Description**: Integrate Ruchy with existing quality gate system
- **Acceptance Criteria**:
  - [x] Zero SATD enforcement
  - [x] Complexity thresholds (≤10 standard)
  - [x] Test coverage requirements (80%)
  - [x] Integration with make commands
- **Definition of Done**: Quality gates enforced for Ruchy files

##### Ticket #014: Ruchy TDD Test Suite
- **Type**: Testing
- **Priority**: Critical
- **Estimate**: 5 story points
- **Assignee**: Claude Agent
- **Description**: Comprehensive test suite using extreme TDD
- **Acceptance Criteria**:
  - [x] Unit tests with RED-GREEN-REFACTOR cycle
  - [x] Property tests with 80% coverage
  - [x] Integration tests against real Ruchy code
  - [x] Zero defects policy maintained
- **Definition of Done**: All tests passing, 80%+ coverage achieved

##### Ticket #015: Ruchy Entropy Refactoring
- **Type**: Enhancement
- **Priority**: Medium
- **Estimate**: 3 story points
- **Assignee**: Claude Agent
- **Description**: Apply entropy-identified refactoring patterns
- **Acceptance Criteria**:
  - [x] DataValidation pattern centralized
  - [x] DataTransformation pipelines extracted
  - [x] ResourceManagement RAII implemented
  - [x] ApiCall abstraction created
- **Definition of Done**: 5%+ LOC reduction achieved, patterns eliminated

---

### Sprint 4.1: Quality Maintenance & Kotlin Support ✅ COMPLETED
**Duration**: 1 week
**Status**: ✅ Completed
**Goal**: Fix test failures, eliminate warnings, and add Kotlin AST support

#### Sprint 4.1 Achievements
- **KotlinAstStrategy**: Full implementation with AST extraction and complexity analysis
- **Zero Defects**: All cargo warnings eliminated (60+ warnings → 0)
- **Zero Test Failures**: All compilation errors and test failures fixed
- **Quality Gates**: All implementations pass with 0 violations
- **Coverage**: Maintained 80%+ test coverage standard
- **Clippy Clean**: Applied automatic fixes and added Default implementations

#### Technical Deliverables
- `KotlinAstStrategy` with full AST support for .kt and .kts files
- Enhanced Ruchy ML implementation (`ruchy_ml.rs`) with 13 tests passing
- Fixed import/compilation issues across language modules
- Eliminated all unused variables, imports, and fields
- Added Default trait implementations where suggested by clippy

---

## Sprint Review Schedule

### Sprint 1 Check-ins
- **Day 3**: Ticket #001 progress review
- **Day 7**: Mid-sprint retrospective  
- **Day 10**: Ticket #002-003 progress review
- **Day 14**: Sprint completion and demo

### Retrospective Topics
1. What worked well in contract implementation?
2. What obstacles were encountered?
3. How can we improve the next sprint?
4. Are our estimates accurate?

---

*Last Updated*: $(date)  
*Next Review*: Sprint 1 Day 3 Check-in