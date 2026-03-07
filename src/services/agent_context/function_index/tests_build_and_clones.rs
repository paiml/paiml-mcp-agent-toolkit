// Build scenario tests (multiple files, empty, binary, ignored dirs),
// merge_fast, manifest accessor, code clones, pattern diversity, fault patterns,
// name_index capping, build filtering

#[test]
fn test_merge_fast() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Build index A
    let proj_a = temp_dir.path().join("a");
    std::fs::create_dir_all(proj_a.join("src")).unwrap();
    std::fs::write(proj_a.join("src/lib.rs"), "fn alpha() {}\n").unwrap();
    let mut index_a = AgentContextIndex::build(&proj_a).unwrap();
    let a_count = index_a.functions.len();

    // Build index B
    let proj_b = temp_dir.path().join("b");
    std::fs::create_dir_all(proj_b.join("src")).unwrap();
    std::fs::write(proj_b.join("src/lib.rs"), "fn beta() {}\n").unwrap();
    let index_b = AgentContextIndex::build(&proj_b).unwrap();
    let b_count = index_b.functions.len();

    index_a.merge_fast(index_b);

    assert_eq!(index_a.functions.len(), a_count + b_count);
    // All functions accessible
    let names: Vec<&str> = index_a
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn test_manifest_accessor() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn foo() {}\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let manifest = index.manifest();
    assert_eq!(manifest.version, "1.4.0");
    assert!(manifest.function_count > 0);
    assert!(manifest.file_count > 0);
}

#[test]
fn test_build_with_multiple_files() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create multiple Rust files
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "/// Documentation for rust_func\nfn rust_func() { if true { println!(\"hello\"); } }\n",
    )
    .unwrap();
    std::fs::write(
        project_path.join("src/helper.rs"),
        "/// Helper function\nfn helper_func() { for i in 0..10 { println!(\"{}\", i); } }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    assert!(index.functions.len() >= 2);
    assert!(index.manifest.file_count >= 2);
    assert_eq!(index.manifest.version, "1.4.0");

    // Verify quality metrics computed
    for func in &index.functions {
        assert!(!func.function_name.is_empty());
        assert!(!func.file_path.is_empty());
        assert!(!func.language.is_empty());
        assert!(!func.quality.tdg_grade.is_empty());
        assert!(!func.quality.big_o.is_empty());
    }
}

#[test]
fn test_build_empty_project() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    // Empty file - no functions
    std::fs::write(project_path.join("src/lib.rs"), "// empty\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    assert_eq!(index.functions.len(), 0);
    assert!((index.manifest.avg_tdg_score - 0.0).abs() < 0.01);
}

#[test]
fn test_build_with_binary_file() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn valid() {}\n").unwrap();
    // Binary file should be skipped
    std::fs::write(project_path.join("src/data.bin"), &[0u8, 1, 2, 255, 254]).unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    // Should index the .rs file but skip .bin
    assert!(index.functions.len() >= 1);
}

#[test]
fn test_build_skips_ignored_dirs() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::create_dir_all(project_path.join("node_modules/pkg")).unwrap();
    std::fs::create_dir_all(project_path.join("target/debug")).unwrap();

    std::fs::write(project_path.join("src/lib.rs"), "fn keep() {}\n").unwrap();
    std::fs::write(
        project_path.join("node_modules/pkg/index.rs"),
        "fn skip_nm() {}\n",
    )
    .unwrap();
    std::fs::write(
        project_path.join("target/debug/output.rs"),
        "fn skip_target() {}\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let names: Vec<&str> = index
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(names.contains(&"keep"));
    assert!(!names.contains(&"skip_nm"));
    assert!(!names.contains(&"skip_target"));
}

#[test]
fn test_detect_code_clones_with_duplicates() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "func_a".to_string(),
            signature: "fn func_a()".to_string(),
            doc_comment: None,
            source: "fn func_a() { let x = 1; let y = 2; x + y }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "func_b".to_string(),
            signature: "fn func_b()".to_string(),
            doc_comment: None,
            // Same source after normalization (whitespace-insensitive)
            source: "fn func_a() { let x = 1; let y = 2; x + y }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "c.rs".to_string(),
            function_name: "func_c".to_string(),
            signature: "fn func_c()".to_string(),
            doc_comment: None,
            source: "fn func_c() { completely_different_code(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "ccc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];

    let clones = detect_code_clones(&funcs);
    // func_a and func_b are clones (same normalized source)
    assert_eq!(clones.get(&0), Some(&2));
    assert_eq!(clones.get(&1), Some(&2));
    // func_c is unique
    assert_eq!(clones.get(&2), None);
}

