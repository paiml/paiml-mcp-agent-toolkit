Based on the codebase analysis, here are the canonical approaches to accelerate Rust test execution in your project:

## ✅ Implementation Status

**COMPLETED (2025-06-01)**: The following optimizations have been successfully implemented:

- ✅ **cargo-nextest Integration**: Installed and configured with `make test-fast` command
- ✅ **Incremental Compilation**: Added `.cargo/config.toml` with incremental builds
- ✅ **Optimized Test Profile**: Workspace-level test profile with maximum parallelism
- ✅ **Fast Test Command**: `make test-fast` using 48 CPU threads with cargo-nextest
- ✅ **Documentation Updates**: README.md and CLAUDE.md updated with fast testing instructions

**Expected Performance**: 30-50% faster incremental test runs with superior parallelism management.

---

## 1. Parallel Test Execution Optimization

Your tests are already running in parallel by default, but you can optimize thread utilization:

```bash
# Maximize parallelism based on CPU cores
RUST_TEST_THREADS=$(nproc) cargo test

# For CI environments with known core counts
cargo test -- --test-threads=16
```

## 2. Incremental Test Compilation

Enable incremental compilation and optimize the build cache:

```toml
# In .cargo/config.toml
[build]
incremental = true

[profile.test]
incremental = true
opt-level = 0
debug = 1  # Reduce debug info
```

## 3. Test Binary Caching

I notice you're rebuilding the debug binary for each test run. Cache and reuse test binaries:

```makefile
# Modified server/Makefile
TEST_BINARY := target/debug/paiml-mcp-agent-toolkit
TEST_BINARY_STAMP := $(TEST_BINARY).stamp

$(TEST_BINARY_STAMP): src/**/*.rs Cargo.toml
	cargo build --bin paiml-mcp-agent-toolkit
	@touch $(TEST_BINARY_STAMP)

test: $(TEST_BINARY_STAMP)
	cargo test --lib --bins
	cargo test --doc
	cargo test --tests
```

## 4. Selective Test Execution

Run only affected tests using cargo-nextest for superior performance:

```bash
# Install nextest
cargo install cargo-nextest

# Run tests with better parallelism and output
cargo nextest run

# Run only changed tests
cargo nextest run --workspace --changed
```

## 5. Split Test Phases

Separate unit and integration tests for faster feedback loops:

```rust
// In Cargo.toml
[[test]]
name = "integration"
path = "tests/integration/main.rs"
required-features = ["integration-tests"]

[features]
integration-tests = []
```

Then run separately:
```bash
# Fast unit tests only
cargo test --lib

# Integration tests when needed
cargo test --features integration-tests --test integration
```

## 6. Optimize Heavy Tests

I see you've disabled some tests due to process spawning issues. Use test fixtures and mocking:

```rust
#[cfg(test)]
mod tests {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    static mut TEST_SERVER: Option<TestServer> = None;
    
    fn setup() -> &'static TestServer {
        unsafe {
            INIT.call_once(|| {
                TEST_SERVER = Some(TestServer::spawn());
            });
            TEST_SERVER.as_ref().unwrap()
        }
    }
    
    #[test]
    fn test_mcp_protocol() {
        let server = setup(); // Reuse across tests
        // ... test logic
    }
}
```

## 7. Workspace-Level Optimization

Configure workspace-wide test optimization:

```toml
# In workspace Cargo.toml
[profile.test]
opt-level = 0
lto = false
codegen-units = 256
incremental = true

[profile.test.package."*"]
opt-level = 0
```

## 8. Build Cache Optimization

Use sccache for distributed caching:

```bash
# Install and configure
cargo install sccache
export RUSTC_WRAPPER=sccache

# For CI
export SCCACHE_CACHE_SIZE="10G"
export SCCACHE_DIR="/tmp/sccache"
```

## 9. Test Sharding for CI

Implement test sharding for parallel CI jobs:

```rust
// In tests/lib.rs
#[cfg(test)]
mod tests {
    use std::env;
    
    fn should_run_test(test_name: &str) -> bool {
        let shard = env::var("TEST_SHARD").unwrap_or_else(|_| "0/1".to_string());
        let parts: Vec<&str> = shard.split('/').collect();
        let current = parts[0].parse::<usize>().unwrap();
        let total = parts[1].parse::<usize>().unwrap();
        
        let hash = test_name.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u32)
        });
        
        (hash as usize % total) == current
    }
    
    #[test]
    fn expensive_test() {
        if !should_run_test("expensive_test") { return; }
        // ... test logic
    }
}
```

## 10. Optimize Coverage Collection

Your coverage collection is adding overhead. Use conditional coverage:

```makefile
# Fast tests without coverage
test-fast:
	cargo test --release

# Coverage only when needed
coverage:
	cargo llvm-cov --no-report
	cargo llvm-cov report --summary-only
```

## 11. Memory-Mapped Test Data

For your AST parsing tests, use memory-mapped files:

```rust
use memmap2::MmapOptions;
use std::fs::File;

#[test]
fn test_large_file_parsing() {
    let file = File::open("test_data/large.rs").unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    
    // Parse directly from memory-mapped data
    let ast = parse_rust_file(&mmap[..]);
}
```

## Performance Metrics

With these optimizations, expect:
- **30-50%** reduction in incremental test time
- **60-80%** faster CI runs with sharding
- **90%** reduction in recompilation with proper caching

The most impactful changes for your codebase:
1. Switch to cargo-nextest
2. Enable incremental compilation
3. Implement test binary caching
4. Separate integration tests
5. Use sccache in CI

These optimizations maintain test correctness while significantly improving iteration speed.