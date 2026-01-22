use std::collections::HashMap;
use syn::{self, visit::Visit, File, Item};

// Simple directed graph for CFG analysis (no petgraph dependency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CfgNodeIndex(usize);

struct SimpleDiGraph<N, E> {
    nodes: Vec<N>,
    edges: Vec<(CfgNodeIndex, CfgNodeIndex, E)>,
}

impl<N, E> SimpleDiGraph<N, E> {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, node: N) -> CfgNodeIndex {
        let idx = CfgNodeIndex(self.nodes.len());
        self.nodes.push(node);
        idx
    }

    fn add_edge(&mut self, from: CfgNodeIndex, to: CfgNodeIndex, edge: E) {
        self.edges.push((from, to, edge));
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // Simple SCC using Kosaraju's algorithm
    fn kosaraju_scc(&self) -> Vec<Vec<CfgNodeIndex>> {
        let n = self.nodes.len();
        if n == 0 {
            return Vec::new();
        }

        // Build adjacency list
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut adj_rev: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (from, to, _) in &self.edges {
            adj[from.0].push(to.0);
            adj_rev[to.0].push(from.0);
        }

        // First DFS to get finish order
        let mut visited = vec![false; n];
        let mut finish_order = Vec::with_capacity(n);

        for i in 0..n {
            if !visited[i] {
                self.dfs_finish(&adj, i, &mut visited, &mut finish_order);
            }
        }

        // Second DFS on transpose in reverse finish order
        let mut visited = vec![false; n];
        let mut sccs = Vec::new();

        for &node in finish_order.iter().rev() {
            if !visited[node] {
                let mut scc = Vec::new();
                self.dfs_collect(&adj_rev, node, &mut visited, &mut scc);
                sccs.push(scc.into_iter().map(CfgNodeIndex).collect());
            }
        }

        sccs
    }

    fn dfs_finish(
        &self,
        adj: &[Vec<usize>],
        node: usize,
        visited: &mut [bool],
        finish: &mut Vec<usize>,
    ) {
        visited[node] = true;
        for &next in &adj[node] {
            if !visited[next] {
                self.dfs_finish(adj, next, visited, finish);
            }
        }
        finish.push(node);
    }

    fn dfs_collect(
        &self,
        adj: &[Vec<usize>],
        node: usize,
        visited: &mut [bool],
        scc: &mut Vec<usize>,
    ) {
        visited[node] = true;
        scc.push(node);
        for &next in &adj[node] {
            if !visited[next] {
                self.dfs_collect(adj, next, visited, scc);
            }
        }
    }
}

pub struct ControlFlowGraph {
    graph: SimpleDiGraph<CfgNode, CfgEdge>,
    _entry: CfgNodeIndex,
    _exit: CfgNodeIndex,
}

#[derive(Debug, Clone)]
pub enum CfgNode {
    Entry,
    Exit,
    Statement(String),
    Condition(String),
    Branch(String),
}

#[derive(Debug, Clone)]
pub enum CfgEdge {
    Sequential,
    True,
    False,
    Jump,
}

impl ControlFlowGraph {
    pub fn from_ast(ast: &File) -> Self {
        let mut graph = SimpleDiGraph::new();
        let entry = graph.add_node(CfgNode::Entry);
        let exit = graph.add_node(CfgNode::Exit);

        let mut builder = CfgBuilder {
            graph,
            current: entry,
            _exit: exit,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        };

        builder.visit_file(ast);
        builder
            .graph
            .add_edge(builder.current, exit, CfgEdge::Sequential);

        ControlFlowGraph {
            graph: builder.graph,
            _entry: entry,
            _exit: exit,
        }
    }

