#!/bin/bash
# Profile context generation with flamegraph

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Setting up flamegraph profiling for context generation...${NC}"

# Check if required tools are installed
if ! command -v perf &> /dev/null; then
    echo -e "${RED}Error: 'perf' is not installed. Install with: sudo apt-get install linux-tools-common linux-tools-generic${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: 'cargo' is not installed.${NC}"
    exit 1
fi

# Install flamegraph if not already installed
if ! command -v flamegraph &> /dev/null; then
    echo -e "${YELLOW}Installing flamegraph...${NC}"
    cargo install flamegraph
fi

# Build release binary with debug symbols
echo -e "${YELLOW}Building release binary with debug symbols...${NC}"
CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --bin pmat

# Create output directory
mkdir -p profiling_results

# Profile the context command
echo -e "${YELLOW}Running context generation with profiling...${NC}"
echo -e "${YELLOW}This may require sudo permissions for perf...${NC}"

# Use flamegraph to profile
flamegraph -o profiling_results/context_flamegraph.svg -- \
    ./target/release/pmat context --toolchain rust -o profiling_results/profile_context.md

echo -e "${GREEN}✅ Flamegraph generated at: profiling_results/context_flamegraph.svg${NC}"

# Also generate a perf report
echo -e "${YELLOW}Generating detailed perf report...${NC}"
sudo perf record -F 997 -g -- ./target/release/pmat context --toolchain rust -o profiling_results/profile_context2.md
sudo perf report --stdio > profiling_results/perf_report.txt

echo -e "${GREEN}✅ Perf report generated at: profiling_results/perf_report.txt${NC}"

# Generate timing analysis
echo -e "${YELLOW}Running with timing analysis...${NC}"
RUST_LOG=paiml_mcp_agent_toolkit=info ./target/release/pmat context --toolchain rust -o profiling_results/profile_context3.md 2> profiling_results/timing_log.txt

echo -e "${GREEN}✅ Timing log generated at: profiling_results/timing_log.txt${NC}"

echo -e "${GREEN}Profiling complete! Results in profiling_results/${NC}"
echo -e "  - Flamegraph: profiling_results/context_flamegraph.svg"
echo -e "  - Perf report: profiling_results/perf_report.txt"
echo -e "  - Timing log: profiling_results/timing_log.txt"