## Functional Testing Checklist: Remote GitHub Project Analysis

### Test Projects Selection
1. **fd** - `https://github.com/sharkdp/fd` (find alternative, ~10K LOC)
2. **bat** - `https://github.com/sharkdp/bat` (cat with syntax highlighting, ~15K LOC)
3. **exa** - `https://github.com/ogham/exa` (modern ls replacement, ~12K LOC)

### 1. Remote Repository Access & Cloning

#### 1.1 URL Parsing & Validation
- [ ] **Valid HTTPS URL** - `https://github.com/sharkdp/fd.git`
- [ ] **URL without .git suffix** - `https://github.com/sharkdp/fd`
- [ ] **SSH URL conversion** - `git@github.com:sharkdp/fd.git` → HTTPS
- [ ] **Branch-specific URL** - `https://github.com/sharkdp/fd/tree/master`
- [ ] **Subpath URL rejection** - `https://github.com/sharkdp/fd/src/main.rs`
- [ ] **Rate limit handling** - 60 requests/hour unauthenticated
- [ ] **Network timeout** - 30s clone timeout, 3 retry attempts

#### 1.2 Clone Performance Boundaries
```
- [ ] Small repo (<50MB): Complete in <10s
- [ ] Medium repo (50-200MB): Complete in <30s
- [ ] Shallow clone fallback: --depth=1 after 1st timeout
- [ ] Disk space check: 3x repo size available
- [ ] Temp directory cleanup: /tmp/pmat-* removed on exit
```

### 2. Repository Structure Detection

#### 2.1 Build System Recognition
- [ ] **Cargo.toml at root** - Primary Rust indicator
- [ ] **Workspace detection** - `[workspace]` section parsing
- [ ] **Binary vs Library** - `[[bin]]` vs `[lib]` differentiation
- [ ] **Multiple binaries** - All `src/bin/*.rs` discovered
- [ ] **Build dependencies** - `[build-dependencies]` impact on complexity

#### 2.2 Language Detection Accuracy
```rust
// Expected detection for mixed codebases:
- [ ] .rs files → Rust AST parser
- [ ] .py files → Python AST parser  
- [ ] .js/.ts → TypeScript parser
- [ ] .md files → Excluded from AST, included in context
- [ ] Binary files → Shannon entropy >7.0 rejection
```

### 3. AST Analysis Completeness

#### 3.1 Rust Symbol Extraction
- [ ] **Public API surface** - All `pub` items counted
- [ ] **Trait implementations** - `impl Trait for Type` captured
- [ ] **Macro definitions** - `macro_rules!` complexity estimation
- [ ] **Generic constraints** - `where T: Clone + Send` parsed
- [ ] **Async functions** - `async fn` cognitive complexity +2
- [ ] **Unsafe blocks** - Additional complexity weight +3

#### 3.2 Cross-File Dependencies
```rust
// Must capture:
- [ ] use statements → import graph
- [ ] mod declarations → module hierarchy
- [ ] extern crate → external dependencies
- [ ] Path imports → resolves to canonical module
- [ ] Glob imports → warns about * usage
```

### 4. Git History Analysis

#### 4.1 Churn Metrics Extraction
- [ ] **Commit parsing** - Last 1000 commits or 1 year
- [ ] **Author deduplication** - Email normalization
- [ ] **File rename tracking** - `--follow` flag active
- [ ] **Merge commit handling** - `--first-parent` for mainline
- [ ] **Binary file exclusion** - No churn for .png, .pdf

#### 4.2 Performance Boundaries
```
- [ ] <1000 commits: Process all
- [ ] 1000-10000: Sample every 5th commit
- [ ] >10000: Last 2000 + statistical sampling
- [ ] Memory limit: <500MB for git operations
- [ ] Incremental processing: Cache SHA-1 hashes
```

### 5. Complexity Analysis Validation

#### 5.1 Cyclomatic Complexity
- [ ] **match expressions** - Each arm = +1
- [ ] **if let** chains - Proper nesting count
- [ ] **? operator** - Counts as branch
- [ ] **Iterator chains** - .filter().map() = +2
- [ ] **Closure complexity** - Nested scope analysis

#### 5.2 Cognitive Complexity
```rust
// Verify weights:
- [ ] Recursion: +10 per occurrence
- [ ] Nested loops: +depth² scoring  
- [ ] Break/continue: +1 + nesting level
- [ ] Macro expansion: Estimated from usage
- [ ] Boolean operators: && || sequences
```

### 6. Dead Code Detection

