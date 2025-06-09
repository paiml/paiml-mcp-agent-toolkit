# Overnight Refactor Execution Plan

## 🎯 Objectives

The automated overnight code repair state machine will:

1. **Fix High Complexity Functions** (43 errors, 91 warnings)
   - Target: Reduce all functions to cyclomatic complexity < 20
   - Priority files: `cli/mod.rs` (functions up to 75 complexity)

2. **Remove TRACKED Comments** (20+ instances)
   - Complete placeholder implementations
   - Replace with production-ready code

3. **Eliminate SATD** (Zero tolerance policy)
   - Remove all TODO, FIXME, HACK, XXX markers
   - Replace with proper implementations

4. **Apply Refactoring Operations**
   - Extract complex functions
   - Flatten deep nesting
   - Remove dead code
   - Consolidate duplicates

## 📊 Current State

### Complexity Hotspots:
1. `handle_analyze_graph_metrics_legacy` - complexity: 75
2. `handle_analyze_duplicates` - complexity: 50
3. `handle_analyze_defect_prediction` - complexity: 46
4. `handle_analyze_name_similarity` - complexity: 45
5. `handle_analyze_proof_annotations` - complexity: 45

### Estimated Technical Debt: 532.8 hours

## 🔄 State Machine Process

```
1. ANALYZING (Scan 434 files)
   ↓
2. PLANNING (Prioritize by TDG score, complexity, churn)
   ↓
3. EXECUTING (Apply refactorings)
   ↓
4. VALIDATING (Run tests, check coverage)
   ↓
5. CHECKPOINT (Save progress every 5 minutes)
```

## ⚡ Execution Strategy

### Batch Processing:
- 50 files per batch
- 8 parallel workers
- Priority: High complexity files first

### Validation After Each Batch:
- Run `make test-fast`
- Run `make lint`
- Check coverage remains > 80%
- Auto-commit successful changes

### Rollback on Failure:
- Revert failed transformations
- Skip problematic files
- Continue with next batch

## 📈 Expected Outcomes

After 12 hours of autonomous operation:

1. **All functions below complexity 20**
2. **Zero SATD markers**
3. **Complete TRACKED implementations**
4. **Improved maintainability score**
5. **Detailed report in `docs/bugs/prepare-release.md`**
6. **Git history of all automated fixes**

## 🚀 To Start

```bash
# Quick start
./start-overnight-repair.sh

# Or full command
./target/release/pmat refactor serve \
  --refactor-mode batch \
  --config refactor-config.json \
  --project . \
  --parallel 8 \
  --memory-limit 16384 \
  --batch-size 50 \
  --checkpoint-dir .refactor_state \
  --resume \
  --auto-commit "refactor: automated fix via state machine [skip ci]" \
  --max-runtime 43200
```

## 📊 Monitoring

```bash
# Watch progress
tail -f refactor_overnight_full.log

# Check current state
cat .refactor_state/checkpoint.json | jq '.current'

# View metrics evolution
grep "FIXED:" refactor_overnight_full.log | wc -l
```

The system is designed to run unattended overnight, systematically improving code quality while maintaining all tests passing.