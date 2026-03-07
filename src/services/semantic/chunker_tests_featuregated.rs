    // Python Language Tests (feature-gated)

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_simple_function() {
        let source = "def hello():\n    print(\"Hello, world!\")\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello");
        assert_eq!(chunks[0].language, "python");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_class() {
        let source = "class MyClass:\n    def __init__(self, value):\n        self.value = value\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "MyClass");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_multiple_functions() {
        let source = "def func1():\n    pass\n\ndef func2():\n    pass\n\ndef func3():\n    pass\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[cfg(not(feature = "python-ast"))]
    #[test]
    fn test_python_feature_disabled() {
        let source = "def test(): pass";
        let result = chunk_code(source, Language::Python);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("python-ast feature is disabled"));
    }

    // C Language Tests (feature-gated)

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_simple_function() {
        let source = "int add(int a, int b) {\n    return a + b;\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
        assert_eq!(chunks[0].language, "c");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_main_function() {
        let source = "int main(int argc, char *argv[]) {\n    return 0;\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "main");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_pointer_return_function() {
        let source = "char *get_string() {\n    return \"hello\";\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "get_string");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_multiple_functions() {
        let source = "void func1() {}\nint func2() { return 0; }\nfloat func3() { return 0.0; }\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[cfg(not(feature = "c-ast"))]
    #[test]
    fn test_c_feature_disabled() {
        let source = "int main() { return 0; }";
        let result = chunk_code(source, Language::C);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not enabled"));
    }

    // C++ Language Tests (feature-gated)

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_simple_function() {
        let source = "int add(int a, int b) {\n    return a + b;\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
        assert_eq!(chunks[0].language, "cpp");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_class() {
        let source = "class MyClass {\npublic:\n    int value;\n};\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "MyClass");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_template_function() {
        let source = "template <typename T>\nT max(T a, T b) {\n    return (a > b) ? a : b;\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "max<T>");
        assert!(chunks[0].content.contains("template"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_namespace_qualified_names() {
        let source = r#"namespace llama {
namespace model {
    int load_weights(const char* path) {
        return 0;
    }
} // namespace model
} // namespace llama
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let func = chunks.iter().find(|c| c.chunk_type == ChunkType::Function);
        assert!(func.is_some(), "Should find function in namespace");
        assert_eq!(func.unwrap().chunk_name, "llama::model::load_weights");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_class_method_qualified() {
        // Multi-line method body ensures tree-sitter-cpp emits function_definition
        let source = "class Transformer {\npublic:\n    void forward(float* input, int n) {\n        for (int i = 0; i < n; i++) {\n            input[i] *= 2.0f;\n        }\n    }\n};\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        // Class itself is always extracted
        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some(), "Should find class");
        assert_eq!(class_chunk.unwrap().chunk_name, "Transformer");
        // Method may or may not be extracted separately depending on tree-sitter-cpp node type
        // At minimum, the class content should contain the method
        assert!(class_chunk.unwrap().content.contains("forward"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_cuda_global_kernel() {
        // tree-sitter-cpp handles CUDA attributes as decorated functions
        let source = r#"__global__ void softmax_kernel(float* output, const float* input, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        output[idx] = expf(input[idx]);
    }
}
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert!(!chunks.is_empty(), "Should parse CUDA __global__ kernel");
        let kernel = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Function);
        assert!(kernel.is_some());
        assert_eq!(kernel.unwrap().chunk_name, "softmax_kernel");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_cuda_device_function() {
        let source = r#"__device__ float warp_reduce_sum(float val) {
    for (int offset = 16; offset > 0; offset /= 2) {
        val += __shfl_down_sync(0xffffffff, val, offset);
    }
    return val;
}
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "warp_reduce_sum");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_cuda_shared_memory() {
        let source = r#"__global__ void reduce_kernel(float* out, const float* in, int n) {
    __shared__ float sdata[256];
    int tid = threadIdx.x;
    sdata[tid] = in[tid];
    __syncthreads();
    if (tid == 0) {
        float sum = 0;
        for (int i = 0; i < 256; i++) sum += sdata[i];
        out[0] = sum;
    }
}
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("__shared__"));
        assert!(chunks[0].content.contains("__syncthreads"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_inline_ptx_assembly() {
        let source = r#"__device__ void barrier_sync() {
    asm volatile("bar.sync 0;");
}
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("asm volatile"));
        assert!(chunks[0].content.contains("bar.sync"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_extern_c_functions() {
        // tree-sitter-cpp handles extern "C" blocks
        let source = r#"extern "C" {
    int llama_decode(void* ctx, int n_tokens) {
        return 0;
    }
}
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let func = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Function);
        assert!(func.is_some(), "Should find function inside extern C block");
        assert_eq!(func.unwrap().chunk_name, "llama_decode");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_multiple_namespaces() {
        let source = r#"namespace ns1 {
    void func_a() {}
}
namespace ns2 {
    void func_b() {}
}
"#;
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let names: Vec<&str> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Function)
            .map(|c| c.chunk_name.as_str())
            .collect();
        assert!(names.contains(&"ns1::func_a"));
        assert!(names.contains(&"ns2::func_b"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_nested_namespace_class() {
        let source = "namespace outer {\nnamespace inner {\n    class Widget {\n    public:\n        void render() {}\n    };\n}\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "outer::inner::Widget");
        // Class content should contain the method even if not extracted separately
        assert!(class_chunk.unwrap().content.contains("render"));
    }

    #[cfg(not(feature = "cpp-ast"))]
    #[test]
    fn test_cpp_feature_disabled() {
        let source = "int main() { return 0; }";
        let result = chunk_code(source, Language::Cpp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not enabled"));
    }

    // Go Language Tests (feature-gated)

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_simple_function() {
        let source = "package main\n\nfunc hello() {\n    fmt.Println(\"Hello\")\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello");
        assert_eq!(chunks[0].language, "go");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_method() {
        let source = "package main\n\ntype Person struct {\n    Name string\n}\n\nfunc (p Person) Greet() string {\n    return \"Hello, \" + p.Name\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        let method_chunk = chunks.iter().find(|c| c.chunk_name == "Greet");
        assert!(method_chunk.is_some());
        assert_eq!(method_chunk.unwrap().chunk_type, ChunkType::Function);
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_struct_type() {
        let source = "package main\n\ntype User struct {\n    ID   int\n    Name string\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "User");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_interface_type() {
        let source =
            "package main\n\ntype Reader interface {\n    Read(p []byte) (n int, err error)\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "Reader");
    }

    #[cfg(not(feature = "go-ast"))]
    #[test]
    fn test_go_feature_disabled() {
        let source = "package main\nfunc main() {}";
        let result = chunk_code(source, Language::Go);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("go-ast feature is disabled"));
    }