#### 6.1 Entry Point Discovery
- [ ] **main() in src/main.rs** - Primary entry
- [ ] **#[test] functions** - Not dead in cfg(test)
- [ ] **#[bench] functions** - Bench profile awareness
- [ ] **pub API in lib.rs** - External interface
- [ ] **Build scripts** - build.rs always live

#### 6.2 Cross-Language References
```
- [ ] FFI exports (#[no_mangle]) → Always live
- [ ] WASM bindings (#[wasm_bindgen]) → Preserved  
- [ ] Serde derives → Field usage inference
- [ ] Macro-generated code → Conservative marking
- [ ] Conditional compilation → cfg() analysis
```

### 7. Technical Debt Identification

#### 7.1 SATD Pattern Matching
- [ ] **TODO/FIXME extraction** - With context ±2 lines
- [ ] **HACK/WORKAROUND** - Severity: High
- [ ] **DEPRECATED markers** - Links to replacements
- [ ] **Performance notes** - "slow", "optimize", "bottleneck"
- [ ] **Safety concerns** - "unsafe", "race", "leak"

#### 7.2 Rust-Specific Debt
```rust
// Patterns to detect:
- [ ] .unwrap() chains → Panic risk
- [ ] .clone() proliferation → Memory overhead
- [ ] Box<dyn Any> → Type erasure debt
- [ ] Mutex<Vec<T>> → Concurrency bottleneck
- [ ] todo!() macro → Incomplete implementation
```

### 8. Mermaid Diagram Generation

#### 8.1 Graph Constraints
- [ ] **Node limit: 150** - PageRank pruning active
- [ ] **Edge budget: 200** - Transitive reduction
- [ ] **Label escaping** - Special chars in quotes
- [ ] **Subgraph creation** - Module clustering
- [ ] **Complexity coloring** - Red/yellow/green gradients

#### 8.2 Determinism Verification
```
- [ ] Identical input → Identical output (hash check)
- [ ] Node ordering → Alphabetical by ID
- [ ] Edge deduplication → Set-based operations
- [ ] Float comparison → 1e-6 epsilon for PageRank
- [ ] Seed-based layout → Fixed random seed
```

### 9. Performance & Resource Boundaries

#### 9.1 Memory Constraints
- [ ] **AST cache size** - Max 100MB per language
- [ ] **Git object cache** - 50MB rolling window
- [ ] **Mermaid string** - <1MB output size
- [ ] **Peak memory** - <2GB for 50K LOC project
- [ ] **OOM handling** - Graceful degradation

#### 9.2 Time Boundaries
```
Project Size | Total Time | AST Parse | Git Analysis
------------ | ---------- | --------- | ------------
<10K LOC     | <30s       | <10s      | <5s
10-50K       | <120s      | <40s      | <20s
50-100K      | <300s      | <120s     | <60s
```

### 10. Output Validation

#### 10.1 Report Completeness
- [ ] **File count matches** - find . -name "*.rs" | wc -l
- [ ] **Symbol count sanity** - >0 for non-empty files
- [ ] **Complexity ranges** - 1-100 typical bounds
- [ ] **TDG calculation** - Score correlates with complexity
- [ ] **No empty sections** - All analyzers produce output

#### 10.2 Error Handling Matrix
```
Scenario              | Expected Behavior       | Validation
--------------------- | ----------------------- | ----------
Network timeout       | Retry 3x, then fail     | Error message
Invalid URL           | Immediate rejection     | Parse error
Empty repository      | Minimal report          | Zero counts
No Rust files         | Language notice         | Warning log
Huge repo (>1GB)      | Shallow clone           | Depth limit
Private repo          | Auth error              | 404 handling
Rate limited          | Backoff + retry         | 429 status
Corrupted git         | Partial analysis        | Skip history
```

### 11. Regression Testing

#### 11.1 Cross-Version Stability
- [ ] **ripgrep v13 vs v14** - Consistent metrics ±5%
- [ ] **Renamed files** - History tracking maintained
- [ ] **Force push handling** - Orphaned commit detection
- [ ] **Submodule analysis** - Optional inclusion flag

#### 11.2 Edge Cases
```rust
// Must handle:
- [ ] Single-file projects
- [ ] 10MB single file parsing  
- [ ] Unicode identifiers (χ²)
- [ ] Proc macro heavy code
- [ ] 500+ file project structure
- [ ] Cyclic module dependencies
- [ ] Workspace with 20 crates
```

### Test Execution Protocol
```bash
# For each test project:
pmat analyze deep-context \
  --repo-url "$URL" \
  --include-git \
  --include-proofs \
  --output-path "./test-results/${PROJECT}.md" \
  2>&1 | tee "./test-logs/${PROJECT}.log"

# Validate: jq '.exit_code == 0' < test-results.json
```