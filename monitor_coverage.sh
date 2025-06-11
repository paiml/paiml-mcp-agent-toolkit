#!/bin/bash
# Monitor coverage analysis progress

PID=$1
REPORT_FILE="coverage_report.txt"

echo "Monitoring coverage analysis (PID: $PID)"
echo "================================================"

while kill -0 $PID 2>/dev/null; do
    # Check if report file exists and has content
    if [ -f "$REPORT_FILE" ]; then
        # Get last few lines to see progress
        echo -n "$(date '+%H:%M:%S') - "
        tail -1 "$REPORT_FILE" | grep -E "(Building|Testing|Coverage)" || echo "Processing..."
        
        # Check for any completed test info
        if grep -q "Coverage Results" "$REPORT_FILE" 2>/dev/null; then
            echo "Coverage analysis appears to be finalizing..."
        fi
    fi
    
    sleep 10
done

echo ""
echo "Coverage analysis completed!"
echo "================================================"

# Show final results
if [ -f "$REPORT_FILE" ]; then
    echo "Final Coverage Report:"
    echo ""
    # Extract coverage summary
    grep -A20 "Coverage Results" "$REPORT_FILE" 2>/dev/null || tail -50 "$REPORT_FILE"
fi