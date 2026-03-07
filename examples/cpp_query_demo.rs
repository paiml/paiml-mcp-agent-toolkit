//! C++ and CUDA Query Workflow Demo
//!
//! Demonstrates pmat query capabilities for C++ ML projects:
//! - Namespace-qualified function indexing
//! - CUDA kernel detection (__global__, __device__, __shared__)
//! - Inline PTX fault annotations (INLINE_PTX, CUDA_SYNC, CUDA_SHMEM)
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

    // 4. Header classification
    println!("\n4. Header Classification");
    println!("{}", "-".repeat(40));
    demo_header_classification();

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
        println!("    Content includes template<>: {}", chunk.content.contains("template"));
    }
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("template"));
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
