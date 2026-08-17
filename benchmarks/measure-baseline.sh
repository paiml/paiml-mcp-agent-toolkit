#!/bin/bash
# Baseline measurement script for dependency reduction benchmarking
# Pattern: Modeled after trueno-db competitive benchmarking methodology
set -euo pipefail

# Ensure we're in the project root FIRST: every path below ($RESULTS_DIR, the
# cargo invocations, target/release/pmat) is relative to it. This cd used to sit
# after the mkdir, so running the script from anywhere but the repo root created
# ./benchmarks/results in the caller's directory and then failed on the first
# `cat > "$RESULT_FILE"` because the real one had never been created.
cd "$(dirname "$0")/.."

RESULTS_DIR="benchmarks/results"
# shellcheck disable=DET002
# Intentional: Timestamp required for benchmark result tracking
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
RESULT_FILE="$RESULTS_DIR/baseline_$TIMESTAMP.md"

mkdir -p "$RESULTS_DIR"

echo "🔬 Starting baseline measurements at $TIMESTAMP"
echo "Results will be saved to: $RESULT_FILE"

# Record environment
cat > "$RESULT_FILE" <<EOF
# Baseline Measurement Results

**Timestamp**: $TIMESTAMP
**Spec**: docs/specifications/dependency-reduction-benchmarking-framework.md

## Environment

