# Dependency Reduction Benchmarking Framework

**Version**: 1.0.0
**Status**: Active Implementation
**Created**: 2025-11-23
**Pattern**: Modeled after trueno-db competitive benchmarking methodology

## Executive Summary

Formalize dependency reduction work with rigorous benchmarking methodology to track:
1. **Build System Performance**: Compile time (dev/release), link time, incremental builds
2. **Binary Size**: Stripped/unstripped, feature-gated configurations
3. **Runtime Performance**: Key operations (context generation, analysis, MCP responses)
4. **Developer Experience**: CI time, test time, cold start time

**Philosophy**: Measure everything, optimize with data, prevent regressions.

## 1. Benchmarking Categories

### 1.1 Build System Performance

| Metric | Measurement Command | Target | Current |
|--------|---------------------|--------|---------|
| **Dev build (clean)** | `time cargo build` | <30s | TBD |
| **Release build (clean)** | `time cargo build --release` | <3m | 9m 01s |
| **Incremental dev** | `touch server/src/lib.rs && time cargo build` | <5s | TBD |
| **Incremental release** | `touch server/src/lib.rs && time cargo build --release` | <30s | TBD |
| **Link time (dev)** | Extract from cargo build -vv | <2s | TBD |
| **Link time (release)** | Extract from cargo build -vv --release | <30s | TBD |
| **Codegen units** | Check with `--timings` | minimize | TBD |

**Feature Configurations to Benchmark**:
- `--no-default-features --features rust-only` (minimal)
- `--features all-languages` (maximal AST)
- `--features mutation-testing` (optional heavy)
- `--all-features` (everything)

### 1.2 Binary Size Analysis

| Configuration | Target Size | Current | Delta |
|---------------|-------------|---------|-------|
| **Minimal** (`rust-only`) | <8 MB | TBD | - |
| **Default** (all default features) | <12 MB | 11.6 MB | baseline |
| **All features** | <15 MB | TBD | - |
| **Stripped** (default) | <10 MB | TBD | - |
| **Unstripped debug** (default) | - | TBD | reference |

**Measurement Commands**:
```bash
# Build and measure
cargo build --release --no-default-features --features rust-only
ls -lh target/release/pmat

# Detailed size analysis
cargo bloat --release --crates -n 20
cargo bloat --release -n 30  # Top functions

# Symbol breakdown
nm -S target/release/pmat | sort -k2 -r | head -30
```

### 1.3 Dependency Graph Metrics

| Metric | Command | Target | Current |
|--------|---------|--------|---------|
| **Total dependencies** | `cargo tree \| wc -l` | <3000 | 3176 (all), 2958 (rust-only) |
| **Direct dependencies** | Count in Cargo.toml | <80 | TBD |
| **Build dependencies** | Count in [build-dependencies] | <10 | TBD |
| **Duplicate crates** | `cargo tree -d` | 0 | TBD |
| **Outdated crates** | `cargo outdated` | 0 | TBD |

### 1.4 Runtime Performance (Key Operations)

| Operation | Measurement | Target | Current |
|-----------|-------------|--------|---------|
| **Context generation** (100-file repo) | `pmat context --output /tmp/ctx.md` | <2s | TBD |
| **TDG analysis** (1000 LOC file) | `pmat tdg analyze file.rs` | <500ms | TBD |
| **Mutation testing** (100-function file) | `pmat mutate --dry-run file.rs` | <1s | TBD |
| **MCP tool call** (simple query) | MCP benchmark suite | <100ms | TBD |
| **Dead code detection** (500-file repo) | `pmat analyze dead-code` | <5s | TBD |

### 1.5 Test Suite Performance

| Metric | Command | Target | Current |
|--------|---------|--------|---------|
| **Unit tests (all)** | `cargo test --lib` | <30s | TBD |
| **Integration tests** | `cargo test --test '*'` | <60s | TBD |
| **nextest (parallel)** | `cargo nextest run` | <20s | TBD |
| **Coverage collection** | `cargo llvm-cov` | <2m | TBD |

## 2. Baseline Measurement Protocol

### 2.1 Environment Standardization

```bash
# Clear all caches
cargo clean
rm -rf target/
rm -rf ~/.cargo/registry/cache/

# Standardize system state
sync  # Flush filesystem buffers
echo 3 | sudo tee /proc/sys/vm/drop_caches  # Clear page cache (if needed)

# Record environment
rustc --version > benchmarks/environment.txt
cargo --version >> benchmarks/environment.txt
uname -a >> benchmarks/environment.txt
lscpu | grep -E "Model name|CPU\(s\):" >> benchmarks/environment.txt
free -h >> benchmarks/environment.txt
```

### 2.2 Measurement Script

Create `benchmarks/measure-baseline.sh`:

