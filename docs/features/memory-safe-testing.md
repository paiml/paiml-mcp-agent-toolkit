# Memory-Safe Testing Configuration

## Problem
`make test-fast` was consuming all system memory and swap, causing system crashes due to excessive parallelism during compilation and testing.

## Solution
Implemented permanent memory-safe settings in the Makefile for both `make test` and `make test-fast`.

### Changes Made

1. **Makefile Updates** (Permanent Solution)
   - Limited `CARGO_BUILD_JOBS` to 2 (4 in CI)
   - Limited test threads to 2 (4-8 in CI)
   - Disabled incremental compilation (`CARGO_INCREMENTAL=0`)
   - Set single codegen unit (`CARGO_PROFILE_TEST_CODEGEN_UNITS=1`)

2. **`.cargo/config.toml`** (Project-wide defaults)
   ```toml
   [build]
   jobs = 2
   
   [profile.test]
   opt-level = 0
   codegen-units = 1
   incremental = false
   ```

### Usage

**Default (memory-safe):**
```bash
make test-fast    # Uses 2 build jobs, 2 test threads
make test         # Same limits, with coverage
```

**Override for more parallelism:**
```bash
CARGO_BUILD_JOBS=4 THREADS=4 make test-fast
```

### Memory Impact
- Before: Used all 30GB RAM + 8GB swap, system crash
- After: Uses ~16GB RAM, 0 swap, stable operation

### Performance Trade-off
- Compilation: ~65 seconds (was ~30 seconds before crash)
- Test execution: Similar speed due to limited test parallelism
- Overall: Slightly slower but guaranteed to complete without OOM

### CI Behavior
In CI environments (`CI=true`), the settings are more aggressive:
- Build jobs: 4
- Test threads: 4-8
- This works because CI runners have dedicated resources