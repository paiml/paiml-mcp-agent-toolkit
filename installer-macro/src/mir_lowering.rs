#![allow(dead_code)]

use crate::shell_ast::{Expression, InterpolationPart, ShellAst, Statement};
use indexmap::IndexMap;
use std::collections::BTreeMap;
use syn::{Block, Expr, ExprIf, ExprMatch, ExprMethodCall, ItemFn, Lit, Pat, Stmt};

pub fn analyze_function(func: &ItemFn) -> ShellAst {
    let mut context = LoweringContext::new();

    // Process function body
    let block = &func.block;
    context.lower_block(block);

    // Ensure deterministic ordering
    context.finalize()
}

struct LoweringContext {
    statements: Vec<Statement>,
    variables: IndexMap<String, usize>, // Preserve declaration order
    string_pool: BTreeMap<String, usize>,
    next_var_id: usize,
    next_string_id: usize,
}

impl LoweringContext {
    fn new() -> Self {
        Self {
            statements: Vec::new(),
            variables: IndexMap::new(),
            string_pool: BTreeMap::new(),
            next_var_id: 0,
            next_string_id: 0,
        }
    }

    fn lower_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.lower_statement(stmt);
        }
    }

    fn lower_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    let var_name = self.extract_identifier(&local.pat);
                    let var_id = self.allocate_variable(&var_name);
                    let value = self.lower_expression(&init.expr);

                    self.statements.push(Statement::LocalAssignment {
                        var: format!("_v{}", var_id),
                        value,
                    });
                }
            }
            Stmt::Expr(expr, _) => {
                self.lower_expression_statement(expr);
            }
            Stmt::Macro(stmt_macro) => {
                // Skip macro statements for now
                let _ = stmt_macro;
            }
            _ => {}
        }
    }

    fn lower_expression_statement(&mut self, expr: &Expr) {
        match expr {
            Expr::MethodCall(method_call) => {
                self.lower_method_call_statement(method_call);
            }
            Expr::If(expr_if) => {
                self.lower_if_statement(expr_if);
            }
            Expr::Match(expr_match) => {
                self.lower_match_statement(expr_match);
            }
            Expr::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    let _ = expr;
                    // Handle error returns
                    self.statements.push(Statement::Exit { code: 1 });
                } else {
                    self.statements.push(Statement::Return { code: 0 });
                }
            }
            _ => {}
        }
    }

    fn lower_if_statement(&mut self, expr_if: &ExprIf) {
        let condition = self.lower_condition(&expr_if.cond);
        let then_block = self.lower_block_to_statements(&expr_if.then_branch);

        let else_block = if let Some((_, else_expr)) = &expr_if.else_branch {
            match else_expr.as_ref() {
                Expr::Block(block) => Some(self.lower_block_to_statements(&block.block)),
                _ => None,
            }
        } else {
            None
        };

        self.statements.push(Statement::If {
            condition,
            then_block,
            else_block,
        });
    }

    fn lower_match_statement(&mut self, expr_match: &ExprMatch) {
        let expr = self.lower_expression(&expr_match.expr);
        let mut patterns = Vec::new();

        for arm in &expr_match.arms {
            if let Pat::Lit(syn::PatLit {
                lit: Lit::Str(s), ..
            }) = &arm.pat
            {
                let pattern = s.value();
                let body = match arm.body.as_ref() {
                    Expr::Block(block) => self.lower_block_to_statements(&block.block),
                    _ => vec![],
                };
                patterns.push((pattern, body));
            }
        }

        self.statements.push(Statement::Case { expr, patterns });
    }

    fn lower_block_to_statements(&mut self, block: &Block) -> Vec<Statement> {
        let mut sub_context = LoweringContext::new();
        sub_context.variables = self.variables.clone();
        sub_context.string_pool = self.string_pool.clone();
        sub_context.next_var_id = self.next_var_id;
        sub_context.next_string_id = self.next_string_id;

        sub_context.lower_block(block);

        self.variables = sub_context.variables;
        self.string_pool = sub_context.string_pool;
        self.next_var_id = sub_context.next_var_id;
        self.next_string_id = sub_context.next_string_id;

        sub_context.statements
    }

    fn lower_condition(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::MethodCall(method) => {
                let receiver = &method.receiver;
                let _ = receiver;
                let method_name = method.method.to_string();

                match method_name.as_str() {
                    "test_dir" => {
                        if let Some(arg) = method.args.first() {
                            let path = self.extract_string_literal(arg);
                            format!("[ -d \"{}\" ]", path)
                        } else {
                            "false".to_string()
                        }
                    }
                    _ => "true".to_string(),
                }
            }
            Expr::Unary(unary) => {
                if matches!(unary.op, syn::UnOp::Not(_)) {
                    let inner = self.lower_condition(&unary.expr);
                    format!("! {}", inner)
                } else {
                    "true".to_string()
                }
            }
            _ => "true".to_string(),
        }
    }

    fn lower_method_call_statement(&mut self, method_call: &ExprMethodCall) {
        let receiver_type = self.infer_receiver_type(&method_call.receiver);
        let method_name = method_call.method.to_string();

        if let ("ShellContext", "command") = (receiver_type.as_str(), method_name.as_str()) {
            if let (Some(cmd_expr), Some(args_expr)) =
                (method_call.args.first(), method_call.args.get(1))
            {
                let cmd = self.extract_string_literal(cmd_expr);
                let args = self.extract_string_array(args_expr);

                // Special handling for certain commands
                match cmd.as_str() {
                    "trap" => {
                        if let Some(trap_cmd) = args.first() {
                            self.statements.push(Statement::SetTrap {
                                command: trap_cmd.clone(),
                                signals: args.into_iter().skip(1).collect(),
                            });
                        }
                    }
                    "readonly" => {
                        if let Some(assignment) = args.first() {
                            if let Some((var, value)) = assignment.split_once('=') {
                                self.statements.push(Statement::Assignment {
                                    var: var.to_string(),
                                    value: Expression::Literal(value.to_string()),
                                });
                            }
                        }
                    }
                    _ => {
                        self.statements.push(Statement::Command {
                            cmd,
                            args: args.into_iter().map(Expression::Literal).collect(),
                        });
                    }
                }
            }
        }
    }

    fn lower_expression(&mut self, expr: &Expr) -> Expression {
        match expr {
            Expr::Lit(expr_lit) => {
                if let Lit::Str(s) = &expr_lit.lit {
                    Expression::Literal(s.value())
                } else {
                    Expression::Literal("".to_string())
                }
            }
            Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();
                    if let Some(var_id) = self.variables.get(&var_name) {
                        Expression::Variable(format!("_v{}", var_id))
                    } else {
                        Expression::Literal(var_name)
                    }
                } else {
                    Expression::Literal("".to_string())
                }
            }
            Expr::MethodCall(method) => self.lower_method_call_expression(method),
            Expr::Macro(mac) => {
                // Handle format! macro
                if mac.mac.path.is_ident("format") {
                    self.lower_format_macro(&mac.mac.tokens)
                } else {
                    Expression::Literal("".to_string())
                }
            }
            _ => Expression::Literal("".to_string()),
        }
    }

    fn lower_method_call_expression(&mut self, method: &ExprMethodCall) -> Expression {
        let method_name = method.method.to_string();

        match method_name.as_str() {
            "trim" => {
                let input = self.lower_expression(&method.receiver);
                Expression::CommandSubstitution {
                    command: "echo".to_string(),
                    args: vec!["-n".to_string(), self.expression_to_string(&input)],
                }
            }
            "command" => {
                if let (Some(cmd_expr), Some(args_expr)) = (method.args.first(), method.args.get(1))
                {
                    let cmd = self.extract_string_literal(cmd_expr);
                    let args = self.extract_string_array(args_expr);

                    Expression::CommandSubstitution { command: cmd, args }
                } else {
                    Expression::Literal("".to_string())
                }
            }
            _ => Expression::Literal("".to_string()),
        }
    }

    fn lower_format_macro(&mut self, tokens: &proc_macro2::TokenStream) -> Expression {
        // Simple format! parsing - in real implementation would be more robust
        let tokens_str = tokens.to_string();
        if let Some(stripped) = tokens_str
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
        {
            let mut parts = Vec::new();
            let mut current = String::new();
            let mut chars = stripped.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '{' && chars.peek() == Some(&'}') {
                    chars.next(); // consume '}'
                    if !current.is_empty() {
                        parts.push(InterpolationPart::Literal(current.clone()));
                        current.clear();
                    }
                    // In real impl, would track format arguments
                    parts.push(InterpolationPart::Variable("_v0".to_string()));
                } else {
                    current.push(ch);
                }
            }

            if !current.is_empty() {
                parts.push(InterpolationPart::Literal(current));
            }

            Expression::StringInterpolation { parts }
        } else {
            Expression::Literal("".to_string())
        }
    }

    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(s) => s.clone(),
            Expression::Variable(v) => format!("${{{}}}", v),
            _ => "".to_string(),
        }
    }

    fn infer_receiver_type(&self, expr: &Expr) -> String {
        if let Expr::Path(path) = expr {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();
                if name == "ctx" {
                    return "ShellContext".to_string();
                }
            }
        }
        "Unknown".to_string()
    }

    fn extract_identifier(&self, pat: &Pat) -> String {
        match pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            _ => "_unknown".to_string(),
        }
    }

    fn extract_string_literal(&self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(expr_lit) => {
                if let Lit::Str(s) = &expr_lit.lit {
                    s.value()
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        }
    }

    fn extract_string_array(&self, expr: &Expr) -> Vec<String> {
        match expr {
            Expr::Reference(ref_expr) => {
                if let Expr::Array(array) = ref_expr.expr.as_ref() {
                    array
                        .elems
                        .iter()
                        .filter_map(|elem| {
                            if let Expr::Lit(expr_lit) = elem {
                                if let Lit::Str(s) = &expr_lit.lit {
                                    Some(s.value())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    fn allocate_variable(&mut self, name: &str) -> usize {
        if let Some(id) = self.variables.get(name) {
            *id
        } else {
            let id = self.next_var_id;
            self.variables.insert(name.to_string(), id);
            self.next_var_id += 1;
            id
        }
    }

    fn allocate_string(&mut self, content: &str) -> usize {
        if let Some(id) = self.string_pool.get(content) {
            *id
        } else {
            let id = self.next_string_id;
            self.string_pool.insert(content.to_string(), id);
            self.next_string_id += 1;
            id
        }
    }

    fn finalize(self) -> ShellAst {
        // Sort string pool by content hash for determinism
        let mut sorted_strings: Vec<_> = self.string_pool.into_iter().collect();

        sorted_strings.sort_by_key(|(s, _)| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(s.as_bytes());
            let hash = hasher.finalize();
            // Convert hash to bytes for sorting
            hash.as_bytes().to_vec()
        });

        ShellAst::Script {
            constants: sorted_strings,
            functions: vec![],
            main: self.statements,
        }
    }
}
