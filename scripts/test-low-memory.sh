#!/bin/bash
# Low memory test script - prevents OOM during testing

echo "🛡️ Running tests with memory-safe settings..."

# 1. Limit cargo build parallelism
export CARGO_BUILD_JOBS=2

# 2. Limit test thread count
export RUST_TEST_THREADS=2

# 3. Disable incremental compilation to save memory
export CARGO_INCREMENTAL=0

# 4. Use single codegen unit to reduce memory usage
export CARGO_PROFILE_TEST_CODEGEN_UNITS=1

# 5. Clear any existing build artifacts that might be consuming memory
echo "🧹 Cleaning build artifacts..."
cargo clean

# 6. Monitor memory during build
echo "💾 Current memory status:"
free -h

echo "🚀 Running tests with reduced memory usage..."
SKIP_SLOW_TESTS=1 cargo test --workspace --features skip-slow-tests --jobs 2 -- --test-threads=2

echo "✅ Tests completed!"
echo "💾 Final memory status:"
free -h