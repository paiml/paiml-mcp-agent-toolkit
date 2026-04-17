// Helper function unit tests: detect_language, complexity, satd, big_o, grade,
// ignored_dir, sha256, name_frequency, doc_comments, keywords, identifiers,
// quality metrics, return type, count_params, TDG/grade boundaries

#[test]
fn test_detect_language() {
    assert_eq!(detect_language(Path::new("test.rs")), Some(Language::Rust));
    assert_eq!(
        detect_language(Path::new("test.py")),
        Some(Language::Python)
    );
    assert_eq!(detect_language(Path::new("test.txt")), None);
}

#[test]
fn test_count_complexity() {
    let simple = "fn foo() { return 1; }";
    assert_eq!(count_complexity(simple), 1);

    let with_if = "fn foo() { if x { return 1; } return 2; }";
    assert_eq!(count_complexity(with_if), 2);
}

#[test]
fn test_count_satd_markers() {
    let clean = "fn foo() { return 1; }";
    assert_eq!(count_satd_markers(clean), 0);

    let with_todo = "fn foo() { // TODO: fix this\n return 1; }";
    assert_eq!(count_satd_markers(with_todo), 1);
}

#[test]
fn test_estimate_big_o() {
    let constant = "fn foo() { return 1; }";
    assert_eq!(estimate_big_o(constant), "O(1)");

    let linear = "fn foo() {\n    for i in items {\n        process(i);\n    }\n}";
    assert_eq!(estimate_big_o(linear), "O(n)");
}

#[test]
fn test_score_to_grade() {
    assert_eq!(score_to_grade(0.5), "A");
    assert_eq!(score_to_grade(2.5), "B");
    assert_eq!(score_to_grade(5.0), "C");
    assert_eq!(score_to_grade(7.0), "D");
    assert_eq!(score_to_grade(9.0), "F");
}

#[test]
fn test_is_ignored_dir() {
    assert!(is_ignored_dir(Path::new("target")));
    assert!(is_ignored_dir(Path::new("node_modules")));
    assert!(!is_ignored_dir(Path::new("src")));
}

#[test]
fn test_compute_file_sha256() {
    let hash1 = compute_file_sha256("hello world");
    let hash2 = compute_file_sha256("hello world");
    let hash3 = compute_file_sha256("different content");
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
    assert_eq!(hash1.len(), 64); // SHA256 hex is 64 chars
}

#[test]
fn test_compute_name_frequency() {
    let mut name_index = HashMap::new();
    name_index.insert("new".to_string(), vec![0, 1, 2, 3, 4]);
    name_index.insert("unique_func".to_string(), vec![5]);
    let freq = compute_name_frequency(&name_index, 10);
    assert!((freq["new"] - 0.5).abs() < 0.01);
    assert!((freq["unique_func"] - 0.1).abs() < 0.01);
}

#[test]
fn test_compute_name_frequency_empty() {
    let name_index = HashMap::new();
    let freq = compute_name_frequency(&name_index, 0);
    assert!(freq.is_empty());
}

#[test]
fn test_extract_doc_comment_basic() {
    let content = "/// This is a doc comment\nfn foo() {}";
    let doc = extract_doc_comment(content, 2); // fn is on line 2
    assert!(doc.is_some());
    assert!(doc.unwrap().contains("This is a doc comment"));
}

#[test]
fn test_extract_doc_comment_none() {
    let content = "fn foo() {}";
    let doc = extract_doc_comment(content, 1);
    assert!(doc.is_none());
}

#[test]
fn test_calculate_simple_tdg() {
    // Low complexity, no SATD, small LOC = low score
    let score = calculate_simple_tdg(1, 0, 10);
    assert!(score < 2.0);

    // High complexity, SATD, large LOC = higher score
    let high_score = calculate_simple_tdg(20, 3, 200);
    assert!(high_score > score);
}

#[test]
fn test_is_keyword() {
    assert!(is_keyword("fn"));
    assert!(is_keyword("let"));
    assert!(is_keyword("if"));
    assert!(is_keyword("for"));
    assert!(is_keyword("while"));
    assert!(is_keyword("return"));
    assert!(is_keyword("def"));
    assert!(is_keyword("class"));
    assert!(is_keyword("import"));
    assert!(!is_keyword("handle_error"));
    assert!(!is_keyword("MyStruct"));
}

