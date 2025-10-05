# PMAT Dogfooding Results - October 5, 2025

**Methodology**: Testing PMAT mutation testing on PMAT itself (Toyota Way)
**Files Tested**: `server/src/services/mutation/types.rs`, `server/src/services/mutation/executor.rs`
**Version**: v2.134.0

## Executive Summary

🔴 **CRITICAL ISSUES FOUND**:
1. Mutation testing **times out** on PMAT's own test suite (>5 minutes)
2. Generated mutant source code is **unreadable** (quote!() formatting)

✅ **GOOD NEWS**:
- File backup/restore mechanism works correctly
- No data loss or corruption
- Compilation still works

## Issues Discovered

### Issue #1: Test Execution Timeout

**Symptom**:
```bash
$ pmat analyze mutate --path server/src/services/mutation/executor.rs
Command timed out after 5m 0s
```

**Root Cause**: PMAT's test suite takes too long
- Running `cargo test --lib` on PMAT takes >2 minutes
- Default timeout is 10 minutes per mutant
- With many mutants, total time becomes impractical

**Impact**:
- Cannot dogfood PMAT on PMAT
- Likely affects other large projects with slow test suites

**Proposed Solutions**:
1. Add `--test-timeout` CLI flag for custom timeouts
2. Add `--test-filter` to run only specific tests
3. Implement test selection based on mutated file
4. Use faster test execution (parallel, cached)

### Issue #2: Unreadable Mutant Source

**Symptom**:
Mutated source from `quote!()` is formatted on one line:

```rust
# ! [doc = ""] use serde :: { Deserialize , Serialize } ; use std :: path :: PathBuf ; # [doc = " Represents a single mutant"] pub struct Mutant { pub id : String , pub original_file : PathBuf , ...
```

**Root Cause**: `quote::quote!(#modified_tree).to_string()` doesn't preserve formatting

**Impact**:
- Mutants are hard to read/debug
- Diffs are useless
- User experience is poor

**Proposed Solutions**:
1. Use `prettyplease` crate to format generated code
2. Use `rustfmt` to format mutated source
3. Preserve original formatting where possible

### Issue #3: Test Suite Performance

**Observation**: PMAT's own test suite is slow
- Large codebase with many dependencies
- Integration tests, property tests, etc.
- Not optimized for mutation testing

**Impact**:
- Dogfooding impractical
- Users with large test suites will have same issue

**Proposed Solutions**:
1. Document best practices for mutation testing (unit tests only)
2. Add test filtering capabilities
3. Recommend mutation testing on specific modules, not entire codebase

## What Worked ✅

### Backup/Restore Mechanism

The executor correctly:
1. Creates `.pmat_backup` file before mutation
2. Writes mutated source to original file
3. Runs tests
4. Restores original from backup
5. Deletes backup file

**Even on timeout**, the restore happened correctly!

### Compilation Success

Generated mutants compile successfully:
- AST replacement works
- Type system is respected
- No syntax errors

## Lessons Learned

### Toyota Way Validation

**"Test on your own product before recommending to customers"**

Dogfooding immediately revealed:
- Timeout issues (wouldn't have found on small test projects)
- Formatting issues (only visible with complex code)
- Performance bottlenecks (PMAT test suite is realistic)

### Real-World Constraints

Testing on pforge was good, but testing on PMAT revealed:
- Large test suites need different strategy
- One-size-fits-all timeout doesn't work
- Need configurability for different project sizes

## Recommendations

### Immediate (v2.135.0)

1. **Add CLI flags**:
   ```bash
   pmat analyze mutate --test-timeout 60 --test-filter "test_parse"
   ```

2. **Add formatting**:
   Use `prettyplease` or `rustfmt` on generated mutants

3. **Document limitations**:
   - Best for unit tests (fast)
   - Not recommended for slow integration tests
   - Suggest per-module mutation testing

### Future Enhancements

1. **Smart Test Selection**:
   - Only run tests that cover mutated code
   - Use coverage data to filter tests
   - Implement test impact analysis

2. **Parallel Execution**:
   - Already have `DistributedExecutor`
   - Integrate with CLI
   - Reduce total time dramatically

3. **Incremental Mutation**:
   - Only mutate changed code
   - Cache previous results
   - Git-aware mutation testing

## Conclusion

### Status

🔴 **Cannot dogfood PMAT on PMAT yet** due to timeouts

✅ **Core functionality works**: backup/restore, compilation, AST replacement

### Priority Fixes

1. Add `--test-timeout` flag (quick win)
2. Add mutant source formatting (user experience)
3. Document best practices (help users avoid our mistakes)

### Long-term Vision

Mutation testing should be:
- **Fast**: Parallel execution, test selection
- **Readable**: Formatted mutants, clear diffs
- **Scalable**: Works on large codebases like PMAT

### Recommendation

**DO NOT ship v2.134.0 as "production ready" without**:
1. Timeout configurability
2. Formatting improvements
3. Documentation of limitations

The dogfooding exercise was **extremely valuable** - it prevented us from shipping a tool that doesn't work on realistic codebases!

---

**Toyota Way Principle Validated**:
"Genchi Genbutsu" (go and see) - By testing on our own complex codebase, we discovered issues that wouldn't appear in simple examples.
