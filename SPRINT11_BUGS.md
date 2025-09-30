# Sprint 11: Multi-Language Deep Context Bugs

## Critical Bug: Go and TypeScript Files Not Analyzed in deep_context.md

### Description
The `pmat context --output deep_context.md` command fails to extract and display AST information for Go and TypeScript files, despite having full AST parsing support compiled in.

### Evidence
When running `pmat context --output deep_context.md` on `/home/noah/src/agentic-ai`:

**Go files show NO analysis:**
```markdown
### ./go-actors/main.go

### ./go-actors/simple.go

### ./go-actors/simple_test.go
```

**TypeScript files show INCORRECT analysis** (parsed as Rust):
```markdown
### ./deno-actors/simple.ts

**File Complexity**: 6 | **Functions**: 6

- **Trait**: `SimpleMessage`  # <-- This is an interface in TS, not a Rust trait!
- **Struct**: `Channel` [fields: 2]  # <-- This is a class in TS, not a Rust struct!
- **Function**: `Channel::send` [complexity: 1]  # <-- Should be Channel.send (TS method)
```

### Root Cause Analysis

1. **Features ARE Enabled**: Verified that `typescript-ast` and `go-ast` are in the `all-languages` feature set which is enabled by default

2. **AST Parsing Code EXISTS**:
   - `/server/src/services/simple_deep_context.rs:336-447` has TypeScript AST parsing
   - `/server/src/services/simple_deep_context.rs:540+` has Go AST parsing

3. **Suspected Issues**:
   - **Issue #1**: Unified context builder may not be calling `analyze_file_complexity()` for all files
   - **Issue #2**: Language detection may be incorrectly identifying files
   - **Issue #3**: Markdown generation may be skipping non-Rust AST items
   - **Issue #4**: TypeScript being parsed as Rust suggests language detection failure

### Test Case
```bash
cd /home/noah/src/agentic-ai
pmat context --output deep_context.md
cat deep_context.md  # Shows empty Go files and incorrectly parsed TS files
```

### Expected Behavior

**Go files should show:**
```markdown
### ./go-actors/simple.go

**File Complexity**: 1 | **Functions**: 2

- **Struct**: `SimpleMessage` [fields: 2]
- **Function**: `SimplePingPong` [complexity: 5] [cognitive: 8] [big-o: O(n)]
```

**TypeScript files should show:**
```markdown
### ./deno-actors/simple.ts

**File Complexity**: 6 | **Functions**: 6

- **Interface**: `SimpleMessage` [properties: 2]
- **Class**: `Channel<T>` [methods: 2]
- **Method**: `Channel.send` [complexity: 2] [cognitive: 2]
- **Method**: `Channel.receive` [complexity: 3] [cognitive: 3]
- **Function**: `simplePingPong` [complexity: 1] [cognitive: 1]
- **Function**: `pingActor` [complexity: 3] [cognitive: 4]
- **Function**: `pongActor` [complexity: 3] [cognitive: 4]
```

### Impact
**CRITICAL - Sprint 11 Priority #1**

- Advertised multi-language support is broken
- Users cannot get proper analysis for 40% of supported languages
- Deep context for polyglot projects is incomplete/wrong
- False advertising in v2.103.0 release

### Affected Languages
Based on review of simple_deep_context.rs:
- ✅ **Rust**: WORKING (full AST analysis)
- ❌ **Go**: BROKEN (empty output)
- ❌ **TypeScript/JavaScript**: BROKEN (incorrectly parsed as Rust or empty)
- ❓ **Python**: NEEDS VERIFICATION
- ❓ **C/C++**: NEEDS VERIFICATION
- ❓ **Java**: NEEDS VERIFICATION
- ❓ **C#**: NEEDS VERIFICATION
- ❓ **Kotlin**: NEEDS VERIFICATION
- ❓ **Ruby**: NEEDS VERIFICATION
- ❓ **Shell**: NEEDS VERIFICATION
- ❓ **WASM**: NEEDS VERIFICATION

### Sprint 11 Tasks

#### Phase 1: Diagnosis (2 hours)
- [ ] **TASK-1**: Add debug logging to unified context builder to track which files are analyzed
- [ ] **TASK-2**: Add debug logging to language detection to verify correct language identification
- [ ] **TASK-3**: Run test suite for each language analyzer individually
- [ ] **TASK-4**: Verify feature flags are actually compiled in (check cargo build output)

#### Phase 2: Fix Go Support (4 hours)
- [ ] **TASK-5**: Write EXTREME TDD test for Go file in deep_context
- [ ] **TASK-6**: Fix unified context builder to call Go AST analysis
- [ ] **TASK-7**: Fix markdown generator to output Go AST items correctly
- [ ] **TASK-8**: Verify Go analysis matches tree-sitter-go output

#### Phase 3: Fix TypeScript Support (4 hours)
- [ ] **TASK-9**: Write EXTREME TDD test for TypeScript file in deep_context
- [ ] **TASK-10**: Fix language detection - TypeScript being parsed as Rust
- [ ] **TASK-11**: Fix unified context builder to call TypeScript AST analysis
- [ ] **TASK-12**: Fix markdown generator to output TypeScript/JavaScript AST items correctly
- [ ] **TASK-13**: Verify TypeScript analysis matches SWC parser output

#### Phase 4: Verify All Languages (6 hours)
- [ ] **TASK-14**: Create test project with files in ALL 12 supported languages
- [ ] **TASK-15**: Run `pmat context` on multi-language project
- [ ] **TASK-16**: Verify each language shows proper AST analysis in output
- [ ] **TASK-17**: Fix any additional broken languages discovered
- [ ] **TASK-18**: Add integration test for multi-language deep_context
- [ ] **TASK-19**: Update ROADMAP.md with language support status

#### Phase 5: Documentation & Release (2 hours)
- [ ] **TASK-20**: Document which AST items each language supports
- [ ] **TASK-21**: Update README with accurate language support matrix
- [ ] **TASK-22**: Add examples of each language's deep_context output
- [ ] **TASK-23**: Bump version to 2.104.0 and release fix

### Acceptance Criteria

1. ✅ All 12 supported languages show AST details in deep_context.md
2. ✅ No language is incorrectly parsed as another language
3. ✅ Function names, complexity, and Big-O are shown for all languages
4. ✅ Struct/class/interface definitions are shown for all languages
5. ✅ All language-specific tests pass
6. ✅ Integration test covers multi-language project
7. ✅ Documentation accurately reflects what works

### Estimated Effort
**18 hours** (2-3 days with EXTREME TDD)

### Priority
**CRITICAL** - This breaks a core feature for polyglot projects

---
*Created: 2025-09-30*
*Detected by: deep_context analysis of agentic-ai project*
*Status: Ready for Sprint 11*