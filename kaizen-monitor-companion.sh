#!/bin/bash
# Kaizen Monitor Companion - Advanced Real-time Analysis
# Provides deep insights during overnight optimization runs

set -euo pipefail

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# Monitoring configuration
REFRESH_INTERVAL=2
ALERT_THRESHOLD_MEMORY_MB=1000
ALERT_THRESHOLD_CPU_PERCENT=80
COMPLEXITY_DANGER_THRESHOLD=30

# Create monitoring directory
mkdir -p monitoring/{logs,metrics,alerts}

# Function to display header
display_header() {
    clear
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║          KAIZEN CONTINUOUS IMPROVEMENT - LIVE MONITORING SYSTEM          ║${NC}"
    echo -e "${CYAN}║                    $(date +'%Y-%m-%d %H:%M:%S') - Iteration $(get_current_iteration)                    ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo
}

# Get current optimization iteration
get_current_iteration() {
    if [[ -f optimization_state.json ]]; then
        jq -r '.iteration // 1' optimization_state.json 2>/dev/null || echo "1"
    else
        echo "0"
    fi
}

# Monitor system resources
monitor_resources() {
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    local mem_usage=$(free -m | awk 'NR==2{printf "%.0f", $3}')
    local disk_io=$(iostat -d 1 2 | tail -n 2 | head -n 1 | awk '{print $3}')
    
    echo -e "${WHITE}=== SYSTEM RESOURCES ===${NC}"
    printf "CPU Usage:  ${YELLOW}%5.1f%%${NC}  " "$cpu_usage"
    
    # Visual CPU bar
    local cpu_bar_length=$((${cpu_usage%.*} / 5))
    printf "["
    for ((i=0; i<20; i++)); do
        if ((i < cpu_bar_length)); then
            printf "${RED}█${NC}"
        else
            printf " "
        fi
    done
    printf "]\n"
    
    printf "Memory:     ${YELLOW}%4dMB${NC}  " "$mem_usage"
    
    # Visual memory bar
    local mem_percent=$((mem_usage * 100 / $(free -m | awk 'NR==2{print $2}')))
    local mem_bar_length=$((mem_percent / 5))
    printf "["
    for ((i=0; i<20; i++)); do
        if ((i < mem_bar_length)); then
            printf "${YELLOW}█${NC}"
        else
            printf " "
        fi
    done
    printf "] %d%%\n" "$mem_percent"
    
    printf "Disk I/O:   ${YELLOW}%5.1fMB/s${NC}\n" "$disk_io"
    
    # Alert if thresholds exceeded
    if (( $(echo "$cpu_usage > $ALERT_THRESHOLD_CPU_PERCENT" | bc -l) )); then
        echo -e "${RED}⚠ HIGH CPU USAGE ALERT${NC}" | tee -a monitoring/alerts/resource_alerts.log
    fi
    
    if (( mem_usage > ALERT_THRESHOLD_MEMORY_MB )); then
        echo -e "${RED}⚠ HIGH MEMORY USAGE ALERT${NC}" | tee -a monitoring/alerts/resource_alerts.log
    fi
    
    echo
}

# Monitor compilation performance
monitor_compilation() {
    echo -e "${WHITE}=== COMPILATION METRICS ===${NC}"
    
    if [[ -f compilation_baseline.json ]] && [[ -f compilation_optimized.json ]]; then
        local baseline=$(jq -r '.mean' compilation_baseline.json 2>/dev/null || echo "0")
        local optimized=$(jq -r '.mean' compilation_optimized.json 2>/dev/null || echo "$baseline")
        local improvement=$(echo "scale=2; (($baseline - $optimized) / $baseline) * 100" | bc 2>/dev/null || echo "0")
        
        printf "Baseline:   ${BLUE}%6.2fs${NC}\n" "$baseline"
        printf "Current:    ${GREEN}%6.2fs${NC}\n" "$optimized"
        printf "Improvement: "
        
        if (( $(echo "$improvement > 0" | bc -l) )); then
            printf "${GREEN}%5.1f%%${NC} ↑\n" "$improvement"
        else
            printf "${RED}%5.1f%%${NC} ↓\n" "${improvement#-}"
        fi
    else
        echo "Waiting for benchmark data..."
    fi
    
    echo
}

