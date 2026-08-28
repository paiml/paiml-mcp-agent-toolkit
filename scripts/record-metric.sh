#!/bin/bash
# Record build/test/lint metrics for O(1) pre-commit validation
# Spec: docs/specifications/quick-test-build-O(1)-checking.md
# Pattern: Hash-based caching (similar to 27fea2ae)
# bashrs disable-file=DET002
# Intentional: Timestamps required for metric recording
set -euo pipefail

METRIC_NAME=${1:-}
METRICS_DIR=".pmat-metrics"

if [ -z "$METRIC_NAME" ]; then
    echo "Usage: $0 <metric-name>" >&2
    echo "Example: $0 lint" >&2
    exit 1
fi

# $METRIC_NAME becomes a path component under $METRICS_DIR, so constrain its
# shape BEFORE any path is built from it. The dispatch `case` below does reject
# unknown names, but it runs too late: $START_FILE is assembled and read with
# `cat` above it. Verified with `bash -x record-metric.sh ../victim/pwned`,
# which ran `cat .pmat-metrics/../victim/pwned.start` and only then printed
# "Unknown metric".
if [ "$METRIC_NAME" != "${METRIC_NAME%%..*}" ] || [ "$METRIC_NAME" != "${METRIC_NAME%%/*}" ]; then
    echo "Invalid metric name (must not contain '..' or '/'): $METRIC_NAME" >&2
    exit 1
fi

# Create metrics directory
mkdir -p "$METRICS_DIR"

# Calculate duration (from start time recorded by Makefile)
START_FILE="$METRICS_DIR/$METRIC_NAME.start"
if [ ! -f "$START_FILE" ]; then
    echo "Warning: No start time found for $METRIC_NAME" >&2
    exit 0
fi

START_MS="$(cat "$START_FILE")"
END_MS="$(date +%s%3N)"
DURATION_MS="$((END_MS - START_MS))"

# Capture result based on metric type
case "$METRIC_NAME" in
    lint)
        # Lint passed if we got here
        cat > "$METRICS_DIR/lint.result" <<EOF
{
  "duration_ms": ${DURATION_MS},
  "passed": true,
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
        ;;

    test-fast)
        # Count tests (rough estimate from cargo output)
        TESTS="$(cargo test --lib --no-run 2>&1 | grep -oP '\d+(?= tests)' | head -1 || echo "0")"
        cat > "$METRICS_DIR/test-fast.result" <<EOF
{
  "duration_ms": ${DURATION_MS},
  "passed": true,
  "tests": ${TESTS},
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
        ;;

    coverage)
        # Extract line coverage percentage with same exclusions as Makefile
        COVERAGE_PCT="$(cargo llvm-cov report --summary-only \
            --ignore-filename-regex='(/tests/|_tests\.rs|_test\.rs|/benches/|/examples/|fixtures/|main\.rs|bin/)' \
            2>/dev/null | grep -E '^TOTAL' | grep -oP '\d+\.\d+(?=%)' | tail -1 || echo "0.0")"
        cat > "$METRICS_DIR/coverage.result" <<EOF
{
  "duration_ms": ${DURATION_MS},
  "coverage_pct": ${COVERAGE_PCT},
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
        # Write trends file for pmat work integration
        TIMESTAMP="$(date +%s)"
        mkdir -p "$METRICS_DIR/trends"
        cat > "$METRICS_DIR/trends/test-coverage.json" <<EOF
[
  {"metric": "test-coverage", "value": ${COVERAGE_PCT}, "timestamp": ${TIMESTAMP}}
]
EOF
        ;;

    build-release)
        # Get binary size
        BINARY_SIZE="$(stat --format=%s target/release/pmat 2>/dev/null || echo "0")"
        cat > "$METRICS_DIR/build-release.result" <<EOF
{
  "duration_ms": ${DURATION_MS},
  "binary_size": ${BINARY_SIZE},
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
        ;;

    deps-default)
        # Count dependencies
        DEPS_COUNT="$(cargo tree 2>/dev/null | wc -l)"
        cat > "$METRICS_DIR/deps-default.result" <<EOF
{
  "count": ${DEPS_COUNT},
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
        ;;

    *)
        echo "Unknown metric: $METRIC_NAME" >&2
        exit 1
        ;;
esac

# Compute hash for cache invalidation
pmat compute-metric-hash "$METRIC_NAME" > "$METRICS_DIR/$METRIC_NAME.hash" 2>/dev/null || {
    # Fallback: simple hash of Cargo.toml + Cargo.lock
    (cat server/Cargo.toml server/Cargo.lock 2>/dev/null || echo "") | sha256sum | cut -d' ' -f1 > "$METRICS_DIR/$METRIC_NAME.hash"
}

# Clean up start file
rm -f "$START_FILE"

echo "✅ Recorded $METRIC_NAME: ${DURATION_MS}ms"
