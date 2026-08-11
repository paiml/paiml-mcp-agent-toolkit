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
