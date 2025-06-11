#!/bin/bash
# Kaizen Continuous Monitoring System - Run all night

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m'

# Configuration
CYCLE_INTERVAL=300  # 5 minutes between optimization cycles
MAX_CYCLES=100      # Run up to 100 cycles overnight
IMPROVEMENT_TARGET=50  # Target 50% total improvement
LOG_DIR="kaizen_logs"
CHECKPOINT_DIR="kaizen_checkpoints"

# Create directories
mkdir -p "$LOG_DIR" "$CHECKPOINT_DIR"

# Initialize metrics
TOTAL_IMPROVEMENT=0
CYCLES_RUN=0
START_TIME=$(date +%s)
BASELINE_BUILD_TIME=74.5

# Log function
log() {
    local msg="$1"
    local level="${2:-INFO}"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "[${timestamp}] [${level}] ${msg}" | tee -a "$LOG_DIR/kaizen_main.log"
}

# Display dashboard
show_dashboard() {
    clear
    local current_time=$(date +%s)
    local elapsed=$((current_time - START_TIME))
    local hours=$((elapsed / 3600))
    local minutes=$(((elapsed % 3600) / 60))
    
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║              KAIZEN CONTINUOUS OPTIMIZATION MONITOR                    ║${NC}"
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════════════════╣${NC}"
    printf "${CYAN}║${NC} Runtime: ${GREEN}%02d:%02d${NC} │ Cycles: ${YELLOW}%d${NC} │ Target: ${BLUE}%d%%${NC} │ Achieved: ${GREEN}%.1f%%${NC}     ${CYAN}║${NC}\n" \
        $hours $minutes $CYCLES_RUN $IMPROVEMENT_TARGET $TOTAL_IMPROVEMENT
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════════╝${NC}"
    echo
}

# Find and apply optimizations
run_optimization_cycle() {
    local cycle_num=$1
    log "Starting optimization cycle $cycle_num" "INFO"
    
    # Change to server directory
    cd server 2>/dev/null || cd /home/noah/src/paiml-mcp-agent-toolkit/server
    
    # Find optimization opportunities
    local opportunities=0
    
    # Pattern 1: Unoptimized allocations
    if rg -q "Vec::new\(\)|String::new\(\)" src/; then
        ((opportunities++))
        log "Found unoptimized allocations" "DEBUG"
    fi
    
    # Pattern 2: Missing inline hints
    if rg -q "^pub fn" src/ | grep -v "#\[inline\]"; then
        ((opportunities++))
        log "Found functions without inline hints" "DEBUG"
    fi
    
    # Pattern 3: Inefficient cloning
    if rg -q "\.clone\(\)\.clone\(\)" src/; then
        ((opportunities++))
        log "Found double cloning" "DEBUG"
    fi
    
    if [[ $opportunities -eq 0 ]]; then
        log "No optimization opportunities found" "WARN"
        return 1
    fi
    
    # Apply optimizations
    local changes_made=0
    
    # Optimization 1: Pre-allocate collections
    for file in $(rg -l "Vec::new\(\)" src/ | head -5); do
        if [[ -f "$file" ]]; then
            cp "$file" "$file.bak"
            sed -i 's/Vec::new()/Vec::with_capacity(128)/g' "$file"
            if cargo check --quiet 2>/dev/null; then
                log "✓ Optimized vectors in $file" "SUCCESS"
                ((changes_made++))
            else
                mv "$file.bak" "$file"
            fi
            rm -f "$file.bak"
        fi
    done
    
    # Optimization 2: Add strategic inline hints
    for file in $(find src -name "*.rs" -type f | head -10); do
        if grep -q "^pub fn.*parse\|analyze\|process" "$file"; then
            cp "$file" "$file.bak"
            sed -i '/^pub fn.*\(parse\|analyze\|process\)/i #[inline]' "$file"
            if cargo check --quiet 2>/dev/null; then
                log "✓ Added inline hints to $file" "SUCCESS"
                ((changes_made++))
            else
                mv "$file.bak" "$file"
            fi
            rm -f "$file.bak"
        fi
    done
    
    # Measure improvement
    if [[ $changes_made -gt 0 ]]; then
        log "Applied $changes_made optimizations, measuring impact..." "INFO"
        
        cargo clean >/dev/null 2>&1
        local start=$(date +%s)
        cargo build --release >/dev/null 2>&1
        local end=$(date +%s)
        local build_time=$((end - start))
        
        local improvement=$(echo "scale=2; (($BASELINE_BUILD_TIME - $build_time) / $BASELINE_BUILD_TIME) * 100" | bc)
        
        if (( $(echo "$improvement > 0" | bc -l) )); then
            TOTAL_IMPROVEMENT=$(echo "$TOTAL_IMPROVEMENT + $improvement" | bc)
            log "Cycle $cycle_num achieved ${improvement}% improvement" "SUCCESS"
            
            # Commit changes
            git add -u 2>/dev/null || true
            git commit -m "perf: Kaizen cycle $cycle_num - ${improvement}% improvement" 2>/dev/null || true
            
            return 0
        fi
    fi
    
    log "No measurable improvement in cycle $cycle_num" "WARN"
    return 1
}

