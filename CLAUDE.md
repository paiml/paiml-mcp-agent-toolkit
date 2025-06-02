# CLAUDE.md

Updates are performed via `gh "Simple Release"`

## Dynamic Context Analysis Protocol

**MANDATORY INITIALIZATION SEQUENCE**:
```bash
# Extract current complexity distribution
awk '/## Complexity Hotspots/,/^##/ {if(/^##/ && NR>1) exit; print}' deep_context.md | \
  awk -F'|' 'NR>2 && $4~/[0-9]/ {print $4,$3}' | sort -rn | head -10

# Identify active technical debt vectors
find docs/bugs -name "*.md" -not -path "*/archived/*" -exec basename {} \;

# Load current AST metrics
grep -E "^\*\*Total Symbols:\*\*|^\*\*Functions:\*\*" deep_context.md | head -20
```

## Architectural Invariants

**Rust Workspace Topology**:
```
workspace/
├── Cargo.toml          # Workspace manifest - source of truth for deps
├── server/             # Primary crate: unified protocol implementation
│   ├── src/
│   │   ├── services/   # Stateful business logic - highest complexity density
│   │   ├── unified_protocol/  # Protocol adapters - thin translation layer
│   │   └── handlers/   # Request routing - minimal logic
│   └── build.rs        # Asset compression pipeline
└── target/release/     # Single binary: all three protocols
```

**Protocol Unification Architecture**:
```rust
// Invariant: All protocols converge to unified service layer
trait ProtocolAdapter: Send + Sync {
    type Input: DeserializeOwned;
    type Output: Serialize;
    type Context: Send;
    
    async fn decode(&self, raw: &[u8]) -> Result<Self::Input>;
    async fn process(&self, req: UnifiedRequest) -> Result<UnifiedResponse>;
    async fn encode(&self, resp: UnifiedResponse) -> Result<Self::Output>;
}
```

**Concurrency Model** (Fixed architecture):
- **Async Runtime**: Tokio multi-threaded, work-stealing scheduler
- **Shared State**: `Arc<RwLock<T>>` for service layer, `DashMap` for caches
- **CPU-Bound Tasks**: Rayon thread pool for AST parsing, DAG generation
- **I/O Pattern**: Buffered stdio for MCP, HTTP/2 for web, epoll-based

## Complexity Analysis Methodology

**Dynamic Hotspot Detection**:
```bash
# Extract functions exceeding cognitive complexity threshold
THRESHOLD=30
awk -v t=$THRESHOLD '
  /^[|].*[|].*[|].*[0-9]+.*[|].*[0-9]+.*[|]$/ {
    cog = $(NF-1); 
    if (cog > t) print $0
  }' deep_context.md
```

**Complexity Decomposition Pattern**:
```rust
// Universal refactoring pattern for high-complexity functions
// FROM: Monolithic function with CC > 25
impl Service {
    fn process_complex(&mut self, input: Input) -> Result<Output> {
        // Multiple nested loops, conditions, error paths
    }
}

// TO: Pipeline architecture with composable stages
impl Service {
    fn process(&self, input: Input) -> Result<Output> {
        Pipeline::new(input)
            .validate(self.validators())
            .transform(self.transformers())
            .analyze(self.analyzers())
            .aggregate(self.aggregators())
            .execute()
    }
}
```

## Performance Profiling Framework

**Memory Hierarchy** (Architectural constants):
```rust
// L1: Thread-local caches (nanosecond access)
thread_local! {
    static AST_CACHE: RefCell<LruCache<Blake3Hash, ParsedAst>> = 
        RefCell::new(LruCache::new(NonZeroUsize::new(100).unwrap()));
}

// L2: Process-wide caches (microsecond access)
lazy_static! {
    static ref TEMPLATE_CACHE: DashMap<String, Arc<Template>> = 
        DashMap::with_capacity(1000);
}

// L3: Persistent cache (millisecond access)
struct PersistentCache {
    conn: Arc<Mutex<rusqlite::Connection>>, // WAL mode, mmap enabled
}
```

**Profiling Extraction Commands**:
```bash
# Current memory footprint analysis
grep -A10 "Total Size:" deep_context.md | awk '/[0-9]+ bytes/ {sum+=$1} END {print sum/1048576 " MB"}'

# Concurrency bottleneck detection
grep -B2 -A2 "RwLock\|Mutex\|parking_lot" deep_context.md | grep -v "^--$"
```