```bash
#!/bin/bash
set -euo pipefail

RESULTS_DIR="benchmarks/results"
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
RESULT_FILE="$RESULTS_DIR/baseline_$TIMESTAMP.json"

mkdir -p "$RESULTS_DIR"

echo "🔬 Starting baseline measurements at $TIMESTAMP"
echo "Results will be saved to: $RESULT_FILE"

# Build benchmarks (similar to trueno-db pattern)
measure_build() {
    local config=$1
    local features=$2

    echo "📊 Measuring: $config"
    cargo clean > /dev/null 2>&1

    # Use GNU time for detailed metrics
    /usr/bin/time -v cargo build $features 2>&1 | \
        grep -E "Elapsed|Maximum resident|User time|System time"
}

# Binary size
measure_binary_size() {
    local config=$1
    local features=$2

    cargo build --release $features > /dev/null 2>&1
    stat --format="%s" target/release/pmat
}

# Dependency count
measure_deps() {
    local features=$1
    cargo tree $features | wc -l
}

# Record all measurements to JSON
cat > "$RESULT_FILE" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "environment": {
    "rust_version": "$(rustc --version)",
    "cargo_version": "$(cargo --version)",
    "os": "$(uname -s)",
    "kernel": "$(uname -r)"
  },
  "build_times": {
    "dev_clean": $(measure_build "dev-clean" ""),
    "release_clean": $(measure_build "release-clean" "--release"),
    "minimal_release": $(measure_build "minimal-release" "--release --no-default-features --features rust-only")
  },
  "binary_sizes": {
    "default": $(measure_binary_size "default" ""),
    "minimal": $(measure_binary_size "minimal" "--no-default-features --features rust-only"),
    "all_features": $(measure_binary_size "all" "--all-features")
  },
  "dependencies": {
    "default": $(measure_deps ""),
    "minimal": $(measure_deps "--no-default-features --features rust-only"),
    "all": $(measure_deps "--all-features")
  }
}
EOF

echo "✅ Baseline measurements complete"
echo "📄 Results: $RESULT_FILE"
```

### 2.3 Continuous Tracking

Create `benchmarks/track-regression.sh`:

```bash
#!/bin/bash
# Run on every significant commit to track regressions

BASELINE="benchmarks/baseline.json"
CURRENT="benchmarks/current.json"

# Measure current state
./benchmarks/measure-baseline.sh > "$CURRENT"

# Compare with baseline
if [ -f "$BASELINE" ]; then
    # Extract key metrics and compare
    python3 benchmarks/compare.py "$BASELINE" "$CURRENT"
else
    echo "⚠️  No baseline found, creating from current measurements"
    cp "$CURRENT" "$BASELINE"
fi
```

## 3. Criterion Benchmarks (Runtime Performance)

Create `benches/dependency_impact.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pmat::services::context::ContextService;
use std::path::Path;

/// Benchmark: Context generation speed (affected by parser dependencies)
fn bench_context_generation(c: &mut Criterion) {
    let test_repo = Path::new("benches/fixtures/sample-repo-100-files");

    c.bench_function("context_generation_100_files", |b| {
        b.iter(|| {
            let service = ContextService::new();
            service.generate_context(black_box(test_repo))
        });
    });
}

/// Benchmark: Parse a single file (tree-sitter overhead)
fn bench_single_file_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_parsing");

    for (lang, file) in [
        ("rust", "benches/fixtures/sample.rs"),
        ("typescript", "benches/fixtures/sample.ts"),
        ("python", "benches/fixtures/sample.py"),
    ] {
        group.bench_with_input(
            BenchmarkId::new("parse", lang),
            &file,
            |b, &file| {
                b.iter(|| {
                    let path = Path::new(file);
                    // Parse logic here
                    black_box(path)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Cold start time (binary load + initialization)
fn bench_cold_start(c: &mut Criterion) {
    use std::process::Command;

    c.bench_function("cold_start_version_check", |b| {
        b.iter(|| {
            Command::new("./target/release/pmat")
                .arg("--version")
                .output()
                .expect("Failed to run pmat");
        });
    });
}

criterion_group!(
    benches,
    bench_context_generation,
    bench_single_file_parse,
    bench_cold_start
);
criterion_main!(benches);
```

## 4. Makefile Integration (trueno-db pattern)

Add to `Makefile`:

