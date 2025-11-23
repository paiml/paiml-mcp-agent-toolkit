#!/bin/bash
# Benchmark cargo builds using bashrs bench
# Pattern: Use existing PAIML tools (bashrs, renacer) instead of custom timing
set -euo pipefail

# shellcheck disable=DET002
# Intentional: Timestamp required for benchmark result tracking (bashrs issue #43)
TIMESTAMP="$(date +%Y-%m-%d_%H-%M-%S)"
RESULTS_DIR="benchmarks/results"
mkdir -p "$RESULTS_DIR"

echo "🔬 Benchmarking cargo builds at $TIMESTAMP"
echo "Using bashrs bench for scientific rigor"
echo ""

# Create individual benchmark scripts
cat > /tmp/bench-dev.sh <<'EOF'
#!/bin/bash
cargo clean >/dev/null 2>&1
cargo build 2>&1 | tail -1
EOF
chmod +x /tmp/bench-dev.sh

cat > /tmp/bench-release.sh <<'EOF'
#!/bin/bash
cargo clean >/dev/null 2>&1
cargo build --release 2>&1 | tail -1
EOF
chmod +x /tmp/bench-release.sh

cat > /tmp/bench-minimal.sh <<'EOF'
#!/bin/bash
cargo clean >/dev/null 2>&1
cargo build --release --no-default-features --features rust-only 2>&1 | tail -1
EOF
chmod +x /tmp/bench-minimal.sh

# Benchmark with bashrs
echo "📊 Benchmarking dev build..."
bashrs bench /tmp/bench-dev.sh \
    --warmup 1 \
    --iterations 3 \
    --output "$RESULTS_DIR/dev-$TIMESTAMP.json" \
    --quiet

echo "📊 Benchmarking release build..."
bashrs bench /tmp/bench-release.sh \
    --warmup 1 \
    --iterations 3 \
    --output "$RESULTS_DIR/release-$TIMESTAMP.json" \
    --quiet

echo "📊 Benchmarking minimal build..."
bashrs bench /tmp/bench-minimal.sh \
    --warmup 1 \
    --iterations 3 \
    --output "$RESULTS_DIR/minimal-$TIMESTAMP.json" \
    --quiet

# Summarize results
echo ""
echo "✅ Benchmarks complete!"
echo ""
echo "📊 Results:"
echo "  Dev:     $RESULTS_DIR/dev-$TIMESTAMP.json"
echo "  Release: $RESULTS_DIR/release-$TIMESTAMP.json"
echo "  Minimal: $RESULTS_DIR/minimal-$TIMESTAMP.json"
echo ""
echo "📖 View results:"
echo "  jq . $RESULTS_DIR/dev-$TIMESTAMP.json"

# Cleanup
rm -f /tmp/bench-./*.sh
