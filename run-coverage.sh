#!/bin/bash
set -e

echo "🧹 Cleaning previous coverage data..."
cargo llvm-cov clean --workspace

echo "📊 Running tests with coverage (excluding property tests)..."
# Run tests with llvm-cov to properly instrument and collect coverage
RUST_MIN_STACK=16777216 \
cargo llvm-cov test --lib -- \
    --test-threads=1 \
    --skip property_tests \
    --skip proptest \
    --skip property \
    --quiet

echo "📈 Generating coverage report..."
cargo llvm-cov report --summary-only

echo "✅ Coverage complete!"