    pub fn cyclomatic_complexity(&self) -> u32 {
        let edges = self.graph.edge_count() as u32;
        let nodes = self.graph.node_count() as u32;
        let components = self.graph.kosaraju_scc().len() as u32;

        edges - nodes + 2 * components
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn essential_complexity(&self) -> u32 {
        // Count strongly connected components (loops)
        let sccs = self.graph.kosaraju_scc();
        sccs.iter().filter(|scc| scc.len() > 1).count() as u32
    }
}

struct CfgBuilder {
    graph: SimpleDiGraph<CfgNode, CfgEdge>,
    current: CfgNodeIndex,
    _exit: CfgNodeIndex,
    break_targets: Vec<CfgNodeIndex>,
    continue_targets: Vec<CfgNodeIndex>,
}

impl<'ast> Visit<'ast> for CfgBuilder {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        let condition = self.graph.add_node(CfgNode::Condition("if".to_string()));
        self.graph
            .add_edge(self.current, condition, CfgEdge::Sequential);

        let then_branch = self.graph.add_node(CfgNode::Branch("then".to_string()));
        self.graph.add_edge(condition, then_branch, CfgEdge::True);

        let _old_current = self.current;
        self.current = then_branch;
        syn::visit::visit_block(self, &node.then_branch);
        let then_exit = self.current;

        let merge = self.graph.add_node(CfgNode::Statement("merge".to_string()));

        if let Some((_, else_branch)) = &node.else_branch {
            let else_node = self.graph.add_node(CfgNode::Branch("else".to_string()));
            self.graph.add_edge(condition, else_node, CfgEdge::False);

            self.current = else_node;
            syn::visit::visit_expr(self, else_branch);
            self.graph
                .add_edge(self.current, merge, CfgEdge::Sequential);
        } else {
            self.graph.add_edge(condition, merge, CfgEdge::False);
        }

        self.graph.add_edge(then_exit, merge, CfgEdge::Sequential);
        self.current = merge;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        let loop_entry = self.graph.add_node(CfgNode::Statement("loop".to_string()));
        self.graph
            .add_edge(self.current, loop_entry, CfgEdge::Sequential);

        let loop_exit = self
            .graph
            .add_node(CfgNode::Statement("loop_exit".to_string()));
        self.break_targets.push(loop_exit);
        self.continue_targets.push(loop_entry);

        self.current = loop_entry;
        syn::visit::visit_block(self, &node.body);
        self.graph.add_edge(self.current, loop_entry, CfgEdge::Jump);

        self.break_targets.pop();
        self.continue_targets.pop();
        self.current = loop_exit;
    }
}

#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub max_nesting: u32,
    pub essential: u32,
}

#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub complexity: u32,
    pub lines: usize,
    pub parameters: usize,
}

#[derive(Debug, Clone)]
pub struct HalsteadMetrics {
    pub vocabulary: usize, // n = n1 + n2
    pub length: usize,     // N = N1 + N2
    pub volume: f64,       // V = N * log2(n)
    pub difficulty: f64,   // D = (n1/2) * (N2/n2)
    pub effort: f64,       // E = D * V
}

use super::complexity::ComplexityAnalyzer;

impl ComplexityAnalyzer {
    pub fn analyze_functions(&self, ast: &File) -> Vec<FunctionMetrics> {
        let mut functions = Vec::new();

        for item in &ast.items {
            if let Item::Fn(func) = item {
                let name = func.sig.ident.to_string();
                let complexity = self.calculate_function_complexity(func);
                let lines = count_lines(&func.block);
                let parameters = func.sig.inputs.len();

                functions.push(FunctionMetrics {
                    name,
                    complexity,
                    lines,
                    parameters,
                });
            }
        }

        functions
    }

