// Analysis methods for RustBorrowChecker
// Included by rust_borrow_checker.rs - no `use` imports or `#!` attributes

impl RustBorrowChecker {
    /// Check if an item contains unsafe code
    ///
    /// #953: this used to read `item_fn.sig.unsafety` and nothing else, so an
    /// `unsafe` BLOCK inside a safe `fn` was invisible and
    /// `fn deref_null() -> u8 { let p: *const u8 = std::ptr::null(); unsafe { *p } }`
    /// collected a `MemorySafety` proof whose assumption reads "Safe Rust
    /// subset" over a body performing guaranteed UB. An unmeasured property is
    /// a gap; a property *proven of code that violates it* is a fabricated
    /// artifact, so the whole body is scanned for `unsafe`, not just the
    /// signature.
    #[cfg(feature = "rust-ast")]
    fn contains_unsafe(&self, item_fn: &ItemFn) -> bool {
        item_fn.sig.unsafety.is_some() || block_contains_unsafe(&item_fn.block)
    }

    #[cfg(not(feature = "rust-ast"))]
    fn contains_unsafe(&self, _content: &str) -> bool {
        // Without syn, do a simple text search
        _content.contains("unsafe")
    }

    /// Check if an impl block contains unsafe code
    ///
    /// Like `contains_unsafe`, the `unsafe` keyword on the `impl` itself is not
    /// the only way unsafe code enters the block: a method body may open an
    /// `unsafe` block (#953).
    #[cfg(feature = "rust-ast")]
    fn contains_unsafe_impl(&self, item_impl: &ItemImpl) -> bool {
        item_impl.unsafety.is_some() || impl_contains_unsafe(item_impl)
    }

    /// Analyze thread safety via trait bounds and type analysis
    #[cfg(feature = "rust-ast")]
    fn analyze_thread_safety(
        &self,
        location: &Location,
        item_fn: &ItemFn,
    ) -> Option<ProofAnnotation> {
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
            Some(self.create_thread_safety_annotation(location))
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
        let content = std::fs::read_to_string(file_path).map_err(ProofCollectionError::Io)?;

        let syntax = syn::parse_file(&content).map_err(|e| ProofCollectionError::Parse {
            path: file_path.to_owned(),
            message: format!("Syntax error: {e}"),
        })?;

        let mut annotations = Vec::new();

        // Byte offset of the start of every line, so a located line can be
        // converted back to the byte span `Location` stores.
        let lines: Vec<&str> = content.lines().collect();
        let line_starts = line_start_offsets(&content);
        // `syn` yields items in source order, so the search for each item's
        // declaration only ever moves forward; this both keeps the scan linear
        // and stops an earlier item's text from matching a later one.
        let mut cursor_line = 0usize;

        for item in &syntax.items {
            let span = measure_item_span(item, &lines, &line_starts, &mut cursor_line);
            let item_annotations = self.analyze_item(item, file_path, span);
            annotations.extend(item_annotations);
        }

        Ok(annotations)
    }

