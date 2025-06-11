#!/bin/bash
# Diagnostic script to identify test crash causes

echo "=== System Diagnostics for Test Crashes ==="
echo

# Check system limits
echo "1. System Resource Limits:"
echo "   Process limit: $(ulimit -u)"
echo "   Open files limit: $(ulimit -n)"
echo "   Max memory size: $(ulimit -m)"
echo "   Virtual memory: $(ulimit -v)"
echo

# Check current resource usage
echo "2. Current System Resources:"
echo "   Memory:"
free -h | grep -E "Mem:|Swap:"
echo
echo "   CPU cores: $(nproc)"
echo "   Load average: $(uptime | awk -F'load average:' '{print $2}')"
echo

# Check for zombie processes
echo "3. Process Status:"
ZOMBIES=$(ps aux | grep -E "defunct|<zombie>" | wc -l)
echo "   Zombie processes: $ZOMBIES"
echo "   Total processes: $(ps aux | wc -l)"
echo "   Cargo processes: $(ps aux | grep cargo | grep -v grep | wc -l)"
echo

# Check disk space
echo "4. Disk Space:"
df -h . | grep -v Filesystem
echo

# Test minimal cargo command
echo "5. Testing minimal cargo command:"
echo "   Running: cargo test --help"
if timeout 5 cargo test --help > /dev/null 2>&1; then
    echo "   ✓ Basic cargo test works"
else
    echo "   ✗ Basic cargo test failed!"
fi
echo

# Check for kernel messages
echo "6. Recent kernel messages (if accessible):"
if command -v dmesg >/dev/null 2>&1; then
    echo "   Last 10 kernel messages:"
    dmesg | tail -10 2>/dev/null || echo "   (requires sudo to read kernel log)"
else
    echo "   dmesg not available"
fi
echo

echo "=== Recommendations ==="
echo "1. Run tests with minimal threads: THREADS=1 make test-safe"
echo "2. Monitor system resources in another terminal: watch -n 1 'free -h; echo; ps aux | grep cargo'"
echo "3. Check system logs after crash: journalctl -xe"
echo "4. Try running a single test file to isolate the issue"