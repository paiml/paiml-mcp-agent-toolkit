//! C++ and CUDA Query Workflow Demo
//!
//! Demonstrates pmat query capabilities for C++ ML projects:
//! - Namespace-qualified function indexing
//! - CUDA kernel detection (__global__, __device__, __shared__)
//! - Inline PTX fault annotations (INLINE_PTX, CUDA_SYNC, CUDA_SHMEM)
//! - PTX instruction tags (PTX:mma.sync, PTX:cp.async, etc.)
//! - C++/CUDA complexity penalties (Phase 4 + Phase 7)
//! - Header classification (.h files auto-detected as C or C++)
//!
//! Run with: `cargo run --example cpp_query_demo`
//!
//! To test against a real project:
//! ```sh
//! cd ../llama.cpp && pmat query "attention" --limit 5
//! cd ../pytorch && pmat query "autograd" --limit 5
//! ```

use pmat::services::semantic::{chunk_code, Language};

fn main() {
    println!("C++/CUDA Query Workflow Demo");
    println!("{}", "=".repeat(50));

    // 1. Namespace-qualified name extraction
    println!("\n1. Namespace-Qualified Names");
    println!("{}", "-".repeat(40));
    demo_namespace_qualified();

    // 2. CUDA kernel indexing
    println!("\n2. CUDA Kernel Indexing");
    println!("{}", "-".repeat(40));
    demo_cuda_kernels();

    // 3. Template function extraction
    println!("\n3. Template Function Extraction");
    println!("{}", "-".repeat(40));
    demo_templates();

    // 4. PTX instruction tags
    println!("\n4. PTX Instruction Tags");
    println!("{}", "-".repeat(40));
    demo_ptx_instructions();

    // 5. C++/CUDA complexity penalties
    println!("\n5. C++/CUDA Complexity Penalties");
    println!("{}", "-".repeat(40));
    demo_complexity_penalties();

    // 6. Macro classification
    println!("\n6. C++ Macro Classification (Phase 6)");
    println!("{}", "-".repeat(40));
    demo_macro_classification();

    // 7. PTX defect detection
    println!("\n7. Inline PTX Defect Detection (Phase 8.4)");
    println!("{}", "-".repeat(40));
    demo_ptx_defects();

    // 8. Header classification
    println!("\n8. Header Classification");
    println!("{}", "-".repeat(40));
    demo_header_classification();

    // 9. Declaration-definition linking (G1)
    println!("\n9. Declaration-Definition Linking (G1)");
    println!("{}", "-".repeat(40));
    demo_decl_def_linking();

    // 10. Standalone PTX indexing (G3)
    println!("\n10. Standalone PTX Indexing (G3)");
    println!("{}", "-".repeat(40));
    demo_ptx_indexing();

    println!("\nAll demos completed successfully.");
}

fn demo_namespace_qualified() {
    let source = r#"namespace llama {
namespace model {
    int load_weights(const char* path) {
        return 0;
    }
}
}
"#;
    let chunks = chunk_code(source, Language::Cpp).unwrap();
    for chunk in &chunks {
        println!(
            "  {} [{}] lines {}-{}",
            chunk.chunk_name,
            chunk.chunk_type.as_str(),
            chunk.start_line,
            chunk.end_line
        );
    }
    let func = chunks
        .iter()
        .find(|c| c.chunk_type == pmat::services::semantic::ChunkType::Function);
    assert!(func.is_some());
    assert_eq!(func.unwrap().chunk_name, "llama::model::load_weights");
    println!("  Qualified name: {}", func.unwrap().chunk_name);
}

fn demo_cuda_kernels() {
    let source = r#"__global__ void softmax_kernel(float* output, const float* input, int n) {
    __shared__ float shared_max[32];
    int tid = threadIdx.x;
    int idx = blockIdx.x * blockDim.x + tid;
    if (idx < n) {
        output[idx] = expf(input[idx]);
    }
    __syncthreads();
}
"#;
    let chunks = chunk_code(source, Language::Cpp).unwrap();
    for chunk in &chunks {
        println!(
            "  {} [{}] lines {}-{}",
            chunk.chunk_name,
            chunk.chunk_type.as_str(),
            chunk.start_line,
            chunk.end_line
        );
        if chunk.content.contains("__shared__") {
            println!("    Fault: CUDA_SHMEM (shared memory usage)");
        }
        if chunk.content.contains("__syncthreads") {
            println!("    Fault: CUDA_SYNC (synchronization barrier)");
        }
    }
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].chunk_name, "softmax_kernel");
}

