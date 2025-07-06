# Speed Example Creation Prompt

## Objective
Rapidly create working examples for `pmat` CLI commands using a fast, iterative approach. Focus on realistic usage scenarios that improve documentation and surface UX inconsistencies. Test each example immediately, then apply to our own codebase.

## Commands Already Processed
**Last Updated**: 2025-07-06

### ✅ Completed Commands
- `pmat analyze complexity` - Fixed AST analysis accuracy issues, validated metrics
  - Fixed base cognitive complexity (was 1, now 0)
  - Fixed double-counting in visitor pattern
  - Fixed nesting level contamination between functions
  - Created validation examples: `complexity_demo.rs`, `complexity_validation.rs`
- `pmat analyze lint-hotspot` - Fixed violation detection and clippy integration
  - Fixed clippy flags (changed from `-D` to `-W` for better collection)
  - Added `--all-targets` to single-file mode to include examples
  - Improved error handling for non-zero clippy exit codes
  - Created validation example: `lint_hotspot_demo.rs` (114 violations)
  - Validated on real codebase: 13,204 violations across 301 files

### 🚧 In Progress
- None currently

### 📋 Not Yet Started
- `pmat analyze satd`
- `pmat analyze dead-code`
- `pmat analyze dag`
- `pmat analyze churn`
- `pmat analyze makefile`
- `pmat quality-gate`
- `pmat refactor auto`
- `pmat refactor extract`
- `pmat report comprehensive`
- `pmat report sarif`
- `pmat enforce standards`
- `pmat diagnose`

## Fast Target Discovery (30 seconds max)

```bash
# Find CLI commands without examples
rg "pub struct.*Command" server/src/cli --type rust | head -10

# Find subcommands in help text
pmat --help | grep -E "^  [a-z]" | head -10

# Find specific analyze subcommands
pmat analyze --help | grep -E "^  [a-z]" | head -10

# Find refactor commands
pmat refactor --help | grep -E "^  [a-z]" | head -10

# Find quality-gate options
pmat quality-gate --help | grep -E "^  " | head -10
```

## Speed Workflow (3-7 minutes per example)

### 1. Pick Command (10 seconds)
```bash
# Start with basic commands first
pmat --help | head -20
```

### 2. Test Command Basics (30 seconds)
```bash
# Run command with --help to understand syntax
pmat [command] --help

# Try minimal invocation to see what happens
pmat [command]
```

### 3. Create Realistic Example (2-3 minutes)

**CRITICAL**: Always create NEW, idiomatic Rust code examples. Never use existing project code as examples.

