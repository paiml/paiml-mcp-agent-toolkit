# DETERMINISTIC Instructions: `pmat work` UX Improvements

## Context
The `pmat work` command has poor UX due to:
1. No short IDs (must type full titles)
2. Quality gates too strict (blocks on unrelated tests)
3. IDs not shown in status output
4. Exact string matching required

## DETERMINISTIC Mitigation Steps

### Issue 1: Long IDs Required
**Problem**: `pmat work continue 'Continue unwrap elimination: 27 more unwraps...'`

**DETERMINISTIC FIX**:
```rust
// File: server/src/models/roadmap.rs
// Location: Line 201, function find_item()

// BEFORE:
pub fn find_item(&self, id: &str) -> Option<&RoadmapItem> {
    self.roadmap.iter().find(|item| item.id == id)
}

// AFTER (add fuzzy matching):
pub fn find_item(&self, id: &str) -> Option<&RoadmapItem> {
    let id_lower = id.to_lowercase();

    // 1. Exact match (fastest)
    if let Some(item) = self.roadmap.iter().find(|item| item.id == id) {
        return Some(item);
    }

    // 2. Case-insensitive match
    if let Some(item) = self.roadmap.iter().find(|item| item.id.to_lowercase() == id_lower) {
        return Some(item);
    }

    // 3. Partial match (prefix)
    if let Some(item) = self.roadmap.iter().find(|item|
        item.id.to_lowercase().starts_with(&id_lower)
    ) {
        return Some(item);
    }

    // 4. Contains match (last resort)
    self.roadmap.iter().find(|item|
        item.id.to_lowercase().contains(&id_lower)
    )
}
```

**TESTING**:
```rust
// File: server/src/models/roadmap.rs (tests module)
#[test]
fn test_fuzzy_id_matching() {
    let mut roadmap = Roadmap::default();
    roadmap.roadmap.push(RoadmapItem {
        id: "Continue unwrap elimination: 27 more unwraps".to_string(),
        // ... other fields
    });

    // Exact match
    assert!(roadmap.find_item("Continue unwrap elimination: 27 more unwraps").is_some());

    // Partial match
    assert!(roadmap.find_item("unwrap elimination").is_some());

    // Case-insensitive
    assert!(roadmap.find_item("UNWRAP").is_some());

    // Prefix match
    assert!(roadmap.find_item("Continue").is_some());
}
```

### Issue 2: Auto-Generate Short IDs
**Problem**: Current IDs are full titles

**DETERMINISTIC FIX**:
```rust
// File: server/src/models/roadmap.rs
// Add method to RoadmapItem impl

impl RoadmapItem {
    /// Generate a short ID from title (e.g., "unwrap-elim-001")
    pub fn generate_short_id(title: &str, index: usize) -> String {
        // Take first 3 words, truncate to 12 chars, slugify
        let words: Vec<&str> = title
            .split_whitespace()
            .take(3)
            .collect();

        let slug = words
            .join("-")
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(12)
            .collect::<String>();

        format!("{}-{:03}", slug, index + 1)
    }
}
```

**TESTING**:
```rust
#[test]
fn test_short_id_generation() {
    assert_eq!(
        RoadmapItem::generate_short_id("Continue unwrap elimination with EXTREME TDD", 0),
        "continue-unw-001"
    );

    assert_eq!(
        RoadmapItem::generate_short_id("Fix critical bugs in parser", 5),
        "fix-critical-006"
    );
}
```

### Issue 3: Status Display Shows IDs
**Problem**: `pmat work status` doesn't show IDs

**DETERMINISTIC FIX**:
```rust
// File: server/src/cli/handlers/work_handlers.rs
// Function: handle_work_status()
// Location: Search for "println!" in status display

// BEFORE:
println!("   {} {} - {} ({}%)",
    status_icon,
    item.title,
    item.status,
    progress
);

// AFTER:
println!("   {} [{}] {} - {} ({}%)",
    status_icon,
    item.id,          // <- ADD THIS
    item.title,
    item.status,
    progress
);
```

**Example Output**:
```
✅ [gh-75] Unify roadmap and GitHub (100%)
⏳ [unwrap-001] Continue unwrap elimination (50%)
```

### Issue 4: Quality Gates Too Strict
**Problem**: `pmat work complete` blocks on unrelated test failures

