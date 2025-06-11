#!/bin/bash
# Simplified Kaizen monitoring without tmux requirement

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}=== KAIZEN OPTIMIZATION - SIMPLE MONITOR ===${NC}"
echo -e "${CYAN}Starting optimization system without tmux...${NC}"
echo

# Initialize state
cat > optimization_state.json << EOF
{
  "current_state": "INITIALIZE",
  "iteration": 1,
  "total_improvement": 0.0,
  "successful_optimizations": 0,
  "failed_attempts": 0,
  "improvement_threshold": 0.05,
  "started_at": "$(date -Iseconds)"
}
EOF

# Create baseline branch
echo "Creating optimization branch..."
git checkout -b "perf/kaizen-$(date +%Y%m%d-%H%M)" 2>/dev/null || true

# Measure initial baseline
echo -e "${YELLOW}Measuring baseline performance...${NC}"
echo "Compilation baseline:"
time -p cargo build --release 2>&1 | grep real || echo "Build completed"

# Start monitoring loop
echo -e "${GREEN}Starting optimization monitor...${NC}"
echo "Press Ctrl+C to stop"
echo

while true; do
    clear
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║          KAIZEN OPTIMIZATION STATUS - $(date +%H:%M:%S)          ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo
    
    # Display current state
    if [[ -f optimization_state.json ]]; then
        echo "Current State: $(jq -r '.current_state' optimization_state.json)"
        echo "Iteration: $(jq -r '.iteration' optimization_state.json)"
        echo "Total Improvement: $(jq -r '.total_improvement' optimization_state.json)%"
        echo
    fi
    
    # Show recent activity
    echo "Recent Activity:"
    if [[ -f optimization_iterations.log ]]; then
        tail -5 optimization_iterations.log 2>/dev/null || echo "No activity yet..."
    fi
    echo
    
    # Basic system stats
    echo "System Resources:"
    echo "CPU: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%"
    echo "Memory: $(free -m | awk 'NR==2{printf "%dMB / %dMB (%.1f%%)", $3, $2, $3*100/$2}')"
    echo
    
    sleep 5
done