#### For Analysis Commands
```bash
# Example: complexity analysis
# Create organized examples/ directory structure
mkdir -p examples/analysis

# Create idiomatic Rust executable example (follows cargo run --example pattern)
cat > examples/analysis/complexity_demo.rs << 'EOF'
use std::collections::HashMap;
use std::time::Duration;

/// HTTP client with retry logic and connection pooling
pub struct HttpClient {
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    pub async fn get(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        self.request_with_retry("GET", &url, None).await
    }

    async fn request_with_retry(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut attempts = 0;
        
        loop {
            match self.make_request(method, url, body).await {
                Ok(response) => return Ok(response),
                Err(e) if attempts < self.max_retries => {
                    attempts += 1;
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn make_request(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Simulate HTTP request complexity
        match method {
            "GET" => self.handle_get_request(url).await,
            "POST" => self.handle_post_request(url, body).await,
            "PUT" => self.handle_put_request(url, body).await,
            "DELETE" => self.handle_delete_request(url).await,
            _ => Err("Unsupported HTTP method".into()),
        }
    }

    async fn handle_get_request(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Complex branching for different URL patterns
        if url.contains("/api/v1/") {
            if url.contains("/users/") {
                self.fetch_user_data(url).await
            } else if url.contains("/orders/") {
                self.fetch_order_data(url).await
            } else {
                self.fetch_generic_data(url).await
            }
        } else if url.contains("/health") {
            Ok("OK".to_string())
        } else {
            Err("Invalid endpoint".into())
        }
    }

    async fn fetch_user_data(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simulate user data fetching logic
        Ok(format!("User data from {}", url))
    }

    async fn fetch_order_data(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simulate order data fetching logic  
        Ok(format!("Order data from {}", url))
    }

    async fn fetch_generic_data(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simulate generic data fetching
        Ok(format!("Generic data from {}", url))
    }

    async fn handle_post_request(
        &self,
        url: &str,
        body: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match body {
            Some(data) if !data.is_empty() => {
                if url.contains("/users") {
                    self.create_user(data).await
                } else if url.contains("/orders") {
                    self.create_order(data).await
                } else {
                    Err("Unknown POST endpoint".into())
                }
            }
            _ => Err("POST request requires body".into()),
        }
    }

    async fn create_user(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Complex user creation logic
        if data.len() < 10 {
            return Err("User data too short".into());
        }
        
        // Validate user data format
        let fields: Vec<&str> = data.split(',').collect();
        if fields.len() < 3 {
            return Err("Missing required user fields".into());
        }
        
        for field in &fields {
            if field.trim().is_empty() {
                return Err("Empty field not allowed".into());
            }
        }
        
        Ok("User created successfully".to_string())
    }

    async fn create_order(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Complex order creation logic with validation
        let order_data: Result<HashMap<&str, &str>, _> = data
            .split(',')
            .map(|pair| {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    Ok((parts[0].trim(), parts[1].trim()))
                } else {
                    Err("Invalid key=value format")
                }
            })
            .collect();
            
        match order_data {
            Ok(map) => {
                if map.contains_key("item") && map.contains_key("quantity") {
                    if let Some(qty_str) = map.get("quantity") {
                        match qty_str.parse::<u32>() {
                            Ok(qty) if qty > 0 => Ok("Order created successfully".to_string()),
                            Ok(_) => Err("Quantity must be greater than 0".into()),
                            Err(_) => Err("Invalid quantity format".into()),
                        }
                    } else {
                        Err("Missing quantity".into())
                    }
                } else {
                    Err("Missing required order fields".into())
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn handle_put_request(
        &self,
        url: &str,
        body: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // PUT request handling with complex logic
        match body {
            Some(data) => {
                if url.contains("/users/") {
                    self.update_user(url, data).await
                } else if url.contains("/orders/") {
                    self.update_order(url, data).await
                } else {
                    Err("Unknown PUT endpoint".into())
                }
            }
            None => Err("PUT request requires body".into()),
        }
    }

    async fn update_user(&self, url: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Extract user ID from URL
        let parts: Vec<&str> = url.split('/').collect();
        let user_id = parts.last().ok_or("Missing user ID")?;
        
        if user_id.parse::<u32>().is_err() {
            return Err("Invalid user ID format".into());
        }
        
        // Validate update data
        if data.is_empty() {
            return Err("Update data cannot be empty".into());
        }
        
        Ok(format!("User {} updated successfully", user_id))
    }

    async fn update_order(&self, url: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Extract order ID and validate
        let parts: Vec<&str> = url.split('/').collect();
        let order_id = parts.last().ok_or("Missing order ID")?;
        
        match order_id.parse::<u32>() {
            Ok(id) if id > 0 => {
                if data.contains("status=") {
                    Ok(format!("Order {} updated successfully", id))
                } else {
                    Err("Status field required for order updates".into())
                }
            }
            _ => Err("Invalid order ID".into()),
        }
    }

    async fn handle_delete_request(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // DELETE request with validation
        if url.contains("/users/") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(user_id) = parts.last() {
                match user_id.parse::<u32>() {
                    Ok(id) if id > 0 => Ok(format!("User {} deleted successfully", id)),
                    _ => Err("Invalid user ID for deletion".into()),
                }
            } else {
                Err("Missing user ID for deletion".into())
            }
        } else {
            Err("DELETE only supported for users".into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage demonstrating the complexity
    let client = HttpClient::new("https://api.example.com".to_string());
    
    // Test various endpoints
    match client.get("/api/v1/users/123").await {
        Ok(response) => println!("User data: {}", response),
        Err(e) => eprintln!("Error fetching user: {}", e),
    }
    
    // Test order creation
    let order_data = "item=laptop,quantity=2,price=1500";
    match client.request_with_retry("POST", "/api/v1/orders", Some(order_data)).await {
        Ok(response) => println!("Order created: {}", response),
        Err(e) => eprintln!("Error creating order: {}", e),
    }
    
    Ok(())
}
EOF

# This creates realistic, idiomatic Rust code with:
# - Proper error handling with Result types
# - Async/await patterns with tokio
# - Complex branching and nesting
# - Real-world HTTP client patterns
# - Multiple complexity levels across functions
# - Runnable main function (cargo run --example http_client_complexity)

# Now test the command with local binary
./target/debug/pmat analyze complexity --include "examples/analysis/*.rs"

# Verify it's runnable (optional)
cargo check --example complexity_demo
```

