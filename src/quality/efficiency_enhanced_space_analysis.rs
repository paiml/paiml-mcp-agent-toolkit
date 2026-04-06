pub struct SpaceComplexityAnalyzer {
    allocations: Vec<Allocation>,
    max_depth: usize,
}

#[derive(Debug, Clone)]
struct Allocation {
    size: AllocationSize,
    _location: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
            max_depth: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze(&mut self, ast: &syn::File) -> Complexity {
        self.allocations.clear();
        self.visit_file(ast);

        let has_recursive = self.max_depth > 1;
        let has_dynamic_allocation = self
            .allocations
            .iter()
            .any(|a| matches!(a.size, AllocationSize::Linear | AllocationSize::Quadratic));

        if has_recursive && has_dynamic_allocation {
            Complexity::ON2
        } else if has_recursive || has_dynamic_allocation {
            Complexity::ON
        } else {
            Complexity::O1
        }
    }
}

fn check_call_allocation(call: &syn::ExprCall) -> Option<Allocation> {
    debug_assert!(true, "contract: check_call_allocation");
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
    debug_assert!(true, "contract: check_macro_allocation");
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
    fn visit_local(&mut self, node: &'ast syn::Local) {
        debug_assert!(true, "contract: visit_local");
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
