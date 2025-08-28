# Contract Refactoring Plan: Uniform Interfaces

## CRITICAL: Current Contract Violations

After auditing the CLI commands, we've identified these violations of the uniform contract principle:

### 1. Inconsistent Path Parameters
- ❌ `Complexity`: uses `project_path` 
- ❌ `DeadCode`: uses `path`
- ❌ `Satd`: uses `path`
- ❌ `Tdg`: uses `path`
- ❌ `LintHotspot`: uses `project_path`
- ✅ **FIX**: All must use `path`

### 2. File vs Files Inconsistency
- ❌ `Complexity`: has BOTH `file` (single) AND `files` (multiple)
- ❌ `LintHotspot`: has `file` parameter
- ❌ `QualityGate`: has `file` parameter
- ✅ **FIX**: Use `files: Option<Vec<PathBuf>>` everywhere for consistency

### 3. Format Type Inconsistency
- ❌ `Complexity`: uses `ComplexityOutputFormat`
- ❌ `DeadCode`: uses `DeadCodeOutputFormat`
- ❌ `Satd`: uses `SatdOutputFormat`
- ✅ **FIX**: All must use unified `OutputFormat` enum

### 4. Missing Common Parameters
- ❌ `Complexity`: missing `include_tests`
- ❌ `Tdg`: missing `include_tests`
- ❌ `Tdg`: missing explicit `timeout`
- ✅ **FIX**: All analysis commands must have base parameters

### 5. Type Inconsistencies
- ❌ `Complexity`: uses `Option<u16>` for thresholds
- ❌ `DeadCode`: uses `u64` for timeout
- ✅ **FIX**: Standardize on `u32` for thresholds, `u64` for timeout

## Refactoring Steps

### Step 1: Update CLI Command Definitions
```rust
// Before (WRONG):
Complexity {
    #[arg(short = 'p', long, default_value = ".")]
    project_path: PathBuf,  // ❌ Should be 'path'
    
    #[arg(long)]
    file: Option<PathBuf>,  // ❌ Inconsistent with 'files'
    
    #[arg(long, value_delimiter = ',')]
    files: Vec<PathBuf>,    // ❌ Both file and files is confusing
}

// After (CORRECT):
Complexity {
    #[arg(short = 'p', long, default_value = ".")]
    path: PathBuf,          // ✅ Uniform parameter name
    
    #[arg(long, value_delimiter = ',')]
    files: Option<Vec<PathBuf>>,  // ✅ Optional list of files
    
    #[arg(long)]
    include_tests: bool,    // ✅ Standard base parameter
}
```

### Step 2: Create Adapter Layer
Until we can refactor all existing code, create an adapter:

```rust
// src/contracts/adapter.rs
impl From<cli::commands::AnalyzeCommands> for UnifiedContract {
    fn from(cmd: cli::commands::AnalyzeCommands) -> Self {
        match cmd {
            AnalyzeCommands::Complexity { project_path, .. } => {
                // Map old 'project_path' to new 'path'
                UnifiedContract {
                    path: project_path,
                    // ... rest of mapping
                }
            }
        }
    }
}
```

### Step 3: Update MCP Tools
Ensure all MCP tools use the exact same parameter names:

```rust
// MCP tool definition
Tool {
    name: "analyze_complexity",
    parameters: {
        "path": PathBuf,        // ✅ Same as CLI
        "files": Vec<PathBuf>,  // ✅ Same as CLI
        "format": OutputFormat, // ✅ Same as CLI
        "include_tests": bool,  // ✅ Same as CLI
        // ... etc
    }
}
```

### Step 4: Create Contract Enforcement Tests
```rust
#[test]
fn enforce_uniform_contracts() {
    // For every command, verify:
    // 1. CLI parameters match contract
    // 2. MCP parameters match contract
    // 3. HTTP parameters match contract
    // 4. No extra parameters in any interface
    // 5. No missing parameters in any interface
}
```

### Step 5: Add to CI/CD
```yaml
# .github/workflows/contract-enforcement.yml
- name: Enforce Uniform Contracts
  run: |
    cargo test --test contract_uniformity
    cargo run --bin contract-validator
```

## Migration Plan

### Phase 1: Add Contracts Module (DONE)
- ✅ Created `src/contracts/mod.rs` with unified definitions
- ✅ Created validation traits
- ✅ Created mapping modules

### Phase 2: Create Adapters (IN PROGRESS)
- 🔄 Map existing CLI to contracts
- 🔄 Map existing MCP to contracts
- 🔄 Add deprecation warnings for old parameters

### Phase 3: Refactor Commands
- [ ] Update all CLI command definitions
- [ ] Update all MCP tool definitions
- [ ] Update all HTTP endpoints

### Phase 4: Enforce in CI
- [ ] Add contract validation to pre-commit hooks
- [ ] Add contract tests to CI pipeline
- [ ] Block PRs that violate contracts

## Benefits

1. **Consistency**: Users learn one set of parameters for all interfaces
2. **Composability**: Tools can be chained together easily
3. **Documentation**: Single source of truth for all parameters
4. **Testing**: One set of tests validates all interfaces
5. **Evolution**: Adding new parameters is done once, available everywhere

## Toyota Way Principle

This refactoring follows **Kaizen** - continuous improvement through small, incremental changes. We're not doing a big-bang rewrite, but gradually migrating to uniform contracts while maintaining backward compatibility through adapters.