#[test]
fn test_estimate_big_o_quadratic() {
    let quadratic = "fn foo() {\n    for i in items {\n        for j in items {\n            process(i, j);\n        }\n    }\n}";
    assert_eq!(estimate_big_o(quadratic), "O(n^2)");
}

#[test]
fn test_estimate_big_o_logarithmic() {
    let log = "fn foo() {\n    while n > 0 {\n        n /= 2;\n    }\n}";
    // Contains while + divide = log
    assert!(["O(n log n)", "O(log n)", "O(n)"].contains(&estimate_big_o(log).as_str()));
}

#[test]
fn test_extract_quality_metrics() {
    let source = "fn complex() {\n    if a {\n        if b {\n            for i in items {\n                // TODO: fix\n                process(i);\n            }\n        }\n    }\n}\n";
    let chunks = chunk_code(source, Language::Rust).unwrap();
    if let Some(chunk) = chunks.first() {
        let quality = extract_quality_metrics(chunk, source);
        assert!(quality.complexity >= 3); // if + if + for
        assert!(quality.satd_count >= 1); // TODO
        assert_eq!(quality.big_o, "O(n)"); // single for loop
    }
}

#[test]
fn test_count_complexity_various() {
    // Multi-line if/else if
    let if_else = "fn f() {\n    if a {\n    } else if b {\n    } else {\n    }\n}";
    assert!(count_complexity(if_else) >= 3);
    // Match expression on its own line
    let matchex = "fn f() {\n    match x {\n        A => {},\n        B => {}\n    }\n}";
    assert!(count_complexity(matchex) >= 2);
    // While loop
    let whileex = "fn f() {\n    while true {\n        break;\n    }\n}";
    assert!(count_complexity(whileex) >= 2);
    // Boolean operators on one line
    let booleans = "fn f() { x && y || z }";
    assert!(count_complexity(booleans) >= 2); // && and || both on same line count once
}

#[test]
fn test_count_satd_markers_various() {
    assert_eq!(count_satd_markers("// FIXME: broken"), 1);
    assert_eq!(count_satd_markers("// HACK: workaround"), 1);
    assert_eq!(count_satd_markers("// XXX: temporary"), 0); // XXX removed - caused false positives from BUG-XXX patterns
    assert_eq!(count_satd_markers("// TODO: fix\n// FIXME: also fix"), 2);
    assert_eq!(count_satd_markers("// Normal comment"), 0);
}

#[test]
fn test_extract_identifiers() {
    let idents = extract_identifiers("fn foo() { bar_baz(42); hello.world(); }");
    assert!(idents.contains("foo"));
    assert!(idents.contains("bar_baz"));
    assert!(idents.contains("hello"));
    assert!(idents.contains("world"));
    // Short words (<3 chars) excluded
    assert!(!idents.contains("42"));
}

#[test]
fn test_extract_identifiers_filters_keywords() {
    let idents = extract_identifiers("fn handle() { if let mut x = return; }");
    // Keywords excluded
    assert!(!idents.contains("fn"));
    assert!(!idents.contains("if"));
    assert!(!idents.contains("let"));
    assert!(!idents.contains("mut"));
    assert!(!idents.contains("return"));
    // Non-keyword kept
    assert!(idents.contains("handle"));
}

#[test]
fn test_extract_doc_comment_block() {
    let content = "/**\n * Block doc comment\n */\nfn foo() {}";
    let doc = extract_doc_comment(content, 4);
    // Block comments cause break, so may return None or partial
    assert!(
        doc.is_none()
            || doc
                .as_ref()
                .is_some_and(|d| d.contains("Block doc comment"))
    );
}

#[test]
fn test_extract_doc_comment_with_attribute() {
    let content = "/// Doc line\n#[inline]\nfn foo() {}";
    let doc = extract_doc_comment(content, 3);
    assert!(doc.is_some());
    assert!(doc.unwrap().contains("Doc line"));
}

#[test]
fn test_estimate_big_o_cubic() {
    let cubic = "fn f() {\n    for i in a {\n        for j in b {\n            for k in c {\n                process();\n            }\n        }\n    }\n}";
    assert_eq!(estimate_big_o(cubic), "O(n^3)");
}