#[test]
fn test_detect_code_clones_no_duplicates() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "unique_a".to_string(),
            signature: "fn unique_a()".to_string(),
            doc_comment: None,
            source: "fn unique_a() { alpha(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "unique_b".to_string(),
            signature: "fn unique_b()".to_string(),
            doc_comment: None,
            source: "fn unique_b() { beta(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];

    let clones = detect_code_clones(&funcs);
    assert!(clones.is_empty());
}

#[test]
fn test_compute_file_pattern_diversity() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f1".to_string(),
            signature: "fn f1() -> bool".to_string(),
            doc_comment: None,
            source: "".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                complexity: 2,
                ..Default::default()
            },
            checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f2".to_string(),
            signature: "fn f2(x: i32) -> String".to_string(),
            doc_comment: None,
            source: "".to_string(),
            start_line: 5,
            end_line: 10,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                complexity: 8,
                ..Default::default()
            },
            checksum: "b".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f3".to_string(),
            signature: "fn f3() -> bool".to_string(),
            doc_comment: None,
            source: "".to_string(),
            start_line: 15,
            end_line: 20,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                complexity: 2,
                ..Default::default()
            },
            checksum: "c".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];
    let mut file_index = HashMap::new();
    file_index.insert("a.rs".to_string(), vec![0, 1, 2]);

    let diversity = compute_file_pattern_diversity(&funcs, &file_index);
    let d = diversity["a.rs"];
    // f1 and f3 have same pattern (bool:0:0), f2 is different (String:1:1)
    // 2 unique / 3 total = 0.667
    assert!(d > 0.5 && d < 0.8, "unexpected diversity: {d}");
}

#[test]
fn test_compute_file_pattern_diversity_empty() {
    let funcs: Vec<FunctionEntry> = vec![];
    let mut file_index = HashMap::new();
    file_index.insert("a.rs".to_string(), Vec::new());

    let diversity = compute_file_pattern_diversity(&funcs, &file_index);
    assert!(!diversity.contains_key("a.rs")); // empty indices skipped
}

#[test]
fn test_detect_fault_patterns() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "risky".to_string(),
            signature: "fn risky()".to_string(),
            doc_comment: None,
            source: "fn risky() { x.unwrap(); y.clone(); // TODO: fix }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "safe".to_string(),
            signature: "fn safe()".to_string(),
            doc_comment: None,
            source: "fn safe() { println!(\"hello\"); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "b".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "c.rs".to_string(),
            function_name: "dangerous".to_string(),
            signature: "fn dangerous()".to_string(),
            doc_comment: None,
            source: "fn dangerous() { unsafe { panic!(\"boom\"); } }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "c".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];

    let faults = detect_fault_patterns(&funcs);
    // risky has: UNWRAP, CLONE, TODO
    let risky_faults = &faults[&0];
    assert!(risky_faults.contains(&"UNWRAP".to_string()));
    assert!(risky_faults.contains(&"CLONE".to_string()));
    assert!(risky_faults.contains(&"TODO".to_string()));
    // safe has no faults
    assert!(!faults.contains_key(&1));
    // dangerous has PANIC, UNSAFE
    let dangerous_faults = &faults[&2];
    assert!(dangerous_faults.contains(&"PANIC".to_string()));
    assert!(dangerous_faults.contains(&"UNSAFE".to_string()));
}

#[test]
fn test_detect_fault_patterns_more() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f".to_string(),
            signature: "fn f()".to_string(),
            doc_comment: None,
            source: "fn f() { x.expect(\"msg\"); // FIXME: broken\n// HACK: workaround\n// XXX: bad\ntodo!(\"later\");\nunimplemented!(\"not yet\");\nunreachable!(\"never\"); }".to_string(),
            start_line: 1, end_line: 1, language: "Rust".to_string(),
            quality: QualityMetrics::default(), checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0, churn_score: 0.0, clone_count: 0,
            pattern_diversity: 0.0, fault_annotations: Vec::new(), linked_definition: None,
        },
    ];

    let faults = detect_fault_patterns(&funcs);
    let f = &faults[&0];
    assert!(f.contains(&"EXPECT".to_string()));
    assert!(f.contains(&"FIXME".to_string()));
    assert!(f.contains(&"HACK".to_string()));
    assert!(f.contains(&"XXX".to_string()));
    assert!(f.contains(&"TODO_MACRO".to_string()));
    assert!(f.contains(&"UNIMPL".to_string()));
    assert!(f.contains(&"UNREACHABLE".to_string()));
}