#### For Quality Gate Commands
```bash
# Example: quality gate check
# Use the same test file from above
pmat quality-gate --file main.rs

# Test with stricter thresholds
pmat quality-gate --file main.rs --max-complexity 5
```

#### For Refactor Commands
```bash
# Example: auto refactor
pmat refactor auto --file main.rs

# Test with dry-run first
pmat refactor auto --file main.rs --dry-run
```

### 4. Document Working Example (60 seconds)

Create examples in one of these locations:
- `examples/cli-usage/` directory (new examples)
- Update existing documentation in `README.md`
- Add to command help text in source

#### Example Documentation Format
```markdown
## Example: Analyzing Code Complexity

```bash
# Create a test file
cat > complex_function.rs << 'EOF'
fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
EOF

# Analyze complexity
pmat analyze complexity --file complex_function.rs

# Expected output:
# Function: factorial, Complexity: 3
```

Why this works:
- Creates realistic code sample
- Shows expected output
- Demonstrates actual tool behavior
```

### 5. Verify Example Works (30 seconds)
```bash
# Ensure the example code compiles and analysis runs
pmat analyze complexity --include "examples/demo-code/*.rs"

# Should show meaningful complexity metrics for our example
```

### 6. Test on Real Codebase (2-3 minutes)
```bash
# Apply the same command to our actual codebase
pmat analyze complexity --top-files 3

# Compare results - does the output make sense?
# Are there any UX issues or confusing messages?

# Test edge cases
pmat analyze complexity --include "nonexistent.rs"
pmat analyze complexity --include "server/src/lib.rs"
```

### 7. Run Quality Checks (2-3 minutes)
**CRITICAL - Toyota Way Zero Tolerance**: ALL tests must pass. NO workarounds!

```bash
# ALL of these must pass - fix any failures immediately
make lint          # Zero clippy warnings
make test-fast     # All unit tests pass
make test-doc      # All doctests pass  
make test-property # All property tests pass

# Verify no self-admitted technical debt
rg "TODO|FIXME|HACK|XXX" server/src --type rust | grep -v "test" || echo "✅ No SATD found"
```

**If ANY test fails**:
1. **STOP** - Do not proceed
2. **FIX** the root cause immediately
3. **RE-RUN** all tests
4. **REPEAT** until 100% pass rate

**Toyota Way Principles**:
- **Jidoka**: Stop the line when defects found
- **Zero Defects**: No compromises on quality
- **Root Cause**: Fix the problem, not the symptom
- **No Workarounds**: Temporary fixes are permanent problems

### 8. Fix Issues Found (2-5 minutes per issue)
**CRITICAL**: Don't just document issues - FIX them immediately!

