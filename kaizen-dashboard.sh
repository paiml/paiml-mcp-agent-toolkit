#!/bin/bash
# Kaizen Real-time Dashboard

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# State tracking
ITERATION=0
TOTAL_IMPROVEMENT=0
START_TIME=$(date +%s)

# Function to display dashboard
display_dashboard() {
    clear
    local current_time=$(date +%s)
    local elapsed=$((current_time - START_TIME))
    local elapsed_min=$((elapsed / 60))
    local elapsed_sec=$((elapsed % 60))
    
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                    KAIZEN OPTIMIZATION DASHBOARD                       ║${NC}"
    echo -e "${CYAN}║                                                                        ║${NC}"
    printf "${CYAN}║${NC}  Runtime: ${GREEN}%02d:%02d${NC}  │  Iteration: ${YELLOW}%d${NC}  │  Total Gain: ${GREEN}%.1f%%${NC}         ${CYAN}║${NC}\n" \
        $elapsed_min $elapsed_sec $ITERATION $TOTAL_IMPROVEMENT
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════════╝${NC}"
    echo
}

# Function to show progress bar
progress_bar() {
    local current=$1
    local total=$2
    local width=50
    local percent=$((current * 100 / total))
    local filled=$((width * current / total))
    
    printf "["
    printf "%${filled}s" | tr ' ' '█'
    printf "%$((width - filled))s" | tr ' ' '░'
    printf "] %3d%%\n" $percent
}

# Main monitoring loop
main() {
    display_dashboard
    
    echo -e "${WHITE}=== CURRENT STATUS ===${NC}"
    echo
    
    # Check if optimization is running
    if [[ -f optimization_state.json ]]; then
        local state=$(jq -r '.current_state // "UNKNOWN"' optimization_state.json 2>/dev/null)
        local iteration=$(jq -r '.iteration // 0' optimization_state.json 2>/dev/null)
        local improvement=$(jq -r '.total_improvement // 0' optimization_state.json 2>/dev/null)
        
        ITERATION=$iteration
        TOTAL_IMPROVEMENT=$improvement
        
        echo -e "State Machine: ${CYAN}$state${NC}"
        echo -n "Progress: "
        
        case "$state" in
            "INITIALIZE") progress_bar 1 8 ;;
            "ANALYZE_BASELINE") progress_bar 2 8 ;;
            "IDENTIFY_BOTTLENECKS") progress_bar 3 8 ;;
            "APPLY_OPTIMIZATION") progress_bar 4 8 ;;
            "RUN_BENCHMARKS") progress_bar 5 8 ;;
            "VALIDATE_IMPROVEMENT") progress_bar 6 8 ;;
            "COMMIT_CHANGES") progress_bar 7 8 ;;
            "COMPLETE") progress_bar 8 8 ;;
            *) progress_bar 0 8 ;;
        esac
        echo
    else
        echo "Optimization not yet started..."
        echo
    fi
    
    # System resources
    echo -e "${WHITE}=== SYSTEM RESOURCES ===${NC}"
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    local mem_total=$(free -m | awk 'NR==2{print $2}')
    local mem_used=$(free -m | awk 'NR==2{print $3}')
    local mem_percent=$((mem_used * 100 / mem_total))
    
    printf "CPU:    [${YELLOW}%5.1f%%${NC}] " "$cpu_usage"
    local cpu_bar=$((${cpu_usage%.*} / 2))
    for ((i=0; i<50; i++)); do
        if ((i < cpu_bar)); then
            printf "${RED}█${NC}"
        else
            printf "░"
        fi
    done
    echo
    
    printf "Memory: [${YELLOW}%5.1f%%${NC}] " "$mem_percent"
    local mem_bar=$((mem_percent / 2))
    for ((i=0; i<50; i++)); do
        if ((i < mem_bar)); then
            printf "${YELLOW}█${NC}"
        else
            printf "░"
        fi
    done
    printf " (%dMB / %dMB)\n" "$mem_used" "$mem_total"
    echo
    
    # Build status
    echo -e "${WHITE}=== BUILD STATUS ===${NC}"
    if pgrep -f "cargo build" >/dev/null; then
        echo -e "Status: ${GREEN}● Building${NC}"
        
        # Count compiled crates
        local compiled=$(ps aux | grep -E "rustc|cargo" | wc -l)
        echo "Active compilations: $compiled"
    else
        echo -e "Status: ${YELLOW}○ Idle${NC}"
    fi
    echo
    
    # Recent optimizations
    if [[ -f optimization_iterations.log ]]; then
        echo -e "${WHITE}=== RECENT ACTIVITY ===${NC}"
        tail -5 optimization_iterations.log | while IFS= read -r line; do
            if [[ "$line" =~ "Successfully" ]]; then
                echo -e "${GREEN}✓${NC} $line"
            elif [[ "$line" =~ "Failed" ]]; then
                echo -e "${RED}✗${NC} $line"
            else
                echo "  $line"
            fi
        done
        echo
    fi
    
    # Performance metrics
    if [[ -f optimization_results.json ]]; then
        echo -e "${WHITE}=== PERFORMANCE METRICS ===${NC}"
        local baseline=$(jq -r '.baseline_seconds // 0' optimization_results.json 2>/dev/null)
        local optimized=$(jq -r '.optimized_seconds // 0' optimization_results.json 2>/dev/null)
        local gain=$(jq -r '.improvement_percent // 0' optimization_results.json 2>/dev/null)
        
        if [[ "$baseline" != "0" ]]; then
            printf "Baseline:  ${BLUE}%3ds${NC}\n" "$baseline"
            printf "Optimized: ${GREEN}%3ds${NC} (%.1f%% improvement)\n" "$optimized" "$gain"
            
            # Visual comparison
            echo -n "Baseline:  "
            for ((i=0; i<baseline/2; i++)); do printf "${BLUE}█${NC}"; done
            echo
            echo -n "Optimized: "
            for ((i=0; i<optimized/2; i++)); do printf "${GREEN}█${NC}"; done
            echo
        fi
    fi
    
    echo
    echo -e "${CYAN}Press Ctrl+C to stop monitoring${NC}"
}

# Signal handler
trap 'echo -e "\n${YELLOW}Monitoring stopped${NC}"; exit 0' INT TERM

# Main loop
while true; do
    main
    sleep 2
done