#[test]
fn test_detect_fault_patterns_cuda_ptx() {
    let funcs = vec![FunctionEntry {
        file_path: "kernel.cu".to_string(),
        function_name: "softmax_kernel".to_string(),
        signature: "__global__ void softmax_kernel(float* out, const float* in, int n)".to_string(),
        doc_comment: None,
        source: concat!(
            "__global__ void softmax_kernel(float* out, const float* in, int n) {\n",
            "    __shared__ float sdata[256];\n",
            "    int tid = threadIdx.x;\n",
            "    sdata[tid] = in[tid];\n",
            "    __syncthreads();\n",
            "    asm volatile(\"bar.sync 0;\");\n",
            "}\n"
        )
        .to_string(),
        start_line: 1,
        end_line: 7,
        language: "C++".to_string(),
        quality: QualityMetrics::default(),
        checksum: "cuda".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(), linked_definition: None,
    }];

    let faults = detect_fault_patterns(&funcs);
    let f = &faults[&0];
    assert!(f.contains(&"CUDA_SHMEM".to_string()));
    assert!(f.contains(&"CUDA_SYNC".to_string()));
    assert!(f.contains(&"INLINE_PTX".to_string()));
    // PTX instruction tags from asm volatile("bar.sync 0;")
    assert!(f.contains(&"PTX:bar.sync".to_string()));
}

#[test]
fn test_detect_fault_patterns_ptx_instructions() {
    let funcs = vec![FunctionEntry {
        file_path: "mma.cuh".to_string(),
        function_name: "mma_A".to_string(),
        signature: "__device__ void mma_A()".to_string(),
        doc_comment: None,
        source: concat!(
            "static __device__ void mma_A(int* x, const int* A, const int* B) {\n",
            "    asm(\"mma.sync.aligned.m16n8k32.row.col.s32.s8.s8.s32 \"\n",
            "        \"{%0, %1, %2, %3}, {%4, %5}, {%6}, {%0, %1, %2, %3};\"\n",
            "        : \"=r\"(x[0]) : \"r\"(A[0]));\n",
            "}\n"
        )
        .to_string(),
        start_line: 1,
        end_line: 5,
        language: "C++".to_string(),
        quality: QualityMetrics::default(),
        checksum: "mma".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(), linked_definition: None,
    }];

    let faults = detect_fault_patterns(&funcs);
    let f = &faults[&0];
    assert!(f.contains(&"INLINE_PTX".to_string()));
    assert!(f.contains(&"PTX:mma.sync".to_string()));
}

#[test]
fn test_detect_fault_patterns_ptx_cp_async() {
    let funcs = vec![FunctionEntry {
        file_path: "cp_async.cuh".to_string(),
        function_name: "async_copy".to_string(),
        signature: "__device__ void async_copy()".to_string(),
        doc_comment: None,
        source: concat!(
            "__device__ void async_copy(void* dst, const void* src) {\n",
            "    asm volatile(\"cp.async.cg.shared.global [%0], [%1], 16;\"\n",
            "                 :: \"r\"(dst), \"l\"(src));\n",
            "}\n"
        )
        .to_string(),
        start_line: 1,
        end_line: 4,
        language: "C++".to_string(),
        quality: QualityMetrics::default(),
        checksum: "cpa".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(), linked_definition: None,
    }];

    let faults = detect_fault_patterns(&funcs);
    let f = &faults[&0];
    assert!(f.contains(&"INLINE_PTX".to_string()));
    assert!(f.contains(&"PTX:cp.async".to_string()));
}

#[test]
fn test_name_index_capped_at_100() {
    // Create 150 functions all named "new"
    let functions: Vec<FunctionEntry> = (0..150)
        .map(|i| FunctionEntry {
            file_path: format!("f{i}.rs"),
            function_name: "new".to_string(),
            signature: "fn new()".to_string(),
            doc_comment: None,
            source: format!("fn new() {{ /* variant {i} */ }}"),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("{i}"),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        })
        .collect();

    let indices = build_indices(&functions);
    // name_index should be capped at 100 entries for "new"
    assert_eq!(indices.name_index["new"].len(), 100);
    // file_index should NOT be capped (all 150 files)
    assert_eq!(indices.file_index.len(), 150);
}

