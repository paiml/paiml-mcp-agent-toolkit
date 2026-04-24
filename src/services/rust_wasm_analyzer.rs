#![cfg_attr(coverage_nightly, coverage(off))]
//! Rust WASM Analyzer Extension
//!
//! Extends the Rust analyzer to detect WASM-specific constructs.
//! Implements DWASM-004: Track #[wasm_bindgen], extern "C", #[no_mangle], and memory patterns.

use syn::{Attribute, FnArg, ForeignItem, Item, ItemFn, Type};

/// WASM boundary function information
#[derive(Debug, Clone)]
pub struct WasmBoundaryFunction {
    pub name: String,
    pub is_wasm_bindgen: bool,
    pub is_extern_c: bool,
    pub is_no_mangle: bool,
    pub has_unsafe: bool,
    pub memory_patterns: Vec<MemoryPattern>,
    pub line: usize,
}

/// Memory management patterns at WASM boundaries
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPattern {
    Box,
    Vec,
    String,
    RawPointer,
    Reference,
}

/// Extern block information
#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String,
    pub functions: Vec<String>,
    pub line: usize,
}

/// WASM-specific analysis result
#[derive(Debug, Default)]
pub struct WasmAnalysis {
    pub boundary_functions: Vec<WasmBoundaryFunction>,
    pub extern_blocks: Vec<ExternBlock>,
    pub unsafe_blocks: Vec<UnsafeBlock>,
}

/// Unsafe block that may affect WASM
#[derive(Debug, Clone)]
pub struct UnsafeBlock {
    pub context: String,
    pub line: usize,
    pub affects_wasm: bool,
}

/// Analyze Rust file for WASM-specific constructs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn analyze_wasm_constructs(file: &syn::File) -> WasmAnalysis {
    let mut analysis = WasmAnalysis::default();

    for item in &file.items {
        match item {
            Item::Fn(func) => {
                if let Some(boundary_fn) = analyze_function(func) {
                    analysis.boundary_functions.push(boundary_fn);
                }
            }
            Item::ForeignMod(foreign_mod) => {
                if let Some(extern_block) = analyze_extern_block(foreign_mod) {
                    analysis.extern_blocks.push(extern_block);
                }
            }
            Item::Impl(impl_block) => {
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        if let Some(boundary_fn) = analyze_impl_method(method, &impl_block.attrs) {
                            analysis.boundary_functions.push(boundary_fn);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    analysis
}

/// Analyze a function for WASM boundary markers
fn analyze_function(func: &ItemFn) -> Option<WasmBoundaryFunction> {
    let is_wasm_bindgen = has_attribute(&func.attrs, "wasm_bindgen");
    let is_no_mangle = has_attribute(&func.attrs, "no_mangle");
    let is_extern_c = is_extern_c_fn(&func.sig.abi);

    // Only track WASM boundary functions
    if !is_wasm_bindgen && !is_no_mangle && !is_extern_c {
        return None;
    }

    let name = func.sig.ident.to_string();
    let has_unsafe = func.sig.unsafety.is_some();
    let memory_patterns = analyze_function_signature(&func.sig);

    Some(WasmBoundaryFunction {
        name,
        is_wasm_bindgen,
        is_extern_c,
        is_no_mangle,
        has_unsafe,
        memory_patterns,
        line: 0, // Will be filled by caller with actual line number
    })
}

/// Analyze an impl method for WASM boundary markers
fn analyze_impl_method(
    method: &syn::ImplItemFn,
    impl_attrs: &[Attribute],
) -> Option<WasmBoundaryFunction> {
    let is_wasm_bindgen =
        has_attribute(&method.attrs, "wasm_bindgen") || has_attribute(impl_attrs, "wasm_bindgen");
    let is_no_mangle = has_attribute(&method.attrs, "no_mangle");
    let is_extern_c = is_extern_c_fn(&method.sig.abi);

    if !is_wasm_bindgen && !is_no_mangle && !is_extern_c {
        return None;
    }

    let name = method.sig.ident.to_string();
    let has_unsafe = method.sig.unsafety.is_some();
    let memory_patterns = analyze_function_signature(&method.sig);

    Some(WasmBoundaryFunction {
        name,
        is_wasm_bindgen,
        is_extern_c,
        is_no_mangle,
        has_unsafe,
        memory_patterns,
        line: 0,
    })
}

/// Analyze extern block
fn analyze_extern_block(foreign_mod: &syn::ItemForeignMod) -> Option<ExternBlock> {
    let abi = foreign_mod
        .abi
        .name
        .as_ref()
        .map(|lit| lit.value())
        .unwrap_or_else(|| "C".to_string());

    let mut functions = Vec::new();
    for item in &foreign_mod.items {
        if let ForeignItem::Fn(func) = item {
            functions.push(func.sig.ident.to_string());
        }
    }

    Some(ExternBlock {
        abi,
        functions,
        line: 0,
    })
}

/// Check if function signature is extern "C"
fn is_extern_c_fn(abi: &Option<syn::Abi>) -> bool {
    if let Some(abi) = abi {
        if let Some(name) = &abi.name {
            return name.value() == "C";
        }
    }
    false
}

/// Analyze function signature for memory patterns
fn analyze_function_signature(sig: &syn::Signature) -> Vec<MemoryPattern> {
    let mut patterns = Vec::new();

    // Analyze inputs
    for input in &sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            patterns.extend(analyze_type(&pat_type.ty));
        }
    }

    // Analyze output
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        patterns.extend(analyze_type(ty));
    }

    patterns
}