# Create checkpoint
create_checkpoint() {
    local checkpoint_name="checkpoint_$(date +%Y%m%d_%H%M%S)"
    local checkpoint_path="$CHECKPOINT_DIR/$checkpoint_name"
    
    mkdir -p "$checkpoint_path"
    
    # Save state
    cat > "$checkpoint_path/state.json" << EOF
{
    "cycles_run": $CYCLES_RUN,
    "total_improvement": $TOTAL_IMPROVEMENT,
    "timestamp": "$(date -Iseconds)",
    "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')"
}
EOF
    
    # Save logs
    cp "$LOG_DIR"/*.log "$checkpoint_path/" 2>/dev/null || true
    
    log "Created checkpoint: $checkpoint_name" "INFO"
}

# Monitor system health
check_system_health() {
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    local mem_usage=$(free -m | awk 'NR==2{printf "%.1f", $3*100/$2}')
    
    if (( $(echo "$cpu_usage > 90" | bc -l) )); then
        log "High CPU usage: ${cpu_usage}%" "WARN"
        return 1
    fi
    
    if (( $(echo "$mem_usage > 90" | bc -l) )); then
        log "High memory usage: ${mem_usage}%" "WARN"
        return 1
    fi
    
    return 0
}

# Main monitoring loop
main() {
    log "Starting Kaizen Continuous Optimization System" "INFO"
    log "Target improvement: ${IMPROVEMENT_TARGET}%" "INFO"
    log "Cycle interval: ${CYCLE_INTERVAL}s" "INFO"
    
    while [[ $CYCLES_RUN -lt $MAX_CYCLES ]]; do
        show_dashboard
        
        # Check if target reached
        if (( $(echo "$TOTAL_IMPROVEMENT >= $IMPROVEMENT_TARGET" | bc -l) )); then
            log "Target improvement of ${IMPROVEMENT_TARGET}% achieved!" "SUCCESS"
            break
        fi
        
        # Check system health
        if ! check_system_health; then
            log "System health check failed, pausing..." "WARN"
            sleep 60
            continue
        fi
        
        # Display current status
        echo -e "${WHITE}=== CYCLE $((CYCLES_RUN + 1)) ===${NC}"
        echo "Current Total Improvement: ${GREEN}${TOTAL_IMPROVEMENT}%${NC}"
        echo
        
        # Run optimization cycle
        if run_optimization_cycle $((CYCLES_RUN + 1)); then
            ((CYCLES_RUN++))
            echo -e "${GREEN}✓ Cycle $CYCLES_RUN completed successfully${NC}"
        else
            echo -e "${YELLOW}⚠ Cycle failed or no improvements found${NC}"
        fi
        
        # Create checkpoint every 10 cycles
        if [[ $((CYCLES_RUN % 10)) -eq 0 ]] && [[ $CYCLES_RUN -gt 0 ]]; then
            create_checkpoint
        fi
        
        # Show next cycle countdown
        echo
        echo -e "${CYAN}Next cycle in ${CYCLE_INTERVAL} seconds...${NC}"
        echo "Press Ctrl+C to stop"
        
        # Countdown with progress bar
        for ((i=0; i<CYCLE_INTERVAL; i+=5)); do
            local progress=$((i * 50 / CYCLE_INTERVAL))
            printf "\r["
            printf "%${progress}s" | tr ' ' '█'
            printf "%$((50 - progress))s" | tr ' ' '░'
            printf "] %3d/%d" $i $CYCLE_INTERVAL
            sleep 5
        done
        echo
    done
    
    # Final report
    show_dashboard
    echo -e "${GREEN}=== OPTIMIZATION COMPLETE ===${NC}"
    echo
    echo "Total Cycles Run: $CYCLES_RUN"
    echo "Total Improvement: ${TOTAL_IMPROVEMENT}%"
    echo "Runtime: $(($(date +%s) - START_TIME))s"
    echo
    
    # Generate final report
    generate_final_report
}

# Generate comprehensive report
generate_final_report() {
    local report_file="$LOG_DIR/kaizen_final_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Kaizen Overnight Optimization Report

Generated: $(date)

## Summary
- Total Cycles: $CYCLES_RUN
- Total Improvement: ${TOTAL_IMPROVEMENT}%
- Runtime: $(($(date +%s) - START_TIME))s
- Target Achievement: $(echo "scale=1; $TOTAL_IMPROVEMENT * 100 / $IMPROVEMENT_TARGET" | bc)%

## Optimization Log
$(tail -50 "$LOG_DIR/kaizen_main.log")

## Git History
$(git log --oneline -10)

## Recommendations
- Review all changes before merging
- Run full test suite: \`cargo test --all-features\`
- Profile with flamegraph for deeper analysis
- Consider manual optimization of remaining hotspots
EOF
    
    log "Final report saved to: $report_file" "INFO"
    echo "Report saved to: $report_file"
}

# Signal handlers
trap 'log "Optimization interrupted by user" "WARN"; create_checkpoint; exit 0' INT TERM

# Start monitoring
main "$@"