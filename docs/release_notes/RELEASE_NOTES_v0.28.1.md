# Release Notes v0.28.1

## Bug Fixes

### Fixed: All analyze commands now display top 10 files by default

Fixed a major defect where analyze sub-commands did not print top 10 files by default. This release ensures consistent behavior across all analysis commands.

#### Commands Fixed:
- **analyze complexity** - Shows "Top Files by Complexity" sorted by total complexity score
- **analyze churn** - Shows "Top Files by Churn" sorted by commit count and churn score  
- **analyze satd** - Shows "Top Files with SATD" sorted by SATD item count
- **analyze lint-hotspot** - Shows "Top Files with Lint Issues" sorted by defect density
- **analyze deep-context** - Shows "Top Files by Complexity" with weighted scoring
- **analyze provability** - Shows "Top Files by Provability" with average scores per file
- **analyze duplicates** - Shows "Top Files by Duplication" sorted by duplication percentage
- **analyze defect-prediction** - Shows "Top Files by Defect Risk" with risk scores
- **analyze comprehensive** - Fixed to use real handler with "Top 10 Hotspot Files"
- **analyze proof-annotations** - Shows "Top Files with Proof Annotations" by annotation count
- **analyze incremental-coverage** - Shows "Top Files by Coverage Change" sorted by delta
- **analyze symbol-table** - Shows "Top Files by Symbol Count" 
- **analyze big-o** - Shows "Top Files by Complexity" with weighted scoring

#### Commands Already Working:
- **analyze dag** - Visualization output (no top files needed)
- **analyze dead-code** - Already had top files support
- **analyze tdg** - Already had top_files support in stub
- **analyze graph-metrics** - Already shows top nodes by centrality
- **analyze name-similarity** - Already supports --top-k parameter

#### Implementation Details:
- All commands respect the `--top-files` parameter (default: 10)
- Added doctests for formatting functions where applicable
- Consistent output formatting across all commands
- Top files sections show relevant metrics for each analysis type

## Other Improvements
- Fixed unused variable warnings in handlers
- Enhanced stub implementations with realistic data
- Improved documentation with examples

## Contributors
- Human-AI collaboration with Claude

## Upgrade Instructions
```bash
cargo install pmat --force
```

Or update your Cargo.toml:
```toml
[dependencies]
pmat = "0.28.1"
```