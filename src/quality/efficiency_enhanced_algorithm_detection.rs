// Algorithm pattern detection for SymbolicExecutor.
// Separated to keep per-file cognitive complexity under threshold.

impl SymbolicExecutor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_algorithm_patterns(&self, ast: &syn::File) -> Vec<AlgorithmPattern> {
        let mut patterns = Vec::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                if self.is_sorting_algorithm(func) {
                    patterns.push(AlgorithmPattern::Sorting);
                }
                if self.is_search_algorithm(func) {
                    patterns.push(AlgorithmPattern::Search);
                }
                if self.is_graph_algorithm(func) {
                    patterns.push(AlgorithmPattern::Graph);
                }
                if self.is_dynamic_programming(func) {
                    patterns.push(AlgorithmPattern::DynamicProgramming);
                }
            }
        }

        patterns
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn is_dynamic_programming(&self, func: &syn::ItemFn) -> bool {
        for stmt in &func.block.stmts {
            if is_dp_cache_statement(stmt) {
                return true;
            }
        }
        false
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn is_dp_cache_statement(stmt: &Stmt) -> bool {
    let Stmt::Local(local) = stmt else {
        return false;
    };
    let Some(local_init) = &local.init else {
        return false;
    };
    match &*local_init.expr {
        syn::Expr::Call(call) => is_cache_constructor_call(call),
        syn::Expr::Macro(mac) => is_cache_macro(mac),
        _ => false,
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn is_cache_constructor_call(call: &syn::ExprCall) -> bool {
    if let syn::Expr::Path(path) = &*call.func {
        let path_str = path_to_string(&path.path);
        path_str.contains("HashMap") || path_str.contains("BTreeMap")
    } else {
        false
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn is_cache_macro(mac: &syn::ExprMacro) -> bool {
    let mac_name = mac
        .mac
        .path
        .segments
        .last()
        .map(|seg| seg.ident.to_string())
        .unwrap_or_default();
    mac_name.contains("hashmap") || mac_name.contains("cache")
}
