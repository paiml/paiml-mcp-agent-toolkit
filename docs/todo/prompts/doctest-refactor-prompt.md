# Speed Doctest Addition Prompt

## Objective
Rapidly add doctests to `pmat` public APIs using a fast, iterative approach. Focus on quick wins that immediately improve documentation coverage. Commit and push after EVERY doctest.

## Fast Target Discovery (30 seconds max)

```bash
# Find constructors (highest value)
rg "pub fn new\(" server/src --type rust | grep -v "///" | head -10

# Find from_* and builder functions
rg "pub fn from_|pub fn builder\(" server/src --type rust | grep -v "///" | head -10

# Find other public functions (exclude new/from_)
rg "pub fn " server/src --type rust | grep -v "///" | grep -v "new\|from_" | head -10

# Find simple getters/checkers
rg "pub fn get_|pub fn is_|pub fn has_" server/src --type rust | grep -v "///" | head -10

# Find calculation/parsing functions
rg "pub fn parse|pub fn analyze|pub fn calculate" server/src --type rust | grep -v "///" | head -10
```

## Speed Workflow (2-5 minutes per function)

### 1. Find Target (10 seconds)
```bash
# Pick first pub fn you see without docs
rg "pub fn" server/src/services/deep_context.rs | grep -v "///" | head -1
```

### 2. Read Function (20 seconds)
- Check signature
- Understand basic purpose
- Note if it's: constructor, getter, transformation, or fallible

### 3. Write Minimal Doctest (60 seconds)

#### For Constructors (`new`, `builder`, `from_*`)
```rust
/// Creates a new [StructName]
///
/// # Examples
/// 
/// ```
/// use pmat::path::to::StructName;
/// 
/// let instance = StructName::new();
/// // One assertion to prove it works
/// assert_eq!(instance.some_method(), expected_value);
/// ```
```

#### For Simple Getters/Pure Functions
```rust
/// Returns the [property description]
///
/// # Examples
/// 
/// ```
/// use pmat::path::to::function;
/// 
/// assert_eq!(function("input"), "expected_output");
/// assert_eq!(function("edge_case"), "edge_result");
/// ```
```

#### For Fallible Operations
```rust
/// [Action description]
///
/// # Examples
/// 
/// ```
/// use pmat::path::to::Type;
/// 
/// let result = Type::try_something("valid")?;
/// assert!(result.is_valid());
/// 
/// assert!(Type::try_something("invalid").is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

### 4. Quick Test (30 seconds)
```bash
# Just compile the doctest - don't run full suite
cargo test --doc --no-run

# If it compiles, run just that test
cargo test --doc -- path::to::module
```

### 5. Verify Changes (60 seconds)
```bash
# Run lint check
make lint

# Run doctests for the specific module
cargo test --doc -- module_name

# If all passes, proceed to commit
```

### 6. Commit and Push (30 seconds)
```bash
# Stage, commit and push in one go
git add server/src/path/to/file.rs && \
git commit -m "docs: Add doctest for function_name

🤖 Generated with Claude Code
https://claude.ai/code

Co-Authored-By: Claude <noreply@anthropic.com>" && \
git push origin master
```

## Speed Heuristics

### Always Worth Doctesting
- `new()` functions
- `builder()` patterns  
- `from_*()` conversions
- Public trait implementations
- Functions with Result/Option returns

### Skip These (for now)
- Internal helper functions
- Simple getters that return primitives
- Functions marked `#[doc(hidden)]`
- Test utilities

## Minimal Quality Bar

✅ **Must Have:**
- Compiles without errors
- At least one assertion
- Uses correct import path

❌ **Don't Need (for speed):**
- Edge case coverage
- Performance tests
- Comprehensive error handling
- Multiple scenarios

## Example Speed Run

```bash
# 1. Find target (10s)
$ rg "pub fn new\(" server/src/services/deep_context.rs
578:    pub fn new(config: DeepContextConfig) -> Self {

# 2. Read function (20s) 
$ sed -n '578,585p' server/src/services/deep_context.rs

# 3. Add doctest (60s)
# 4. Quick verify (30s)
$ cargo test --doc --package pmat -- deep_context::DeepContextAnalyzer::new 2>&1 | grep "test result"

# 5. Commit and push (30s)
$ git add server/src/services/deep_context.rs && \
  git commit -m "docs: Add doctest for DeepContextAnalyzer::new" && \
  git push origin master

# Total: ~2.5 minutes
```

## Key Success Factors

### 1. One Doctest Per Commit
- Add ONE doctest
- Test it immediately
- Commit and push right away
- This prevents losing work and maintains momentum

### 2. Minimal Viable Doctests
```rust
/// Brief description
///
/// # Examples
/// 
/// ```
/// use pmat::path::to::Type;
/// 
/// let instance = Type::new();
/// // One simple assertion
/// ```
```

### 3. Quick Testing
```bash
# Test just the specific doctest (faster)
cargo test --doc --package pmat -- module::Type::function 2>&1 | grep "test result"
```

### 4. Efficient Commit Pattern
```bash
# Use command chaining for speed
git add file.rs && git commit -m "docs: Add doctest for function_name

🤖 Generated with Claude Code
https://claude.ai/code

Co-Authored-By: Claude <noreply@anthropic.com>" && git push origin master
```

## Results
Using this approach, you can add 20-30 doctests in 30 minutes, each properly tested and pushed to GitHub.
