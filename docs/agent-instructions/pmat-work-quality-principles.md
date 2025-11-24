# DETERMINISTIC Instructions: `pmat work` Quality Principles

## MANDATORY Quality Gates for `pmat work complete`

### 1. Rust Project Requirements ✅ IMPLEMENTED

**IF project contains Cargo.toml**:

```bash
# Before completing work, MUST verify:
cargo run --example <relevant_example>  # Test examples work
cargo test --examples                   # All examples compile
pmat rust-project-score                 # Capture score (O(1) cached)
```

**Implementation**:
```rust
// File: server/src/cli/handlers/work_handlers.rs
// Function: run_quality_gates()
// Add before clippy:

// Check if Rust project
if project_path.join("Cargo.toml").exists() {
    println!("   🦀 Rust project detected...");

    // 1. Test examples compile
    println!("      📦 Checking examples...");
    let examples = Command::new("cargo")
        .args(&["test", "--examples", "--no-run"])
        .current_dir(project_path)
        .status()?;

    if !examples.success() {
        println!("      ❌ Examples failed to compile");
        all_passed = false;
    } else {
        println!("      ✅ Examples compile");
    }

    // 2. Capture rust-project-score (O(1) from cache)
    println!("      📊 Capturing rust-project-score...");
    let score = Command::new("pmat")
        .args(&["rust-project-score", "--format", "json"])
        .current_dir(project_path)
        .output()?;

    // Store score in .pmat-metrics/ for commit linking
    // (Implementation below)
}
```

---

### 2. Five Whys - ONLY Debugging Method

**CRITICAL**: Five Whys is the ONLY acceptable debugging approach.

**NEVER**:
- ❌ Random print statements
- ❌ Guessing root cause
- ❌ Applying fixes without understanding

**ALWAYS**:
```bash
# When debugging ANY issue:
pmat five-whys --issue "description"

# Example: Stack overflow in test
pmat five-whys --issue "thread panicked: stack overflow in test_multiple_parameter_types"

# Output: Five iterations of "why?" to reach root cause
Why 1: Test recursion exceeds stack limit
Why 2: Deep AST traversal without tail recursion
Why 3: Parser generates deeply nested nodes
Why 4: Grammar allows unlimited nesting
Why 5: No max depth validation in parser config

Root Cause: Parser lacks max_depth parameter
Fix: Add max_depth: 1000 to parser config
```

**Enforcement**:
- `pmat work complete` MUST ask: "Did you use Five Whys for any bugs?"
- Commit message MUST include Five Whys output if debugging occurred

---

### 3. Renacer Golden Tracing - Transpile/Distributed Projects ✅ IMPLEMENTED

**MANDATORY for**:
- Transpilers (Rust→JS, Python→C, etc.)
- Distributed systems
- Multi-process workflows
- Cross-language integrations

**Golden Tracing** = Record expected execution traces, validate against future runs

**Setup**:
```toml
# renacer.toml (project root)
[golden_traces]
enabled = true
trace_dir = "golden_traces/"

[[golden_traces.scenarios]]
name = "transpile_rust_to_js"
command = "pmat transpile --input test.rs --output test.js"
golden_trace = "golden_traces/transpile_rust_to_js.trace"

[[golden_traces.scenarios]]
name = "distributed_worker_handshake"
command = "cargo run --example distributed_demo"
golden_trace = "golden_traces/worker_handshake.trace"
```

**Capture Golden Trace**:
```bash
# First run: Capture golden trace
renacer capture --scenario transpile_rust_to_js

# Creates: golden_traces/transpile_rust_to_js.trace
# Contains: function calls, I/O, timing, state transitions
```

**Validate**:
```bash
# Before completing work
renacer validate --all

# Output:
✅ transpile_rust_to_js: MATCH (100% identical)
❌ distributed_worker_handshake: DIVERGED at step 42
   Expected: Worker sent ACK
   Actual:   Worker timeout after 5s
```

**Integration**:
```rust
// File: server/src/cli/handlers/work_handlers.rs
// Function: run_quality_gates()

// Check for renacer.toml
if project_path.join("renacer.toml").exists() {
    println!("   🎯 Golden traces detected...");

    let renacer = Command::new("renacer")
        .args(&["validate", "--all"])
        .current_dir(project_path)
        .status()?;

    if !renacer.success() {
        println!("      ❌ Golden traces diverged");
        all_passed = false;
    } else {
        println!("      ✅ Golden traces match");
    }
}
```

---

### 4. Commit Metadata Linking (O(1) Capture) ✅ IMPLEMENTED

**EVERY commit MUST link**:
- A. Prompt used (from work item)
- B. TDG score
- C. Repo score
- D. Rust project score (if Rust)

**Implementation**:
```bash
# On `pmat work complete <id>`:

# 1. Capture scores (O(1) from .pmat-metrics/ cache)
pmat tdg-score > .pmat-metrics/commit-$(git rev-parse --short HEAD)-tdg.json
pmat repo-score > .pmat-metrics/commit-$(git rev-parse --short HEAD)-repo.json
pmat rust-project-score > .pmat-metrics/commit-$(git rev-parse --short HEAD)-rust.json

# 2. Create commit metadata file
cat > .pmat-metrics/commit-$(git rev-parse --short HEAD)-meta.json << EOF
{
  "commit_sha": "$(git rev-parse HEAD)",
  "work_item_id": "${ID}",
  "prompt": "$(cat docs/roadmaps/roadmap.yaml | yq '.roadmap[] | select(.id == \"${ID}\") | .title')",
  "tdg_score": $(cat .pmat-metrics/commit-*-tdg.json),
  "repo_score": $(cat .pmat-metrics/commit-*-repo.json),
  "rust_project_score": $(cat .pmat-metrics/commit-*-rust.json),
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

# 3. Add to git commit message footer
git commit -m "feat: ${TITLE}

Work-Item: ${ID}
TDG-Score: ${TDG_SCORE}/100
Repo-Score: ${REPO_SCORE}/100
Rust-Score: ${RUST_SCORE}/134

Metrics: .pmat-metrics/commit-${SHORT_SHA}-meta.json"
```

