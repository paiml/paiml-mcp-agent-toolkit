    /// Test: WASM export names should be extracted instead of generic function_N
    #[test]
    fn test_wasm_export_names_extraction() {
        let wat_code = r#"
(module
  ;; Import external function
  (import "env" "log" (func $log (param i32)))

  ;; Exported functions with meaningful names
  (func $add (export "add") (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add
  )

  (func $multiply (export "multiply") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (if (i32.lt_s (local.get $n) (i32.const 2))
      (then (local.get $n))
      (else
        (i32.add
          (call $fibonacci (i32.sub (local.get $n) (i32.const 1)))
          (call $fibonacci (i32.sub (local.get $n) (i32.const 2)))
        )
      )
    )
  )

  ;; Memory and table exports
  (memory 1)
  (export "memory" (memory 0))

  (table 10 funcref)
  (export "function_table" (table 0))
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("math.wasm"));
        let items = analyzer
            .analyze_wat_text(wat_code)
            .expect("Should parse WAT text with exports");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract meaningful export names instead of generic names
        assert!(
            function_names.iter().any(|name| name.contains("add")),
            "Should extract 'add' function name from export, got: {:?}",
            function_names
        );
        assert!(
            function_names.iter().any(|name| name.contains("multiply")),
            "Should extract 'multiply' function name from export, got: {:?}",
            function_names
        );
        assert!(
            function_names.iter().any(|name| name.contains("fibonacci")),
            "Should extract 'fibonacci' function name from export, got: {:?}",
            function_names
        );

        // Should NOT contain generic function_N names when export names are available
        assert!(
            !function_names
                .iter()
                .any(|name| name.starts_with("function_")
                    && name.chars().last().unwrap().is_ascii_digit()),
            "Should not use generic function_N names when export names available, got: {:?}",
            function_names
        );
    }

    /// Test: WASM import functions should be tracked with module context
    #[test]
    fn test_wasm_import_functions_tracking() {
        let wat_code = r#"
(module
  ;; Various imports from different modules
  (import "env" "console_log" (func $console_log (param i32)))
  (import "env" "memory" (memory 1))
  (import "js" "Math.random" (func $random (result f64)))
  (import "js" "Date.now" (func $now (result f64)))
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

  ;; Functions that use imports
  (func $log_message (export "log_message") (param $msg_ptr i32)
    local.get $msg_ptr
    call $console_log
  )

  (func $get_random_number (export "get_random_number") (result f64)
    call $random
  )

  (func $write_to_stdout (export "write_to_stdout") (param $ptr i32) (param $len i32) (result i32)
    i32.const 1  ;; stdout fd
    local.get $ptr
    local.get $len
    i32.const 0  ;; nwritten_ptr
    call $fd_write
  )
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("imports.wasm"));
        let items = analyzer
            .analyze_wat_text(wat_code)
            .expect("Should parse WAT text with imports");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract exported functions that use imports
        assert!(
            function_names
                .iter()
                .any(|name| name.contains("log_message")),
            "Should extract function that uses import"
        );
        assert!(
            function_names
                .iter()
                .any(|name| name.contains("get_random_number")),
            "Should extract function that calls imported JS function"
        );
        assert!(
            function_names
                .iter()
                .any(|name| name.contains("write_to_stdout")),
            "Should extract function that uses WASI import"
        );

        // TODO: Import tracking is not yet implemented - this test defines the requirement
        // When implemented, should also track import statements:
        // assert!(items.iter().any(|item| matches!(item, AstItem::Use { path, .. } if path.contains("env.console_log"))));
    }

    /// Test: Complex WASM modules with nested functions and locals
    #[test]
    fn test_complex_wasm_module_analysis() {
        let wat_code = r#"
(module
  ;; Type definitions
  (type $binary_op (func (param i32 i32) (result i32)))
  (type $unary_op (func (param i32) (result i32)))

  ;; Function table for indirect calls
  (table $functions 4 funcref)
  (elem (i32.const 0) $add $sub $mul $div)

  ;; Basic arithmetic operations
  (func $add (type $binary_op)
    local.get 0
    local.get 1
    i32.add
  )

  (func $sub (type $binary_op)
    local.get 0
    local.get 1
    i32.sub
  )

  (func $mul (type $binary_op)
    local.get 0
    local.get 1
    i32.mul
  )

  (func $div (type $binary_op)
    local.get 0
    local.get 1
    i32.div_s
  )

  ;; Higher-order function using function table
  (func $apply_operation (export "apply_operation") (param $a i32) (param $b i32) (param $op_index i32) (result i32)
    local.get $a
    local.get $b
    local.get $op_index
    call_indirect (type $binary_op)
  )

  ;; Recursive factorial function
  (func $factorial (export "factorial") (param $n i32) (result i32)
    (local $result i32)
    (local $i i32)

    ;; Initialize result to 1
    i32.const 1
    local.set $result

    ;; Initialize counter to 1
    i32.const 1
    local.set $i

    ;; Loop from 1 to n
    (loop $factorial_loop
      ;; Check if i <= n
      local.get $i
      local.get $n
      i32.le_s

      (if
        (then
          ;; result = result * i
          local.get $result
          local.get $i
          i32.mul
          local.set $result

          ;; i = i + 1
          local.get $i
          i32.const 1
          i32.add
          local.set $i

          ;; Continue loop
          br $factorial_loop
        )
      )
    )

    local.get $result
  )

  ;; Array processing function with memory operations
  (func $sum_array (export "sum_array") (param $ptr i32) (param $len i32) (result i32)
    (local $sum i32)
    (local $i i32)
    (local $current_ptr i32)

    ;; Initialize sum to 0
    i32.const 0
    local.set $sum

    ;; Initialize index to 0
    i32.const 0
    local.set $i

    ;; Initialize current pointer
    local.get $ptr
    local.set $current_ptr

    (loop $sum_loop
      ;; Check if i < len
      local.get $i
      local.get $len
      i32.lt_s

      (if
        (then
          ;; Load value from memory and add to sum
          local.get $sum
          local.get $current_ptr
          i32.load
          i32.add
          local.set $sum

          ;; Move to next array element (increment by 4 bytes for i32)
          local.get $current_ptr
          i32.const 4
          i32.add
          local.set $current_ptr

          ;; Increment index
          local.get $i
          i32.const 1
          i32.add
          local.set $i

          ;; Continue loop
          br $sum_loop
        )
      )
    )

    local.get $sum
  )

  ;; Memory allocation
  (memory $mem 1)
  (export "memory" (memory $mem))
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("complex.wasm"));
        let items = analyzer
            .analyze_wat_text(wat_code)
            .expect("Should parse complex WAT module");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract all function names including internal and exported
        assert!(
            function_names.iter().any(|name| name.contains("add")),
            "Should extract arithmetic function name"
        );
        assert!(
            function_names
                .iter()
                .any(|name| name.contains("apply_operation")),
            "Should extract higher-order function name"
        );
        assert!(
            function_names.iter().any(|name| name.contains("factorial")),
            "Should extract recursive function name"
        );
        assert!(
            function_names.iter().any(|name| name.contains("sum_array")),
            "Should extract memory processing function name"
        );

        // Should have reasonable number of functions (not just exports)
        assert!(
            function_names.len() >= 6,
            "Should extract both internal and exported functions, got {} functions: {:?}",
            function_names.len(),
            function_names
        );
    }

    /// Test: WASM binary analysis should extract function information
    #[test]
    fn test_wasm_binary_function_extraction() {
        // This is a minimal valid WASM binary that exports an "add" function
        let wasm_binary: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version
            // Type section
            0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // Function section
            0x03, 0x02, 0x01, 0x00, // Export section
            0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // Code section
            0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];

        let analyzer = WasmModuleAnalyzer::new(Path::new("add.wasm"));
        let items = analyzer
            .analyze_wasm_binary(wasm_binary)
            .expect("Should parse WASM binary");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract function from binary (currently limited)
        assert!(
            !function_names.is_empty(),
            "Should extract at least one function from WASM binary"
        );

        // Note: This test currently expects the basic implementation
        // Enhanced implementation should extract export names from binary sections
        println!("Extracted functions from WASM binary: {:?}", function_names);
    }

    /// Test: WASM module validation should catch errors but preserve names
    #[test]
    fn test_wasm_validation_with_name_preservation() {
        let invalid_wat_code = r#"
(module
  ;; This has some issues but should still extract function names where possible
  (func $valid_function (export "valid") (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.add
  )

  ;; This function has a naming issue but name should still be extracted
  (func $another-function (export "another")
    ;; Missing return type but has a name
    nop
  )
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("mixed.wasm"));
        // Should not panic even with parsing issues
        let result = analyzer.analyze_wat_text(invalid_wat_code);

        match result {
            Ok(items) => {
                let function_names: Vec<String> = items
                    .iter()
                    .filter_map(|item| match item {
                        AstItem::Function { name, .. } => Some(name.clone()),
                        _ => None,
                    })
                    .collect();

                // Should extract names where possible despite validation issues
                assert!(
                    function_names
                        .iter()
                        .any(|name| name.contains("valid_function")),
                    "Should extract valid function names even with module issues"
                );
            }
            Err(_) => {
                // Parsing might fail, which is acceptable for malformed WAT
                // The key point is that we don't panic and handle errors gracefully
                println!("WAT parsing failed gracefully as expected for invalid code");
            }
        }
    }
