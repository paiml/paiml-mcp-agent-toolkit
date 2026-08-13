/// Space complexity analyzer.
pub struct SpaceComplexityAnalyzer {
    allocations: Vec<Allocation>,
    /// Number of self-recursive functions seen in the file being analyzed.
    /// Recursion costs stack frames proportional to the recursion depth, so it
    /// is a space signal in its own right. (Replaces the former `max_depth`
    /// field, which was only ever assigned `0` and therefore could never make
    /// `has_recursive` true.)
    recursive_functions: usize,
}

#[derive(Debug, Clone)]
struct Allocation {
    size: AllocationSize,
    _location: String,
}

#[derive(Debug, Clone)]
enum AllocationSize {
    Constant(usize),
    Linear,
    Quadratic,
    Dynamic,
}

impl Default for SpaceComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceComplexityAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
            recursive_functions: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Analyze.
    pub fn analyze(&mut self, ast: &syn::File) -> Complexity {
        self.allocations.clear();
        self.recursive_functions = 0;
        self.visit_file(ast);

        let has_recursive = self.recursive_functions > 0;
        // `Dynamic` must be counted here: it is the size class the visitor
        // actually produces for `Vec`/`String`/`vec![]`, i.e. an allocation
        // whose size is not statically bounded. Omitting it (the predicate
        // used to match only `Linear | Quadratic`, which nothing constructs)
        // made this whole function a constant that returned `O1` for every
        // input — measurement collected, then thrown away.
        let has_dynamic_allocation = self.allocations.iter().any(|a| {
            matches!(
                a.size,
                AllocationSize::Linear | AllocationSize::Quadratic | AllocationSize::Dynamic
            )
        });

        if has_recursive && has_dynamic_allocation {
            Complexity::ON2
        } else if has_recursive || has_dynamic_allocation {
            Complexity::ON
        } else {
            Complexity::O1
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn check_call_allocation(call: &syn::ExprCall) -> Option<Allocation> {
    if let syn::Expr::Path(path) = &*call.func {
        let path_str = path_to_string(&path.path);
        if path_str.contains("Vec") || path_str.contains("String") {
            return Some(Allocation {
                size: AllocationSize::Dynamic,
                _location: "vec/string".to_string(),
            });
        }
    }
    None
}

fn check_macro_allocation(mac: &syn::ExprMacro) -> Option<Allocation> {
    let mac_name = mac
        .mac
        .path
        .segments
        .last()
        .map(|seg| seg.ident.to_string())
        .unwrap_or_default();

    if mac_name == "vec" || mac_name.contains("string") {
        Some(Allocation {
            size: AllocationSize::Dynamic,
            _location: "macro".to_string(),
        })
    } else {
        None
    }
}

impl<'ast> Visit<'ast> for SpaceComplexityAnalyzer {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Same recursion rule `SymbolicExecutor::analyze_function` uses — one
        // detector, not a second hand-rolled one.
        let mut detector = RecursionDetector {
            function_name: node.sig.ident.to_string(),
            is_recursive: false,
        };
        detector.visit_block(&node.block);
        if detector.is_recursive {
            self.recursive_functions += 1;
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(local_init) = &node.init {
            match &*local_init.expr {
                syn::Expr::Array(_) => {
                    self.allocations.push(Allocation {
                        size: AllocationSize::Constant(1),
                        _location: "array".to_string(),
                    });
                }
                syn::Expr::Call(call) => {
                    if let Some(alloc) = check_call_allocation(call) {
                        self.allocations.push(alloc);
                    }
                }
                syn::Expr::Macro(mac) => {
                    if let Some(alloc) = check_macro_allocation(mac) {
                        self.allocations.push(alloc);
                    }
                }
                _ => {}
            }
        }

        syn::visit::visit_local(self, node);
    }
}