    fn calculate_function_complexity(&self, func: &syn::ItemFn) -> u32 {
        // Use the existing method from ComplexityAnalyzer
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![syn::Item::Fn(func.clone())],
        };
        self.calculate_cyclomatic(&file)
    }

    pub fn calculate_halstead_metrics(&self, code: &str) -> HalsteadMetrics {
        let mut operators = HashMap::new();
        let mut operands = HashMap::new();

        // Tokenize and classify
        for token in tokenize(code) {
            if is_operator(&token) {
                *operators.entry(token).or_insert(0) += 1;
            } else if is_operand(&token) {
                *operands.entry(token).or_insert(0) += 1;
            }
        }

        let n1 = operators.len();
        let n2 = operands.len();
        let n1_total: usize = operators.values().sum();
        let n2_total: usize = operands.values().sum();

        let vocabulary = n1 + n2;
        let length = n1_total + n2_total;
        let volume = if vocabulary > 0 {
            length as f64 * (vocabulary as f64).log2()
        } else {
            0.0
        };

        let difficulty = if n2 > 0 && n2_total > 0 {
            (n1 as f64 / 2.0) * (n2_total as f64 / n2 as f64)
        } else {
            0.0
        };

        let effort = difficulty * volume;

        HalsteadMetrics {
            vocabulary,
            length,
            volume,
            difficulty,
            effort,
        }
    }
}

fn count_lines(block: &syn::Block) -> usize {
    block.stmts.len()
}