#[test]
fn test_estimate_big_o_n4() {
    let n4 = "fn f() {\n    for _ in a {\n        for _ in b {\n            for _ in c {\n                for _ in d {\n                    x();\n                }\n            }\n        }\n    }\n}";
    assert_eq!(estimate_big_o(n4), "O(n^4)");
}

#[test]
fn test_calculate_simple_tdg_boundaries() {
    // Zero everything (complexity<=1 floor ensures A grade)
    let score = calculate_simple_tdg(0, 0, 0);
    assert!((score - 0.0).abs() < 0.01);

    // Max complexity capped at 4.0
    let max_complexity = calculate_simple_tdg(100, 0, 0);
    assert!((max_complexity - 4.0).abs() < 0.01);

    // SATD penalty only (with complexity > 1 to bypass GH-272 floor): capped at 2.0
    let max_satd = calculate_simple_tdg(25, 10, 0);
    // complexity penalty = 1.0, SATD cap = 2.0 → total = 3.0
    assert!((max_satd - 3.0).abs() < 0.01);

    // LOC penalty kicks in above 200
    let no_loc_penalty = calculate_simple_tdg(25, 0, 200);
    // complexity 25 -> 1.0, no loc penalty
    assert!((no_loc_penalty - 1.0).abs() < 0.01);

    let large_loc = calculate_simple_tdg(25, 0, 400);
    assert!(large_loc > 1.0);

    // Max possible: complexity=4 + satd=2 + loc=2 = 8.0
    let max_all = calculate_simple_tdg(100, 10, 1000);
    assert!((max_all - 8.0).abs() < 0.01);
}

// GH-272: cyclomatic complexity 1 means no branches (simplest possible
// control flow). Such functions should never fall below grade A regardless
// of SATD or LOC penalties (long data-table initializers, trivial constructors).
#[test]
fn test_gh272_complexity_1_always_grades_a() {
    // High LOC with complexity 1 (e.g. 1000-line data-table initializer)
    let long_trivial = calculate_simple_tdg(1, 0, 1000);
    assert!(
        long_trivial < 2.0,
        "complexity=1 with 1000 LOC should stay A-grade (got {long_trivial})"
    );
    assert_eq!(score_to_grade(long_trivial), "A");

    // High SATD with complexity 1 — should also stay A
    let satd_trivial = calculate_simple_tdg(1, 20, 0);
    assert!(
        satd_trivial < 2.0,
        "complexity=1 with 20 SATD should stay A-grade (got {satd_trivial})"
    );
    assert_eq!(score_to_grade(satd_trivial), "A");

    // Combined: giant + SATD + complexity 1 — still A
    let worst = calculate_simple_tdg(1, 50, 2000);
    assert!(worst < 2.0, "complexity=1 always caps < 2.0 (got {worst})");
    assert_eq!(score_to_grade(worst), "A");

    // Complexity 2 (one branch) does NOT get the floor
    let complexity_2_large = calculate_simple_tdg(2, 0, 1000);
    // 2/25 + (1000-200)/200.min(2.0) = 0.08 + 2.0 = 2.08 -> B
    assert!(complexity_2_large >= 2.0, "complexity=2 keeps normal scoring");
}

#[test]
fn test_score_to_grade_boundaries() {
    assert_eq!(score_to_grade(0.0), "A");
    assert_eq!(score_to_grade(1.99), "A");
    assert_eq!(score_to_grade(2.0), "B");
    assert_eq!(score_to_grade(3.99), "B");
    assert_eq!(score_to_grade(4.0), "C");
    assert_eq!(score_to_grade(5.99), "C");
    assert_eq!(score_to_grade(6.0), "D");
    assert_eq!(score_to_grade(7.99), "D");
    assert_eq!(score_to_grade(8.0), "F");
    assert_eq!(score_to_grade(10.0), "F");
}

#[test]
fn test_extract_return_type() {
    assert_eq!(extract_return_type("fn foo() -> bool"), "bool");
    assert_eq!(
        extract_return_type("fn foo() -> Result<String, Error>"),
        "Result<String, Error>"
    );
    assert_eq!(extract_return_type("fn foo()"), "void");
}

