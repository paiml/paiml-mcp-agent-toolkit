#!/bin/bash
# Configure swap settings to prevent system crashes

echo "🔧 Configuring swap settings for large builds..."

# 1. Check current swap
echo "Current swap status:"
free -h

# 2. Set swappiness to a lower value (less aggressive swapping)
echo "Setting vm.swappiness to 10 (from default 60)..."
sudo sysctl vm.swappiness=10

# 3. Increase swap if needed
if [ "$1" == "--increase-swap" ]; then
    echo "Creating additional 16GB swap file..."
    sudo fallocate -l 16G /swapfile2
    sudo chmod 600 /swapfile2
    sudo mkswap /swapfile2
    sudo swapon /swapfile2
    echo "New swap status:"
    free -h
fi

# 4. Clear system caches to free memory
if [ "$1" == "--clear-cache" ]; then
    echo "Clearing system caches..."
    sync
    echo 3 | sudo tee /proc/sys/vm/drop_caches
    echo "Memory after cache clear:"
    free -h
fi

echo "✅ Swap configuration complete!"
echo ""
echo "Recommended test commands:"
echo "  Low memory:  CARGO_BUILD_JOBS=2 make test-fast"
echo "  Very low:    ./scripts/test-low-memory.sh"
echo "  Clear cache: ./scripts/config-swap.sh --clear-cache"