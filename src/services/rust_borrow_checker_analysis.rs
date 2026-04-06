// Analysis methods for RustBorrowChecker
// Included by rust_borrow_checker.rs - no `use` imports or `#!` attributes

impl RustBorrowChecker {
    /// Check if an item contains unsafe code
    #[cfg(feature = "rust-ast")]
    fn contains_unsafe(&self, item_fn: &ItemFn) -> bool {
        // Simple check for unsafe keyword
        item_fn.sig.unsafety.is_some()
    }

    #[cfg(not(feature = "rust-ast"))]
    fn contains_unsafe(&self, _content: &str) -> bool {
        debug_assert!(!_content.is_empty(), "_content must not be empty");
        // Without syn, do a simple text search
        _content.contains("unsafe")
    }

    /// Check if an impl block contains unsafe code
    #[cfg(feature = "rust-ast")]
    fn contains_unsafe_impl(&self, item_impl: &ItemImpl) -> bool {
        item_impl.unsafety.is_some()
    }

    /// Analyze thread safety via trait bounds and type analysis
    #[cfg(feature = "rust-ast")]
    fn analyze_thread_safety(&self, item_fn: &ItemFn) -> Option<ProofAnnotation> {
        // Conservative analysis: only if all parameters appear to be Send+Sync
        let params_likely_send_sync = item_fn.sig.inputs.iter().all(|arg| {
            match arg {
                syn::FnArg::Typed(pat_type) => {
                    // Simple heuristic: check if the type looks like it implements Send+Sync
                    self.type_likely_implements_send_sync(&pat_type.ty)
                }
                _ => true, // self is Send+Sync if the type is
            }
        });

        if params_likely_send_sync {
            Some(self.create_thread_safety_annotation())
        } else {
            None
        }
    }

    /// Simple heuristic to check if a type likely implements Send+Sync
    #[cfg(all(feature = "rust-ast", feature = "quote"))]
    fn type_likely_implements_send_sync(&self, ty: &Type) -> bool {
        match ty {
            Type::Path(path) => {
                let path_str = quote::quote!(#path).to_string();
                // Common Send+Sync types
                matches!(
                    path_str.as_str(),
                    "String"
                        | "i32"
                        | "u32"
                        | "i64"
                        | "u64"
                        | "f32"
                        | "f64"
                        | "bool"
                        | "char"
                        | "usize"
                        | "isize"
                        | "Vec"
                        | "HashMap"
                        | "BTreeMap"
                        | "Arc"
                        | "Mutex"
                        | "RwLock"
                )
            }
            Type::Reference(_) => true, // &T is Send+Sync if T is
            _ => false,                 // Conservative default
        }
    }

    /// Fallback implementation without quote
    #[cfg(all(feature = "rust-ast", not(feature = "quote")))]
    fn type_likely_implements_send_sync(&self, _ty: &Type) -> bool {
        // Conservative default when we can't analyze the type
        false
    }

    /// Check if a trait path is an auto trait (Send, Sync, etc.)
    #[cfg(all(feature = "rust-ast", feature = "quote"))]
    fn is_auto_trait(&self, trait_path: &syn::Path) -> bool {
        let path_str = quote::quote!(#trait_path).to_string();
        matches!(
            path_str.as_str(),
            "Send" | "Sync" | "Unpin" | "UnwindSafe" | "RefUnwindSafe"
        )
    }

    /// Fallback implementation without quote
    #[cfg(all(feature = "rust-ast", not(feature = "quote")))]
    fn is_auto_trait(&self, trait_path: &syn::Path) -> bool {
        // Simple check based on the last segment
        if let Some(segment) = trait_path.segments.last() {
            matches!(
                segment.ident.to_string().as_str(),
                "Send" | "Sync" | "Unpin" | "UnwindSafe" | "RefUnwindSafe"
            )
        } else {
            false
        }
    }

    /// Analyze a Rust file and extract proof annotations
    #[cfg(feature = "rust-ast")]
    fn analyze_rust_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<(Location, ProofAnnotation)>, ProofCollectionError> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let content = std::fs::read_to_string(file_path).map_err(ProofCollectionError::Io)?;

        let syntax = syn::parse_file(&content).map_err(|e| ProofCollectionError::Parse {
            path: file_path.to_owned(),
            message: format!("Syntax error: {e}"),
        })?;

        let mut annotations = Vec::new();

        for item in &syntax.items {
            let item_annotations = self.analyze_item(item, file_path);
            annotations.extend(item_annotations);
        }

        Ok(annotations)
    }

    /// Analyze an item and generate proof annotations
    #[cfg(feature = "rust-ast")]
    fn analyze_item(&self, item: &Item, file_path: &Path) -> Vec<(Location, ProofAnnotation)> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let mut annotations = Vec::new();

        match item {
            Item::Fn(item_fn) if !self.contains_unsafe(item_fn) => {
                // Extract location from span (simplified)
                let start = 0u32; // Would need proper span handling
                let end = 100u32;
                let loc = Location::new(file_path.to_owned(), start, end);

                // Memory safety guarantee for safe functions
                annotations.push((loc.clone(), self.memory_safety_annotation()));

                // Thread safety analysis
                if let Some(thread_safety) = self.analyze_thread_safety(item_fn) {
                    annotations.push((loc.clone(), thread_safety));
                }

                // Termination analysis for const fn
                if item_fn.sig.constness.is_some() {
                    annotations.push((loc, self.const_fn_termination()));
                }
            }
            Item::Impl(item_impl) if !self.contains_unsafe_impl(item_impl) => {
                // Analyze impl blocks for trait safety guarantees
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    if self.is_auto_trait(trait_path) {
                        let start = 0u32;
                        let end = 100u32;
                        let loc = Location::new(file_path.to_owned(), start, end);
                        annotations.push((loc, self.auto_trait_annotation(trait_path)));
                    }
                }
            }
            _ => {}
        }

        annotations
    }

    /// Analyze a Rust file without syn (fallback)
    #[cfg(not(feature = "rust-ast"))]
    fn analyze_rust_file_simple(
        &self,
        file_path: &Path,
    ) -> Result<Vec<(Location, ProofAnnotation)>, ProofCollectionError> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let content = std::fs::read_to_string(file_path).map_err(ProofCollectionError::Io)?;

        let mut annotations = Vec::new();

        // Simple text-based analysis
        if !self.contains_unsafe(&content) {
            // If no unsafe code found, assume memory safety
            let loc = Location::new(file_path.to_owned(), 0, content.len() as u32);
            annotations.push((loc, self.memory_safety_annotation()));
        }

        Ok(annotations)
    }
}
