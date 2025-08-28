# Sprint 3 Refactoring Plan - handle_analyze_complexity

**Issue**: #49  
**Current Complexity**: 41 cyclomatic, 60 cognitive  
**Target**: ≤8 cyclomatic (Toyota Way)  
**File**: `server/src/cli/handlers/complexity_handlers.rs`

## Analysis of Current Function

### Responsibilities (Violation of Single Responsibility Principle)
1. **Input Validation**: Watch mode check, file existence
2. **Configuration**: Toolchain detection, threshold building  
3. **Analysis Routing**: Single file vs multiple files vs project mode
4. **Data Processing**: File content reading, analysis execution
5. **Post-Processing**: Filtering by thresholds, sorting by complexity
6. **Output Formatting**: Multiple format handling (JSON, Summary, etc.)
7. **Output Writing**: File vs console output

### Complexity Contributors
- **Nested conditionals**: file vs files vs project mode (3 branches)
- **Toolchain handling**: detected vs undetected (2 branches) 
- **Filtering logic**: threshold filtering (multiple conditions)
- **Format matching**: Multiple output formats (5+ branches)
- **Error handling**: Multiple Result<> chains

## Refactoring Strategy

### Phase 1: Extract Analysis Functions
- Create single-purpose functions for each analysis mode
- Reduce main function branching logic
- Improve testability through smaller units

### Phase 2: Extract Configuration  
- Create configuration struct to reduce parameter passing
- Centralize toolchain and threshold logic
- Simplify main function signature

### Implementation Timeline
- **Day 1**: Begin with configuration extraction and single file analysis
- **Total**: 4 days following Toyota Way quality-first approach