    /// Analyze an item and generate proof annotations
    #[cfg(feature = "rust-ast")]
    fn analyze_item(
        &self,
        item: &Item,
        file_path: &Path,
        span: (u32, u32),
    ) -> Vec<(Location, ProofAnnotation)> {
        let mut annotations = Vec::new();
        // MEASURED byte span (#712). Both ends used to be the literals
        // `0` and `100` for every item in every file, so all 2964 annotations
        // this command produced over src/ shared ONE location and the derived
        // annotationId could only tell them apart by property/method.
        let (start, end) = span;

        match item {
            Item::Fn(item_fn) if !self.contains_unsafe(item_fn) => {
                let loc = Location::new(file_path.to_owned(), start, end);

                // Memory safety guarantee for safe functions
                annotations.push((loc.clone(), self.memory_safety_annotation(&loc)));

                // Thread safety analysis
                if let Some(thread_safety) = self.analyze_thread_safety(&loc, item_fn) {
                    annotations.push((loc.clone(), thread_safety));
                }

                // Termination analysis for const fn
                if item_fn.sig.constness.is_some() {
                    let annotation = self.const_fn_termination(&loc);
                    annotations.push((loc, annotation));
                }
            }
            Item::Impl(item_impl) if !self.contains_unsafe_impl(item_impl) => {
                // Analyze impl blocks for trait safety guarantees
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    if self.is_auto_trait(trait_path) {
                        let loc = Location::new(file_path.to_owned(), start, end);
                        let annotation = self.auto_trait_annotation(&loc, trait_path);
                        annotations.push((loc, annotation));
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
        let content = std::fs::read_to_string(file_path).map_err(ProofCollectionError::Io)?;

        let mut annotations = Vec::new();

        // Simple text-based analysis
        if !self.contains_unsafe(&content) {
            // If no unsafe code found, assume memory safety
            let loc = Location::new(file_path.to_owned(), 0, content.len() as u32);
            let annotation = self.memory_safety_annotation(&loc);
            annotations.push((loc, annotation));
        }

        Ok(annotations)
    }
}

/// Finds any `unsafe` reachable from the node it is walked over.
///
/// `unsafe` enters a function four ways and all four must count, because each
/// one is enough to break the "Safe Rust subset" assumption the collected
/// `MemorySafety` annotation carries (#953):
///   * an `unsafe { .. }` expression in the body (including nested closures,
///     `async` blocks, macro-free nested blocks and nested items);
///   * an `unsafe fn` declared inside the body;
///   * an `unsafe impl` or `unsafe trait` declared inside the body;
///   * the `unsafe` on the signature itself, which the callers check directly.
#[cfg(feature = "rust-ast")]
#[derive(Default)]
struct UnsafeFinder {
    found: bool,
}

#[cfg(feature = "rust-ast")]
impl<'ast> syn::visit::Visit<'ast> for UnsafeFinder {
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.found = true;
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_signature(&mut self, node: &'ast syn::Signature) {
        if node.unsafety.is_some() {
            self.found = true;
        }
        syn::visit::visit_signature(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if node.unsafety.is_some() {
            self.found = true;
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if node.unsafety.is_some() {
            self.found = true;
        }
        syn::visit::visit_item_trait(self, node);
    }
}

/// Does this block contain `unsafe` anywhere inside it?
#[cfg(feature = "rust-ast")]
fn block_contains_unsafe(block: &syn::Block) -> bool {
    use syn::visit::Visit;
    let mut finder = UnsafeFinder::default();
    finder.visit_block(block);
    finder.found
}

/// Does this impl block contain `unsafe` anywhere inside it?
#[cfg(feature = "rust-ast")]
fn impl_contains_unsafe(item_impl: &ItemImpl) -> bool {
    use syn::visit::Visit;
    let mut finder = UnsafeFinder::default();
    for item in &item_impl.items {
        finder.visit_impl_item(item);
    }
    finder.found
}

/// Byte offset at which each line of `content` begins.
///
/// `content.lines()` drops the terminators, so the offsets are accumulated from
/// the raw bytes instead of from the split lines.
#[cfg(feature = "rust-ast")]
fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (idx, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(idx + 1);
        }
    }
    offsets
}

/// Measure the byte span of one top-level item.
///
/// #712: this replaces the literals `start = 0` / `end = 100` that every item in
/// every file used to receive ("Would need proper span handling"). Real byte
/// offsets are not available from `syn` here -- `proc_macro2::Span::byte_range`
/// needs the `span-locations` feature, which would add a thread-local source map
/// that grows for the life of the process and pmat parses thousands of files --
/// so the extent is measured from the source text instead: the declaration line
/// is located by its keyword anchor, and the closing line comes from
/// `find_brace_balanced_end`, the codebase's string-aware end-of-block finder
/// (#652, #656).
///
/// `cursor_line` advances past each item found so the scan stays linear and an
/// earlier item cannot match a later one.
#[cfg(feature = "rust-ast")]
fn measure_item_span(
    item: &Item,
    lines: &[&str],
    line_starts: &[usize],
    cursor_line: &mut usize,
) -> (u32, u32) {
    let anchor = match item {
        Item::Fn(item_fn) => format!("fn {}", item_fn.sig.ident),
        Item::Impl(_) => "impl".to_string(),
        _ => return span_of_lines(line_starts, lines, *cursor_line, *cursor_line),
    };

    let Some(start_line) = (*cursor_line..lines.len()).find(|&i| lines[i].contains(&anchor)) else {
        // Anchor not found (for example a macro-shaped declaration). Report the
        // remaining region rather than a fixed constant.
        return span_of_lines(line_starts, lines, *cursor_line, lines.len().saturating_sub(1));
    };

    let end_line = crate::cli::language_analyzer::find_brace_balanced_end(lines, start_line, true);
    // Never let the cursor stall: an item always consumes at least its own line.
    *cursor_line = end_line.max(start_line) + 1;

    span_of_lines(line_starts, lines, start_line, end_line)
}

/// Convert an inclusive line range into a `[start_byte, end_byte)` span.
#[cfg(feature = "rust-ast")]
fn span_of_lines(
    line_starts: &[usize],
    lines: &[&str],
    start_line: usize,
    end_line: usize,
) -> (u32, u32) {
    if line_starts.is_empty() || start_line >= line_starts.len() {
        return (0, 0);
    }
    let end_line = end_line.min(line_starts.len().saturating_sub(1));
    let start = line_starts[start_line];
    let end = line_starts[end_line] + lines.get(end_line).map_or(0, |l| l.len());
    (
        u32::try_from(start).unwrap_or(u32::MAX),
        u32::try_from(end.max(start)).unwrap_or(u32::MAX),
    )
}

/// #953: a `MemorySafety` proof must never be collected over a body that opens
/// an `unsafe` block.
///
/// RED on the old code: `contains_unsafe` read only `item_fn.sig.unsafety`, so
/// every one of these bodies was classified safe and received a `MemorySafety`
/// annotation asserting "Safe Rust subset".
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(all(test, feature = "rust-ast"))]
mod unsafe_detection_tests {
    use super::*;

    /// Property types proven over `source`, in item order.
    fn proven(source: &str) -> Vec<PropertyType> {
        let file = syn::parse_file(source).expect("fixture must parse");
        let checker = RustBorrowChecker::default();
        let path = std::path::PathBuf::from("fixture.rs");
        file.items
            .iter()
            .flat_map(|item| checker.analyze_item(item, &path, (0, 1)))
            .map(|(_, annotation)| annotation.property_proven)
            .collect()
    }

    fn proves_memory_safety(source: &str) -> bool {
        proven(source)
            .iter()
            .any(|p| matches!(p, PropertyType::MemorySafety))
    }

    #[test]
    fn an_unsafe_block_inside_a_safe_fn_defeats_the_memory_safety_proof() {
        assert!(
            !proves_memory_safety(
                "pub fn deref_null() -> u8 { let p: *const u8 = std::ptr::null(); unsafe { *p } }"
            ),
            "a body that dereferences a null pointer inside `unsafe` must not \
             collect a MemorySafety proof asserting the Safe Rust subset"
        );
    }

    #[test]
    fn an_unsafe_block_nested_in_a_closure_or_branch_also_defeats_it() {
        assert!(
            !proves_memory_safety(
                "pub fn nested(flag: bool) -> u8 { \
                 let f = || unsafe { *(std::ptr::null::<u8>()) }; \
                 if flag { f() } else { 0 } }"
            ),
            "unsafe reached through a closure must still be found"
        );
        assert!(
            !proves_memory_safety(
                "pub fn inner_item() { unsafe fn hidden() {} let _ = hidden as usize; }"
            ),
            "an `unsafe fn` declared inside the body must still be found"
        );
    }

    #[test]
    fn a_genuinely_safe_fn_still_collects_the_proof() {
        assert!(
            proves_memory_safety("pub fn add(a: i32) -> i32 { a + 1 }"),
            "the fix must not silence the proof for code that really is in the \
             safe subset — an over-broad rule would make the command useless"
        );
    }

    #[test]
    fn an_unsafe_fn_signature_is_still_rejected() {
        assert!(
            !proves_memory_safety("pub unsafe fn raw(p: *const u8) -> u8 { *p }"),
            "the pre-existing signature check must survive the fix"
        );
    }

    #[test]
    fn an_impl_whose_method_opens_unsafe_is_not_treated_as_unsafe_free() {
        let file = syn::parse_file(
            "struct S; impl S { fn m(&self) -> u8 { unsafe { *(std::ptr::null::<u8>()) } } }",
        )
        .expect("fixture must parse");
        let checker = RustBorrowChecker::default();
        let item_impl = file
            .items
            .iter()
            .find_map(|i| match i {
                Item::Impl(item_impl) => Some(item_impl),
                _ => None,
            })
            .expect("fixture has an impl");
        assert!(
            checker.contains_unsafe_impl(item_impl),
            "an impl block carrying an unsafe method body contains unsafe code"
        );
    }
}