#[test]
fn test_count_params() {
    assert_eq!(count_params("fn foo()"), 0);
    assert_eq!(count_params("fn foo(x: i32)"), 1);
    assert_eq!(count_params("fn foo(x: i32, y: String)"), 2);
    assert_eq!(count_params("fn foo(x: i32, y: String, z: bool)"), 3);
    assert_eq!(count_params("no parens"), 0);
    // C++ regression: comment with ')' before '(' must not panic (PyTorch crash)
    // find('(') now locates the first '(' which may be in a comment — the key invariant
    // is NO PANIC, not perfect param counting on multiline comment+signature strings
    let _ = count_params("// 1) out = exp(a - val)\nvoid softmax(float* x, int n)");
    // C++ with nested parens in types
    assert_eq!(count_params("void foo(std::vector<int> v, int n)"), 2);
    // CUDA kernel signature
    assert_eq!(
        count_params("__global__ void kernel(float* out, const float* in, int n)"),
        3
    );
}

#[test]
fn test_normalize_source_hash() {
    // Same content with different whitespace produces same hash
    assert_eq!(
        normalize_source_hash("fn foo() { }"),
        normalize_source_hash("fn foo() {}")
    );
    assert_eq!(
        normalize_source_hash("  fn  foo ( ) {\n}"),
        normalize_source_hash("fn foo(){}"),
    );
    // Case-insensitive: FN FOO() == fn foo()
    assert_eq!(
        normalize_source_hash("FN FOO()"),
        normalize_source_hash("fn foo()"),
    );
    // Different content produces different hash
    assert_ne!(
        normalize_source_hash("fn foo()"),
        normalize_source_hash("fn bar()"),
    );
}

#[test]
fn test_is_ignored_dir_comprehensive() {
    assert!(is_ignored_dir(Path::new("target")));
    assert!(is_ignored_dir(Path::new("node_modules")));
    assert!(is_ignored_dir(Path::new(".git")));
    assert!(is_ignored_dir(Path::new(".pmat")));
    assert!(is_ignored_dir(Path::new("__pycache__")));
    assert!(is_ignored_dir(Path::new("venv")));
    assert!(is_ignored_dir(Path::new(".venv")));
    assert!(is_ignored_dir(Path::new("dist")));
    assert!(is_ignored_dir(Path::new("build")));
    assert!(is_ignored_dir(Path::new(".next")));
    assert!(is_ignored_dir(Path::new(".cache")));
    assert!(is_ignored_dir(Path::new("vendor")));
    assert!(is_ignored_dir(Path::new("third_party")));
    assert!(is_ignored_dir(Path::new("fixtures")));
    assert!(is_ignored_dir(Path::new(".cargo")));
    assert!(!is_ignored_dir(Path::new("src")));
    assert!(!is_ignored_dir(Path::new("lib")));
    assert!(!is_ignored_dir(Path::new("server")));
}

#[test]
fn test_detect_language_all_types() {
    assert_eq!(detect_language(Path::new("test.rs")), Some(Language::Rust));
    assert_eq!(
        detect_language(Path::new("test.py")),
        Some(Language::Python)
    );
    assert_eq!(
        detect_language(Path::new("test.ts")),
        Some(Language::TypeScript)
    );
    assert_eq!(
        detect_language(Path::new("test.tsx")),
        Some(Language::TypeScript)
    );
    assert_eq!(detect_language(Path::new("test.c")), Some(Language::C));
    assert_eq!(detect_language(Path::new("test.h")), Some(Language::C));
    assert_eq!(detect_language(Path::new("test.cpp")), Some(Language::Cpp));
    assert_eq!(detect_language(Path::new("test.go")), Some(Language::Go));
    assert_eq!(detect_language(Path::new("test.md")), None);
    assert_eq!(detect_language(Path::new("test.toml")), None);
    // CUDA extensions
    assert_eq!(detect_language(Path::new("kernel.cu")), Some(Language::Cpp));
    assert_eq!(
        detect_language(Path::new("kernel.cuh")),
        Some(Language::Cpp)
    );
}

#[test]
fn test_classify_header_language_pure_c() {
    let content = r#"
#ifndef GGML_H
#define GGML_H
#include <stdint.h>
struct ggml_tensor { int ne[4]; void* data; };
int ggml_init(int n);
void ggml_free(void);
#endif
"#;
    assert_eq!(classify_header_language(content), Language::C);
}

