# Analyze Commands Top Files Audit

## Commands WITH top-files functionality ✅

1. **complexity** - ✅ Has `top_files: usize` with default 10
2. **dead-code** - ✅ Has `top_files: Option<usize>` (no default, shows all)
3. **satd** - ✅ Has `top_files: usize` with default 10
4. **tdg** - ✅ Has `top: usize` with default 10 (different name but same functionality)
5. **duplicates** - ✅ Has `top_files: usize` with default 10
6. **big-o** - ✅ Has `top_files: usize` with default 10
7. **graph-metrics** - ✅ Has `top_k: usize` with default 20 (different name but same functionality)
8. **name-similarity** - ✅ Has `top_k: usize` with default 10 (different name but same functionality)

## Commands WITHOUT top-files functionality ❌

1. **churn** - ❌ No top files parameter
2. **dag** - ❌ No top files parameter (has target_nodes for graph reduction)
3. **deep-context** - ❌ No top files parameter
4. **lint-hotspot** - ❌ No top files parameter (analyzes single hotspot file)
5. **makefile** - ❌ No top files parameter (analyzes single makefile)
6. **provability** - ❌ No top files parameter
7. **defect-prediction** - ❌ No top files parameter
8. **comprehensive** - ❌ No top files parameter (aggregates all analyses)
9. **proof-annotations** - ❌ No top files parameter
10. **incremental-coverage** - ❌ No top files parameter
11. **symbol-table** - ❌ No top files parameter
12. **assembly-script** - ❌ No top files parameter
13. **web-assembly** - ❌ No top files parameter

## Summary

- **8 out of 21** analyze commands have top-files functionality
- Commands that produce file-based results generally have this feature
- Commands that analyze specific files or produce different kinds of output don't have it