**DETERMINISTIC FIX A (Default --skip-quality)**:
```rust
// File: server/src/cli/commands.rs
// Find WorkComplete command struct

#[derive(Parser)]
pub struct WorkComplete {
    pub id: String,

    // BEFORE:
    // #[arg(long)]
    // pub skip_quality: bool,

    // AFTER (default to true):
    #[arg(long, default_value = "true")]
    pub run_quality: bool,  // Inverted logic
}
```

**DETERMINISTIC FIX B (Git-aware testing)**:
```rust
// File: server/src/cli/handlers/work_handlers.rs
// Add function before handle_work_complete

/// Run tests only for changed files
async fn run_targeted_tests(project_path: &Path) -> Result<bool> {
    use std::process::Command;

    // Get list of changed files
    let output = Command::new("git")
        .args(&["diff", "--name-only", "HEAD"])
        .current_dir(project_path)
        .output()?;

    let changed_files = String::from_utf8_lossy(&output.stdout);

    // Extract changed modules
    let modules: Vec<String> = changed_files
        .lines()
        .filter(|line| line.ends_with(".rs"))
        .filter_map(|path| {
            // Extract module path from file path
            // e.g., "server/src/services/progress.rs" -> "services::progress"
            path.strip_prefix("server/src/")?
                .strip_suffix(".rs")?
                .replace('/', "::")
                .into()
        })
        .collect();

    if modules.is_empty() {
        println!("   ℹ️  No Rust files changed, skipping tests");
        return Ok(true);
    }

    println!("   🧪 Running tests for changed modules: {}", modules.join(", "));

    // Run cargo test with module filters
    let test_result = Command::new("cargo")
        .args(&["test", "--lib", &modules.join("|")])
        .current_dir(project_path)
        .status()?;

    Ok(test_result.success())
}
```

**Usage in handle_work_complete**:
```rust
// Replace full test run with:
if run_quality {
    println!("🔍 Running quality gates...");
    println!();

    // Use targeted testing
    let tests_pass = run_targeted_tests(&project_path).await?;

    if !tests_pass {
        println!("   ❌ Tests failed for changed modules");
        println!();
        println!("💡 Tip: Use --no-run-quality to skip quality gates");
        return Err(anyhow::anyhow!("Quality gates failed"));
    }

    println!("   ✅ Tests passed");
}
```

## Agent Usage Instructions

### When User Reports UX Issues with `pmat work`:

1. **Identify the specific UX issue**:
   - Long IDs? → Apply Issue 1 fix
   - Can't find items? → Apply Issue 1 + 2 fixes
   - Status unclear? → Apply Issue 3 fix
   - Quality gates blocking? → Apply Issue 4 fix

2. **Apply fixes in this order**:
   ```bash
   # 1. Add fuzzy matching (fastest win)
   # Edit: server/src/models/roadmap.rs:201

   # 2. Update status display
   # Edit: server/src/cli/handlers/work_handlers.rs (status function)

   # 3. Fix quality gates
   # Edit: server/src/cli/commands.rs
   # Edit: server/src/cli/handlers/work_handlers.rs

   # 4. Add short ID generation (larger change)
   # Edit: server/src/models/roadmap.rs (add method)
   ```

3. **Test each fix independently**:
   ```bash
   # After each change:
   cargo test roadmap::tests
   cargo build --bin pmat

   # Manual test:
   pmat work status
   pmat work continue unwrap  # Test fuzzy match
   ```

4. **Commit atomically**:
   ```bash
   git add server/src/models/roadmap.rs
   git commit -m "fix(ux): Add fuzzy ID matching to pmat work commands

   - Support partial match (prefix/contains)
   - Case-insensitive matching
   - Fallback to exact match first (performance)

   Fixes: typing full titles for pmat work continue"
   ```

## Success Criteria

After fixes:
- ✅ `pmat work continue unwrap` works (partial match)
- ✅ `pmat work status` shows `[id]` before title
- ✅ `pmat work complete` doesn't block on unrelated tests
- ✅ Tests pass: `cargo test roadmap::tests`

## Priority Order

1. **URGENT**: Fuzzy ID matching (Issue 1) - 15 min fix
2. **HIGH**: Status display (Issue 3) - 5 min fix
3. **MEDIUM**: Quality gates (Issue 4) - 30 min fix
4. **LOW**: Short ID generation (Issue 2) - 1 hour (optional, fuzzy matching solves UX)
