# First-Class C++, CUDA, and PTX/Kernel Assembly Support for `pmat query`

## Problem Statement

`pmat query` does not index C++ functions **by default**. The `cpp-ast` feature is gated behind `extended-languages`, not in the `core-languages` default. When built with defaults, querying llama.cpp yields 2,073 "functions" — all Python/CMake/Markdown — with zero C++ function definitions.

When built with `--features extended-languages`, C++ indexing **works**: llama.cpp yields **12,510 functions** with TDG grades, complexity metrics, and 187K call edges. However, critical quality gaps remain that prevent first-class parity with Rust support.

### Falsification Evidence (POC: 6 C++ ML Projects)

Tested with `cargo install --path . --features extended-languages` against real projects:

| Project | Files | Functions Indexed | Call Edges | Index Time | Status |
|---------|-------|-------------------|------------|------------|--------|
| **llama.cpp** | 1,074 | 12,510 | 187,701 | 7.4s | OK |
| **whisper.cpp** | 481 | 9,508 | 60,034 | 9.8s | OK |
| **llamafile** | 379 | 4,582 | 16,383 | 7.2s | OK |
| **kernels-community** (CUDA) | 719 | 1,850 | 17,275 | 1.3s | OK |
| **vllm** (csrc) | 168 | est. ~800 | — | — | Not tested |
| **PyTorch** | 8,768 | 48,419 (pre-crash) | — | CRASH | **PANIC** |

### Verified Working (cpp-ast enabled)

```
$ cd llama.cpp && pmat query "attention" --limit 3
llama.cpp/src/llama.cpp:5382-6161 │ llm_load_hparams │ TDG: C │ O(n)
   C:27 │ L:780 │ calls: llama_model_loader, gguf_get_n_kv, ... │ ← llama_model_load

$ cd kernels-community && pmat query "flash attention" --limit 1
flash-attn3/flash-attn/flash_api.cpp:255-365 │ run_mha_fwd_constexpr │ TDG: A │ O(1)
   C:30 │ L:111 │ calls: Flash_fwd_params, cutlass, flash │ ← run_mha_fwd

$ cd llamafile && pmat query "tokenize" --limit 1
llamafile/llama.cpp:47-63 │ llamafile_tokenize │ TDG: A │ O(1)
   C:2 │ L:17 │ calls: resize │ ← eval_plain_text, append_tokens
```

### Remaining Defects

```
# BUG-1 (P0): PyTorch crashes the indexer
$ cd pytorch && pmat query "autograd"
thread 'main' panicked at helpers_annotations.rs:213:
begin <= end (16 <= 4) when slicing `// 1) out = exp(a - val)`
# Root cause: count_params() calls sig.find(')') which returns the FIRST ')'
# C++ comments like "// 1) out = exp(a - val)" have ')' before '(' in the signature string

# BUG-2 (P1): Call graph edges not displayed in compact query mode
$ cd llama.cpp && pmat query "llama_decode" --limit 1
# Shows: C:2 │ L:10 │ 106c 42%
# Missing: calls/callers lines (187K edges exist in SQLite but aren't rendered)

# BUG-3 (P1): No qualified names (namespace::class::method)
$ cd llama.cpp && pmat query --regex "llama_model" --limit 5
# Shows: llm_load_hparams, not llama::model::load_hparams
# C++ functions are indexed with flat names, losing namespace context

# BUG-4 (P2): .cu/.cuh files not routed to C++ parser
$ find vllm -name "*.cu" | wc -l   # 144 CUDA files
# detect_language() does NOT handle .cu extension → not indexed

# BUG-5 (P2): cpp-ast not in default features
# Users must know to pass --features extended-languages
```

### Corrected Root Cause

The original spec incorrectly stated "AgentContextIndex::build() does not route .cpp files through the C++ AST visitor." The **actual** root cause:

1. `chunk_code()` in `chunker_types.rs` gates C++ parsing on `#[cfg(feature = "c-ast")]`
2. `c-ast` is NOT in `core-languages` (the default feature bundle)
3. With `extended-languages` enabled, C++ indexing **works** — the pipeline IS wired up
4. The **real** gaps are: crash robustness (PyTorch), qualified names, CUDA/PTX, display

The tree-sitter-based approach is validated by ATLAS **[R4]** for C/C++ multi-view code representation without build system dependency, and by the broader code search literature **[R1]** **[R2]** showing BM25+AST hybrid retrieval outperforms pure lexical or pure neural approaches.

## Target Projects (Verified)

| Project | C/C++ Files | LOC (est.) | Key Patterns | Status |
|---------|-------------|------------|--------------|--------|
| llama.cpp | 1,095 | ~120K | C-style API, GGML macros (931 refs), minimal templates (4 uses) | **WORKS** |
| whisper.cpp | 647 | ~80K | C++ classes, audio processing, miniaudio.h (93K header-only) | **WORKS** |
| llamafile | 532 | ~60K | Mixed C/C++, cosmopolitan libc | **WORKS** |
| kernels-community | 1,494 | ~50K | Flash Attention CUDA, cutlass templates | **WORKS** |
| vllm (csrc) | 168 | ~20K | CUDA kernels, pybind11 bindings | Untested |
| PyTorch | 3,045 | ~280K | Deep templates, pybind11, autograd, 48K functions | **CRASHES** |

