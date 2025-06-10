#!/bin/bash
# Quick start script for Kaizen overnight optimization

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         KAIZEN OVERNIGHT OPTIMIZATION - QUICK START              ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check for tmux
if ! command -v tmux &> /dev/null; then
    echo "Installing tmux..."
    sudo apt-get update && sudo apt-get install -y tmux || brew install tmux
fi

# Check for required Rust tools
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Cargo not found. Please install Rust first.${NC}"
    exit 1
fi

# Check for monitoring tools
if ! command -v iostat &> /dev/null; then
    echo "Installing sysstat for I/O monitoring..."
    sudo apt-get install -y sysstat || brew install sysstat
fi

echo -e "${GREEN}✓ Prerequisites satisfied${NC}"
echo

# Create necessary directories
mkdir -p {monitoring/{logs,metrics,alerts},checkpoints,benchmarks}

# Initialize git branch if needed
if ! git rev-parse --verify "perf/kaizen-$(date +%Y%m%d)" >/dev/null 2>&1; then
    echo "Creating performance optimization branch..."
    git checkout -b "perf/kaizen-$(date +%Y%m%d)"
fi

# Launch monitoring dashboard
echo -e "${CYAN}Launching monitoring dashboard...${NC}"
if tmux has-session -t kaizen_monitor 2>/dev/null; then
    tmux kill-session -t kaizen_monitor
fi

# Start the companion monitor in a new terminal (if available)
if command -v gnome-terminal &> /dev/null; then
    gnome-terminal --title="Kaizen Monitor" -- bash -c "./kaizen-monitor-companion.sh; exec bash" &
elif command -v xterm &> /dev/null; then
    xterm -title "Kaizen Monitor" -e "./kaizen-monitor-companion.sh" &
else
    echo -e "${YELLOW}Launch companion monitor manually: ./kaizen-monitor-companion.sh${NC}"
fi

# Main optimization process
echo -e "${CYAN}Starting main optimization process...${NC}"
echo -e "${YELLOW}This will run in foreground mode for active monitoring${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop at any time (progress will be saved)${NC}"
echo

# Show initial status
echo -e "${GREEN}Initial Codebase Statistics:${NC}"
echo -n "Total Rust files: "
find src -name "*.rs" -type f | wc -l
echo -n "Total lines of code: "
find src -name "*.rs" -type f -exec wc -l {} + | tail -1 | awk '{print $1}'
echo -n "Current compilation time: "
time -p cargo build --release 2>&1 | grep real | awk '{print $2 "s"}'
echo

# Countdown
for i in {5..1}; do
    echo -ne "\rStarting optimization in ${YELLOW}$i${NC} seconds... "
    sleep 1
done
echo -e "\r${GREEN}Starting optimization now!${NC}                "
echo

# Launch main optimization
exec ./kaizen-overnight-optimization.sh