## Token-Efficient Navigation Strategies

**Semantic Chunking Protocol**:
```bash
# Extract service layer (business logic concentration)
sed -n '/^### \.\/server\/src\/services\//,/^### \.\/[^s]/ {
    /^### \.\/[^s]/d; p
}' deep_context.md | head -8000

# Extract protocol adapters (thin layer, low complexity)
awk '/unified_protocol\/adapters/,/^### / {print}' deep_context.md | head -3000

# High-value extraction (functions with complexity gradient)
awk '/Cyclomatic/ && $NF > 15 {print; for(i=1;i<=5;i++) {getline; print}}' deep_context.md
```

**Subsystem Isolation Patterns**:
```rust
// Pattern 1: AST Analysis Subsystem
ast_*() -> UnifiedAstNode -> Language-specific visitors

// Pattern 2: Cache Subsystem  
cache::manager -> strategies -> persistent/session/content

// Pattern 3: Template Subsystem
template_service -> renderer -> embedded_templates
```

## Development Workflow Invariants

**Triple-Interface Validation** (Protocol correctness invariant):
```bash
# Semantic equivalence testing across all protocols
test_equivalence() {
    local input='{"method":"analyze_complexity","params":{"path":"."}}'
    
    # CLI interface
    local cli_hash=$(echo "$input" | \
        cargo run -- analyze complexity . --format json | \
        jq -S . | sha256sum)
    
    # MCP interface  
    local mcp_hash=$(echo "$input" | \
        cargo run -- | jq -S .result | sha256sum)
    
    # HTTP interface
    local http_hash=$(curl -s -X POST localhost:3000/api/v1/analyze \
        -H "Content-Type: application/json" -d "$input" | \
        jq -S . | sha256sum)
    
    [[ "$cli_hash" == "$mcp_hash" && "$mcp_hash" == "$http_hash" ]]
}
```

**Incremental Refactoring Protocol**:
1. **Measure**: Extract current complexity metrics dynamically
2. **Identify**: Functions where `cognitive/cyclomatic > 2.5` (high branching)
3. **Decompose**: Apply domain-specific patterns (visitor, pipeline, state machine)
4. **Validate**: Ensure protocol equivalence via triple-interface tests
5. **Benchmark**: Confirm no regression in p99 latency

## Cache Coherency Protocol

**Invalidation Strategy** (Deterministic):
```rust
// Content-addressed caching with mtime validation
fn cache_key(path: &Path, content: &[u8]) -> CacheKey {
    let mtime = fs::metadata(path)?.modified()?;
    let content_hash = blake3::hash(content);
    CacheKey {
        path_hash: blake3::hash(path.as_os_str().as_bytes()),
        content_hash,
        mtime_ns: mtime.duration_since(UNIX_EPOCH)?.as_nanos(),
    }
}
```

**Cache Hierarchy Traversal**:
```rust
async fn get_with_cache<T>(&self, key: &str) -> Result<T> {
    // L1: Thread-local (fastest)
    if let Some(v) = self.thread_cache.get(key) { return Ok(v); }
    
    // L2: Process-wide (fast)
    if let Some(v) = self.shared_cache.get(key) { 
        self.thread_cache.insert(key, v.clone());
        return Ok(v);
    }
    
    // L3: Persistent (slower)
    if let Some(v) = self.persistent_cache.get(key).await? {
        self.shared_cache.insert(key, v.clone());
        self.thread_cache.insert(key, v.clone());
        return Ok(v);
    }
    
    // L4: Compute (slowest)
    let v = self.compute(key).await?;
    self.persist_through_hierarchy(key, &v).await?;
    Ok(v)
}
```

## Critical Path Analysis

**Performance Bottleneck Detection**:
```bash
# Extract I/O-bound operations
grep -E "(tokio::fs|std::fs::read|AsyncRead|AsyncWrite)" deep_context.md | \
    grep -B2 -A2 "async fn"

# Identify lock contention points
grep -E "(\.write\(\)|\.lock\(\)|RwLock.*write|Mutex.*lock)" deep_context.md | \
    awk -F: '{count[$1]++} END {for(f in count) if(count[f]>3) print f, count[f]}'
```

