### Summary

The `pmat quality-gate --strict` command fails with an 'unexpected argument' error. The non-strict version of the command, `pmat quality-gate`, runs successfully.

### Steps to Reproduce

1. Build the release binary:
   ```
   cargo build --release
   ```
2. Run the quality gate with the `--strict` flag:
   ```
   ./target/release/pmat quality-gate --strict
   ```

### Expected Behavior

The quality gate should run with stricter checks enabled.

### Actual Behavior

The command fails with the following error:
```
error: unexpected argument found
```

### Workaround

Running the command without the `--strict` flag works as expected.

### Environment

- **pmat version**: Tested on commit `ca94118`
- **Tool**: Gemini CLI
