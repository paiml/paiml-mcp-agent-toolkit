#!/bin/bash
# Kaizen Optimization for Server Directory with Active Monitoring

set -euo pipefail

# Colors
RED='\033[0;31m'
WHITE='\033[1;37m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
WORK_DIR="/home/noah/src/paiml-mcp-agent-toolkit/server"
STATE_FILE="$WORK_DIR/optimization_state.json"
LOG_FILE="$WORK_DIR/kaizen_log_$(date +%Y%m%d_%H%M%S).log"
BASELINE_TIME=74.5  # Known baseline

cd "$WORK_DIR"

log() {
    echo "[$(date +%H:%M:%S)] $1" | tee -a "$LOG_FILE"
}

show_header() {
    clear
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║          KAIZEN SERVER OPTIMIZATION - ACTIVE MONITOR             ║${NC}"
    echo -e "${CYAN}║                    $(date +'%Y-%m-%d %H:%M:%S')                    ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo
}

# Initialize optimization state
init_state() {
    cat > "$STATE_FILE" << EOF
{
  "current_state": "ANALYZE",
  "iteration": 1,
  "total_improvement": 0.0,
  "optimizations_applied": 0,
  "files_optimized": [],
  "baseline_time": $BASELINE_TIME
}
EOF
}

# Find optimization opportunities
find_opportunities() {
    log "Searching for optimization opportunities..."
    
    # Find files with potential optimizations
    local opportunities=0
    
    # Pattern 1: Vec::new() that could be pre-allocated
    log "Checking for vector allocations..."
    if rg -l "Vec::new\(\)" src/ 2>/dev/null | head -5; then
        ((opportunities++))
    fi
    
    # Pattern 2: String::new() that could be pre-allocated
    log "Checking for string allocations..."
    if rg -l "String::new\(\)" src/ 2>/dev/null | head -5; then
        ((opportunities++))
    fi
    
    # Pattern 3: Nested loops
    log "Checking for nested loops..."
    if rg -U "for.*\{[\s\S]*?for" src/ 2>/dev/null | head -5; then
        ((opportunities++))
    fi
    
    # Pattern 4: Large functions
    log "Checking for large functions..."
    find src -name "*.rs" -size +50k 2>/dev/null | while read file; do
        log "Large file found: $file"
        ((opportunities++))
    done
    
    echo $opportunities
}

# Apply specific optimizations
apply_optimizations() {
    local applied=0
    
    show_header
    echo -e "${YELLOW}=== APPLYING OPTIMIZATIONS ===${NC}"
    echo
    
    # Optimization 1: Pre-allocate vectors in hot paths
    log "Optimizing vector allocations..."
    for file in src/services/{dag_builder,mermaid_generator,complexity,ast_based_dependency_analyzer}.rs; do
        if [[ -f "$file" ]]; then
            if grep -q "Vec::new()" "$file"; then
                cp "$file" "$file.backup"
                sed -i 's/Vec::new()/Vec::with_capacity(256)/g' "$file"
                if cargo check --quiet 2>/dev/null; then
                    log "✓ Optimized vectors in $(basename $file)"
                    ((applied++))
                else
                    mv "$file.backup" "$file"
                    log "✗ Failed to optimize $(basename $file)"
                fi
                rm -f "$file.backup"
            fi
        fi
    done
    
    # Optimization 2: Pre-allocate strings
    log "Optimizing string allocations..."
    for file in src/services/{mermaid_generator,template_service,renderer}.rs; do
        if [[ -f "$file" ]]; then
            if grep -q "String::new()" "$file"; then
                cp "$file" "$file.backup"
                sed -i 's/String::new()/String::with_capacity(4096)/g' "$file"
                if cargo check --quiet 2>/dev/null; then
                    log "✓ Optimized strings in $(basename $file)"
                    ((applied++))
                else
                    mv "$file.backup" "$file"
                    log "✗ Failed to optimize $(basename $file)"
                fi
                rm -f "$file.backup"
            fi
        fi
    done
    
    # Optimization 3: Add inline hints
    log "Adding inline hints to hot functions..."
    for file in src/services/{ast_typescript,ast_rust,ast_python}.rs; do
        if [[ -f "$file" ]]; then
            cp "$file" "$file.backup"
            # Add inline to parse functions
            sed -i 's/pub fn parse/\#[inline]\npub fn parse/g' "$file"
            sed -i 's/\#\[inline\]\n\#\[inline\]/\#[inline]/g' "$file"  # Remove duplicates
            if cargo check --quiet 2>/dev/null; then
                log "✓ Added inline hints to $(basename $file)"
                ((applied++))
            else
                mv "$file.backup" "$file"
            fi
            rm -f "$file.backup"
        fi
    done
    
    echo
    echo -e "${GREEN}Applied $applied optimizations${NC}"
    
    # Update state
    jq --arg n "$applied" '.optimizations_applied += ($n | tonumber)' "$STATE_FILE" > tmp.$$ && \
        mv tmp.$$ "$STATE_FILE"
    
    return $applied
}