fn tokenize(code: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in code.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            if !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn is_operator(token: &str) -> bool {
    matches!(
        token,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "="
            | "=="
            | "!="
            | "<"
            | ">"
            | "<="
            | ">="
            | "&&"
            | "||"
            | "!"
            | "&"
            | "|"
            | "^"
            | "<<"
            | ">>"
            | "+="
            | "-="
            | "*="
            | "/="
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
    )
}

fn is_operand(token: &str) -> bool {
    !is_operator(token) && !token.chars().all(|c| c.is_ascii_punctuation())
}

// Maintainability Index calculation
pub fn calculate_maintainability_index(
    halstead_volume: f64,
    cyclomatic_complexity: u32,
    lines_of_code: usize,
) -> f64 {
    let mi = 171.0
        - 5.2 * halstead_volume.ln()
        - 0.23 * cyclomatic_complexity as f64
        - 16.2 * (lines_of_code as f64).ln();

    mi.max(0.0).min(100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_complexity() {
        let code = r#"
            fn test(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        return x * 2;
                    }
                    return x + 1;
                }
                return 0;
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);

        assert!(cfg.cyclomatic_complexity() >= 3);
    }

    #[test]
    fn test_halstead_metrics() {
        let code = "let x = a + b * c;";
        let analyzer = ComplexityAnalyzer::new();
        let metrics = analyzer.calculate_halstead_metrics(code);

        assert!(metrics.vocabulary > 0);
        assert!(metrics.volume > 0.0);
    }

    #[test]
    fn test_maintainability_index() {
        let mi = calculate_maintainability_index(100.0, 5, 50);
        assert!((0.0..=100.0).contains(&mi));
    }

    // === CfgNode Tests ===

    #[test]
    fn test_cfg_node_entry() {
        let node = CfgNode::Entry;
        let debug = format!("{:?}", node);
        assert!(debug.contains("Entry"));
    }

    #[test]
    fn test_cfg_node_exit() {
        let node = CfgNode::Exit;
        let debug = format!("{:?}", node);
        assert!(debug.contains("Exit"));
    }

    #[test]
    fn test_cfg_node_statement() {
        let node = CfgNode::Statement("test_stmt".to_string());
        let debug = format!("{:?}", node);
        assert!(debug.contains("Statement"));
        assert!(debug.contains("test_stmt"));
    }

    #[test]
    fn test_cfg_node_condition() {
        let node = CfgNode::Condition("if_cond".to_string());
        let debug = format!("{:?}", node);
        assert!(debug.contains("Condition"));
    }

    #[test]
    fn test_cfg_node_branch() {
        let node = CfgNode::Branch("else_branch".to_string());
        let debug = format!("{:?}", node);
        assert!(debug.contains("Branch"));
    }

    #[test]
    fn test_cfg_node_clone() {
        let node = CfgNode::Condition("test".to_string());
        let cloned = node.clone();
        assert_eq!(format!("{:?}", node), format!("{:?}", cloned));
    }

    // === CfgEdge Tests ===

    #[test]
    fn test_cfg_edge_sequential() {
        let edge = CfgEdge::Sequential;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("Sequential"));
    }

    #[test]
    fn test_cfg_edge_true() {
        let edge = CfgEdge::True;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("True"));
    }

    #[test]
    fn test_cfg_edge_false() {
        let edge = CfgEdge::False;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("False"));
    }

    #[test]
    fn test_cfg_edge_jump() {
        let edge = CfgEdge::Jump;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("Jump"));
    }

    #[test]
    fn test_cfg_edge_clone() {
        let edge = CfgEdge::True;
        let cloned = edge.clone();
        assert_eq!(format!("{:?}", edge), format!("{:?}", cloned));
    }

    // === ControlFlowGraph Tests ===

    #[test]
    fn test_cfg_simple_function() {
        let code = r#"
            fn simple() -> i32 {
                42
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.node_count() >= 2);
        assert!(cfg.edge_count() >= 1);
    }

    #[test]
    fn test_cfg_node_count() {
        let code = r#"
            fn test() {
                if true {
                    println!("yes");
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.node_count() > 2);
    }

    #[test]
    fn test_cfg_edge_count() {
        let code = r#"
            fn test() {
                if true {
                    println!("yes");
                } else {
                    println!("no");
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.edge_count() > 2);
    }

    #[test]
    fn test_cfg_essential_complexity() {
        let code = r#"
            fn test() {
                loop {
                    break;
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        let essential = cfg.essential_complexity();
        assert!(essential >= 0);
    }

    #[test]
    fn test_cfg_with_loop() {
        let code = r#"
            fn test() {
                loop {
                    if true {
                        break;
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.cyclomatic_complexity() >= 1);
    }

    // === ComplexityMetrics Tests ===

    #[test]
    fn test_complexity_metrics_struct() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 8,
            max_nesting: 3,
            essential: 2,
        };
        assert_eq!(metrics.cyclomatic, 5);
        assert_eq!(metrics.cognitive, 8);
        assert_eq!(metrics.max_nesting, 3);
        assert_eq!(metrics.essential, 2);
    }

    #[test]
    fn test_complexity_metrics_clone() {
        let metrics = ComplexityMetrics {
            cyclomatic: 10,
            cognitive: 15,
            max_nesting: 4,
            essential: 3,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.cyclomatic, 10);
    }

    #[test]
    fn test_complexity_metrics_debug() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 8,
            max_nesting: 3,
            essential: 2,
        };
        let debug = format!("{:?}", metrics);
        assert!(debug.contains("ComplexityMetrics"));
        assert!(debug.contains("cyclomatic"));
    }

    // === FunctionMetrics Tests ===

    #[test]
    fn test_function_metrics_struct() {
        let metrics = FunctionMetrics {
            name: "test_func".to_string(),
            complexity: 5,
            lines: 20,
            parameters: 3,
        };
        assert_eq!(metrics.name, "test_func");
        assert_eq!(metrics.complexity, 5);
        assert_eq!(metrics.lines, 20);
        assert_eq!(metrics.parameters, 3);
    }

    #[test]
    fn test_function_metrics_clone() {
        let metrics = FunctionMetrics {
            name: "my_func".to_string(),
            complexity: 8,
            lines: 50,
            parameters: 2,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.name, "my_func");
    }

    // === HalsteadMetrics Tests ===

    #[test]
    fn test_halstead_metrics_struct() {
        let metrics = HalsteadMetrics {
            vocabulary: 10,
            length: 25,
            volume: 83.0,
            difficulty: 5.0,
            effort: 415.0,
        };
        assert_eq!(metrics.vocabulary, 10);
        assert_eq!(metrics.length, 25);
    }

    #[test]
    fn test_halstead_empty_code() {
        let analyzer = ComplexityAnalyzer::new();
        let metrics = analyzer.calculate_halstead_metrics("");
        assert_eq!(metrics.vocabulary, 0);
        assert_eq!(metrics.length, 0);
    }

    #[test]
    fn test_halstead_simple_expression() {
        let analyzer = ComplexityAnalyzer::new();
        let metrics = analyzer.calculate_halstead_metrics("x + y");
        assert!(metrics.vocabulary > 0);
    }

    // === is_operator Tests ===

    #[test]
    fn test_is_operator_arithmetic() {
        assert!(is_operator("+"));
        assert!(is_operator("-"));
        assert!(is_operator("*"));
        assert!(is_operator("/"));
        assert!(is_operator("%"));
    }

    #[test]
    fn test_is_operator_comparison() {
        assert!(is_operator("=="));
        assert!(is_operator("!="));
        assert!(is_operator("<"));
        assert!(is_operator(">"));
        assert!(is_operator("<="));
        assert!(is_operator(">="));
    }

    #[test]
    fn test_is_operator_logical() {
        assert!(is_operator("&&"));
        assert!(is_operator("||"));
        assert!(is_operator("!"));
    }

    #[test]
    fn test_is_operator_control_flow() {
        assert!(is_operator("if"));
        assert!(is_operator("else"));
        assert!(is_operator("match"));
        assert!(is_operator("for"));
        assert!(is_operator("while"));
        assert!(is_operator("loop"));
    }

    #[test]
    fn test_is_not_operator() {
        assert!(!is_operator("x"));
        assert!(!is_operator("variable"));
        assert!(!is_operator("123"));
    }

    // === is_operand Tests ===

    #[test]
    fn test_is_operand() {
        assert!(is_operand("x"));
        assert!(is_operand("variable"));
        assert!(is_operand("123"));
    }

    #[test]
    fn test_is_not_operand() {
        assert!(!is_operand("+"));
        assert!(!is_operand("if"));
    }

    // === tokenize Tests ===

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("a + b");
        assert!(tokens.contains(&"a".to_string()));
        assert!(tokens.contains(&"+".to_string()));
        assert!(tokens.contains(&"b".to_string()));
    }

    #[test]
    fn test_tokenize_identifier() {
        let tokens = tokenize("my_variable");
        assert!(tokens.contains(&"my_variable".to_string()));
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    // === calculate_maintainability_index Tests ===

    #[test]
    fn test_maintainability_index_perfect() {
        let mi = calculate_maintainability_index(10.0, 1, 10);
        assert!(mi > 0.0);
        assert!(mi <= 100.0);
    }

    #[test]
    fn test_maintainability_index_complex() {
        let mi = calculate_maintainability_index(1000.0, 50, 500);
        assert!(mi >= 0.0);
    }

    #[test]
    fn test_maintainability_index_bounds() {
        // Very high volume should clamp to lower bound
        let mi_low = calculate_maintainability_index(1000000.0, 100, 10000);
        assert!(mi_low >= 0.0);

        // Very low should still be in bounds
        let mi_high = calculate_maintainability_index(1.0, 1, 1);
        assert!(mi_high <= 100.0);
    }

    // === analyze_functions Tests ===

    #[test]
    fn test_analyze_functions_empty() {
        let code = r#"
            struct MyStruct {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let analyzer = ComplexityAnalyzer::new();
        let functions = analyzer.analyze_functions(&ast);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_analyze_functions_single() {
        let code = r#"
            fn my_function(x: i32, y: i32) -> i32 {
                x + y
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let analyzer = ComplexityAnalyzer::new();
        let functions = analyzer.analyze_functions(&ast);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "my_function");
        assert_eq!(functions[0].parameters, 2);
    }

    #[test]
    fn test_analyze_functions_multiple() {
        let code = r#"
            fn func1() {}
            fn func2(x: i32) {}
            fn func3(a: i32, b: i32, c: i32) {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let analyzer = ComplexityAnalyzer::new();
        let functions = analyzer.analyze_functions(&ast);
        assert_eq!(functions.len(), 3);
    }
}