\`\`\`
Rust: $(rustc --version)
Cargo: $(cargo --version)
OS: $(uname -s) $(uname -r)
CPU: $(lscpu | grep "Model name" | sed 's/Model name: *//')
Cores: $(nproc)
RAM: $(free -h | grep Mem | awk '{print $2}')
\`\`\`

EOF

echo "📦 Measuring dependency counts..."

# Dependency counts
DEPS_DEFAULT=$(cargo tree 2>/dev/null | wc -l)
DEPS_MINIMAL=$(cargo tree --no-default-features --features rust-only 2>/dev/null | wc -l)
DEPS_ALL=$(cargo tree --all-features 2>/dev/null | wc -l)

cat >> "$RESULT_FILE" <<EOF
## Dependency Counts

| Configuration | Count | Delta from Default |
|---------------|-------|-------------------|
| Minimal (rust-only) | $DEPS_MINIMAL | -$((DEPS_DEFAULT - DEPS_MINIMAL)) (-$(echo "scale=1; 100 * ($DEPS_DEFAULT - $DEPS_MINIMAL) / $DEPS_DEFAULT" | bc)%) |
| Default | $DEPS_DEFAULT | baseline |
| All features | $DEPS_ALL | +$((DEPS_ALL - DEPS_DEFAULT)) (+$(echo "scale=1; 100 * ($DEPS_ALL - $DEPS_DEFAULT) / $DEPS_DEFAULT" | bc)%) |

\`\`\`bash
# Commands used
cargo tree | wc -l  # Default
cargo tree --no-default-features --features rust-only | wc -l  # Minimal
cargo tree --all-features | wc -l  # All
\`\`\`

EOF

echo "⏱️  Measuring build times (this will take several minutes)..."

# Function to measure build time
measure_build() {
    local config=$1
    local args=$2

    echo "  Building: $config"
    cargo clean > /dev/null 2>&1

    # Use GNU time for detailed metrics (fallback to time if not available)
    if command -v /usr/bin/time > /dev/null 2>&1; then
        /usr/bin/time -f "%E real, %U user, %S sys, %M KB max resident" \
            cargo build $args > /dev/null 2>&1
    else
        # Fallback to bash time
        TIMEFORMAT='%R real, %U user, %S sys'
        time cargo build $args > /dev/null 2>&1
    fi
}

cat >> "$RESULT_FILE" <<EOF
## Build Times (Clean Builds)

| Configuration | Time | Command |
|---------------|------|---------|
EOF

# Dev build (default)
echo "  Measuring: dev-default"
BUILD_TIME_DEV=$(measure_build "dev-default" "" 2>&1 | tail -1)
echo "| Dev (default) | $BUILD_TIME_DEV | \`cargo build\` |" >> "$RESULT_FILE"

# Release build (default)
echo "  Measuring: release-default"
BUILD_TIME_RELEASE=$(measure_build "release-default" "--release" 2>&1 | tail -1)
echo "| Release (default) | $BUILD_TIME_RELEASE | \`cargo build --release\` |" >> "$RESULT_FILE"

# Minimal release
echo "  Measuring: release-minimal"
BUILD_TIME_MINIMAL=$(measure_build "release-minimal" "--release --no-default-features --features rust-only" 2>&1 | tail -1)
echo "| Release (minimal) | $BUILD_TIME_MINIMAL | \`cargo build --release --features rust-only\` |" >> "$RESULT_FILE"

echo "📏 Measuring binary sizes..."

# Binary sizes
cargo build --release > /dev/null 2>&1
SIZE_DEFAULT=$(stat --format=%s target/release/pmat 2>/dev/null || stat -f%z target/release/pmat 2>/dev/null)
SIZE_DEFAULT_H=$(numfmt --to=iec-i --suffix=B $SIZE_DEFAULT 2>/dev/null || echo "$SIZE_DEFAULT bytes")

cargo build --release --no-default-features --features rust-only > /dev/null 2>&1
SIZE_MINIMAL=$(stat --format=%s target/release/pmat 2>/dev/null || stat -f%z target/release/pmat 2>/dev/null)
SIZE_MINIMAL_H=$(numfmt --to=iec-i --suffix=B $SIZE_MINIMAL 2>/dev/null || echo "$SIZE_MINIMAL bytes")

cargo build --release --all-features > /dev/null 2>&1
SIZE_ALL=$(stat --format=%s target/release/pmat 2>/dev/null || stat -f%z target/release/pmat 2>/dev/null)
SIZE_ALL_H=$(numfmt --to=iec-i --suffix=B $SIZE_ALL 2>/dev/null || echo "$SIZE_ALL bytes")

cat >> "$RESULT_FILE" <<EOF

## Binary Sizes (Release, Stripped)

| Configuration | Size | Delta from Default |
|---------------|------|-------------------|
| Minimal (rust-only) | $SIZE_MINIMAL_H | -$((SIZE_DEFAULT - SIZE_MINIMAL)) bytes (-$(echo "scale=1; 100 * ($SIZE_DEFAULT - $SIZE_MINIMAL) / $SIZE_DEFAULT" | bc)%) |
| Default | $SIZE_DEFAULT_H | baseline |
| All features | $SIZE_ALL_H | +$((SIZE_ALL - SIZE_DEFAULT)) bytes (+$(echo "scale=1; 100 * ($SIZE_ALL - $SIZE_DEFAULT) / $SIZE_DEFAULT" | bc)%) |

\`\`\`bash
# Measurement commands
ls -lh target/release/pmat
stat --format=%s target/release/pmat  # Linux
stat -f%z target/release/pmat  # macOS
\`\`\`

EOF

echo "🔍 Analyzing largest crate contributors..."

# Top crate contributors (requires cargo-bloat)
if command -v cargo-bloat > /dev/null 2>&1; then
    cat >> "$RESULT_FILE" <<EOF
## Binary Composition (Top 10 Crates)

\`\`\`
$(cargo bloat --release --crates -n 10 2>/dev/null)
\`\`\`

## Binary Composition (Top 20 Functions)

\`\`\`
$(cargo bloat --release -n 20 2>/dev/null)
\`\`\`

EOF
else
    cat >> "$RESULT_FILE" <<EOF
## Binary Composition

⚠️ cargo-bloat not installed. Install with:
\`\`\`bash
cargo install cargo-bloat
\`\`\`

EOF
fi

# Summary
cat >> "$RESULT_FILE" <<EOF
## Summary

### Key Metrics

- **Dependency Reduction**: Minimal config uses $DEPS_MINIMAL deps vs $DEPS_DEFAULT default (-$(echo "scale=1; 100 * ($DEPS_DEFAULT - $DEPS_MINIMAL) / $DEPS_DEFAULT" | bc)%)
- **Binary Size Reduction**: Minimal binary is $SIZE_MINIMAL_H vs $SIZE_DEFAULT_H default (-$(echo "scale=1; 100 * ($SIZE_DEFAULT - SIZE_MINIMAL) / $SIZE_DEFAULT" | bc)%)
- **Build Time**: See table above for timing comparisons

### Recommendations

1. Use \`--features rust-only\` for fast development iterations
2. Use default features for full functionality
3. Use \`--all-features\` only when testing complete feature matrix

### Next Steps

1. Run \`make bench-runtime\` to measure runtime performance
2. Compare with previous baseline in \`benchmarks/results/\`
3. Update \`benchmarks/RESULTS.md\` with findings

---

**Generated**: $TIMESTAMP
**Tool**: benchmarks/measure-baseline.sh
**Pattern**: trueno-db competitive benchmarking methodology
EOF

echo "✅ Baseline measurements complete!"
echo ""
echo "📊 Results saved to: $RESULT_FILE"
echo ""
echo "📖 View results:"
echo "   cat $RESULT_FILE"
echo ""
echo "🔄 Next steps:"
echo "   1. Review results: cat $RESULT_FILE"
echo "   2. Set baseline: cp $RESULT_FILE benchmarks/baseline.md"
echo "   3. Run runtime benchmarks: make bench-runtime"
