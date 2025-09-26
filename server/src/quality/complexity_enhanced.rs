use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use syn::{self, visit::Visit, File, Item};

pub struct ControlFlowGraph {
    graph: DiGraph<CfgNode, CfgEdge>,
    _entry: NodeIndex,
    _exit: NodeIndex,
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
        let mut graph = DiGraph::new();
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
        let components = kosaraju_scc(&self.graph).len() as u32;

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
        let sccs = kosaraju_scc(&self.graph);
        sccs.iter().filter(|scc| scc.len() > 1).count() as u32
    }
}

struct CfgBuilder {
    graph: DiGraph<CfgNode, CfgEdge>,
    current: NodeIndex,
    _exit: NodeIndex,
    break_targets: Vec<NodeIndex>,
    continue_targets: Vec<NodeIndex>,
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
}
