# Binary Size Reduction Specification

## Executive Summary

This specification outlines strategies to reduce the binary size of the PAIML MCP Agent Toolkit while maintaining performance characteristics. Based on empirical analysis of Rust binary optimization techniques, we can achieve a 60-80% size reduction through systematic application of compiler optimizations, dependency management, and architectural refinements.

## Current State Analysis

### Baseline Metrics

Before optimization, establish baseline measurements:

```bash
# Measure current binary size
ls -lh target/release/paiml-mcp-agent-toolkit

# Analyze binary composition
cargo bloat --release --crates
cargo bloat --release -n 30

# Symbol analysis
nm -S target/release/paiml-mcp-agent-toolkit | sort -k2 -r | head -20
```

### Primary Size Contributors

1. **Embedded Templates**: Static string inclusion via `include_str!`
2. **AST Parsing Infrastructure**: Tree-sitter and language-specific parsers
3. **JSON-RPC/MCP Protocol**: Serde and protocol handling
4. **Handlebars Engine**: Template rendering system
5. **Tokio Runtime**: Async runtime (if used)

## Optimization Strategies

### 1. Compiler-Level Optimizations

#### Release Profile Configuration

```toml
[profile.release]
opt-level = "z"          # Optimize for size (-Os equivalent)
lto = true               # Enable Link Time Optimization
codegen-units = 1        # Single codegen unit for maximum optimization
panic = "abort"          # Remove unwinding machinery (~200KB savings)
strip = true             # Strip symbols (60-80% reduction)
overflow-checks = false  # Disable overflow checks in release

# Advanced optimizations (requires nightly)
[profile.release.package."*"]
opt-level = "z"
strip = "symbols"
```

#### Cargo Configuration

```toml
# .cargo/config.toml
[build]
rustflags = [
    "-C", "link-arg=-s",     # Strip symbols during linking
    "-C", "prefer-dynamic",   # Use dynamic linking where possible
    "-C", "target-cpu=native" # Not for distribution, but for local builds
]

[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "link-arg=-Wl,--gc-sections",  # Remove unused sections
    "-C", "link-arg=-Wl,--strip-all",    # Strip all symbols
]
```

### 2. Dependency Optimization

#### Audit Current Dependencies

```bash
# Analyze dependency weight
cargo tree --duplicate
cargo audit --deny warnings
cargo machete  # Find unused dependencies
```

#### Strategic Replacements

```toml
[dependencies]
# Replace heavy dependencies
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde_json = { version = "1.0", default-features = false, features = ["std"] }

# Consider alternatives
# tokio → smol (if full async runtime not needed)
# reqwest → ureq (for simple HTTP)
# regex → regex-lite (if full regex not needed)

# Disable default features systematically
clap = { version = "4.0", default-features = false, features = ["std", "derive"] }
```

### 3. Template Embedding Optimization

#### Compression Strategy

```rust
// build.rs
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

fn main() {
    // Compress templates at build time
    let templates = include_bytes!("templates/");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(templates).unwrap();
    let compressed = encoder.finish().unwrap();
    
    // Generate const with compressed data
    println!("cargo:rustc-env=COMPRESSED_TEMPLATES={:?}", compressed);
}

// In code
lazy_static! {
    static ref TEMPLATES: HashMap<String, String> = {
        let compressed = include_bytes!(env!("COMPRESSED_TEMPLATES"));
        let decompressed = decompress(compressed);
        parse_templates(decompressed)
    };
}
```

#### Template Deduplication

```rust
// Identify common template fragments
const COMMON_HEADER: &str = include_str!("common/header.hbs");
const COMMON_FOOTER: &str = include_str!("common/footer.hbs");

// Use string interning for repeated patterns
use string_cache::DefaultAtom;

struct Template {
    fragments: Vec<DefaultAtom>,
}
```

### 4. AST Infrastructure Optimization

#### Conditional Compilation

```rust
#[cfg(feature = "rust-ast")]
mod rust_parser;

#[cfg(feature = "typescript-ast")]
mod typescript_parser;

#[cfg(feature = "python-ast")]
mod python_parser;

// Allow users to build with only needed parsers
// cargo build --no-default-features --features rust-ast
```

#### Parser Minimization

```rust
// Use tree-sitter's minimal API
use tree_sitter::{Parser, Query};

// Avoid pulling in full language grammars
#[cfg(feature = "minimal-ast")]
pub fn analyze_complexity(source: &str) -> ComplexityMetrics {
    // Hand-rolled minimal parser for complexity only
    let mut complexity = 0;
    for line in source.lines() {
        complexity += count_control_flow_keywords(line);
    }
    ComplexityMetrics { cyclomatic: complexity }
}
```

### 5. Dead Code Elimination

#### Aggressive Inlining

```rust
#[inline(always)]
fn small_frequent_function() -> u32 {
    // Forces inlining, eliminating function overhead
    42
}

#[inline(never)]
fn large_infrequent_function() {
    // Prevents inlining of rarely-used code
}
```

#### Visibility Restrictions

```rust
// Use pub(crate) instead of pub where possible
pub(crate) mod internal;

// Seal traits to enable more aggressive optimization
mod private {
    pub trait Sealed {}
}
pub trait MyTrait: private::Sealed {}
```

### 6. Binary Packing Strategies

#### UPX Compression (Post-Build)

```makefile
# Makefile addition
release-compressed: release
	upx --best --lzma target/release/paiml-mcp-agent-toolkit
	# Typically achieves 50-70% compression
	# Trade-off: ~100ms startup overhead
```

#### Static vs Dynamic Linking