/// Analyze a type for memory patterns
fn analyze_type(ty: &Type) -> Vec<MemoryPattern> {
    let mut patterns = Vec::new();

    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "Box" => patterns.push(MemoryPattern::Box),
                    "Vec" => patterns.push(MemoryPattern::Vec),
                    "String" => patterns.push(MemoryPattern::String),
                    _ => {}
                }

                // Recursively analyze type arguments (e.g., Box<String>)
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            patterns.extend(analyze_type(inner_ty));
                        }
                    }
                }
            }
        }
        Type::Ptr(_) => {
            patterns.push(MemoryPattern::RawPointer);
        }
        Type::Reference(_) => {
            patterns.push(MemoryPattern::Reference);
        }
        _ => {}
    }

    patterns
}

/// Check if attributes contain a specific attribute name
fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|seg| seg.ident == name)
            .unwrap_or(false)
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_detect_wasm_bindgen_function() {
        let code = quote! {
            #[wasm_bindgen]
            /// Add.
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        };

        let func: ItemFn = syn::parse2(code).unwrap();
        let result = analyze_function(&func).unwrap();

        assert!(result.is_wasm_bindgen);
        assert!(!result.is_extern_c);
        assert!(!result.is_no_mangle);
        assert_eq!(result.name, "add");
    }

    #[test]
    fn test_detect_extern_c_function() {
        let code = quote! {
            #[no_mangle]
            pub extern "C" fn ffi_function() -> i32 {
                42
            }
        };

        let func: ItemFn = syn::parse2(code).unwrap();
        let result = analyze_function(&func).unwrap();

        assert!(!result.is_wasm_bindgen);
        assert!(result.is_extern_c);
        assert!(result.is_no_mangle);
    }

    #[test]
    fn test_detect_memory_patterns() {
        let code = quote! {
            #[wasm_bindgen]
            #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
            /// Process data.
            pub fn process_data(input: Vec<u8>) -> Box<String> {
                Box::new(String::from_utf8_lossy(&input).to_string())
            }
        };

        let func: ItemFn = syn::parse2(code).unwrap();
        let result = analyze_function(&func).unwrap();

        assert!(result.memory_patterns.contains(&MemoryPattern::Vec));
        assert!(result.memory_patterns.contains(&MemoryPattern::Box));
        assert!(result.memory_patterns.contains(&MemoryPattern::String));
    }

    #[test]
    fn test_detect_unsafe_function() {
        let code = quote! {
            #[wasm_bindgen]
            pub unsafe fn raw_pointer_access(ptr: *mut u8) -> i32 {
                *ptr as i32
            }
        };

        let func: ItemFn = syn::parse2(code).unwrap();
        let result = analyze_function(&func).unwrap();

        assert!(result.has_unsafe);
        assert!(result.memory_patterns.contains(&MemoryPattern::RawPointer));
    }

    #[test]
    fn test_non_boundary_function_ignored() {
        let code = quote! {
            #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
            /// Regular function.
            pub fn regular_function() -> i32 {
                42
            }
        };

        let func: ItemFn = syn::parse2(code).unwrap();
        let result = analyze_function(&func);

        assert!(result.is_none());
    }

    #[test]
    fn test_extern_block_detection() {
        let code = quote! {
            extern "C" {
                fn external_function(x: i32) -> i32;
                fn another_function();
            }
        };

        let foreign_mod: syn::ItemForeignMod = syn::parse2(code).unwrap();
        let result = analyze_extern_block(&foreign_mod).unwrap();

        assert_eq!(result.abi, "C");
        assert_eq!(result.functions.len(), 2);
        assert!(result.functions.contains(&"external_function".to_string()));
        assert!(result.functions.contains(&"another_function".to_string()));
    }

    #[test]
    fn test_full_file_analysis() {
        let code = quote! {
            #[wasm_bindgen]
            #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
            /// Wasm func.
            pub fn wasm_func() {}

            #[no_mangle]
            pub extern "C" fn c_func() {}

            extern "C" {
                fn external();
            }

            #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
            /// Regular func.
            pub fn regular_func() {}
        };

        let file: syn::File = syn::parse2(code).unwrap();
        let analysis = analyze_wasm_constructs(&file);

        assert_eq!(analysis.boundary_functions.len(), 2);
        assert_eq!(analysis.extern_blocks.len(), 1);
    }

    // analyze_impl_method guard at line 124:
    //   if !is_wasm_bindgen && !is_no_mangle && !is_extern_c { return None; }
    // and the `method_attr || impl_attr` disjunction at line 120.

    fn parse_impl(code: proc_macro2::TokenStream) -> syn::ItemImpl {
        syn::parse2(code).expect("parse impl")
    }

    fn first_method(item_impl: &syn::ItemImpl) -> &syn::ImplItemFn {
        item_impl
            .items
            .iter()
            .find_map(|i| if let syn::ImplItem::Fn(f) = i { Some(f) } else { None })
            .expect("impl has a method")
    }

    #[test]
    fn test_analyze_impl_method_bindgen_on_method() {
        // wasm_bindgen on the method itself → Some, is_wasm_bindgen=true.
        let item_impl = parse_impl(quote! {
            impl Widget {
                #[wasm_bindgen]
                pub fn new() -> Self { Widget }
            }
        });
        let method = first_method(&item_impl);
        let out = analyze_impl_method(method, &item_impl.attrs).expect("bindgen method is a boundary");
        assert!(out.is_wasm_bindgen);
        assert!(!out.is_extern_c);
        assert!(!out.is_no_mangle);
        assert_eq!(out.name, "new");
    }

    #[test]
    fn test_analyze_impl_method_bindgen_on_impl_block() {
        // wasm_bindgen on the impl block (not the method) must still mark the method as a boundary
        // via the RHS of `has_attribute(&method.attrs, "wasm_bindgen") || has_attribute(impl_attrs, "wasm_bindgen")`.
        let item_impl = parse_impl(quote! {
            #[wasm_bindgen]
            impl Widget {
                pub fn bare_method(&self) -> i32 { 0 }
            }
        });
        let method = first_method(&item_impl);
        let out =
            analyze_impl_method(method, &item_impl.attrs).expect("inherited impl-block attr must mark boundary");
        assert!(out.is_wasm_bindgen, "impl-block attr must be inherited");
        assert!(!out.is_no_mangle);
        assert!(!out.is_extern_c);
        assert_eq!(out.name, "bare_method");
    }

    #[test]
    fn test_analyze_impl_method_non_boundary_returns_none() {
        // No wasm_bindgen, no no_mangle, no extern "C" → early return None.
        // Kills the `&&` → `||` mutation at line 124 (would otherwise return Some(..)).
        let item_impl = parse_impl(quote! {
            impl Widget {
                pub fn plain(&self) {}
            }
        });
        let method = first_method(&item_impl);
        assert!(analyze_impl_method(method, &item_impl.attrs).is_none());
    }

    #[test]
    fn test_analyze_impl_method_extern_c_abi() {
        // `extern "C"` on the method's sig.abi alone is enough to make it a boundary.
        let item_impl = parse_impl(quote! {
            impl Widget {
                pub extern "C" fn ffi_call(x: i32) -> i32 { x }
            }
        });
        let method = first_method(&item_impl);
        let out = analyze_impl_method(method, &item_impl.attrs).expect("extern C is a boundary");
        assert!(out.is_extern_c);
        assert!(!out.is_wasm_bindgen);
        assert!(!out.is_no_mangle);
    }

    #[test]
    fn test_analyze_impl_method_no_mangle_only() {
        // #[no_mangle] alone (without extern "C") also flags the method as a boundary.
        let item_impl = parse_impl(quote! {
            impl Widget {
                #[no_mangle]
                pub fn exported(&self) {}
            }
        });
        let method = first_method(&item_impl);
        let out = analyze_impl_method(method, &item_impl.attrs).expect("no_mangle method is a boundary");
        assert!(out.is_no_mangle);
        assert!(!out.is_wasm_bindgen);
        assert!(!out.is_extern_c);
    }

    #[test]
    fn test_full_file_analysis_picks_up_impl_methods() {
        // analyze_wasm_constructs dispatcher at lines 72-80 must collect impl-method boundaries.
        let file: syn::File = syn::parse2(quote! {
            #[wasm_bindgen]
            impl Api {
                pub fn greet() {}
                pub fn farewell() {}
            }

            impl Plain {
                pub fn unrelated(&self) {}
            }
        })
        .unwrap();
        let analysis = analyze_wasm_constructs(&file);
        assert_eq!(
            analysis.boundary_functions.len(),
            2,
            "both methods on #[wasm_bindgen] impl are boundaries; method on plain impl is not"
        );
        let names: Vec<&str> = analysis
            .boundary_functions
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert!(names.contains(&"greet"));
        assert!(names.contains(&"farewell"));
        assert!(!names.contains(&"unrelated"));
    }
}
