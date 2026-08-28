#!/bin/bash
# shellcheck disable=SC2032  # Variables are used within this script
#
# The line that used to sit here, "# bashrs-disable: DET002 SEC010", suppressed
# nothing: bashrs has no inline suppression syntax at all — only the `--ignore`
# flag and `.bashrsignore` — so the comment was decoration and both rules had
# been firing on this file the whole time. The timestamps below are now derived
# from SOURCE_DATE_EPOCH, which is the actual remedy DET002 asks for, verified
# to clear the rule rather than assumed to.
set -e

OUTPUT_DIR=".pmat-metrics/build-benchmarks"
# shellcheck disable=SC2174
mkdir -p "$OUTPUT_DIR"
# Honour SOURCE_DATE_EPOCH (reproducible-builds.org) so a benchmark run can be
# reproduced byte-for-byte; fall back to wall-clock when it is unset.
TIMESTAMP="$(date -u -d "@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m%d_%H%M%S)"
BASELINE_FILE="$OUTPUT_DIR/baseline_$TIMESTAMP.txt"

echo "=== Build Performance Benchmark ===" | tee "$BASELINE_FILE"
echo "Date: $(date -u -d "@${SOURCE_DATE_EPOCH:-$(date +%s)}")" | tee -a "$BASELINE_FILE"
echo "Git commit: $(git rev-parse HEAD)" | tee -a "$BASELINE_FILE"
echo "" | tee -a "$BASELINE_FILE"

# 1. Clean build
echo "1. Clean build (dev)..." | tee -a "$BASELINE_FILE"
cargo clean
{ time cargo build 2>&1; } 2>&1 | tee -a "$BASELINE_FILE"

# 2. Incremental build (touch one file)
echo "" | tee -a "$BASELINE_FILE"
echo "2. Incremental build (one file)..." | tee -a "$BASELINE_FILE"
touch server/src/cli/commands.rs
{ time cargo build 2>&1; } 2>&1 | tee -a "$BASELINE_FILE"

# 3. Dependency count
echo "" | tee -a "$BASELINE_FILE"
echo "3. Dependency analysis..." | tee -a "$BASELINE_FILE"
DEP_COUNT="$(cargo tree 2>/dev/null | wc -l)"
echo "Total dependencies: $DEP_COUNT" | tee -a "$BASELINE_FILE"

# 4. Duplicate versions
echo "" | tee -a "$BASELINE_FILE"
echo "4. Duplicate dependency versions:" | tee -a "$BASELINE_FILE"
cargo tree -d 2>/dev/null | tee -a "$BASELINE_FILE"

echo "" | tee -a "$BASELINE_FILE"
echo "=== Baseline Results Saved ===" | tee -a "$BASELINE_FILE"
echo "File: $BASELINE_FILE" | tee -a "$BASELINE_FILE"