**Code Implementation**:
```rust
// File: server/src/cli/handlers/work_handlers.rs
// Function: handle_work_complete()
// Add after marking complete, before git commit suggestion:

async fn capture_commit_metadata(
    project_path: &Path,
    item: &RoadmapItem,
) -> Result<CommitMetadata> {
    let short_sha = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .current_dir(project_path)
        .output()?;
    let short_sha = String::from_utf8_lossy(&short_sha.stdout).trim().to_string();

    // Capture scores (O(1) from cache)
    let tdg_score = capture_tdg_score(project_path).await?;
    let repo_score = capture_repo_score(project_path).await?;
    let rust_score = if project_path.join("Cargo.toml").exists() {
        Some(capture_rust_project_score(project_path).await?)
    } else {
        None
    };

    let metadata = CommitMetadata {
        commit_sha: None, // Will be filled after commit
        work_item_id: item.id.clone(),
        prompt: item.title.clone(),
        tdg_score,
        repo_score,
        rust_project_score: rust_score,
        timestamp: chrono::Utc::now(),
    };

    // Write to .pmat-metrics/
    let metrics_dir = project_path.join(".pmat-metrics");
    std::fs::create_dir_all(&metrics_dir)?;

    let meta_file = metrics_dir.join(format!("commit-{}-meta.json", short_sha));
    let json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(meta_file, json)?;

    Ok(metadata)
}

#[derive(Serialize, Deserialize)]
struct CommitMetadata {
    commit_sha: Option<String>,
    work_item_id: String,
    prompt: String,
    tdg_score: f64,
    repo_score: f64,
    rust_project_score: Option<f64>,
    timestamp: chrono::DateTime<chrono::Utc>,
}
```

---

## Updated `pmat work complete` Flow

```bash
pmat work complete <id>

✅ Completing work on: <id>

🔍 Running quality gates...

   🧪 Running tests...
      📋 Testing changed modules: services::progress
      ✅ Tests passed (12 tests, 3s)

   🦀 Rust project detected...
      📦 Checking examples...
      ✅ Examples compile (5 examples)
      📊 Capturing rust-project-score... (82.5/134)

   🎯 Golden traces detected...
      ✅ Golden traces match (3/3 scenarios)

   📎 Running clippy...
      ✅ No clippy warnings

   📊 Capturing commit metadata...
      ✅ TDG Score: 87.3/100
      ✅ Repo Score: 92.1/100
      ✅ Rust Project Score: 82.5/134

✅ Marked as complete: Continue unwrap elimination
✅ Updated roadmap: ./docs/roadmaps/roadmap.yaml
✅ Commit metadata: .pmat-metrics/commit-abc123-meta.json

🎯 Next steps:
   1. git commit -m "feat: Continue unwrap elimination (Refs <id>)

   Work-Item: Continue unwrap elimination: 27 more unwraps...
   TDG-Score: 87.3/100
   Repo-Score: 92.1/100
   Rust-Score: 82.5/134

   Metrics: .pmat-metrics/commit-abc123-meta.json"
```

---

## Agent Instructions for Debugging

**When user reports bug/error**:

```bash
# 1. MANDATORY: Use Five Whys
cat docs/agent-instructions/pmat-work-quality-principles.md
# Section 2: Five Whys

# 2. IF transpile/distributed: Check golden traces
cd golden_traces/
ls *.trace
renacer validate --scenario <relevant>

# 3. Apply fix
# 4. Re-validate golden traces
renacer capture --scenario <relevant>  # Update if behavior changed intentionally

# 5. Document in commit
git commit -m "fix: <issue>

Five Whys Root Cause: <from five-whys output>
Golden Trace: Updated <scenario> (behavior changed as intended)
"
```

---

## Files to Modify

1. `server/src/cli/handlers/work_handlers.rs`
   - Add `capture_commit_metadata()`
   - Add Rust example checking to `run_quality_gates()`
   - Add renacer validation to `run_quality_gates()`

2. `CLAUDE.md`
   - Add section: "Five Whys - ONLY Debugging Method"
   - Add section: "Renacer Golden Tracing - Transpile/Distributed"
   - Link to this document

3. `.gitignore`
   - Exclude `.pmat-metrics/*.json` (too many files)
   - Keep `.pmat-metrics/*-meta.json` (commit metadata tracked)

---

## Priority Implementation Order

1. ✅ **URGENT**: Five Whys documentation (CLAUDE.md) - COMPLETED
2. ✅ **HIGH**: Commit metadata capture - COMPLETED
3. ✅ **MEDIUM**: Rust example validation - COMPLETED
4. ✅ **LOW**: Renacer integration - COMPLETED

**ALL TASKS COMPLETED! ✅**

---

## Success Criteria ✅ ALL ACHIEVED

After implementation:
- ✅ `pmat work complete` captures all scores (O(1)) - IMPLEMENTED
- ✅ Commit messages include Work-Item + scores - IMPLEMENTED
- ✅ Rust projects validate examples - IMPLEMENTED
- ✅ Five Whys mandatory for debugging - DOCUMENTED in CLAUDE.md
- ✅ Renacer golden traces validated (if applicable) - IMPLEMENTED
- ✅ `.pmat-metrics/commit-*-meta.json` created per commit - IMPLEMENTED

**Implementation Date**: 2025-11-24
**Status**: Production-ready, all quality principles integrated into `pmat work complete`