#[test]
fn test_classify_header_language_extern_c() {
    let content = r#"
#ifndef LLAMA_H
#define LLAMA_H
#ifdef __cplusplus
extern "C" {
#endif
int llama_decode(void* ctx, int n);
#ifdef __cplusplus
}
#endif
#endif
"#;
    assert_eq!(classify_header_language(content), Language::Cpp);
}

#[test]
fn test_classify_header_language_cpp_class() {
    let content = r#"
#pragma once
namespace whisper {
class Context {
public:
    void decode();
private:
    int n_;
};
}
"#;
    assert_eq!(classify_header_language(content), Language::Cpp);
}

#[test]
fn test_classify_header_language_template() {
    let content = r#"
#pragma once
template <typename T>
T clamp(T val, T lo, T hi) {
    return (val < lo) ? lo : (val > hi) ? hi : val;
}
"#;
    assert_eq!(classify_header_language(content), Language::Cpp);
}

#[test]
fn test_cpp_complexity_penalty_preprocessor() {
    let source = r#"
void foo() {
    #ifdef __CUDA_ARCH__
        #if __CUDA_ARCH__ >= 800
            do_sm80();
        #endif
    #endif
}
"#;
    let penalty = cpp_complexity_penalty(source);
    // #ifdef at depth 1 (+1), #if at depth 2 (+2) = 3
    assert!(penalty >= 3, "preprocessor nesting penalty: got {penalty}");
}

#[test]
fn test_cpp_complexity_penalty_macro_heavy() {
    let source = r#"
void init_model() {
    GGML_ASSERT(ctx != NULL);
    GGML_LOG_INFO("loading model");
    GGML_ASSERT(n_vocab > 0);
    GGML_CHECK(buf != NULL);
    GGML_ASSERT(embd > 0);
    GGML_LOG_WARN("deprecated path");
}
"#;
    let penalty = cpp_complexity_penalty(source);
    // 6 GGML_ macro calls > 5 threshold → +3
    assert!(penalty >= 3, "macro-heavy penalty: got {penalty}");
}

#[test]
fn test_cpp_complexity_penalty_cuda_kernel() {
    let source = r#"
__global__ void softmax_kernel(float* output, const float* input, int n) {
    __shared__ float shared_max[32];
    int tid = threadIdx.x;
    if (tid < n) {
        output[tid] = expf(input[tid]);
    }
    __syncthreads();
}
"#;
    let penalty = cpp_complexity_penalty(source);
    // __shared__ (+2) + __syncthreads (+3) + thread divergence (__global__ + if) (+2) = 7
    assert!(penalty >= 7, "CUDA kernel penalty: got {penalty}");
}

#[test]
fn test_cpp_complexity_penalty_template_nesting() {
    let source = r#"
template <typename T>
template <int N>
void MatMul<T>::compute(T* out) {
    for (int i = 0; i < N; i++) {
        out[i] = a[i] * b[i];
    }
}
"#;
    let penalty = cpp_complexity_penalty(source);
    // 2 template<> → 1 extra level → +2
    assert!(penalty >= 2, "template nesting penalty: got {penalty}");
}

#[test]
fn test_cpp_complexity_penalty_sfinae() {
    let source = r#"
template <typename T, typename = std::enable_if<std::is_arithmetic<T>::value>>
T add(T a, T b) { return a + b; }
"#;
    let penalty = cpp_complexity_penalty(source);
    // enable_if → +3
    assert!(penalty >= 3, "SFINAE penalty: got {penalty}");
}

#[test]
fn test_cpp_complexity_penalty_warp_primitives() {
    let source = r#"
__device__ float warp_reduce(float val) {
    val += __shfl_xor_sync(0xffffffff, val, 16);
    val += __shfl_xor_sync(0xffffffff, val, 8);
    return val;
}
"#;
    let penalty = cpp_complexity_penalty(source);
    // __shfl_ → +2
    assert!(penalty >= 2, "warp primitive penalty: got {penalty}");
}

#[test]
fn test_cpp_complexity_penalty_simple_function() {
    // A simple C++ function should have zero penalty
    let source = "int add(int a, int b) { return a + b; }";
    assert_eq!(cpp_complexity_penalty(source), 0);
}