### Macro Density (Measured)

| Project | `#define` Count | Macro Refs | Template Uses | Verdict |
|---------|-----------------|------------|---------------|---------|
| ggml (core) | 16 | 931 (`GGML_*`) | 0 | Macro-API, not macro-code-gen |
| llama.cpp/src | ~30 | ~200 | 4 | Low macro, low template |
| PyTorch csrc | ~100 | ~2,000 | 34 per hot file | High template density |
| kernels-community | ~50 | ~300 | Heavy (cutlass) | Template-heavy CUDA |

**Key insight**: llama.cpp is NOT template-heavy or SFINAE-heavy. The spec's cognitive complexity penalties for SFINAE (+3) and template nesting (+2) are real concerns only for PyTorch, not for the primary POC target.

## Technical Architecture

### Phase 0: Bug Fixes (P0 - Blocking)

These must be fixed before any new feature work.

#### 0.1 PyTorch Crash: `count_params()` Slice Panic

**Location**: `src/services/agent_context/function_index/helpers_annotations.rs:213`

```rust
// CURRENT (panics on C++ comments with ')' before '('):
pub(super) fn count_params(sig: &str) -> usize {
    if let Some(start) = sig.find('(') {
        if let Some(end) = sig.find(')') {
            let params = &sig[start + 1..end];  // PANIC: start=16, end=4
            ...
        }
    }
    0
}

// FIX: find matching ')' AFTER '('
pub(super) fn count_params(sig: &str) -> usize {
    if let Some(start) = sig.find('(') {
        if let Some(end) = sig[start..].find(')') {
            let params = &sig[start + 1..start + end];
            if params.trim().is_empty() {
                return 0;
            }
            return params.split(',').count();
        }
    }
    0
}
```

**Impact**: Blocks indexing of PyTorch (48K functions), any project with C-style comments containing `)` before function signatures.

#### 0.2 CUDA File Extension Registration

```rust
// In detect_language():
"cu" | "cuh" => Some(Language::Cpp),  // Parse as C++ (tree-sitter-cpp handles CUDA attributes)
```

No new `Language::Cuda` variant needed — tree-sitter-cpp already parses `__global__`, `__device__`, `__shared__` as attributed function definitions.

### Phase 1: Promote to Default Features (P0)

**Goal**: `pmat query` works on C++ projects out of the box.

```toml
# Cargo.toml change:
core-languages = ["rust-ast", "typescript-ast", "javascript-ast", "python-ast",
                   "lua-ast", "lean-ast", "c-ast", "cpp-ast"]  # ADD c-ast, cpp-ast
```

**Rationale**: C/C++ are top-3 languages by codebase size. Users querying llama.cpp, PyTorch, or any ML infrastructure should not need `--features extended-languages`. Build time impact is ~10-15s incremental.

**Validation**:
```bash
cargo install --path .  # default features
cd ../llama.cpp && pmat query "attention" --limit 3
# Must return C++ functions, not Python
```

### Phase 2: Qualified Name Extraction (P1)

Current: functions indexed as flat names (`llm_load_hparams`).
Required: namespace-qualified names (`llama::model_loader::llm_load_hparams`).

```rust
fn extract_qualified_name(node: &Node, source: &[u8]) -> String {
    let mut parts = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "namespace_definition" => {
                if let Some(name) = parent.child_by_field_name("name") {
                    parts.push(name.utf8_text(source).unwrap_or("anon").to_string());
                }
            }
            "class_specifier" | "struct_specifier" => {
                if let Some(name) = parent.child_by_field_name("name") {
                    parts.push(name.utf8_text(source).unwrap_or("anon").to_string());
                }
            }
            _ => {}
        }
        current = parent.parent();
    }
    parts.reverse();
    // Append function name
    if let Some(declarator) = node.child_by_field_name("declarator") {
        parts.push(extract_declarator_name(&declarator, source));
    }
    parts.join("::")
}
```

### Phase 3: Call Graph Display Fix (P1)

Call edges exist (187K for llama.cpp) but don't render in compact query mode for C++ functions. The display threshold or rendering path skips non-Rust functions.

**Investigation**: Compare the rendering path for Rust vs C++ `FunctionEntry` records in the query output formatter. The issue is likely in `format_result()` where callers/callees are conditionally displayed.

### Phase 4: C++-Aware Complexity Metrics (P1)

#### Cyclomatic Complexity

Standard decision-point counting plus C++-specific nodes:

| Node Kind | Cyclomatic +1 | Notes |
|-----------|---------------|-------|
| `if_statement` | +1 | |
| `for_statement` / `for_range_loop` | +1 | |
| `while_statement` / `do_statement` | +1 | |
| `switch_statement` | +1 per `case_statement` | |
| `catch_clause` | +1 | |
| `conditional_expression` (ternary) | +1 | |
| `&&` / `\|\|` | +1 | Boolean operators |
| `co_await` / `co_yield` | +1 | C++20 coroutines |

#### Cognitive Complexity

C++-specific penalties (ordered by real-world impact based on POC data). Domain-specific complexity penalties are validated by **[R9]** (CCTR shows domain-aware metrics outperform generic ones) and **[R10]** (cognitive behavioral metrics correlate with developer-perceived difficulty):