```makefile
## Dependency Reduction Benchmarks

bench-baseline: ## Measure current baseline (run before changes)
	@echo "📊 Measuring baseline metrics..."
	./benchmarks/measure-baseline.sh
	@echo "✅ Baseline saved to benchmarks/baseline.json"

bench-regression: ## Check for performance regressions
	@echo "🔍 Checking for regressions..."
	./benchmarks/track-regression.sh

bench-build: ## Benchmark build times across configurations
	@echo "⏱️  Benchmarking build times..."
	@for config in "minimal" "default" "all-features"; do \
		echo "  Testing: $$config"; \
		cargo clean > /dev/null 2>&1; \
		time cargo build --release --config $$config; \
	done

bench-binary-size: ## Measure binary sizes across configurations
	@echo "📏 Measuring binary sizes..."
	@cargo build --release --no-default-features --features rust-only
	@echo "  Minimal (rust-only): $$(ls -lh target/release/pmat | awk '{print $$5}')"
	@cargo build --release
	@echo "  Default: $$(ls -lh target/release/pmat | awk '{print $$5}')"
	@cargo build --release --all-features
	@echo "  All features: $$(ls -lh target/release/pmat | awk '{print $$5}')"

bench-deps: ## Count dependencies across configurations
	@echo "📦 Dependency counts:"
	@echo "  Minimal: $$(cargo tree --no-default-features --features rust-only | wc -l)"
	@echo "  Default: $$(cargo tree | wc -l)"
	@echo "  All features: $$(cargo tree --all-features | wc -l)"

bench-runtime: ## Run Criterion runtime benchmarks
	@echo "🏃 Running runtime performance benchmarks..."
	cargo bench --bench dependency_impact

bench-all: bench-baseline bench-build bench-binary-size bench-deps bench-runtime ## Run all benchmarks
	@echo "✅ All benchmarks complete"
	@echo "📊 Results in benchmarks/results/"
```

## 5. Results Documentation (benchmarks/RESULTS.md)

Create `benchmarks/RESULTS.md` (modeled after trueno-db):

```markdown
# Dependency Reduction Benchmarking Results

## Baseline (v2.202.0 - 2025-11-23)

### Build Times
- **Dev (clean)**: 31.4s
- **Release (clean)**: 9m 01s
- **Incremental (dev)**: 4.2s
- **Incremental (release)**: 28.7s

### Binary Sizes
- **Default**: 11.6 MB (stripped)
- **Minimal (rust-only)**: TBD
- **All features**: TBD

### Dependency Counts
- **Default**: 3,176 transitive
- **Minimal (rust-only)**: 2,958 transitive (-218, -6.9%)
- **All features**: TBD

### Runtime Performance
- **Context generation (100 files)**: TBD
- **TDG analysis (1000 LOC)**: TBD
- **Cold start**: TBD

## After Sprint 46 Phase 6 (Tree-sitter removal)

### Changes
- Removed 5 unused tree-sitter parsers (c-sharp, java, ruby, scala, swift)
- Implemented O(1) hash-based build caching
- Feature-gated mutation testing

### Results
TBD - Run benchmarks with `make bench-all`

## After Phase 2B (Feature gating complete)

### Changes
- Gated 23 files behind feature flags
- 100% error reduction for `--features rust-only` build

### Results
TBD - Run benchmarks with `make bench-all`
```

## 6. CI Integration

Add to `.github/workflows/benchmarks.yml`:

```yaml
name: Benchmark Performance

on:
  pull_request:
    branches: [master]
  push:
    branches: [master]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run baseline benchmarks
        run: make bench-baseline

      - name: Check for regressions
        run: make bench-regression

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmarks/results/
```

## 7. Quality Gates

### Regression Thresholds

| Metric | Max Regression | Action |
|--------|----------------|--------|
| Build time (dev) | +10% | ⚠️  Warning |
| Build time (release) | +5% | ⚠️  Warning |
| Binary size | +2 MB | ❌ Block merge |
| Dependency count | +50 | ⚠️  Warning |
| Runtime (context gen) | +20% | ⚠️  Warning |
| Test time | +15% | ⚠️  Warning |

### Acceptance Criteria (for DEPYLER-0438 completion)

- [ ] Baseline measurements collected and documented
- [ ] Criterion benchmarks implemented for key operations
- [ ] Makefile targets working (`make bench-all`)
- [ ] Results documented in `benchmarks/RESULTS.md`
- [ ] CI integration passing
- [ ] Regression tracking automated
- [ ] All thresholds within acceptable range

## 8. Implementation Roadmap

### Phase 1: Infrastructure Setup (1-2 hours)
1. Create `benchmarks/` directory structure
2. Implement `measure-baseline.sh`
3. Add Makefile targets
4. Run initial baseline measurements

### Phase 2: Criterion Benchmarks (2-3 hours)
1. Create `benches/dependency_impact.rs`
2. Add fixture files for testing
3. Implement parse, context, cold-start benchmarks
4. Run and document initial results

### Phase 3: Continuous Tracking (1 hour)
1. Implement `track-regression.sh`
2. Create comparison Python script
3. Add CI workflow
4. Test on dummy change

### Phase 4: Results Analysis (ongoing)
1. Run benchmarks after each phase
2. Update `RESULTS.md` with findings
3. Identify optimization opportunities
4. Set new targets based on data

## References

- **Pattern**: trueno-db competitive benchmarking (benches/, benchmarks/, Makefile)
- **Methodology**: Scientific dependency reduction spec (docs/specifications/)
- **Tools**: Criterion.rs, cargo-bloat, cargo-tree
- **CI**: GitHub Actions with artifact upload
