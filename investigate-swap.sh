#!/bin/bash
# Deep investigation of swap usage

echo "=== Deep Swap Investigation ==="
echo

# Show current swap status
echo "1. Current Memory Status:"
free -h
echo

# Find processes using swap
echo "2. Top 10 Processes Using Swap:"
echo "PID    SWAP(KB)  COMMAND"
echo "---    --------  -------"
for file in /proc/*/status; do
    if [ -r "$file" ]; then
        pid=$(basename $(dirname "$file"))
        swap=$(grep VmSwap "$file" 2>/dev/null | awk '{print $2}')
        if [ -n "$swap" ] && [ "$swap" != "0" ]; then
            cmd=$(ps -p "$pid" -o comm= 2>/dev/null)
            if [ -n "$cmd" ]; then
                echo "$pid    $swap    $cmd"
            fi
        fi
    fi
done 2>/dev/null | sort -k2 -nr | head -10
echo

# Check for specific problematic processes
echo "3. Checking for stuck processes:"
echo "   Zombie processes: $(ps aux | grep -c '<defunct>')"
echo "   D-state (uninterruptible): $(ps aux | awk '$8 ~ /D/ {print $2, $11}' | wc -l)"
echo

# Show D-state processes
echo "4. Uninterruptible sleep processes (D-state):"
ps aux | awk '$8 ~ /D/ {print $2, $8, $11}' | head -10
echo

# Check kernel memory info
echo "5. Kernel swap info:"
if [ -r /proc/meminfo ]; then
    grep -E "Swap|Dirty|Writeback" /proc/meminfo
fi
echo

# Check for memory cgroups limiting swap
echo "6. Memory cgroup info:"
if [ -d /sys/fs/cgroup/memory ]; then
    echo "   System cgroup swap limit: $(cat /sys/fs/cgroup/memory/memory.memsw.limit_in_bytes 2>/dev/null || echo 'N/A')"
fi
echo

# Check swap partition/file
echo "7. Swap devices:"
cat /proc/swaps
echo

# System uptime (maybe it needs a reboot)
echo "8. System uptime:"
uptime
echo

echo "=== Analysis ==="
echo "If swap won't clear after swapoff/swapon, possible causes:"
echo "1. Processes in D-state (uninterruptible sleep) holding swap"
echo "2. Kernel modules or drivers with memory leaks"
echo "3. Filesystem cache or buffers stuck in swap"
echo "4. System needs a reboot to fully clear kernel structures"