# Monitor complexity improvements
monitor_complexity() {
    echo -e "${WHITE}=== COMPLEXITY ANALYSIS ===${NC}"
    
    if [[ -f high_complexity_functions.txt ]]; then
        local total_functions=$(wc -l < high_complexity_functions.txt 2>/dev/null || echo 0)
        local danger_functions=$(awk -F'|' -v t="$COMPLEXITY_DANGER_THRESHOLD" '$2 > t' high_complexity_functions.txt | wc -l)
        
        echo "High Complexity Functions: $total_functions"
        echo "Critical (>$COMPLEXITY_DANGER_THRESHOLD): ${RED}$danger_functions${NC}"
        
        # Show top 3 complex functions
        echo -e "\nTop Complex Functions:"
        head -3 high_complexity_functions.txt | while IFS='|' read -r func complexity file; do
            printf "  %-30s ${YELLOW}%3d${NC} %s\n" "${func:0:30}" "$complexity" "${file##*/}"
        done
    else
        echo "No complexity analysis available yet..."
    fi
    
    echo
}

# Monitor optimization progress
monitor_optimization_progress() {
    echo -e "${WHITE}=== OPTIMIZATION PROGRESS ===${NC}"
    
    if [[ -f optimization_state.json ]]; then
        local state=$(jq -r '.current_state' optimization_state.json)
        local iteration=$(jq -r '.iteration' optimization_state.json)
        local total_improvement=$(jq -r '.total_improvement' optimization_state.json)
        local successful=$(jq -r '.successful_optimizations // 0' optimization_state.json)
        local failed=$(jq -r '.failed_attempts // 0' optimization_state.json)
        
        # State machine visualization
        local states=("INITIALIZE" "ANALYZE_BASELINE" "IDENTIFY_BOTTLENECKS" "APPLY_OPTIMIZATION" "RUN_BENCHMARKS" "VALIDATE_IMPROVEMENT" "COMMIT_CHANGES" "NEXT_ITERATION")
        
        echo -n "Pipeline: "
        for s in "${states[@]}"; do
            if [[ "$s" == "$state" ]]; then
                printf "${GREEN}[%s]${NC} " "${s:0:3}"
            else
                printf "${WHITE}%s${NC} " "${s:0:3}"
            fi
        done
        echo
        
        printf "\nCurrent State: ${CYAN}%-20s${NC}\n" "$state"
        printf "Iteration:     ${YELLOW}%d${NC}\n" "$iteration"
        printf "Success Rate:  "
        
        local total_attempts=$((successful + failed))
        if ((total_attempts > 0)); then
            local success_rate=$(echo "scale=1; $successful * 100 / $total_attempts" | bc)
            printf "${GREEN}%.1f%%${NC} (%d/%d)\n" "$success_rate" "$successful" "$total_attempts"
        else
            printf "N/A\n"
        fi
        
        printf "Total Gain:    "
        if (( $(echo "$total_improvement > 0" | bc -l) )); then
            printf "${GREEN}%.2f%%${NC}\n" "$total_improvement"
        else
            printf "${YELLOW}%.2f%%${NC}\n" "$total_improvement"
        fi
    else
        echo "Optimization not started yet..."
    fi
    
    echo
}

# Monitor Kaizen adaptive parameters
monitor_kaizen_parameters() {
    echo -e "${WHITE}=== KAIZEN PARAMETERS ===${NC}"
    
    if [[ -f optimization_state.json ]]; then
        local threshold=$(jq -r '.improvement_threshold' optimization_state.json)
        local complexity_weight=$(jq -r '.adaptive_parameters.complexity_weight' optimization_state.json)
        local frequency_weight=$(jq -r '.adaptive_parameters.frequency_weight' optimization_state.json)
        
        printf "Improvement Threshold: ${YELLOW}%.3f%%${NC}\n" "$threshold"
        printf "Complexity Weight:     ${CYAN}%.2f${NC}\n" "$complexity_weight"
        printf "Frequency Weight:      ${CYAN}%.2f${NC}\n" "$frequency_weight"
    fi
    
    echo
}