**IMPORTANT**: Always test with local binary after fixes:
```bash
# Build with fixes
cargo build --package pmat

# Test with local binary (NOT system PATH binary)
./target/debug/pmat analyze complexity --include "server/examples/*.rs"
```

For each UX issue discovered:
1. **Identify root cause** in codebase
2. **Implement fix** with proper error handling
3. **Add property test** to prevent regression
4. **Add doctest** showing correct usage
5. **Verify fix works** with original failing case

Example fix workflow:
```bash
# Issue: --include pattern doesn't work
# 1. Find the file pattern matching code
rg "include.*pattern" server/src --type rust

# 2. Fix the implementation
# 3. Add property test
# 4. Add doctest
# 5. Test the fix with local binary
./target/debug/pmat analyze complexity --include "server/examples/*.rs"
```

### 9. Validate Metric Accuracy (2-3 minutes)
**CRITICAL**: Verify the analysis metrics are actually correct!

For complexity analysis:
1. **Manual calculation** of cyclomatic complexity for simple functions
2. **Compare with established tools** (if available)
3. **Verify cognitive complexity** against known algorithms
4. **Check edge cases**: empty functions, single return, deep nesting

Example validation:
```rust
// This function should have cyclomatic complexity = 4
fn test_complexity(x: i32) -> i32 {
    if x > 0 {        // +1
        if x > 10 {   // +1
            x * 2
        } else {      // +1
            x + 1
        }
    } else {          // +1
        0
    }
}
// Expected: Cyclomatic = 4, Cognitive = 2 (nesting)
```

### 10. Add Property Tests (1-2 minutes)
Create property tests for the functionality:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn complexity_analysis_never_panics(
        code in ".*{1,1000}",
        include_pattern in ".*{1,100}"
    ) {
        // Should never panic regardless of input
        let result = analyze_complexity_with_pattern(&code, &include_pattern);
        prop_assert!(result.is_ok() || result.is_err()); // Never panics
    }
    
    #[test]
    fn complexity_metrics_are_non_negative(
        valid_rust_code in generate_valid_rust_functions()
    ) {
        let metrics = calculate_complexity(&valid_rust_code)?;
        prop_assert!(metrics.cyclomatic >= 1); // Minimum complexity is 1
        prop_assert!(metrics.cognitive >= 0);   // Can be 0 for simple functions
    }
}
```

### 11. Add Doctests (1 minute)
Add doctests to the fixed functions:

```rust
/// Analyzes complexity of Rust code files matching the given pattern
/// 
/// # Examples
/// 
/// ```
/// use pmat::analyze_complexity;
/// 
/// // Analyze all Rust files in examples directory
/// let result = analyze_complexity("examples/*.rs")?;
/// assert!(result.files_analyzed > 0);
/// assert!(result.total_functions > 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn analyze_complexity(pattern: &str) -> Result<ComplexityResult, Error> {
    // Implementation with proper pattern matching
}
```

### 12. Commit and Push (30 seconds)
```bash
# If creating example files
git add examples/cli-usage/complexity-example.md
git commit -m "docs: Add complexity analysis example

🤖 Generated with Claude Code
https://claude.ai/code

