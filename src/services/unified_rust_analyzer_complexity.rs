/// Per-function complexity collector for the unified (single-parse) analyzer.
///
/// #686: this used to be a private `SimpleComplexityVisitor` marked
/// "GREEN PHASE" that
///   * counted only *top-level* statements, so `pmat context` reported
///     cyclomatic 3 for a function `quality-gate` scored 9;
///   * set `cognitive = cyclomatic` verbatim ("Simplified for GREEN phase"),
///     so every function's two metrics were identical;
///   * emitted `line_start: 0`, `line_end: 0` and a hard-coded `lines: 10`,
///     with the file's own `lines` computed as `functions.len() * 10`.
///
/// It now delegates to `accurate_complexity_analyzer::measure_block` — the same
/// visitor `analyze complexity` and `quality-gate` use — and reads real line
/// spans off the source text.
struct UnifiedComplexityVisitor<'a> {
    functions: Vec<FunctionComplexity>,
    spans: &'a mut FunctionSpans,
}

impl<'a> UnifiedComplexityVisitor<'a> {
    fn new(spans: &'a mut FunctionSpans) -> Self {
        Self {
            functions: Vec::new(),
            spans,
        }
    }

    fn record(&mut self, name: String, block: &syn::Block) {
        let measured = measure_block(&name, block);
        // Spans are consumed in textual order, matching syn's visit order.
        let span = self.spans.take(&name).unwrap_or(LineSpan::UNKNOWN);

        self.functions.push(FunctionComplexity {
            name,
            line_start: span.start,
            line_end: span.end,
            metrics: ComplexityMetrics {
                cyclomatic: narrow_u16(measured.cyclomatic),
                cognitive: narrow_u16(measured.cognitive),
                nesting_max: narrow_u8(measured.max_nesting),
                // 0 when the definition could not be located in the text.
                lines: narrow_u16(span.line_count()),
                halstead: None,
            },
        });
    }
}

impl<'ast> syn::visit::Visit<'ast> for UnifiedComplexityVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.record(node.sig.ident.to_string(), &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.record(node.sig.ident.to_string(), &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }
}

fn narrow_u16(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn narrow_u8(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

impl UnifiedRustAnalyzer {
    /// Extract complexity metrics from the parsed syntax tree.
    ///
    /// `content` is the source text the tree was parsed from; it is required
    /// because line extents are measured, not guessed.
    fn extract_complexity_metrics(
        &self,
        syntax_tree: &syn::File,
        content: &str,
    ) -> FileComplexityMetrics {
        use syn::visit::Visit;

        let mut spans = FunctionSpans::from_source(content);
        let total_lines = spans.total_lines();

        let mut visitor = UnifiedComplexityVisitor::new(&mut spans);
        visitor.visit_file(syntax_tree);
        let functions = visitor.functions;

        let total_cyclomatic: u32 = functions.iter().map(|f| u32::from(f.metrics.cyclomatic)).sum();
        let total_cognitive: u32 = functions.iter().map(|f| u32::from(f.metrics.cognitive)).sum();

        let avg_cyclomatic = if functions.is_empty() {
            1
        } else {
            total_cyclomatic / functions.len() as u32
        };
        let avg_cognitive = if functions.is_empty() {
            0
        } else {
            total_cognitive / functions.len() as u32
        };
        let nesting_max = functions
            .iter()
            .map(|f| f.metrics.nesting_max)
            .max()
            .unwrap_or(0);

        FileComplexityMetrics {
            path: self.file_path.display().to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: narrow_u16(avg_cyclomatic),
                cognitive: narrow_u16(avg_cognitive),
                nesting_max,
                // Real file length; was `functions.len() * 10`.
                lines: narrow_u16(total_lines),
                halstead: None,
            },
            functions,
            classes: Vec::new(), // Rust doesn't have classes
        }
    }
}
