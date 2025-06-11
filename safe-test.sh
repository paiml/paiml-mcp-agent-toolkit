#\!/bin/bash
# Safe test runner that prevents system overload

set -euo pipefail

echo "🛡️ Safe Test Runner"
echo "=================="

# Check current system load
LOAD=$(uptime | awk -F'load average:' '{print $2}' | awk -F, '{print $1}' | xargs)
CORES=$(nproc)

echo "Current load: $LOAD (on $CORES cores)"

# If load is already high, refuse to run
if command -v bc >/dev/null && (( $(echo "$LOAD > $CORES" | bc -l) )); then
    echo "❌ System load is too high ($LOAD > $CORES cores)"
    echo "   Wait for load to decrease before running tests"
    exit 1
fi

# Use minimal parallelism to avoid crashes
echo "🔧 Using minimal configuration:"
echo "  - Debug mode (no --release flag)"
echo "  - 2 test threads only"
echo "  - Single package at a time"
echo "  - No workspace-wide testing"

# Run tests with extreme safety
cd server
export RUST_TEST_THREADS=2
export RUST_BACKTRACE=1

echo ""
echo "Running tests with minimal resource usage..."

# Just run the specific test file we care about
cargo test --lib --test-threads=2 2>&1 | head -100

echo ""
echo "✅ Test run completed safely"