# Show recent optimization activities
show_recent_activities() {
    echo -e "${WHITE}=== RECENT ACTIVITIES ===${NC}"
    
    if [[ -f optimization_iterations.log ]]; then
        tail -5 optimization_iterations.log | while read -r line; do
            if [[ "$line" =~ "Successfully optimized" ]]; then
                echo -e "${GREEN}✓${NC} $line"
            elif [[ "$line" =~ "Failed to optimize" ]]; then
                echo -e "${RED}✗${NC} $line"
            elif [[ "$line" =~ "State transition" ]]; then
                echo -e "${BLUE}→${NC} $line"
            else
                echo "  $line"
            fi
        done
    else
        echo "No activities logged yet..."
    fi
    
    echo
}

# Generate improvement graph (ASCII)
show_improvement_graph() {
    echo -e "${WHITE}=== IMPROVEMENT TREND ===${NC}"
    
    if [[ -f kaizen_metrics.json ]]; then
        # Extract last 10 improvements
        local improvements=$(jq -r '.improvement_history[-10:][].improvement' kaizen_metrics.json 2>/dev/null)
        
        if [[ -n "$improvements" ]]; then
            local max_imp=$(echo "$improvements" | sort -nr | head -1)
            local scale=$(echo "scale=2; 10 / ($max_imp + 0.1)" | bc)
            
            echo "$improvements" | tail -10 | while read -r imp; do
                local bar_length=$(echo "scale=0; $imp * $scale" | bc)
                printf "%5.1f%% |" "$imp"
                
                for ((i=0; i<bar_length; i++)); do
                    printf "${GREEN}█${NC}"
                done
                echo
            done
            
            echo "       └────────────────────────"
            echo "        0%                    ${max_imp}%"
        else
            echo "No improvement data yet..."
        fi
    fi
    
    echo
}

# Main monitoring loop
main_loop() {
    while true; do
        display_header
        
        # Create two-column layout
        {
            # Left column
            echo -e "${CYAN}┌─ System & Performance ─────────────┐${NC}"
            monitor_resources
            monitor_compilation
            monitor_complexity
            echo -e "${CYAN}└────────────────────────────────────┘${NC}"
        } | pr -t -w 80 > /tmp/left_col.tmp
        
        {
            # Right column
            echo -e "${CYAN}┌─ Optimization Progress ────────────┐${NC}"
            monitor_optimization_progress
            monitor_kaizen_parameters
            show_improvement_graph
            echo -e "${CYAN}└────────────────────────────────────┘${NC}"
        } | pr -t -w 80 > /tmp/right_col.tmp
        
        # Display side by side
        paste /tmp/left_col.tmp /tmp/right_col.tmp
        
        echo
        show_recent_activities
        
        # Show alerts if any
        if [[ -f monitoring/alerts/resource_alerts.log ]] && [[ -s monitoring/alerts/resource_alerts.log ]]; then
            echo -e "${RED}=== ALERTS ===${NC}"
            tail -3 monitoring/alerts/resource_alerts.log
        fi
        
        # Footer with controls
        echo -e "\n${CYAN}───────────────────────────────────────────────────────────────────────────${NC}"
        echo -e "Press ${YELLOW}Ctrl+C${NC} to stop monitoring | Refresh: ${GREEN}${REFRESH_INTERVAL}s${NC} | Logs: ${BLUE}monitoring/logs/${NC}"
        
        sleep $REFRESH_INTERVAL
    done
}

# Signal handler for graceful exit
trap 'echo -e "\n${YELLOW}Monitoring stopped.${NC}"; exit 0' INT TERM

# Check if optimization is running
check_optimization_running() {
    if [[ -f monitoring.pid ]]; then
        local pid=$(cat monitoring.pid)
        if ps -p "$pid" > /dev/null 2>&1; then
            echo -e "${GREEN}Optimization process is running (PID: $pid)${NC}"
        else
            echo -e "${RED}Optimization process not found (stale PID: $pid)${NC}"
        fi
    else
        echo -e "${YELLOW}No optimization process detected${NC}"
    fi
}

# Start monitoring
echo -e "${CYAN}Starting Kaizen Monitor Companion...${NC}"
check_optimization_running
echo -e "${CYAN}Beginning real-time monitoring...${NC}\n"

main_loop