| Pattern | Penalty | Prevalence | Projects Affected |
|---------|---------|------------|-------------------|
| Macro-heavy functions (>5 macro calls) | +3 | **HIGH** | llama.cpp, ggml, whisper.cpp |
| Preprocessor conditionals (`#if`/`#ifdef`) | +1 per nesting | **HIGH** | All projects |
| Operator overloads | +1 | MEDIUM | PyTorch |
| Template specialization nesting | +2 per level | MEDIUM | PyTorch, kernels-community |
| Multiple inheritance | +2 per base | LOW | PyTorch only |
| SFINAE (`enable_if`, `requires`) | +3 | LOW | PyTorch only |
| `const_cast` / `reinterpret_cast` | +2 | LOW | ggml backends |

### Phase 5: Header/Implementation Linking (P2)

#### `.h` File Ambiguity (Confirmed Real)

Tested against real headers:
- `llama.cpp/include/llama.h` — Uses `#ifdef __cplusplus` guard, API is `extern "C"`. **Ambiguous**.
- `ggml/include/ggml.h` — Pure ANSI C. **No C++ features**.
- `whisper.cpp/include/whisper.h` — C++ classes. **Clearly C++**.

Current `detect_language()` maps all `.h` → `Language::C`. This is wrong for llama.h and whisper.h.

```rust
fn classify_header(path: &Path, content: &str) -> Language {
    // 1. Explicit guard: #ifdef __cplusplus with extern "C" → C-compatible C++ header
    if content.contains("extern \"C\"") || content.contains("extern \"C\"") {
        // Still C++ (can contain C++ types outside extern "C" block)
        return Language::Cpp;
    }

    // 2. C++ keywords in non-comment context
    let cpp_indicators = ["class ", "namespace ", "template<", "template <",
                          "virtual ", "constexpr ", "nullptr", "::"];
    if cpp_indicators.iter().any(|kw| content.contains(kw)) {
        return Language::Cpp;
    }

    // 3. Co-located .cpp/.cc file → C++
    if let (Some(stem), Some(dir)) = (path.file_stem(), path.parent()) {
        let stem = stem.to_string_lossy();
        if dir.join(format!("{stem}.cpp")).exists()
            || dir.join(format!("{stem}.cc")).exists() {
            return Language::Cpp;
        }
    }

    // 4. Default: C
    Language::C
}
```

#### Declaration-Definition Linking

```rust
struct DeclDefLink {
    /// "ggml_backend_buffer_init_tensor" → definition in ggml-backend.cpp:143
    definition: Location,
    /// Declarations in ggml-backend.h:45
    declarations: Vec<Location>,
}
```

### Phase 6: Macro-Aware Analysis (P2)

#### tree-sitter-cpp Preprocessor Limitations (Research-Confirmed)

