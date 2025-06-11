#!/bin/bash
# Script to safely clear swap and clean up system

echo "=== System Swap Recovery Tool ==="
echo

# Show current state
echo "Current swap usage:"
free -h | grep -E "Mem:|Swap:"
echo

# Check for zombie processes
echo "Checking for zombie processes..."
ZOMBIES=$(ps aux | grep -E "defunct|<zombie>" | grep -v grep)
if [ -n "$ZOMBIES" ]; then
    echo "Found zombie processes:"
    echo "$ZOMBIES"
    echo
    echo "Attempting to find and kill parent processes..."
    ps aux | grep -E "cargo|test" | grep -v grep | awk '{print $2}' | xargs -r kill -9 2>/dev/null
    sleep 2
fi

# Check if we can clear swap (requires sudo)
echo
echo "To clear swap, run these commands with sudo:"
echo "  sudo swapoff -a    # Disable swap (moves data back to RAM)"
echo "  sudo swapon -a     # Re-enable swap"
echo
echo "WARNING: Only do this if you have enough free RAM!"
echo "Current free RAM: $(free -h | grep Mem | awk '{print $4}')"
echo

# Kill any remaining test processes
echo "Cleaning up any remaining test processes..."
pkill -f "cargo test" 2>/dev/null
pkill -f "cargo nextest" 2>/dev/null
pkill -f "target/debug/deps" 2>/dev/null
pkill -f "target/release/deps" 2>/dev/null

# Clear PageCache, dentries and inodes (safe, doesn't require sudo)
echo
echo "To free up memory caches (requires sudo):"
echo "  sudo sync && echo 3 | sudo tee /proc/sys/vm/drop_caches"
echo

# Show final state
echo
echo "Current state after cleanup:"
free -h | grep -E "Mem:|Swap:"
ps aux | grep -E "cargo|test" | grep -v grep | wc -l | xargs echo "Remaining test processes:"

echo
echo "=== Recommendations ==="
echo "1. If swap is still full, reboot the system for a clean state"
echo "2. Or use the sudo commands above to manually clear swap"
echo "3. Set vm.swappiness=10 to reduce swap usage:"
echo "   echo 'vm.swappiness=10' | sudo tee -a /etc/sysctl.conf"
echo "4. Consider increasing swap size or RAM"