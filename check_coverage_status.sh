#!/bin/bash
# Quick coverage status check

PID=262918
REPORT_FILE="coverage_report.txt"
TOTAL_TESTS=1090

if ps -p $PID > /dev/null 2>&1; then
    # Count completed tests
    COMPLETED=$(grep -c "^test " $REPORT_FILE 2>/dev/null || echo "0")
    PERCENT=$((COMPLETED * 100 / TOTAL_TESTS))
    
    echo "Coverage Status at $(date '+%H:%M:%S')"
    echo "=========================="
    echo "Progress: $COMPLETED/$TOTAL_TESTS tests ($PERCENT%)"
    echo "Status: Running..."
    
    # Show last test
    LAST_TEST=$(grep "^test " $REPORT_FILE 2>/dev/null | tail -1 || echo "Starting...")
    echo "Last test: $LAST_TEST"
else
    echo "Coverage Status at $(date '+%H:%M:%S')"
    echo "=========================="
    echo "Status: COMPLETED"
    
    # Check for final results
    if grep -q "Coverage Results" $REPORT_FILE 2>/dev/null; then
        echo ""
        echo "Final Coverage Report:"
        grep -A30 "Coverage Results" $REPORT_FILE | head -40
    else
        echo "Checking for test results..."
        grep "test result:" $REPORT_FILE 2>/dev/null | tail -5
    fi
fi