```toml
# For musl target (smaller static binaries)
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]

# Build command
cargo build --release --target x86_64-unknown-linux-musl
```

### 7. Architecture-Specific Optimizations

#### Feature Modularization

```toml
[features]
default = ["mcp-server", "cli"]
mcp-server = ["tokio", "json-rpc"]
cli = ["clap", "colored"]
minimal = []  # Bare minimum functionality

# Users can build minimal version:
# cargo build --no-default-features --features minimal
```

#### Lazy Loading

```rust
use once_cell::sync::Lazy;

static COMPLEX_DATA: Lazy<HashMap<String, Template>> = Lazy::new(|| {
    // Initialize only when first accessed
    load_templates()
});
```

## Measurement Protocol

### Continuous Monitoring

```yaml
# .github/workflows/binary-size.yml
- name: Check binary size
  run: |
    SIZE=$(stat -c%s target/release/paiml-mcp-agent-toolkit)
    echo "Binary size: $SIZE bytes"
    
    # Fail if size exceeds threshold
    if [ $SIZE -gt 10485760 ]; then  # 10MB
      echo "Binary size exceeds 10MB threshold"
      exit 1
    fi
```

### Size Tracking

```rust
// tests/binary_size.rs
#[test]
fn binary_size_regression() {
    let binary = std::fs::metadata("target/release/paiml-mcp-agent-toolkit")
        .expect("Binary not found");
    
    assert!(binary.len() < 10 * 1024 * 1024, 
            "Binary size {} exceeds 10MB limit", binary.len());
}
```

## Implementation Roadmap

### Phase 1: Low-Hanging Fruit (1-2 days)
1. Apply release profile optimizations
2. Strip symbols
3. Remove unused dependencies
4. Expected reduction: 60-70%

### Phase 2: Structural Changes (3-5 days)
1. Implement template compression
2. Add feature flags for parsers
3. Optimize dependency features
4. Expected additional reduction: 15-20%

### Phase 3: Advanced Optimizations (1 week)
1. Custom allocator investigation
2. No-std evaluation for core components
3. Binary packing with UPX
4. Expected additional reduction: 10-15%

## Trade-off Analysis

### Performance Impact

| Optimization | Size Reduction | Performance Impact | Recommendation |
|--------------|----------------|-------------------|----------------|
| `opt-level = "z"` | 20-30% | 5-10% slower | ✓ Recommended |
| LTO | 10-20% | Longer build times | ✓ Recommended |
| `panic = "abort"` | 5-10% | No stack unwinding | ✓ Recommended |
| Strip symbols | 60-80% | No debug info | ✓ Production only |
| UPX compression | 50-70% | 100ms startup | ⚠️ Optional |
| No-std | 80-90% | Limited functionality | ❌ Not suitable |

### Maintenance Considerations

1. **Debug Builds**: Maintain separate debug profile for development
2. **Symbol Preservation**: Keep unstripped binaries for crash analysis
3. **Feature Testing**: CI must test all feature combinations
4. **Documentation**: Update build instructions for different profiles

## Benchmarking Suite

```rust
// benches/binary_metrics.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_startup_time(c: &mut Criterion) {
    c.bench_function("cold_start", |b| {
        b.iter(|| {
            std::process::Command::new("./target/release/paiml-mcp-agent-toolkit")
                .arg("--version")
                .output()
                .unwrap();
        });
    });
}

fn benchmark_template_generation(c: &mut Criterion) {
    c.bench_function("template_render", |b| {
        b.iter(|| {
            // Measure template generation performance
        });
    });
}

criterion_group!(benches, benchmark_startup_time, benchmark_template_generation);
criterion_main!(benches);
```

## Recommended Configuration

Based on analysis, the optimal configuration for the PAIML MCP Agent Toolkit:

```toml
# Cargo.toml
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.release-debug]
inherits = "release"
strip = false
debug = true

[dependencies]
# Minimal features for all dependencies
serde = { version = "1.0", default-features = false, features = ["derive", "std"] }
serde_json = { version = "1.0", default-features = false, features = ["std"] }
clap = { version = "4.0", default-features = false, features = ["std", "derive", "help"] }

[features]
default = ["full"]
full = ["mcp", "cli", "all-languages"]
minimal = ["cli", "rust-only"]
mcp = ["tokio", "async"]
all-languages = ["rust-ast", "typescript-ast", "python-ast"]
rust-only = ["rust-ast"]
```

## Expected Results

With systematic application of these optimizations:

- **Baseline**: ~15-20MB (estimated current size)
- **Phase 1**: ~5-7MB (strip + basic optimizations)
- **Phase 2**: ~3-4MB (dependency optimization)
- **Phase 3**: ~2-3MB (architectural changes)
- **With UPX**: ~1-1.5MB (post-build compression)

## Monitoring and Maintenance

```bash
# Add to Makefile
size-report:
	@echo "=== Binary Size Report ==="
	@ls -lh target/release/paiml-mcp-agent-toolkit
	@echo ""
	@echo "=== Size by Crate ==="
	@cargo bloat --release --crates -n 10
	@echo ""
	@echo "=== Largest Functions ==="
	@cargo bloat --release -n 10

size-track: release
	@SIZE=$$(stat -f%z target/release/paiml-mcp-agent-toolkit 2>/dev/null || stat -c%s target/release/paiml-mcp-agent-toolkit); \
	echo "$$(date +%Y-%m-%d),$$SIZE" >> size-history.csv
	@echo "Binary size: $$SIZE bytes"
```

This specification provides a systematic approach to reducing binary size while maintaining the functionality and performance characteristics required for the PAIML MCP Agent Toolkit.