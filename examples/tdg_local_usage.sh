#!/bin/bash
# Sprint 31 - Local TDG System Usage Examples

echo "=== TDG System Local Usage Examples ==="
echo

# Build the project with minimal features for faster compilation
echo "1. Building the project (Rust-only for speed)..."
cargo build --package pmat --no-default-features --features rust-only,demo

# Basic TDG analysis on a single file
echo -e "\n2. Analyzing a single Rust file..."
./target/debug/pmat tdg server/src/tdg/analyzer_ast.rs

# Analyze entire TDG module
echo -e "\n3. Analyzing the TDG module directory..."
./target/debug/pmat tdg server/src/tdg --top-files 5

# Compare two implementations
echo -e "\n4. Comparing simple vs AST analyzer implementations..."
./target/debug/pmat tdg compare server/src/tdg/analyzer_simple.rs server/src/tdg/analyzer_ast.rs

# Start the web dashboard (will run in background)
echo -e "\n5. Starting TDG Web Dashboard..."
echo "   Dashboard will be available at http://localhost:8081"
echo "   Press Ctrl+C to stop"
./target/debug/pmat tdg dashboard --port 8081 --host 127.0.0.1 &
DASHBOARD_PID=$!

# Wait a moment for dashboard to start
sleep 2

# Test dashboard endpoints
echo -e "\n6. Testing dashboard API endpoints..."
echo "   Getting system metrics:"
curl -s http://localhost:8081/api/metrics | jq '.health_status.overall' 2>/dev/null || echo "Dashboard starting..."

echo "   Getting storage statistics:"
curl -s http://localhost:8081/api/storage/stats | jq '.total_entries' 2>/dev/null || echo "Storage initializing..."

# Show diagnostics
echo -e "\n7. Running TDG diagnostics..."
./target/debug/pmat tdg diagnostics --all

# Export analysis results
echo -e "\n8. Exporting analysis results in different formats..."
./target/debug/pmat tdg server/src/main.rs --format json > tdg_analysis.json
./target/debug/pmat tdg server/src/main.rs --format markdown > tdg_analysis.md
echo "   Exported to tdg_analysis.json and tdg_analysis.md"

# Clean up
echo -e "\n9. Cleaning up..."
kill $DASHBOARD_PID 2>/dev/null
echo "   Dashboard stopped"

echo -e "\n=== TDG System Ready for Local Development ==="
echo "Key features available:"
echo "  - TDG analysis with 6 orthogonal metrics"
echo "  - Web dashboard with real-time monitoring"
echo "  - MCP tools integration for external clients"
echo "  - Multi-format export (JSON, CSV, SARIF, HTML, Markdown)"
echo "  - Performance profiling and bottleneck detection"
echo "  - Alert system with configurable thresholds"
echo "  - Metrics aggregation and trending"
echo
echo "To use MCP tools, start the MCP server:"
echo "  ./target/debug/pmat mcp serve"
echo
echo "For more options, run:"
echo "  ./target/debug/pmat tdg --help"