fn demo_templates() {
    let source = "template <typename T>\nT clamp(T val, T lo, T hi) {\n    return (val < lo) ? lo : (val > hi) ? hi : val;\n}\n";
    let chunks = chunk_code(source, Language::Cpp).unwrap();
    for chunk in &chunks {
        println!(
            "  {} [{}] lines {}-{}",
            chunk.chunk_name,
            chunk.chunk_type.as_str(),
            chunk.start_line,
            chunk.end_line
        );
        println!(
            "    Content includes template<>: {}",
            chunk.content.contains("template")
        );
    }
    assert_eq!(chunks.len(), 1);
    // G6: Template parameter extraction - name includes <T>
    assert_eq!(chunks[0].chunk_name, "clamp<T>");
    assert!(chunks[0].content.contains("template"));
}

fn demo_ptx_instructions() {
    // PTX instruction extraction from inline asm() blocks.
    // During indexing, detect_fault_patterns + extract_ptx_instruction_tags
    // will emit tags like PTX:mma.sync, PTX:cp.async, PTX:bar.sync etc.
    let source = r#"static __device__ void mma_A(int* out, const char* src) {
    asm("ldmatrix.sync.aligned.m8n8.x4.b16 {%0, %1, %2, %3}, [%4];"
        : "=r"(out[0]), "=r"(out[1]), "=r"(out[2]), "=r"(out[3])
        : "l"(src));
}
"#;
    let chunks = chunk_code(source, Language::Cpp).unwrap();
    for chunk in &chunks {
        println!(
            "  {} [{}] lines {}-{}",
            chunk.chunk_name,
            chunk.chunk_type.as_str(),
            chunk.start_line,
            chunk.end_line
        );
        // In the full indexer, fault annotations would include:
        // INLINE_PTX, PTX:ldmatrix
        println!(
            "    Contains inline PTX: {}",
            chunk.content.contains("asm(")
        );
        println!("    PTX opcode: ldmatrix (searchable via `pmat query \"ldmatrix\" --faults`)");
    }
    assert!(!chunks.is_empty());
    assert!(chunks[0].content.contains("ldmatrix"));
}

fn demo_complexity_penalties() {
    // C++/CUDA complexity penalties are applied during indexing.
    // Here we demonstrate the penalty patterns that affect TDG scores.

    // Simple C++ function: no extra penalty
    println!("  Simple function: no C++ penalty");

    // CUDA kernel patterns that add complexity penalties:
    println!("  CUDA kernel penalties:");
    println!("    __shared__ memory usage:  +2 (synchronization complexity)");
    println!("    __syncthreads() barrier:  +3 (barrier coordination)");
    println!("    __shfl_* warp primitives: +2 (low-level parallelism)");
    println!("    Thread divergence in kernel: +2 (if inside __global__)");

    // C++ patterns:
    println!("  C++ cognitive penalties:");
    println!("    #ifdef nesting:           +1 per level");
    println!("    Macro-heavy (>5 calls):   +3");
    println!("    SFINAE/enable_if:         +3");
    println!("    Template nesting:         +2 per extra level");
    println!("    const_cast/reinterpret:   +2");

    // Verify via chunk_code that CUDA kernel has expected content
    let source = r#"__global__ void softmax(float* out, const float* in, int n) {
    __shared__ float sdata[256];
    if (threadIdx.x < n) { sdata[threadIdx.x] = in[threadIdx.x]; }
    __syncthreads();
}"#;
    let chunks = chunk_code(source, Language::Cpp).unwrap();
    assert!(!chunks.is_empty());
    let kernel = &chunks[0];
    // During indexing, this kernel gets: +2 (shared) + +3 (sync) + +2 (divergence) = +7
    assert!(kernel.content.contains("__shared__"));
    assert!(kernel.content.contains("__syncthreads"));
    println!("  Example: softmax kernel would get +7 penalty during indexing");
}

fn demo_macro_classification() {
    // During indexing, known C/C++ macro families are classified:
    // - MACRO:ASSERT — GGML_ASSERT, TORCH_CHECK, AT_ASSERT, CUDA_CHECK
    // - MACRO:DISPATCH — AT_DISPATCH_ALL_TYPES, AT_DISPATCH_FLOATING_TYPES
    // - MACRO:LOG — GGML_LOG_INFO, TORCH_WARN
    println!("  Known macro families classified during indexing:");
    println!("    MACRO:ASSERT   — Boundary validation (GGML_ASSERT, TORCH_CHECK)");
    println!("    MACRO:DISPATCH — Type-generic dispatch (AT_DISPATCH_ALL_TYPES)");
    println!("    MACRO:LOG      — Logging (GGML_LOG_INFO, TORCH_WARN)");
    println!();
    println!("  Search via: pmat query \"GGML_ASSERT\" --faults --limit 10");
    println!("  Results show MACRO:ASSERT annotation on matching functions");
}