Co-Authored-By: Claude <noreply@anthropic.com>"
git push origin master
```

## Example Organization Strategy

Follow idiomatic Rust project structure like tch-rs, pytorch, etc:

```
examples/
├── analysis/           # pmat analyze commands
│   ├── complexity_demo.rs
│   ├── lint_hotspot_demo.rs
│   ├── satd_demo.rs
│   └── dead_code_demo.rs
├── quality_gate/       # pmat quality-gate commands  
│   ├── basic_checks.rs
│   ├── custom_thresholds.rs
│   └── ci_integration.rs
├── refactor/          # pmat refactor commands
│   ├── auto_refactor_demo.rs
│   ├── extract_function_demo.rs
│   └── complexity_reduction.rs
├── reports/           # pmat report commands
│   ├── comprehensive_analysis.rs
│   └── sarif_output.rs
└── README.md          # Index of all examples
```

### Example Categories

Each example should be:
- **Runnable**: `cargo run --example <name>`
- **Self-contained**: No external dependencies beyond tokio/serde if needed
- **Realistic**: Demonstrates real-world code patterns
- **Documented**: Shows expected pmat output in comments

## Speed Heuristics

### Always Worth Exemplifying
- Core analysis commands (`complexity`, `lint-hotspot`, `satd`)
- Quality gate scenarios (pass/fail cases)
- Refactor workflows (`auto`, `extract`)
- Common CLI flag combinations
- Error scenarios and edge cases

### Skip These (for now)
- Internal debugging commands
- Commands requiring complex setup
- Commands that modify the codebase significantly
- Platform-specific behaviors

## Minimal Quality Bar

✅ **Must Have:**
- Command actually runs without crashes
- Example produces expected output type
- Shows realistic usage scenario
- Tests both success and common failure cases

❌ **Don't Need (for speed):**
- Comprehensive flag coverage
- Complex multi-step workflows
- Performance benchmarking
- Advanced configuration scenarios

## Example Speed Run

```bash
# 1. Pick target (10s)
$ pmat --help | grep analyze
analyze     Analyze code quality metrics

# 2. Test basics (30s)
$ pmat analyze --help
$ pmat analyze complexity --help

# 3. Create example (2-3m)
$ cd /tmp && mkdir pmat-test && cd pmat-test
$ echo 'fn test() { println!("hello"); }' > test.rs
$ pmat analyze complexity --file test.rs

# 4. Document (60s)
# Create examples/cli-usage/basic-complexity.md

# 5. Test on real code (2-3m)
$ cd /home/noah/src/paiml-mcp-agent-toolkit
$ pmat analyze complexity --top-files 3

# 6. Commit (30s)
$ git add examples/cli-usage/basic-complexity.md
$ git commit -m "docs: Add basic complexity analysis example"
$ git push origin master

# Total: ~6 minutes
```

## Key Success Factors

### 1. Real Usage First
- Always test commands yourself before documenting
- Use realistic code samples, not toy examples
- Show both successful and error scenarios
- Verify output makes sense to a new user

### 2. Minimal Viable Examples
```bash
# Good: Shows real command with real output
pmat analyze complexity --file src/main.rs
# Output: Function: main, Complexity: 2

# Better: Shows multiple scenarios
pmat analyze complexity --top-files 5
# Output: Top 5 most complex functions...
```

### 3. Quick Testing Pattern
```bash
# Test the example immediately
cd /tmp/pmat-example
pmat [command] [args]

# Then test on real codebase
cd /home/noah/src/paiml-mcp-agent-toolkit
pmat [command] [args]
```

### 4. Document UX Issues
```markdown
## Issues Found
- Error message unclear when file not found
- Output format inconsistent between commands
- Missing progress indicators for long operations
- Help text doesn't match actual behavior
```

## Validation Workflow

### 1. External Validation (Test Example)
```bash
# Create clean test environment
cd /tmp && rm -rf pmat-validation && mkdir pmat-validation && cd pmat-validation

# Follow the example exactly as documented
# Does it work for a new user?
```

### 2. Internal Validation (Our Codebase)
```bash
# Apply same command to our code
cd /home/noah/src/paiml-mcp-agent-toolkit
pmat [command] [args]

# Questions to ask:
# - Are the results useful?
# - Is the output format helpful?
# - Are there obvious bugs or UX issues?
# - Does it work as documented?
```

### 3. Edge Case Testing
```bash
# Test common failure modes
pmat [command] --file nonexistent.rs
pmat [command] --invalid-flag
pmat [command] # (missing required args)
```

## Results
Using this approach, you can create 10-15 working examples in 60 minutes, each tested on both synthetic and real code, with UX issues documented for future improvement.