# Measure build performance
measure_performance() {
    show_header
    echo -e "${BLUE}=== MEASURING PERFORMANCE ===${NC}"
    echo
    
    log "Cleaning build artifacts..."
    cargo clean >/dev/null 2>&1
    
    log "Building release version..."
    local start=$(date +%s)
    
    # Show progress
    cargo build --release 2>&1 | while read line; do
        if [[ "$line" =~ Compiling ]]; then
            echo -ne "\r${YELLOW}Building:${NC} $line                    "
        fi
    done
    
    local end=$(date +%s)
    local build_time=$((end - start))
    
    echo -e "\n${GREEN}Build completed in ${build_time}s${NC}"
    
    # Calculate improvement
    local improvement=$(echo "scale=2; (($BASELINE_TIME - $build_time) / $BASELINE_TIME) * 100" | bc)
    
    log "Build time: ${build_time}s (improvement: ${improvement}%)"
    
    # Update state
    jq --arg t "$build_time" --arg i "$improvement" '
        .last_build_time = ($t | tonumber) |
        .last_improvement = ($i | tonumber) |
        .total_improvement += ($i | tonumber)
    ' "$STATE_FILE" > tmp.$$ && mv tmp.$$ "$STATE_FILE"
    
    echo "$improvement"
}

# Main optimization loop
main() {
    log "Starting Kaizen Server Optimization"
    
    # Initialize if needed
    if [[ ! -f "$STATE_FILE" ]]; then
        init_state
    fi
    
    local iteration=1
    local total_improvement=0
    
    while true; do
        show_header
        
        # Display current status
        echo -e "${WHITE}Iteration: $iteration${NC}"
        echo -e "${WHITE}Total Improvement: ${GREEN}${total_improvement}%${NC}"
        echo
        
        # Find opportunities
        echo -e "${CYAN}Phase 1: Analysis${NC}"
        local opportunities=$(find_opportunities)
        echo "Found $opportunities optimization opportunities"
        echo
        
        if [[ $opportunities -eq 0 ]]; then
            log "No more optimization opportunities found"
            break
        fi
        
        # Apply optimizations
        echo -e "${CYAN}Phase 2: Optimization${NC}"
        if apply_optimizations; then
            echo
            
            # Measure improvement
            echo -e "${CYAN}Phase 3: Validation${NC}"
            local improvement=$(measure_performance)
            
            if (( $(echo "$improvement > 0" | bc -l) )); then
                total_improvement=$(echo "$total_improvement + $improvement" | bc)
                log "Iteration $iteration: ${improvement}% improvement"
                
                # Commit changes
                echo
                echo -e "${CYAN}Phase 4: Commit${NC}"
                git add -u 2>/dev/null || true
                git commit -m "perf: Kaizen iteration $iteration - ${improvement}% improvement" 2>/dev/null || {
                    log "No changes to commit"
                }
            else
                log "No improvement detected"
            fi
        fi
        
        # Check if we should continue
        ((iteration++))
        if [[ $iteration -gt 10 ]] || (( $(echo "$total_improvement > 30" | bc -l) )); then
            log "Optimization goals reached"
            break
        fi
        
        # Update state
        jq --arg i "$iteration" '.iteration = ($i | tonumber)' "$STATE_FILE" > tmp.$$ && \
            mv tmp.$$ "$STATE_FILE"
        
        echo
        echo -e "${YELLOW}Starting next iteration in 10 seconds...${NC}"
        echo "Press Ctrl+C to stop"
        sleep 10
    done
    
    # Final report
    show_header
    echo -e "${GREEN}=== OPTIMIZATION COMPLETE ===${NC}"
    echo
    echo "Total Iterations: $((iteration - 1))"
    echo "Total Improvement: ${total_improvement}%"
    echo "Final Build Time: $(jq -r '.last_build_time // "unknown"' "$STATE_FILE")s"
    echo
    echo "Optimizations Applied:"
    jq -r '.files_optimized[]? // empty' "$STATE_FILE" | sort -u
    echo
    echo "Log file: $LOG_FILE"
}

# Signal handler
trap 'echo -e "\n${YELLOW}Optimization interrupted${NC}"; exit 1' INT TERM

# Run optimization
main