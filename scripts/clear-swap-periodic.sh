#!/bin/bash
# Periodic swap clearing script for long-running processes
# Can be run via cron during overnight refactoring

SWAP_THRESHOLD=50  # Clear swap if usage exceeds this percentage
LOG_FILE="${LOG_FILE:-/tmp/swap-clear.log}"

log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

check_and_clear_swap() {
    if ! command -v sudo >/dev/null 2>&1; then
        log_message "ERROR: sudo not available"
        return 1
    fi

    # Get swap usage
    SWAP_USED=$(free -b | grep Swap | awk '{print $3}')
    SWAP_TOTAL=$(free -b | grep Swap | awk '{print $2}')
    
    if [ "$SWAP_TOTAL" -eq 0 ]; then
        log_message "INFO: No swap configured"
        return 0
    fi
    
    SWAP_PERCENT=$((SWAP_USED * 100 / SWAP_TOTAL))
    
    log_message "INFO: Swap usage: ${SWAP_PERCENT}% ($(numfmt --to=iec $SWAP_USED) / $(numfmt --to=iec $SWAP_TOTAL))"
    
    if [ "$SWAP_PERCENT" -gt "$SWAP_THRESHOLD" ]; then
        log_message "WARNING: Swap usage exceeds threshold (${SWAP_PERCENT}% > ${SWAP_THRESHOLD}%)"
        
        # Check if overnight refactor is running
        if [ -f ".refactor_state/refactor.pid" ]; then
            PID=$(cat .refactor_state/refactor.pid)
            if ps -p "$PID" > /dev/null 2>&1; then
                log_message "INFO: Overnight refactor running (PID: $PID), clearing swap..."
                
                # Clear swap
                sudo sync
                sudo sh -c "echo 3 > /proc/sys/vm/drop_caches" 2>/dev/null || true
                sudo swapoff -a && sudo swapon -a 2>/dev/null || {
                    log_message "ERROR: Failed to clear swap"
                    return 1
                }
                
                # Check new usage
                NEW_SWAP_USED=$(free -b | grep Swap | awk '{print $3}')
                NEW_SWAP_PERCENT=$((NEW_SWAP_USED * 100 / SWAP_TOTAL))
                log_message "SUCCESS: Swap cleared (${SWAP_PERCENT}% -> ${NEW_SWAP_PERCENT}%)"
            else
                log_message "INFO: Refactor process not running, skipping swap clear"
            fi
        else
            log_message "INFO: No active refactor process, clearing swap anyway..."
            sudo sync
            sudo sh -c "echo 3 > /proc/sys/vm/drop_caches" 2>/dev/null || true
            sudo swapoff -a && sudo swapon -a 2>/dev/null || true
            log_message "SUCCESS: Swap cleared"
        fi
    else
        log_message "INFO: Swap usage below threshold, no action needed"
    fi
}

# Main execution
main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --threshold)
                SWAP_THRESHOLD="$2"
                shift 2
                ;;
            --log)
                LOG_FILE="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [--threshold PERCENT] [--log FILE]"
                echo "  --threshold PERCENT  Clear swap if usage exceeds PERCENT (default: 50)"
                echo "  --log FILE          Log file path (default: /tmp/swap-clear.log)"
                echo ""
                echo "Example cron entry (every 30 minutes):"
                echo "*/30 * * * * cd /path/to/project && ./scripts/clear-swap-periodic.sh"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    check_and_clear_swap
}

# Run if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi