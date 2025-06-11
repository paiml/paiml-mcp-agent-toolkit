#!/bin/bash
# Pre-test system health check

echo "🔍 Pre-test System Health Check"
echo "==============================="

# Check swap usage
SWAP_USED=$(free -m | grep Swap | awk '{print $3}')
SWAP_TOTAL=$(free -m | grep Swap | awk '{print $2}')
SWAP_PERCENT=0
if [ "$SWAP_TOTAL" -gt 0 ]; then
    SWAP_PERCENT=$((SWAP_USED * 100 / SWAP_TOTAL))
fi

# Check free memory
FREE_MEM=$(free -m | grep Mem | awk '{print $4}')

# Check load
LOAD=$(uptime | awk '{print $(NF-2)}' | sed 's/,//')
CORES=$(nproc)

echo "System Status:"
echo "  CPU Load: $LOAD (on $CORES cores)"
echo "  Free Memory: ${FREE_MEM}MB"
echo "  Swap Used: ${SWAP_USED}MB / ${SWAP_TOTAL}MB ($SWAP_PERCENT%)"
echo

# Warn if swap is too full  
if [ "$SWAP_PERCENT" -gt 90 ]; then
    echo "⚠️  WARNING: Swap usage is very high ($SWAP_PERCENT%)"
    echo "   This may cause performance issues during testing."
    echo
    echo "   To fix this, run:"
    echo "   sudo swapoff -a && sudo swapon -a"
    echo
    echo "   Or reboot the system for a clean state."
    echo
    echo "   Continuing with tests despite high swap usage..."
fi

# Fail if memory is too low
if [ "$FREE_MEM" -lt 2000 ]; then
    echo "❌ ERROR: Insufficient free memory (${FREE_MEM}MB)"
    echo "   Need at least 2GB free to run tests safely."
    exit 1
fi

# Warn if load is high
if command -v bc >/dev/null 2>&1; then
    if (( $(echo "$LOAD > $CORES" | bc -l) )); then
        echo "⚠️  WARNING: System load ($LOAD) is higher than CPU cores ($CORES)"
        echo "   Tests may run slowly or fail."
    fi
fi

echo "✅ System health check passed. Safe to run tests."
echo