#[test]
fn test_build_filters_test_functions() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn real_func() { }\nfn test_something() { }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let names: Vec<&str> = index
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(
        names.contains(&"real_func"),
        "non-test function should be indexed"
    );
    assert!(
        !names.contains(&"test_something"),
        "test_ function should be filtered"
    );
}

#[test]
fn test_classify_cpp_macros_assert() {
    let mut faults = Vec::new();
    let source = r#"void validate(ggml_context* ctx) {
    GGML_ASSERT(ctx != NULL);
    GGML_ASSERT(ctx->n_tensors > 0);
}"#;
    classify_cpp_macros(source, &mut faults);
    assert!(faults.contains(&"MACRO:ASSERT".to_string()), "faults={faults:?}");
    assert!(!faults.contains(&"MACRO:DISPATCH".to_string()));
}

#[test]
fn test_classify_cpp_macros_dispatch() {
    let mut faults = Vec::new();
    let source = r#"void compute(at::Tensor input) {
    AT_DISPATCH_ALL_TYPES(input.scalar_type(), "compute", [&] {
        auto data = input.data_ptr<scalar_t>();
    });
}"#;
    classify_cpp_macros(source, &mut faults);
    assert!(faults.contains(&"MACRO:DISPATCH".to_string()), "faults={faults:?}");
}

#[test]
fn test_classify_cpp_macros_logging() {
    let mut faults = Vec::new();
    let source = r#"void init() {
    GGML_LOG_INFO("initializing model");
    GGML_LOG_WARN("deprecated API");
}"#;
    classify_cpp_macros(source, &mut faults);
    assert!(faults.contains(&"MACRO:LOG".to_string()), "faults={faults:?}");
}

#[test]
fn test_classify_cpp_macros_none() {
    let mut faults = Vec::new();
    let source = "int add(int a, int b) { return a + b; }";
    classify_cpp_macros(source, &mut faults);
    assert!(faults.is_empty(), "simple function should have no macro faults");
}

#[test]
fn test_detect_inline_ptx_defects_missing_barrier() {
    let mut faults = Vec::new();
    let source = r#"__device__ void unsafe_shared(float* sdata) {
    asm volatile("st.shared.f32 [%0], %1;" : : "l"(sdata), "f"(val));
    asm volatile("ld.shared.f32 %0, [%1];" : "=f"(result) : "l"(sdata));
}"#;
    detect_inline_ptx_defects(source, &mut faults);
    assert!(faults.contains(&"PTX_MISSING_BARRIER".to_string()), "faults={faults:?}");
}

#[test]
fn test_detect_inline_ptx_defects_barrier_divergence() {
    let mut faults = Vec::new();
    let source = r#"__device__ void divergent_barrier(int tid) {
    asm volatile("st.shared.f32 [%0], %1;" : : "l"(sdata), "f"(val));
    if (tid < 16) {
        asm volatile("bar.sync 0;");
    }
}"#;
    detect_inline_ptx_defects(source, &mut faults);
    assert!(faults.contains(&"PTX_BARRIER_DIV".to_string()), "faults={faults:?}");
}

#[test]
fn test_detect_inline_ptx_defects_high_regs() {
    let mut faults = Vec::new();
    // 10 register outputs -> PTX_HIGH_REGS
    let source = r#"__device__ void many_regs() {
    asm("mma.sync.aligned.m16n8k16 {%0,%1,%2,%3,%4,%5,%6,%7,%8,%9}, ..."
        : "=r"(d0), "=r"(d1), "=r"(d2), "=r"(d3), "=r"(d4),
          "=r"(d5), "=r"(d6), "=r"(d7), "=r"(d8), "=r"(d9)
        : "r"(a0));
}"#;
    detect_inline_ptx_defects(source, &mut faults);
    assert!(faults.contains(&"PTX_HIGH_REGS".to_string()), "faults={faults:?}");
}

#[test]
fn test_detect_inline_ptx_defects_safe() {
    let mut faults = Vec::new();
    // Safe: has barrier between store and load
    let source = r#"__device__ void safe_shared(float* sdata) {
    asm volatile("st.shared.f32 [%0], %1;" : : "l"(sdata), "f"(val));
    asm volatile("bar.sync 0;");
    asm volatile("ld.shared.f32 %0, [%1];" : "=f"(result) : "l"(sdata));
}"#;
    detect_inline_ptx_defects(source, &mut faults);
    assert!(!faults.contains(&"PTX_MISSING_BARRIER".to_string()), "should not flag when barrier present");
}
