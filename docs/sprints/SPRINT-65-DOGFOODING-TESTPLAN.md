# Sprint 65: Git-Commit Correlation Dogfooding Test Plan

**Version**: v2.179.0
**Feature**: `pmat tdg history` command
**Purpose**: Validate git-commit correlation on PMAT's own codebase
**Date**: October 28, 2025

---

## Prerequisites

1. ✅ pmat v2.179.0 built with appropriate storage backend
2. ✅ PMAT repository has git history
3. ⏳ Initial TDG analysis with git context

---

## Test Suite

### Test 1: Initial Analysis with Git Context

**Command**:
```bash
./target/release/pmat tdg server/src/cli/handlers/tdg_handlers.rs --with-git-context
```

**Expected**:
- ✅ Analysis completes successfully
- ✅ Output shows git context section:
  ```
  🔗 Git Context:
    ├─ Commit:  <sha>
    ├─ Branch:  master
    └─ Author:  <name>
  ```
- ✅ TDG score displayed (A+ through F)
- ✅ Record stored with git_context

**Validates**: Phase 2 git context capture

---

### Test 2: Query Specific Commit

**Command**:
```bash
# By short SHA (Sprint 65 Phase 3 commit)
./target/release/pmat tdg history --commit 3ca7373

# By tag (latest release)
./target/release/pmat tdg history --commit v2.178.0
```

**Expected**:
- ✅ Table output with matching records
- ✅ Shows: commit SHA, grade, score, branch, author, date, file
- ✅ Emoji 📝 displayed
- ✅ Box-drawing characters formatted correctly

**Validates**: Storage query by commit/tag, table formatter

---

### Test 3: Query History Since Reference

**Command**:
```bash
# Last 10 commits
./target/release/pmat tdg history --since HEAD~10

# Since v2.178.0
./target/release/pmat tdg history --since v2.178.0
```

**Expected**:
- ✅ Multiple records displayed (if available)
- ✅ Records sorted by timestamp (newest first)
- ✅ Only shows records after specified commit
- ✅ Table format correct

**Validates**: git2 integration, time filtering, sorting

---

### Test 4: Query Commit Range

**Command**:
```bash
# Last 10 commits
./target/release/pmat tdg history --range HEAD~10..HEAD

# Between releases
./target/release/pmat tdg history --range v2.177.0..v2.178.0
```

**Expected**:
- ✅ Shows records within specified range
- ✅ Excludes records outside range
- ✅ Correct timestamp filtering

**Validates**: Range parsing, boundary conditions

---

### Test 5: Filter by File Path

**Command**:
```bash
# Specific file history
./target/release/pmat tdg history --path server/src/cli/handlers/tdg_handlers.rs --since HEAD~10

# File at specific commit
./target/release/pmat tdg history --path server/src/lib.rs --commit HEAD
```

**Expected**:
- ✅ Only shows records for specified file
- ✅ Combines with other filters correctly
- ✅ Path matching is exact

**Validates**: Path filtering

---

### Test 6: JSON Output Format

**Command**:
```bash
./target/release/pmat tdg history --commit HEAD --format json
```

**Expected**:
- ✅ Valid JSON output
- ✅ Contains "history" array
- ✅ Contains "total_records" count
- ✅ Each record has:
  - file_path
  - score (with all 7 metrics)
  - git_context (commit_sha, branch, author, timestamp, tags)

**Validates**: JSON formatter, complete metadata

---

### Test 7: Quality Archaeology Workflow

**Commands**:
```bash
# 1. Find when quality dropped below B+
./target/release/pmat tdg history --since HEAD~50 --format json | \
  jq '.history[] | select(.score.grade | test("C|D|F"))'

# 2. Quality delta between releases
./target/release/pmat tdg history --range v2.177.0..v2.178.0

# 3. Per-file quality trend
./target/release/pmat tdg history --path server/src/lib.rs --since HEAD~20
```

**Expected**:
- ✅ jq pipeline processes JSON correctly
- ✅ Results show quality regressions (if any)
- ✅ Release comparison shows differences
- ✅ Per-file trend reveals quality evolution

**Validates**: Real-world use case, JSON + jq integration

---

### Test 8: Error Handling

**Commands**:
```bash
# Non-existent commit
./target/release/pmat tdg history --commit nonexistent123

# Invalid range syntax
./target/release/pmat tdg history --range invalid-range

# Non-existent file path
./target/release/pmat tdg history --path does/not/exist.rs
```

**Expected**:
- ✅ Clear error messages
- ✅ Non-zero exit codes
- ✅ Helpful suggestions for fix

**Validates**: Error handling, user experience

---

### Test 9: Performance

**Command**:
```bash
# Query large history
time ./target/release/pmat tdg history --since HEAD~100
```

**Expected**:
- ✅ Completes in <5 seconds
- ✅ No memory leaks
- ✅ Efficient storage iteration

**Validates**: Performance, scalability

---

### Test 10: Help Output

**Command**:
```bash
./target/release/pmat tdg history --help
```

**Expected**:
- ✅ Shows all 5 flags: --commit, --since, --range, --path, --format
- ✅ Clear descriptions
- ✅ Example usage
- ✅ Default values shown

**Validates**: CLI UX, documentation

---

## Success Criteria

**Critical (Must Pass)**:
- [ ] Test 1: Initial analysis with git context
- [ ] Test 2: Query specific commit
- [ ] Test 3: Query since reference
- [ ] Test 6: JSON output format

**Important (Should Pass)**:
- [ ] Test 4: Query commit range
- [ ] Test 5: Filter by file path
- [ ] Test 7: Quality archaeology workflow
- [ ] Test 10: Help output

**Nice-to-Have (May Pass)**:
- [ ] Test 8: Error handling
- [ ] Test 9: Performance

---

## Actual Results

### Test Execution Log

**Date**: October 28, 2025
**Binary**: `./target/release/pmat` v2.179.0
**Storage Backend**: [sled-backend | libsql]

#### Test 1: Initial Analysis with Git Context
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 2: Query Specific Commit
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 3: Query History Since Reference
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 4: Query Commit Range
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 5: Filter by File Path
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 6: JSON Output Format
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 7: Quality Archaeology Workflow
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 8: Error Handling
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 9: Performance
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

#### Test 10: Help Output
**Status**: ⏳ Pending
**Output**:
```
[To be filled during execution]
```
**Notes**:

---

## Issues Found

### Issue 1: [Title]
**Severity**: [Critical | High | Medium | Low]
**Test**: Test X
**Description**: [What went wrong]
**Reproduction**: [Steps to reproduce]
**Expected**: [What should happen]
**Actual**: [What actually happened]
**Fix**: [How to fix, if known]

---

## Summary

**Tests Run**: 0/10
**Tests Passed**: 0/10
**Tests Failed**: 0/10
**Critical Failures**: 0
**Issues Found**: 0

**Recommendation**: ⏳ PENDING EXECUTION

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
