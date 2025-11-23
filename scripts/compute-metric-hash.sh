#!/bin/bash
# Compute metric hash for cache invalidation (Phase 1 - simple version)
# Spec: docs/specifications/quick-test-build-O(1)-checking.md
# Full Merkle tree implementation in Phase 2 (Rust CLI)
set -euo pipefail

METRIC_NAME="${1:-}"

if [ -z "$METRIC_NAME" ]; then
    echo "Usage: $0 <metric-name>" >&2
    echo "Example: $0 lint" >&2
    exit 1
fi

# Simple hash based on Cargo.toml + Cargo.lock
# Phase 2 will implement full Merkle tree hashing per spec
case "$METRIC_NAME" in
    lint|test-fast|coverage|build-release|deps-default)
        # For Phase 1, all metrics depend on Cargo.toml + Cargo.lock
        # Phase 2 will differentiate (lint = src + .clippy.toml, etc.)
        (cat server/Cargo.toml server/Cargo.lock 2>/dev/null || echo "no-cache") | sha256sum | cut -d' ' -f1
        ;;
    *)
        echo "Unknown metric: $METRIC_NAME" >&2
        exit 1
        ;;
esac