Per **[R14]** [tree-sitter-c#108](https://github.com/tree-sitter/tree-sitter-c/issues/108) (still OPEN):
- Preprocessor macros "can appear around pretty much any token of the language while this grammar only allows for it in a couple of places"
- Split identifiers across `#if`/`#else` branches produce ERROR nodes
- `#if` inside array initializers breaks parsing
- `#define` after control flow misparses nesting

**Impact on real projects**: llama.cpp has 931 `GGML_*` macro references but only 16 `#define` statements in ggml.c — macros are USED heavily but DEFINED sparingly. tree-sitter-cpp handles macro USAGE fine (treats as identifiers). It fails on macro DEFINITIONS that contain partial syntax **[R15]**.

**Practical strategy**: Don't expand macros. Classify known macro patterns **[R15]**:

```toml
# .pmat/cpp-macros.toml
[macros.ggml]
assert = ["GGML_ASSERT", "GGML_ABORT"]
logging = ["GGML_LOG_INFO", "GGML_LOG_WARN", "GGML_LOG_ERROR"]
attribute = ["GGML_API", "GGML_DEPRECATED", "GGML_CALL"]

[macros.pytorch]
assert = ["TORCH_CHECK", "TORCH_INTERNAL_ASSERT", "AT_ASSERT"]
logging = ["TORCH_WARN", "TORCH_LOG"]
dispatch = ["AT_DISPATCH_ALL_TYPES", "AT_DISPATCH_FLOATING_TYPES"]
```

### Phase 7: CUDA Kernel Quality (P2)

Already partially working — kernels-community indexed 1,850 functions including CUDA template kernels (e.g., `run_mha_fwd_constexpr` with `C:30`). Additional work:

| Pattern | Penalty | Rationale |
|---------|---------|-----------|
| `__shared__` memory | +2 | Synchronization complexity |
| `__syncthreads()` | +3 | Barrier coordination |
| Warp primitives (`__shfl_*`, `__ballot_*`) | +2 | Low-level parallelism |
| Thread divergence (if inside kernel) | +2 | Performance cliff |

### Phase 8: PTX and Kernel Assembly Indexing (P1)

pmat and trueno already have **significant PTX infrastructure** that is not wired into `pmat query`. This phase unifies existing PTX analysis with the query index so users can search GPU kernel assembly alongside C++/CUDA source.

#### Existing Infrastructure (Already Built)

| Component | Location | Lines | What It Does |
|-----------|----------|-------|--------------|
| PTX defect detection | `src/tdg/cuda_simd/detection_ptx.rs` | 210 | P0/P1/P2 bug patterns (barrier, shared mem, register spills) |
| PTX dataflow tracing | `src/services/agent_context/query/ptx_flow.rs` | 290 | Cross-project PTX flow (emitter→loader→analyzer→consumer) |
| PTX diagnostics | `src/services/agent_context/query/ptx_diagnostics.rs` | 421 | Register counts, branch density, shared memory metrics |
| PTX state machine | `src/tdg/cuda_simd/detection_state.rs` | — | Multi-line pattern analysis, loop/barrier tracking |
| trueno PTX builder | `trueno-gpu/src/ptx/builder/` | 4,422+ | 100+ PTX instructions, pure Rust codegen |
| trueno PTX optimizer | `trueno-gpu/src/ptx/optimize/` | — | Barrier safety, FMA fusion, loop split, tile validation |

**Key insight**: pmat already detects 10 PTX defect classes (SHARED_U64, MISSING_BARRIER, EARLY_EXIT_BARRIER, REG_SPILLS, etc.) and traces PTX dataflow across projects. But none of this surfaces through `pmat query`. This static PTX analysis approach is validated by FlipFlop **[R5]**, which demonstrates 83% accuracy in identifying optimal GPU configurations through static PTX analysis alone, without kernel execution.

#### Inline PTX Assembly Density (Measured)

| Project | Inline `asm()` | `.ptx` Files | Key PTX Patterns |
|---------|----------------|--------------|------------------|
| **llama.cpp** | 299 | 7 (build artifacts) | cp.async, mma.sync, movmatrix, ldmatrix |
| **vllm** | 257 | 0 | CUDA kernels with inline PTX for attention |
| **kernels-community** | 171 | 0 | philox RNG, numeric conversions, Flash Attention |
| **pytorch** | 69 | 0 | Tensor core ops, CUTLASS integration |
| **whisper.cpp** | 54 | 0 | Audio processing CUDA kernels |
| **llamafile** | 5 | 0 | Minimal CUDA usage |

**Total: 855 inline PTX assembly blocks across 6 ML projects.** KernelBench **[R8]** confirms these hand-written kernels represent the performance frontier — LLMs match PyTorch Eager on <20% of kernel tasks, making human-written PTX in llama.cpp and kernels-community the ground truth for kernel quality.

#### 8.1 Inline PTX Extraction from CUDA Source

Inline PTX lives inside `asm()` / `asm volatile()` blocks in `.cu`/`.cuh` files. These should be extracted as searchable "sub-functions" attached to their parent CUDA function:

```cpp
// In llama.cpp/ggml/src/ggml-cuda/mma.cuh
static __device__ void mma_A(mma_A_K32 & mma_A, const char * src) {
    asm("ldmatrix.sync.aligned.m8n8.x4.b16 {%0, %1, %2, %3}, [%4];"
        : "=r"(mma_A.x[0]), "=r"(mma_A.x[1]), "=r"(mma_A.x[2]), "=r"(mma_A.x[3])
        : "l"(src));
}
```

Index entry:

```
FunctionEntry {
    name: "mma_A",
    ptx_instructions: ["ldmatrix.sync.aligned.m8n8.x4.b16"],
    ptx_instruction_count: 1,
    gpu_attributes: ["__device__"],
    file: "ggml/src/ggml-cuda/mma.cuh",
    ...
}
```

#### 8.2 PTX Instruction Extraction

Parse inline PTX strings to extract instruction mnemonics for search and analysis:

```rust
/// Extract PTX instructions from asm() block content
fn extract_ptx_instructions(asm_content: &str) -> Vec<String> {
    // PTX instruction format: "op.modifier.type dest, src1, src2;"
    // Examples:
    //   "cp.async.cg.shared.global.L2::256B [%0], [%1], 16;"
    //   "mma.sync.aligned.m16n8k16.row.col.s32.s8.s8.s32 {%0,...}, {%4,...}, {%6}, {%0,...};"
    //   "bar.sync 0;"
    let mut instructions = Vec::new();
    for line in asm_content.lines() {
        let trimmed = line.trim().trim_start_matches("\\n").trim();
        if let Some(mnemonic) = trimmed.split_whitespace().next() {
            if !mnemonic.starts_with('%') && !mnemonic.starts_with(':')
                && !mnemonic.starts_with('"') && !mnemonic.is_empty() {
                instructions.push(mnemonic.trim_end_matches(';').to_string());
            }
        }
    }
    instructions
}
```

#### 8.3 Queryable PTX Patterns

Enable semantic search over PTX instruction patterns:

```bash
# Find all functions using tensor core MMA instructions
pmat query "mma.sync" --include-project ../llama.cpp --limit 10

# Find functions with async copy (memory pipeline)
pmat query "cp.async" --include-project ../llama.cpp --limit 10

# Find barrier synchronization patterns
pmat query "bar.sync" --include-project ../kernels-community --limit 10

# Find shared memory operations
pmat query "ld.shared" --faults --limit 10
# Shows: MISSING_BARRIER, SHARED_U64 defect annotations

# Find register-heavy kernels (spill risk)
pmat query --regex "__global__" --include-project ../llama.cpp --limit 10
# Shows: register count, spill risk from ptx_diagnostics

# Cross-project PTX dataflow (already in ptx_flow.rs)
pmat query "ptx" --ptx-flow --include-project ../trueno
# Shows: trueno (Emitter) → llama.cpp (Consumer) dataflow
```

#### 8.4 PTX Defect Annotations in Query Results

Wire existing `detection_ptx.rs` defect patterns into query fault annotations:

| Defect Code | Severity | Description |
|-------------|----------|-------------|
| `PTX_BARRIER_DIV` | P0 | Branch before `bar.sync` — thread divergence deadlock **[R11]** |
| `PTX_SHARED_U64` | P1 | 64-bit register for shared memory address (should be U32) |
| `PTX_MISSING_BARRIER` | P0 | Store to shared → load without `bar.sync` **[R11]** |
| `PTX_EARLY_EXIT` | P0 | Thread exits before reaching barrier (PARITY-114) |
| `PTX_REG_SPILL` | P1 | Register spills to local memory (perf regression) |
| `PTX_PRED_OVERFLOW` | P1 | >8 predicate registers cause spills |
| `PTX_EMPTY_LOOP` | P2 | Loop body with no computation |
| `PTX_REDUNDANT_MOV` | P2 | Redundant register move chains |

These defect classes are grounded in GPUVerify's **[R11]** SDV (synchronous delayed visibility) semantics for barrier divergence and data race formalization, and validated by VOLTA **[R6]** which demonstrates that formal verification catches bugs that testing misses. They should appear in `pmat query` output alongside existing fault annotations (UNWRAP, CLONE, TODO, etc.):

```
ggml/src/ggml-cuda/mma.cuh:32-35 │ transpose_b16 │ TDG: A │ O(1)
   C:1 │ L:4 │ PTX:1 │ gpu:__device__
   ptx: movmatrix.sync.aligned.m8n8.trans.b16
```

#### 8.5 Standalone .ptx File Indexing

For projects that ship or generate `.ptx` files (build artifacts, trueno output):

```rust
// In detect_language():
"ptx" => Some(Language::Ptx),  // New language variant

// PTX "functions" are .entry and .func blocks:
// .entry vector_add(.param .u64 a, .param .u64 b, .param .u64 c, .param .u32 n)
// .func (.reg .f32 result) warp_reduce(.reg .f32 val)
```

Extract PTX kernel entries as `FunctionEntry`:

```
FunctionEntry {
    name: "vector_add",
    language: Language::Ptx,
    signature: ".entry vector_add(.param .u64, .param .u64, .param .u64, .param .u32)",
    ptx_target: "sm_80",
    ptx_version: "7.0",
    register_count: 24,
    shared_memory_bytes: 4096,
    barrier_count: 2,
    ...
}
```

#### 8.6 Shader/Kernel Format Coverage

Beyond PTX, other GPU kernel formats exist in ML projects:

| Format | Extension | Found In | Parser | Priority |
|--------|-----------|----------|--------|----------|
| **PTX** | `.ptx` | trueno, llama.cpp build/ | Custom (regex-based) | P1 |
| **Inline PTX** | in `.cu`/`.cuh` | All 6 ML projects (855 blocks) | asm() extraction | P1 |
| **CUDA C++** | `.cu`, `.cuh` | All projects | tree-sitter-cpp | P0 (this spec) |
| **Metal** | `.metal` | PyTorch MPS | tree-sitter-c (close enough) | P3 |
| **GLSL/Vulkan** | `.glsl` | PyTorch vulkan/ | tree-sitter-glsl (exists) | P3 |
| **WGSL** | `.wgsl` | WebGPU projects | tree-sitter-wgsl (exists) | P3 |
| **OpenCL** | `.cl` | Not found in POC set | tree-sitter-c | P3 |

### Phase 9: Cross-Language Call Boundary (P3)

llama.cpp mixes C and C++:
- **ggml core**: Pure C (`.c` + `.h` with `extern "C"`)
- **llama API**: C++ (`.cpp` + `.hpp` with classes)
- **GPU backends**: Mixed (C wrappers around CUDA kernels)

Call graph must cross C/C++/CUDA boundaries. Current naive approach (match by function name) works for C-style APIs but fails for overloaded C++ methods.

## Implementation Plan (Revised)

### Phase 0: Critical Fixes (1 day)

| Step | Description | Effort |
|------|-------------|--------|
| 0.1 | Fix `count_params()` slice panic (PyTorch crash) | 0.5h |
| 0.2 | Add `.cu`/`.cuh` → `Language::Cpp` in `detect_language()` | 0.5h |
| 0.3 | Validate: PyTorch indexes without crash, vllm CUDA files indexed | 2h |

### Phase 1: Default Feature Promotion (0.5 day)

| Step | Description | Effort |
|------|-------------|--------|
| 1.1 | Add `c-ast`, `cpp-ast` to `core-languages` default feature bundle | 0.5h |
| 1.2 | Update CI to test C++ indexing in default build | 1h |
| 1.3 | Validate: `cargo install --path . && cd ../llama.cpp && pmat query "attention"` works | 1h |

### Phase 2: Quality Improvements (3 days)

| Step | Description | Effort |
|------|-------------|--------|
| 2.1 | Qualified name extraction (namespace::class::method) | 1d |
| 2.2 | Fix call graph display for C++ functions in compact mode | 0.5d |
| 2.3 | C++-specific cognitive complexity penalties | 1d |
| 2.4 | `.h` file C/C++ classification heuristic | 0.5d |

### Phase 3: Cross-File Analysis (5 days)

| Step | Description | Effort |
|------|-------------|--------|
| 3.1 | Header discovery and include path resolution | 2d |
| 3.2 | Declaration-definition linking | 2d |
| 3.3 | Cross-file call graph resolution | 1d |

### Phase 4: PTX and Kernel Assembly Integration (3 days, P1)

| Step | Description | Effort |
|------|-------------|--------|
| 4.1 | Extract inline PTX instructions from `asm()` blocks in indexed C++/CUDA functions | 1d |
| 4.2 | Wire `detection_ptx.rs` defect patterns into query fault annotations | 0.5d |
| 4.3 | Wire `ptx_diagnostics.rs` metrics (register count, branch density) into FunctionEntry | 0.5d |
| 4.4 | Add `.ptx` file indexing (`.entry`/`.func` extraction, Language::Ptx variant) | 0.5d |
| 4.5 | Wire `ptx_flow.rs` cross-project dataflow into `--ptx-flow` query flag | 0.5d |

### Phase 5: Advanced (5 days, P2)

| Step | Description | Effort |
|------|-------------|--------|
| 5.1 | Known macro database (ggml, pytorch, cuda macros) | 1d |
| 5.2 | CUDA kernel complexity penalties | 1d |
| 5.3 | `compile_commands.json` integration | 1d |
| 5.4 | Template instantiation tracking | 2d |

## Performance (Measured)

| Metric | Measured | Target | Status |
|--------|----------|--------|--------|
| Index llama.cpp (1,074 files, 12.5K functions) | **7.4s** | <30s | EXCEEDED |
| Index whisper.cpp (481 files, 9.5K functions) | **9.8s** | <15s | MET |
| Index llamafile (379 files, 4.6K functions) | **7.2s** | <10s | MET |
| Index kernels-community (719 files, 1.8K functions) | **1.3s** | <5s | EXCEEDED |
| Index PyTorch (8,768 files, 48K functions) | **CRASH** | <30s | BLOCKED |
| SQLite size: llama.cpp | **43.5 MB** | <200MB | EXCEEDED |
| SQLite size: whisper.cpp | **30.5 MB** | <100MB | EXCEEDED |
| Functions extracted: llama.cpp | **12,510** | >8,000 | EXCEEDED |
| Call edges: llama.cpp | **187,701** | >50,000 | EXCEEDED |

## Validation (Updated with Real Data)

### Smoke Test (PASSING with extended-languages)

```bash
# C++ function extraction
cd ../llama.cpp && pmat query "attention" --limit 3
# ACTUAL: aclnn_add_alibi (C++), aclnn_get_slope (C++), _get_unpad_data (Python)

cd ../whisper.cpp && pmat query "encoder" --limit 3
# ACTUAL: ma_encoder_uninit (C), ma_encoder_init (C), whisper functions

cd ../llamafile && pmat query "tokenize" --limit 3
# ACTUAL: cleanup_tokenize_params (C++), llamafile_tokenize (C++)

cd ../kernels-community && pmat query "flash attention" --limit 1
# ACTUAL: run_mha_fwd_constexpr (CUDA C++ template, C:30, L:111)
```

### PTX Smoke Test (Target: After Phase 4)

```bash
# Find functions using tensor core MMA instructions
cd ../llama.cpp && pmat query "mma.sync" --limit 5
# Expected: mma_A, mma_B functions from ggml-cuda/mma.cuh with PTX annotations

# Find async copy patterns
pmat query "cp.async" --limit 5
# Expected: functions from ggml-cuda/cp-async.cuh with cp.async.cg.shared.global

# Find barrier-related defects
pmat query "bar.sync" --faults --limit 10
# Expected: PTX_MISSING_BARRIER, PTX_BARRIER_DIV annotations where applicable

# Cross-project PTX dataflow
pmat query "ptx" --ptx-flow --include-project ../trueno
# Expected: trueno (Emitter) → pmat (Analyzer) → llama.cpp (Consumer)
```

### Regression Tests Required

Add fixtures under `fixtures/cpp/` for:

1. **Signature with comments containing parentheses** (PyTorch crash case)
2. **CUDA kernel with `__global__`/`__device__`** (`.cu` file)
3. **Inline PTX assembly** (`asm("mma.sync.aligned...")` pattern from llama.cpp mma.cuh)
4. **Inline PTX with barrier** (`asm volatile("bar.sync 0;")` for defect detection)
5. **Namespace-heavy code** (deep nesting, anonymous namespaces)
6. **`extern "C"` mixed header** (llama.h pattern)
7. **Macro-heavy function** (>10 GGML_* calls)
8. **Template function** (cutlass-style `template<int Arch, int Split, ...>`)
9. **Standalone `.ptx` file** (`.entry` kernel with registers, shared memory, barriers)
10. **Async copy PTX** (`cp.async.cg.shared.global` from llama.cpp cp-async.cuh)

## Full Language Stack (After All Phases)

```
              pmat query
                  │
    ┌─────────────┼──────────────┐
    │             │              │
  C/C++        CUDA           PTX
  (.c .cpp     (.cu .cuh)    (.ptx)
   .h .hpp)        │              │
    │             │              │
 tree-sitter   tree-sitter    regex-based
  -c / -cpp      -cpp         .entry/.func
    │             │              │
    │        ┌────┴────┐         │
    │    high-level  inline      │
    │    CUDA C++    asm()       │
    │                  │         │
    │           PTX instruction  │
    │           extraction       │
    │                  │         │
    └────────┬─────────┼─────────┘
             │         │
        FunctionEntry  │
        + complexity   │
        + call graph   │
        + TDG grade    │
                       │
              PTX defect annotations
              (detection_ptx.rs)
              + register metrics
              (ptx_diagnostics.rs)
              + dataflow tracing
              (ptx_flow.rs)
```

## Known Limitations

1. **tree-sitter-cpp cannot expand macros** **[R14]**: Functions hidden inside macro expansions (`DEFINE_HANDLER(foo)`) are not indexed. This is a [known open issue](https://github.com/tree-sitter/tree-sitter-c/issues/108) in tree-sitter-c/cpp with no upstream fix planned.
2. **No type inference**: Overloaded function calls and template instantiations cannot be resolved to specific targets without clang-level semantic analysis.
3. **No build system integration** (Phase 0-2): Include paths are heuristic-based until `compile_commands.json` support lands.
4. **Header-only libraries**: Functions in headers indexed once per header, not per inclusion site.
5. **Generated code**: Files in `build/`, `cmake-build-*/` excluded by `.gitignore`, but generated-in-tree code is indexed.
6. **Virtual dispatch**: Call graph cannot resolve virtual method calls to concrete implementations without class hierarchy analysis.
7. **PTX from NVRTC**: Runtime-compiled PTX (PyTorch Inductor, NVRTC) is not indexable — PTX is generated at runtime, not present in source tree. CuAsmRL **[R7]** demonstrates that even post-compilation SASS optimization yields 9% average gains, but this requires GPU execution which pmat avoids by design.
8. **Inline PTX extraction is string-based**: Cannot validate PTX instruction correctness without NVIDIA's `ptxas` assembler. Relies on regex extraction from `asm()` string literals.
9. **No SASS support**: SASS (native GPU machine code) is binary and not human-readable. Only PTX (intermediate assembly) is indexed.

## Comparison with Existing Tools

| Capability | clangd | cscope | ctags | nsight | pmat (proposed) |
|-----------|--------|--------|-------|--------|-----------------|
| Function index | Full semantic | Regex | Regex | CUDA only | AST (tree-sitter) |
| Cross-file resolution | Full | Basic | None | CUDA call graph | Name-based (planned: header-linked) |
| Complexity metrics | None | None | None | None | Cyclomatic + Cognitive |
| TDG grading | None | None | None | None | Full pipeline |
| Call graph | Full | Basic | None | CUDA only | 187K edges (llama.cpp) |
| Semantic search | None | None | None | None | BM25 + TF-IDF |
| Git history fusion | None | None | None | None | RRF with commit intent |
| Code clone detection | None | None | None | None | MinHash + LSH |
| **PTX defect detection** | None | None | None | Basic | **10 defect classes, P0/P1/P2** |
| **Inline PTX extraction** | None | None | None | Via profiler | **asm() block parsing** |
| **PTX dataflow tracing** | None | None | None | None | **Cross-project (ptx_flow.rs)** |
| **GPU register metrics** | None | None | None | Yes (binary) | **Source-level estimation** |
| Works without build | **No** | Yes | Yes | **No** | **Yes** |
| Handles 48K functions | Yes | Yes | Yes | N/A | Crashes (PyTorch) → **fix in Phase 0** |

Key differentiator: pmat provides **quality-enriched semantic search** without requiring a working build system or `compile_commands.json`. clangd needs both to function. pmat's hybrid BM25+TF-IDF approach is validated by **[R1]** (80% improvement over baseline BM25 via commit-aware retrieval) and **[R2]** (dependency-graph-guided retrieval achieving +40.9 Pass@1).

## References

### Code Search and Retrieval

- **[R1]** Gandhi, Gao, Callan. [Repository-level Code Search with Neural Retrieval Methods](https://arxiv.org/abs/2502.07067) (arXiv 2502.07067, Feb 2025). BM25 over commit messages + CodeBERT neural reranking achieves 80% improvement in MAP/MRR/P@1 over baseline BM25. *Validates pmat's hybrid BM25+TF-IDF approach and git history fusion via RRF.*

- **[R2]** Li et al. [GraphCodeAgent: Dual Graph-Guided LLM Agent for Retrieval-Augmented Repo-Level Code Generation](https://arxiv.org/abs/2504.10046) (arXiv 2504.10046, Apr 2025). Constructs requirement graphs + DS-code graphs for dependency-aware retrieval. Achieves +40.9 Pass@1 on GPT-4o vs no-RAG. *Validates pmat's call graph + PageRank ranking for code search quality.*

- **[R3]** [TreeRanker: Fast and Model-agnostic Ranking System for Code Suggestions](https://arxiv.org/abs/2508.02455) (arXiv 2508.02455, 2025). Lightweight masking for valid code completions without model modification. *Relevant to pmat's TDG-based ranking of search results.*

- **[R4]** [ATLAS: Automated Tree-based Language Analysis for C/C++](https://arxiv.org/abs/2512.12507) (arXiv 2512.12507, Dec 2025). Tree-sitter-based CST→AST extraction for C/C++, generates multi-view code representations for ML tasks. *Confirms tree-sitter as the right parser choice for C++ analysis without build system.*

### GPU Kernel Analysis and PTX

- **[R5]** Rajput, Brandt, Elisseev, Sharma. [FlipFlop: A Static Analysis-based Energy Optimization Framework for GPU Kernels](https://arxiv.org/abs/2601.13345) (arXiv 2601.13345, Jan 2026). **Static PTX analysis** predicts energy-efficient thread block configurations without kernel execution. 83% accuracy, 93.4% search space reduction, up to 79% energy savings for multi-head attention kernels. *Directly validates pmat's approach of analyzing PTX statically (detection_ptx.rs) without requiring GPU execution.*

- **[R6]** Dubey et al. [VOLTA: Equivalence Checking of ML GPU Kernels](https://arxiv.org/abs/2511.12638) (arXiv 2511.12638, Nov 2025). First formal verification tool for GPU kernels — verifies convolutions, matmuls, and attention mechanisms are mathematically equivalent across manual tuning, LLM generation, and compiler optimization. *Motivates pmat's PTX defect detection: static analysis catches bugs that testing misses.*

- **[R7]** He, Yoneki. [CuAsmRL: Optimizing GPU SASS Schedules via Deep Reinforcement Learning](https://arxiv.org/abs/2501.08071) (arXiv 2501.08071, Jan 2025). RL agent optimizes GPU SASS assembly schedules, achieving 9% average / 26% peak speedup over -O3. Documents the PTX→SASS compilation pipeline. *Confirms that PTX-level analysis captures optimization opportunities invisible at CUDA C++ level.*

- **[R8]** Ouyang et al. [KernelBench: Can LLMs Write Efficient GPU Kernels?](https://arxiv.org/abs/2502.10517) (arXiv 2502.10517, Feb 2025). 250-kernel benchmark: best LLMs match PyTorch Eager on <20% of tasks. Levels: single ops, fusion patterns, full architectures (MobileNet, VGG, MiniGPT). *Motivates indexing real-world kernel code (llama.cpp, kernels-community) as reference for LLM-generated kernel quality comparison.*

### Complexity Metrics

- **[R9]** Ouedraogo et al. [Rethinking Cognitive Complexity for Unit Tests (CCTR)](https://arxiv.org/abs/2506.06764) (arXiv 2506.06764, Jun 2025, ICSME 2025). Introduces test-aware cognitive complexity integrating assertion density, annotation roles, and test composition patterns. *Validates pmat's domain-specific complexity penalties (template nesting, macro density) beyond generic cyclomatic/cognitive metrics.*

- **[R10]** [NRevisit: A Cognitive Behavioral Metric for Code Understandability Assessment](https://arxiv.org/abs/2504.18345) (arXiv 2504.18345, Apr 2025). Bridges behavioral and static complexity measures using developer perception. *Supports adding C++-specific cognitive penalties (SFINAE, operator overloads) that correlate with developer confusion.*

### GPU Kernel Verification

- **[R11]** Betts, Chong et al. [GPUVerify: A Verifier for GPU Kernels](https://nchong.github.io/papers/oopsla12.pdf) (OOPSLA 2012). Defines synchronous delayed visibility (SDV) semantics for barrier divergence and data race detection. *Foundational theory behind pmat's PTX_BARRIER_DIV and PTX_MISSING_BARRIER defect classes.*

- **[R12]** [GPURepair: Automated Repair of GPU Kernels](https://www.ias.ac.in/article/fulltext/sadh/049/0010) (Sadhana 2024). Proposes fixes for both intra-block data races and barrier divergence in CUDA/OpenCL. *Motivates extending pmat from detection to automated fix suggestions for PTX defects.*

### Code Clone Detection

- **[R13]** Moumoula et al. [Large Language Models for Cross-lingual Code Clone Detection](https://arxiv.org/abs/2408.04430) (arXiv 2408.04430, Aug 2024, updated May 2025). GPT-3.5-Turbo achieves F1=0.99 on XLCoST for cross-language clone detection. *Validates pmat's MinHash+LSH approach for clone detection across C/C++/CUDA boundaries.*

### Tooling and Limitations

- **[R14]** [tree-sitter-c#108: Preprocessor macro handling is not general enough](https://github.com/tree-sitter/tree-sitter-c/issues/108) (Open issue, tree-sitter-c). Macros at arbitrary positions produce ERROR nodes. No upstream fix planned. *Documents the fundamental limitation of tree-sitter for macro-heavy C/C++ code.*

- **[R15]** [Tree-sitter and Preprocessing: A Syntax Showdown](https://habr.com/en/articles/835192/) (2024). Practical strategies: extras field method, limited context-specific rules. *Informs pmat's approach of classifying known macros rather than expanding them.*