**Optimization Priorities** (Ordered by impact):
1. **Reduce allocations**: Use `SmallVec`, arena allocators for AST nodes
2. **Minimize syscalls**: Batch file operations, use memory-mapped I/O
3. **Lock-free algorithms**: Replace `RwLock` with `ArcSwap` where possible
4. **SIMD opportunities**: String scanning, hash computations

## Correctness Invariants

**Determinism Requirements**:
- **File ordering**: Always sort by UTF-8 byte order
- **Hash stability**: Use platform-independent hashers (blake3)
- **Time handling**: UTC only, nanosecond precision truncated to seconds
- **Floating point**: No float comparisons, use integer scoring

**Safety Boundaries**:
```rust
// All external inputs must pass through validation layer
#[must_use]
fn validate_input<T: Validate>(input: T) -> Result<ValidatedInput<T>> {
    input.validate_structure()?
        .validate_semantics()?  
        .validate_security()?
        .seal()
}
```

**Remember**: This codebase optimizes for deterministic correctness across three protocols. Performance optimizations are secondary to maintaining behavioral equivalence. When analyzing complexity, focus on cognitive load reduction rather than pure cyclomatic metrics—the goal is maintainable code that junior engineers can modify safely.

## Release Process

### Simple Release Workflow (RECOMMENDED)

This project uses an automated GitHub Actions workflow for creating releases. Follow this streamlined process:

#### 1. Prepare for Release
```bash
# Ensure all changes are committed and pushed
git status  # Should show clean working directory
git push origin master

# Verify all tests pass
make lint && make test
```

#### 2. Trigger Automated Release
```bash
# Use GitHub CLI to trigger the Simple Release workflow
gh workflow run "Simple Release" --field version_bump=[patch|minor|major]

# Monitor the release progress
gh run list --limit 5
```

The automated workflow will:
- ✅ **Bump version numbers** in both `Cargo.toml` files automatically
- ✅ **Build optimized binaries** for all supported platforms:
  - `x86_64-unknown-linux-gnu` (Linux x86-64)
  - `aarch64-unknown-linux-gnu` (Linux ARM64)
  - `x86_64-apple-darwin` (Intel Mac)
  - `aarch64-apple-darwin` (Apple Silicon Mac)
- ✅ **Create GitHub release** with auto-generated release notes
- ✅ **Attach binary artifacts** to the release
- ✅ **Tag the repository** with the new version

#### 3. Version Bump Guidelines
Choose the appropriate bump type:

| Change Type | Version Bump | Example | Use Cases |
|-------------|--------------|---------|-----------|
| **patch** | 0.18.2 → 0.18.3 | Bug fixes, documentation, minor improvements | HTTP interface fixes, documentation updates |
| **minor** | 0.18.2 → 0.19.0 | New features, new analysis tools | Deep context analysis, new CLI commands |
| **major** | 0.18.2 → 1.0.0 | Breaking changes, API changes | Protocol breaking changes, major refactoring |

#### 4. Manual Release (Advanced)

If you need to create a release manually for specific reasons:

```bash
# 1. Update documentation first
# Update README.md and RELEASE_NOTES.md with new features

# 2. Commit all changes
git add README.md RELEASE_NOTES.md server/src/
git commit -m "feat: [feature description]

[Detailed implementation description]

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 3. Push changes
git push origin master

# 4. Create release manually
gh release create v[X.Y.Z] \
  --title "v[X.Y.Z]: [Feature Title]" \
  --notes "Release description"
```

### Version Numbering Policy

**DO NOT manually update Cargo.toml version numbers.** The GitHub Actions automatically handle version bumping when changes are pushed.

**Version Scheme:**
- **Major (X.0.0)**: Breaking changes, major architecture changes
- **Minor (0.X.0)**: New features, new analysis tools, significant enhancements  
- **Patch (0.0.X)**: Bug fixes, documentation updates, minor improvements

**Examples:**
- `v0.15.0`: Dead code analysis feature (new analysis tool = minor)
- `v0.14.1`: Bug fixes and documentation (patch)
- `v1.0.0`: Major API breaking changes (major)