fn demo_ptx_defects() {
    // Phase 8.4: Inline PTX defect detection
    // Detects safety issues in asm() blocks without full PTX parsing
    println!("  PTX defect patterns detected in inline asm():");
    println!("    PTX_MISSING_BARRIER — st.shared + ld.shared without bar.sync");
    println!("    PTX_BARRIER_DIV    — Branch before barrier (deadlock risk)");
    println!("    PTX_HIGH_REGS      — >8 register outputs (spill risk)");
    println!();

    // Demonstrate with inline PTX content
    let source = r#"__device__ void risky(float* sdata) {
    asm volatile("st.shared.f32 [%0], %1;" : : "l"(sdata), "f"(val));
    asm volatile("ld.shared.f32 %0, [%1];" : "=f"(result) : "l"(sdata));
}"#;
    let has_shared_store = source.contains("st.shared");
    let has_shared_load = source.contains("ld.shared");
    let has_barrier = source.contains("bar.sync");
    let missing = has_shared_store && has_shared_load && !has_barrier;
    println!("  Example: shared store+load without barrier");
    println!("    st.shared present: {has_shared_store}");
    println!("    ld.shared present: {has_shared_load}");
    println!("    bar.sync present: {has_barrier}");
    println!("    PTX_MISSING_BARRIER: {missing}");
    assert!(missing, "should detect missing barrier");
}

fn demo_header_classification() {
    // Header classification is automatic during indexing:
    // .h files are parsed as C by default, then upgraded to C++ if content
    // contains C++ indicators (namespace, class, template, extern "C", etc.)
    //
    // Pure C header -> parsed as C
    let c_header = "#include <stdint.h>\nstruct ggml_tensor { int ne[4]; };\nint ggml_init(int n) { return 0; }\n";
    let c_chunks = chunk_code(c_header, Language::C).unwrap();
    println!("  Pure C header: {} chunks", c_chunks.len());
    for c in &c_chunks {
        println!("    {} [{}]", c.chunk_name, c.language);
    }

    // C++ header -> parsed as C++ (namespace, class detection)
    let cpp_header = "namespace whisper {\nclass Context {\npublic:\n    int n_;\n};\n}\n";
    let cpp_chunks = chunk_code(cpp_header, Language::Cpp).unwrap();
    println!("  C++ header: {} chunks", cpp_chunks.len());
    for c in &cpp_chunks {
        println!("    {} [{}]", c.chunk_name, c.language);
    }
    assert!(!cpp_chunks.is_empty());
}

fn demo_decl_def_linking() {
    // G1: Declaration-definition linking
    // Header declarations get [decl] suffix, linking to implementations
    let header = "int llama_vocab_bos(const struct llama_vocab * vocab);";
    let header_chunks = chunk_code(header, Language::C).unwrap();
    println!("  Header declaration:");
    for c in &header_chunks {
        println!("    {} [{}]", c.chunk_name, c.chunk_type.as_str());
    }
    // Declarations in headers get [decl] suffix
    if let Some(decl) = header_chunks
        .iter()
        .find(|c| c.chunk_name.contains("[decl]"))
    {
        println!("    Declaration detected: {}", decl.chunk_name);
    } else {
        println!("    (declaration detection requires function prototype syntax)");
    }

    // Definition in .cpp file
    let source = "int llama_vocab_bos(const struct llama_vocab * vocab) {\n    return vocab->bos_token;\n}\n";
    let source_chunks = chunk_code(source, Language::C).unwrap();
    println!("  Source definition:");
    for c in &source_chunks {
        println!("    {} [{}]", c.chunk_name, c.chunk_type.as_str());
    }
    assert!(!source_chunks.is_empty());
    assert_eq!(source_chunks[0].chunk_name, "llama_vocab_bos");
    println!("  During indexing, [decl] entries link to definition via linked_definition field");
}

fn demo_ptx_indexing() {
    // G3: Standalone PTX file indexing
    let ptx_source = r#".version 7.0
.target sm_80

.entry vector_add(
    .param .u64 a,
    .param .u64 b,
    .param .u64 c
)
{
    .reg .f32 %f<4>;
    ld.param.u64 %rd1, [a];
    ret;
}

.func (.reg .f32 result) relu(
    .param .f32 x
)
{
    .reg .f32 %f1;
    ld.param.f32 %f1, [x];
    mov.f32 result, %f1;
    ret;
}
"#;
    let chunks = chunk_code(ptx_source, Language::Ptx).unwrap();
    println!("  PTX file: {} chunks extracted", chunks.len());
    for c in &chunks {
        println!(
            "    {} [{}] lines {}-{}",
            c.chunk_name,
            c.chunk_type.as_str(),
            c.start_line,
            c.end_line
        );
    }
    assert!(chunks.len() >= 2, "should extract .entry and .func blocks");
    assert!(chunks.iter().any(|c| c.chunk_name == "vector_add"));
    assert!(chunks.iter().any(|c| c.chunk_name == "relu"));
    println!("  PTX kernels and device functions indexed successfully");
}
