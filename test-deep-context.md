# Deep Context Analysis: 
**Generated:** 2025-06-04 18:32:44 UTC
**Tool Version:** 0.21.0
**Analysis Time:** 5.07620139s

## Executive Summary
**Overall Health Score:** 75.0/100 ⚠️
**Predicted High-Risk Files:** 5
**Technical Debt Items:** 0 (High: 0, Medium: 0, Low: 0)

## Key Metrics
### Complexity
- **Median Cyclomatic:** 0.0
- **Max Cyclomatic:** 32 (./server/src/handlers/tools.rs:handle_analyze_system_architecture)
- **Violations:** 238

### Code Churn (30 days)
- **Median Changes:** 1
- **Max Changes:** 102 (server/Cargo.toml)
- **Hotspot Files:** 48

### Technical Debt (SATD)
- **Total Items:** 0
- **High Severity:** 0
- **Debt Hotspots:** 0 files

### Duplicates
- **Clone Coverage:** 0.0%
- **Type-1/2 Clones:** 0
- **Type-3/4 Clones:** 0

### Dead Code
- **Unreachable Functions:** 0
- **Dead Code %:** 0.0%

## AST Network Analysis
**Module Centrality (PageRank):**
1. main (score: 0.25)
2. lib (score: 0.20)
3. services (score: 0.15)

**Function Importance:**
1. main (connections: 15)
2. analyze_project (connections: 12)
3. process_files (connections: 8)

## Top 5 Predicted Defect Files
1. ./server/src/handlers/tools.rs (risk score: 12.8)
   - Complexity: 32, Churn: 0, SATD: 0
2. ./scripts/deep-context.ts (risk score: 11.6)
   - Complexity: 29, Churn: 0, SATD: 0
3. ./scripts/archive/dead-scripts/docker-setup.ts (risk score: 11.2)
   - Complexity: 28, Churn: 0, SATD: 0
4. ./server/scripts/docker-setup.ts (risk score: 11.2)
   - Complexity: 28, Churn: 0, SATD: 0
5. ./scripts/archive/dead-scripts/mcp-install-deterministic.ts (risk score: 10.0)
   - Complexity: 25, Churn: 0, SATD: 0
