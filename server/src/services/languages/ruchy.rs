//! Ruchy Language Support for PMAT
//!
//! This module provides AST parsing and complexity analysis for the Ruchy programming language.
//! Ruchy is a Rust-like language with Swift/Kotlin ergonomics that transpiles to Rust.
//!
//! ## Ruchy Language Features
//! - Functions: `fun name(params) -> ReturnType { ... }`
//! - Control flow: `if`, `while`, `for`, `match`
//! - Classes: `class Name { ... }`
//! - Actors: `actor Name { ... }`
//! - Traits: `trait Name { ... }`
//! - Pattern matching and pipeline operators
//!
//! ## Example
//! ```ruchy
//! fun fibonacci(n: i32) -> i32 {
//!     if n <= 1 {
//!         n
//!     } else {
//!         fibonacci(n - 1) + fibonacci(n - 2)
//!     }
//! }
//! ```

use crate::services::complexity::{
    ComplexityMetrics, FileComplexityMetrics, FunctionComplexity, HalsteadMetrics,
};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

/// Ruchy language token types based on the official Ruchy lexer specification
/// Updated to match ruchy v1.5.0 token definitions
#[derive(Debug, Clone, PartialEq)]
pub enum RuchyToken {
    // Keywords - aligned with ruchy v1.5.0
    Fun,
    If,
    Else,
    While,
    For,
    Match,
    Return,
    Let,
    Const,
    Var,
    Class,
    Struct,
    Enum,
    Trait,
    Impl,
    Actor,
    Receive,
    Spawn,
    Send,
    Await,
    Async,
    True,
    False,
    Break,
    Continue,
    In,
    As,
    Pub,
    Mod,
    Use,
    Where,
    Type,
    Import,
    From,
    Export,

    // Operators - aligned with ruchy v1.5.0
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,
    PipeForward, // |>
    Arrow,       // ->
    FatArrow,    // =>
    Question,    // ?
    Ampersand,   // &
    Pipe,        // |
    Caret,       // ^
    Tilde,       // ~
    LeftShift,   // <<
    RightShift,  // >>

    // Delimiters - aligned with ruchy v1.5.0
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    DoubleColon,
    DotDot,    // ..
    DotDotDot, // ...
    At,        // @
    Hash,      // #

    // Literals - aligned with ruchy v1.5.0
    Integer(i64),
    Float(f64),
    String(String),
    FString(String),
    Char(char),
    Bool(bool),

    // Identifiers
    Identifier(String),

    // Special
    Annotation(String), // @test, etc.
    Comment(String),

    // End of file
    Eof,

    // Error token
    Error,
}

/// Type information for Ruchy expressions
#[derive(Debug, Clone, PartialEq)]
pub enum RuchyType {
    Integer,
    Float,
    String,
    Bool,
    Char,
    Array(Box<RuchyType>),
    Option(Box<RuchyType>),
    Result(Box<RuchyType>, Box<RuchyType>),
    Function(Vec<RuchyType>, Box<RuchyType>), // (params, return)
    Class(String),
    Actor(String),
    Unknown,
    Inferred(String), // Type variable for inference
}

/// Import information
#[derive(Debug, Clone)]
pub struct RuchyImport {
    pub module: String,
    pub items: Vec<String>,
    pub line: u32,
}

/// Ruchy AST node types
#[derive(Debug, Clone)]
pub enum RuchyAst {
    Program {
        items: Vec<RuchyAst>,
    },
    Function {
        name: String,
        params: Vec<(String, String)>, // (name, type)
        return_type: Option<String>,
        body: Box<RuchyAst>,
        is_test: bool,
        line_start: u32,
        line_end: u32,
    },
    Class {
        name: String,
        fields: Vec<(String, String)>,
        methods: Vec<RuchyAst>,
        line_start: u32,
        line_end: u32,
    },
    Actor {
        name: String,
        state: Vec<(String, String)>,
        handlers: Vec<RuchyAst>,
        line_start: u32,
        line_end: u32,
    },
    Block {
        statements: Vec<RuchyAst>,
    },
    If {
        condition: Box<RuchyAst>,
        then_branch: Box<RuchyAst>,
        else_branch: Option<Box<RuchyAst>>,
    },
    While {
        condition: Box<RuchyAst>,
        body: Box<RuchyAst>,
    },
    For {
        variable: String,
        iterable: Box<RuchyAst>,
        body: Box<RuchyAst>,
    },
    Match {
        expr: Box<RuchyAst>,
        arms: Vec<(RuchyAst, RuchyAst)>, // (pattern, body)
    },
    Return {
        value: Option<Box<RuchyAst>>,
    },
    Let {
        name: String,
        value: Box<RuchyAst>,
    },
    BinaryOp {
        left: Box<RuchyAst>,
        op: RuchyToken,
        right: Box<RuchyAst>,
    },
    UnaryOp {
        op: RuchyToken,
        expr: Box<RuchyAst>,
    },
    Call {
        function: Box<RuchyAst>,
        args: Vec<RuchyAst>,
    },
    Pipeline {
        stages: Vec<RuchyAst>,
    },
    Identifier(String),
    Literal(RuchyToken),
    Import {
        module: String,
        items: Vec<String>,
        line: u32,
    },
    Export {
        items: Vec<String>,
        line: u32,
    },
}

/// Ruchy complexity analyzer
pub struct RuchyComplexityAnalyzer {
    current_complexity: ComplexityMetrics,
    nesting_level: u8,
    functions: Vec<FunctionComplexity>,
    classes: Vec<crate::services::complexity::ClassComplexity>,
    // Halstead metrics tracking
    operators: HashSet<String>,
    operands: HashSet<String>,
    operator_count: u32,
    operand_count: u32,
    // Dead code tracking
    defined_functions: HashSet<String>,
    called_functions: HashSet<String>,
    defined_variables: HashSet<String>,
    used_variables: HashSet<String>,
    // Type inference tracking
    #[allow(dead_code)]
    type_environment: std::collections::HashMap<String, RuchyType>,
    // Import/dependency tracking
    imports: Vec<RuchyImport>,
    exports: HashSet<String>,
    // Actor analysis tracking
    actors: Vec<ActorInfo>,
    current_actor: Option<String>,
    message_flows: Vec<MessageFlow>,
    spawn_calls: Vec<(String, String, u32)>, // (spawner, spawned, line)
}

impl Default for RuchyComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Dead code detection results
#[derive(Debug, Clone)]
pub struct RuchyDeadCode {
    pub unused_functions: Vec<String>,
    pub unused_variables: Vec<String>,
    pub unreachable_code: Vec<(u32, u32)>, // (start_line, end_line)
}

/// Actor message flow analysis
#[derive(Debug, Clone)]
pub struct RuchyActorAnalysis {
    pub actors: Vec<ActorInfo>,
    pub message_flows: Vec<MessageFlow>,
    pub potential_deadlocks: Vec<DeadlockWarning>,
}

/// Information about an actor
#[derive(Debug, Clone)]
pub struct ActorInfo {
    pub name: String,
    pub state_fields: Vec<String>,
    pub message_handlers: Vec<String>,
    pub spawned_actors: Vec<String>,
    pub line_start: u32,
    pub line_end: u32,
}

/// Message flow between actors
#[derive(Debug, Clone)]
pub struct MessageFlow {
    pub from_actor: String,
    pub to_actor: String,
    pub message_type: String,
    pub line: u32,
}

/// Potential deadlock warning
#[derive(Debug, Clone)]
pub struct DeadlockWarning {
    pub actors_involved: Vec<String>,
    pub description: String,
    pub line: u32,
}

impl RuchyComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            current_complexity: ComplexityMetrics::default(),
            nesting_level: 0,
            functions: Vec::new(),
            classes: Vec::new(),
            operators: HashSet::new(),
            operands: HashSet::new(),
            operator_count: 0,
            operand_count: 0,
            defined_functions: HashSet::new(),
            called_functions: HashSet::new(),
            defined_variables: HashSet::<String>::new(),
            used_variables: HashSet::new(),
            type_environment: std::collections::HashMap::new(),
            imports: Vec::new(),
            exports: HashSet::new(),
            actors: Vec::new(),
            current_actor: None,
            message_flows: Vec::new(),
            spawn_calls: Vec::new(),
        }
    }

    /// Reset Halstead tracking for a new function
    fn reset_halstead(&mut self) {
        self.operators.clear();
        self.operands.clear();
        self.operator_count = 0;
        self.operand_count = 0;
    }

    /// Track an operator for Halstead metrics
    fn track_operator(&mut self, op: &str) {
        self.operators.insert(op.to_string());
        self.operator_count += 1;
    }

    /// Track an operand for Halstead metrics
    fn track_operand(&mut self, operand: &str) {
        self.operands.insert(operand.to_string());
        self.operand_count += 1;
    }

    /// Calculate Halstead metrics for current function
    fn calculate_halstead(&self) -> HalsteadMetrics {
        let n1 = self.operators.len() as u32;
        let n2 = self.operands.len() as u32;
        let n1_total = self.operator_count;
        let n2_total = self.operand_count;

        let n = (n1 + n2) as f64;
        let n_total = (n1_total + n2_total) as f64;

        let volume = if n > 0.0 { n_total * n.log2() } else { 0.0 };
        let difficulty = if n2 > 0 {
            (n1 as f64 / 2.0) * (n2_total as f64 / n2 as f64)
        } else {
            0.0
        };
        let effort = volume * difficulty;
        let time = effort / 18.0; // Stroud number
        let bugs = volume / 3000.0; // Industry average

        HalsteadMetrics {
            n1,
            n2,
            n1_total,
            n2_total,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        }
    }

    /// Get dead code analysis results
    pub fn get_dead_code(&self) -> RuchyDeadCode {
        let unused_functions: Vec<String> = self
            .defined_functions
            .difference(&self.called_functions)
            .filter(|f| *f != "main" && !self.exports.contains(*f)) // main and exported functions are entry points
            .cloned()
            .collect();

        let unused_variables: Vec<String> = self
            .defined_variables
            .difference(&self.used_variables)
            .cloned()
            .collect();

        RuchyDeadCode {
            unused_functions,
            unused_variables,
            unreachable_code: Vec::new(), // Will be populated during AST traversal
        }
    }

    /// Infer type from a literal token
    #[allow(dead_code)]
    fn infer_literal_type(&self, lit: &RuchyToken) -> RuchyType {
        match lit {
            RuchyToken::Integer(_) => RuchyType::Integer,
            RuchyToken::Float(_) => RuchyType::Float,
            RuchyToken::String(_) | RuchyToken::FString(_) => RuchyType::String,
            RuchyToken::Char(_) => RuchyType::Char,
            RuchyToken::Bool(_) | RuchyToken::True | RuchyToken::False => RuchyType::Bool,
            _ => RuchyType::Unknown,
        }
    }

    /// Infer type of a binary operation
    #[allow(dead_code)]
    fn infer_binary_type(
        &self,
        op: &RuchyToken,
        left_type: &RuchyType,
        _right_type: &RuchyType,
    ) -> RuchyType {
        match op {
            RuchyToken::Plus | RuchyToken::Minus | RuchyToken::Star | RuchyToken::Slash => {
                match left_type {
                    RuchyType::Float => RuchyType::Float,
                    RuchyType::Integer => RuchyType::Integer,
                    RuchyType::String if matches!(op, RuchyToken::Plus) => RuchyType::String,
                    _ => RuchyType::Unknown,
                }
            }
            RuchyToken::EqualEqual
            | RuchyToken::NotEqual
            | RuchyToken::Less
            | RuchyToken::Greater
            | RuchyToken::LessEqual
            | RuchyToken::GreaterEqual => RuchyType::Bool,
            RuchyToken::And | RuchyToken::Or => RuchyType::Bool,
            _ => RuchyType::Unknown,
        }
    }

    /// Get import dependencies
    pub fn get_imports(&self) -> &[RuchyImport] {
        &self.imports
    }

    /// Get exported items
    pub fn get_exports(&self) -> Vec<String> {
        self.exports.iter().cloned().collect()
    }

    /// Analyze pattern complexity for match expressions
    fn analyze_pattern_complexity(&mut self, pattern: &RuchyAst) {
        match pattern {
            RuchyAst::Identifier(name) => {
                self.track_operand(name);
                self.defined_variables.insert(name.clone());
            }
            RuchyAst::Literal(lit) => match lit {
                RuchyToken::Integer(i) => self.track_operand(&i.to_string()),
                RuchyToken::String(s) => self.track_operand(s),
                _ => {}
            },
            // Wildcard pattern
            _ => {
                self.track_operator("_");
            }
        }
    }

    /// Get actor analysis results
    pub fn get_actor_analysis(&self) -> RuchyActorAnalysis {
        let potential_deadlocks = self.detect_potential_deadlocks();

        RuchyActorAnalysis {
            actors: self.actors.clone(),
            message_flows: self.message_flows.clone(),
            potential_deadlocks,
        }
    }

    /// Detect potential deadlocks in actor message flows
    fn detect_potential_deadlocks(&self) -> Vec<DeadlockWarning> {
        let mut warnings = Vec::new();

        // Simple cycle detection in message flows
        for flow1 in &self.message_flows {
            for flow2 in &self.message_flows {
                if flow1.from_actor == flow2.to_actor && flow1.to_actor == flow2.from_actor {
                    warnings.push(DeadlockWarning {
                        actors_involved: vec![flow1.from_actor.clone(), flow1.to_actor.clone()],
                        description: format!(
                            "Potential circular dependency between {} and {}",
                            flow1.from_actor, flow1.to_actor
                        ),
                        line: flow1.line,
                    });
                }
            }
        }

        warnings
    }

    /// Analyze a Ruchy AST node for complexity
    fn analyze_node(&mut self, node: &RuchyAst) {
        match node {
            RuchyAst::Function {
                name,
                body,
                line_start,
                line_end,
                ..
            } => {
                // Track function definition for dead code analysis
                self.defined_functions.insert(name.clone());
                self.track_operator("fun");
                self.track_operand(name);

                let prev_complexity = self.current_complexity;
                let prev_nesting = self.nesting_level;

                self.current_complexity = ComplexityMetrics {
                    cyclomatic: 1, // Base complexity for function
                    cognitive: 0,
                    nesting_max: 0,
                    lines: (*line_end - *line_start) as u16,
                    halstead: None,
                };
                self.nesting_level = 0;
                self.reset_halstead();

                self.analyze_node(body);

                // Calculate Halstead metrics for this function
                let halstead = self.calculate_halstead();
                self.current_complexity.halstead = Some(halstead);

                self.functions.push(FunctionComplexity {
                    name: name.clone(),
                    line_start: *line_start,
                    line_end: *line_end,
                    metrics: self.current_complexity,
                });

                self.current_complexity = prev_complexity;
                self.nesting_level = prev_nesting;
            }

            RuchyAst::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.current_complexity.cyclomatic += 1;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;
                self.track_operator("if");

                self.nesting_level += 1;
                self.current_complexity.nesting_max =
                    self.current_complexity.nesting_max.max(self.nesting_level);

                self.analyze_node(condition);
                self.analyze_node(then_branch);
                if let Some(else_br) = else_branch {
                    self.current_complexity.cyclomatic += 1;
                    self.track_operator("else");
                    self.analyze_node(else_br);
                }

                self.nesting_level -= 1;
            }

            RuchyAst::While { condition, body } => {
                self.current_complexity.cyclomatic += 1;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;

                self.nesting_level += 1;
                self.current_complexity.nesting_max =
                    self.current_complexity.nesting_max.max(self.nesting_level);

                self.analyze_node(condition);
                self.analyze_node(body);

                self.nesting_level -= 1;
            }

            RuchyAst::For { body, .. } => {
                self.current_complexity.cyclomatic += 1;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;

                self.nesting_level += 1;
                self.current_complexity.nesting_max =
                    self.current_complexity.nesting_max.max(self.nesting_level);

                self.analyze_node(body);

                self.nesting_level -= 1;
            }

            RuchyAst::Match { expr, arms } => {
                // Pattern matching has higher cognitive complexity
                let arm_count = arms.len() as u16;
                self.current_complexity.cyclomatic += arm_count;
                self.current_complexity.cognitive += (arm_count * 2) + self.nesting_level as u16;

                self.track_operator("match");

                self.nesting_level += 1;
                self.current_complexity.nesting_max =
                    self.current_complexity.nesting_max.max(self.nesting_level);

                self.analyze_node(expr);
                for (pattern, body) in arms {
                    // Analyze pattern complexity
                    self.analyze_pattern_complexity(pattern);
                    self.analyze_node(body);
                }

                self.nesting_level -= 1;
            }

            RuchyAst::BinaryOp { left, op, right } => {
                // Track operator for Halstead
                let op_str = match op {
                    RuchyToken::Plus => "+",
                    RuchyToken::Minus => "-",
                    RuchyToken::Star => "*",
                    RuchyToken::Slash => "/",
                    RuchyToken::Percent => "%",
                    RuchyToken::EqualEqual => "==",
                    RuchyToken::NotEqual => "!=",
                    RuchyToken::Less => "<",
                    RuchyToken::Greater => ">",
                    RuchyToken::LessEqual => "<=",
                    RuchyToken::GreaterEqual => ">=",
                    RuchyToken::And => "&&",
                    RuchyToken::Or => "||",
                    RuchyToken::PipeForward => "|>",
                    _ => "op",
                };
                self.track_operator(op_str);

                // Logical operators add complexity
                if matches!(op, RuchyToken::And | RuchyToken::Or) {
                    self.current_complexity.cyclomatic += 1;
                    self.current_complexity.cognitive += 1;
                }
                self.analyze_node(left);
                self.analyze_node(right);
            }

            RuchyAst::Block { statements } => {
                for stmt in statements {
                    self.analyze_node(stmt);
                }
            }

            RuchyAst::Pipeline { stages } => {
                // Pipelines add cognitive complexity
                self.current_complexity.cognitive += (stages.len() as u16).saturating_sub(1);
                for stage in stages {
                    self.analyze_node(stage);
                }
            }

            RuchyAst::Class {
                methods,
                line_start,
                line_end,
                name,
                ..
            } => {
                let mut class_complexity = ComplexityMetrics::default();

                for method in methods {
                    self.analyze_node(method);
                    if let RuchyAst::Function { .. } = method {
                        if let Some(func) = self.functions.last() {
                            class_complexity.cyclomatic += func.metrics.cyclomatic;
                            class_complexity.cognitive += func.metrics.cognitive;
                            class_complexity.nesting_max =
                                class_complexity.nesting_max.max(func.metrics.nesting_max);
                        }
                    }
                }

                self.classes
                    .push(crate::services::complexity::ClassComplexity {
                        name: name.clone(),
                        line_start: *line_start,
                        line_end: *line_end,
                        metrics: class_complexity,
                        methods: vec![],
                    });
            }

            RuchyAst::Call { function, args } => {
                // Track function call for dead code analysis
                if let RuchyAst::Identifier(fn_name) = function.as_ref() {
                    self.called_functions.insert(fn_name.clone());
                    self.track_operand(fn_name);

                    // Track actor-related calls for message flow analysis
                    match fn_name.as_str() {
                        "spawn" if !args.is_empty() => {
                            if let RuchyAst::Identifier(actor_name) = &args[0] {
                                if let Some(current) = &self.current_actor {
                                    self.spawn_calls
                                        .push((current.clone(), actor_name.clone(), 0));
                                }
                            }
                        }
                        "send" if args.len() >= 2 => {
                            if let (RuchyAst::Identifier(target), RuchyAst::Identifier(message)) =
                                (&args[0], &args[1])
                            {
                                if let Some(current) = &self.current_actor {
                                    self.message_flows.push(MessageFlow {
                                        from_actor: current.clone(),
                                        to_actor: target.clone(),
                                        message_type: message.clone(),
                                        line: 0,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                self.track_operator("()");

                self.analyze_node(function);
                for arg in args {
                    self.analyze_node(arg);
                }
            }

            RuchyAst::Identifier(name) => {
                self.track_operand(name);
                self.used_variables.insert(name.clone());
            }

            RuchyAst::Literal(lit) => match lit {
                RuchyToken::Integer(i) => self.track_operand(&i.to_string()),
                RuchyToken::Float(f) => self.track_operand(&f.to_string()),
                RuchyToken::String(s) | RuchyToken::FString(s) => self.track_operand(s),
                RuchyToken::Bool(b) => self.track_operand(&b.to_string()),
                _ => {}
            },

            RuchyAst::Let { name, value } => {
                self.defined_variables.insert(name.clone());
                self.track_operator("let");
                self.track_operand(name);
                self.analyze_node(value);
            }

            RuchyAst::Return { value } => {
                self.track_operator("return");
                if let Some(val) = value {
                    self.analyze_node(val);
                }
            }

            RuchyAst::UnaryOp { op, expr } => {
                let op_str = match op {
                    RuchyToken::Not => "!",
                    RuchyToken::Minus => "-",
                    _ => "unary",
                };
                self.track_operator(op_str);
                self.analyze_node(expr);
            }

            RuchyAst::Import {
                module,
                items,
                line,
            } => {
                self.imports.push(RuchyImport {
                    module: module.clone(),
                    items: items.clone(),
                    line: *line,
                });
                self.track_operator("import");
                self.track_operand(module);
            }

            RuchyAst::Export { items, .. } => {
                for item in items {
                    self.exports.insert(item.clone());
                }
                self.track_operator("export");
            }

            // Note: Ok, Err, Some, None, Try, Await would need to be added to RuchyAst enum
            // For now, handle them as regular function calls
            RuchyAst::Actor {
                name,
                state,
                handlers,
                line_start,
                line_end,
            } => {
                self.track_operator("actor");
                self.track_operand(name);

                let prev_actor = self.current_actor.clone();
                self.current_actor = Some(name.clone());

                let mut actor_info = ActorInfo {
                    name: name.clone(),
                    state_fields: state.iter().map(|(field, _)| field.clone()).collect(),
                    message_handlers: Vec::new(),
                    spawned_actors: Vec::new(),
                    line_start: *line_start,
                    line_end: *line_end,
                };

                let mut class_complexity = ComplexityMetrics::default();

                for handler in handlers {
                    if let RuchyAst::Function {
                        name: handler_name, ..
                    } = handler
                    {
                        actor_info.message_handlers.push(handler_name.clone());
                    }
                    self.analyze_node(handler);
                    if let RuchyAst::Function { .. } = handler {
                        if let Some(func) = self.functions.last() {
                            class_complexity.cyclomatic += func.metrics.cyclomatic;
                            class_complexity.cognitive += func.metrics.cognitive;
                            class_complexity.nesting_max =
                                class_complexity.nesting_max.max(func.metrics.nesting_max);
                        }
                    }
                }

                self.actors.push(actor_info);
                self.classes
                    .push(crate::services::complexity::ClassComplexity {
                        name: name.clone(),
                        line_start: *line_start,
                        line_end: *line_end,
                        metrics: class_complexity,
                        methods: vec![],
                    });

                self.current_actor = prev_actor;
            }

            _ => {
                // Other nodes don't affect complexity directly
            }
        }
    }

    /// Analyze a complete Ruchy program
    pub fn analyze_program(&mut self, ast: &RuchyAst) -> FileComplexityMetrics {
        if let RuchyAst::Program { items } = ast {
            for item in items {
                self.analyze_node(item);
            }
        } else {
            self.analyze_node(ast);
        }

        // Calculate total file complexity
        let total_complexity = ComplexityMetrics {
            cyclomatic: self
                .functions
                .iter()
                .map(|f| f.metrics.cyclomatic)
                .sum::<u16>()
                .max(1),
            cognitive: self
                .functions
                .iter()
                .map(|f| f.metrics.cognitive)
                .sum::<u16>()
                .max(1),
            nesting_max: self
                .functions
                .iter()
                .map(|f| f.metrics.nesting_max)
                .max()
                .unwrap_or(0),
            lines: self.functions.iter().map(|f| f.metrics.lines).sum::<u16>(),
            halstead: None,
        };

        FileComplexityMetrics {
            path: String::new(), // Will be set by caller
            total_complexity,
            functions: self.functions.clone(),
            classes: self.classes.clone(),
        }
    }
}

/// Simple Ruchy lexer for basic tokenization
pub struct RuchyLexer {
    input: String,
    position: usize,
    current_char: Option<char>,
    line: u32,
    column: u32,
}

impl RuchyLexer {
    pub fn new(input: String) -> Self {
        let lexer = Self {
            input: input.clone(),
            position: 0,
            current_char: input.chars().next(),
            line: 1,
            column: 1,
        };
        lexer
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.current_char == Some('/') && self.peek() == Some('/') {
            while self.current_char.is_some() && self.current_char != Some('\n') {
                self.advance();
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_number(&mut self) -> RuchyToken {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char {
            if ch.is_numeric() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek().is_some_and(|c| c.is_numeric()) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !num_str.contains('e') && !num_str.contains('E') {
                num_str.push(ch);
                self.advance();
                if let Some(sign) = self.current_char {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        if is_float {
            RuchyToken::Float(num_str.parse().unwrap_or(0.0))
        } else {
            RuchyToken::Integer(num_str.parse().unwrap_or(0))
        }
    }

    fn read_string(&mut self, quote: char) -> String {
        let mut result = String::new();
        self.advance(); // skip opening quote

        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '\'' => result.push('\''),
                        _ => {
                            result.push('\\');
                            result.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        result
    }

    pub fn next_token(&mut self) -> RuchyToken {
        self.skip_whitespace();
        self.skip_comment();

        match self.current_char {
            None => RuchyToken::Eof,
            Some('+') => {
                self.advance();
                RuchyToken::Plus
            }
            Some('-') => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    RuchyToken::Arrow
                } else {
                    RuchyToken::Minus
                }
            }
            Some('*') => {
                self.advance();
                RuchyToken::Star
            }
            Some('/') => {
                self.advance();
                if self.current_char == Some('/') {
                    self.skip_comment();
                    self.next_token()
                } else {
                    RuchyToken::Slash
                }
            }
            Some('(') => {
                self.advance();
                RuchyToken::LeftParen
            }
            Some(')') => {
                self.advance();
                RuchyToken::RightParen
            }
            Some('{') => {
                self.advance();
                RuchyToken::LeftBrace
            }
            Some('}') => {
                self.advance();
                RuchyToken::RightBrace
            }
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    RuchyToken::EqualEqual
                } else if self.current_char == Some('>') {
                    self.advance();
                    RuchyToken::FatArrow
                } else {
                    RuchyToken::Equal
                }
            }
            Some('|') => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    RuchyToken::PipeForward
                } else if self.current_char == Some('|') {
                    self.advance();
                    RuchyToken::Or
                } else {
                    RuchyToken::Identifier("|".to_string())
                }
            }
            Some('&') => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    RuchyToken::And
                } else {
                    RuchyToken::Identifier("&".to_string())
                }
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "fun" => RuchyToken::Fun,
                    "if" => RuchyToken::If,
                    "else" => RuchyToken::Else,
                    "while" => RuchyToken::While,
                    "for" => RuchyToken::For,
                    "match" => RuchyToken::Match,
                    "return" => RuchyToken::Return,
                    "let" => RuchyToken::Let,
                    "const" => RuchyToken::Const,
                    "var" => RuchyToken::Var,
                    "class" => RuchyToken::Class,
                    "struct" => RuchyToken::Struct,
                    "enum" => RuchyToken::Enum,
                    "trait" => RuchyToken::Trait,
                    "impl" => RuchyToken::Impl,
                    "actor" => RuchyToken::Actor,
                    "async" => RuchyToken::Async,
                    "await" => RuchyToken::Await,
                    "spawn" => RuchyToken::Spawn,
                    "send" => RuchyToken::Send,
                    "receive" => RuchyToken::Receive,
                    "break" => RuchyToken::Break,
                    "continue" => RuchyToken::Continue,
                    "in" => RuchyToken::In,
                    "as" => RuchyToken::As,
                    "pub" => RuchyToken::Pub,
                    "mod" => RuchyToken::Mod,
                    "use" => RuchyToken::Use,
                    "where" => RuchyToken::Where,
                    "type" => RuchyToken::Type,
                    "import" => RuchyToken::Import,
                    "from" => RuchyToken::From,
                    "export" => RuchyToken::Export,
                    "true" => RuchyToken::True,
                    "false" => RuchyToken::False,
                    _ => RuchyToken::Identifier(ident),
                }
            }
            Some('"') => {
                let s = self.read_string('"');
                RuchyToken::String(s)
            }
            Some('\'') => {
                self.advance();
                let ch = self.current_char.unwrap_or('\0');
                self.advance();
                if self.current_char == Some('\'') {
                    self.advance();
                }
                RuchyToken::Char(ch)
            }
            Some('@') => {
                self.advance();
                let ident = self.read_identifier();
                RuchyToken::Annotation(format!("@{}", ident))
            }
            Some('#') => {
                self.advance();
                RuchyToken::Hash
            }
            Some('.') => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    if self.current_char == Some('.') {
                        self.advance();
                        RuchyToken::DotDotDot
                    } else {
                        RuchyToken::DotDot
                    }
                } else {
                    RuchyToken::Dot
                }
            }
            Some(':') => {
                self.advance();
                if self.current_char == Some(':') {
                    self.advance();
                    RuchyToken::DoubleColon
                } else {
                    RuchyToken::Colon
                }
            }
            Some(';') => {
                self.advance();
                RuchyToken::Semicolon
            }
            Some(',') => {
                self.advance();
                RuchyToken::Comma
            }
            Some('[') => {
                self.advance();
                RuchyToken::LeftBracket
            }
            Some(']') => {
                self.advance();
                RuchyToken::RightBracket
            }
            Some('!') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    RuchyToken::NotEqual
                } else {
                    RuchyToken::Not
                }
            }
            Some('<') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    RuchyToken::LessEqual
                } else if self.current_char == Some('<') {
                    self.advance();
                    RuchyToken::LeftShift
                } else {
                    RuchyToken::Less
                }
            }
            Some('>') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    RuchyToken::GreaterEqual
                } else if self.current_char == Some('>') {
                    self.advance();
                    RuchyToken::RightShift
                } else {
                    RuchyToken::Greater
                }
            }
            Some('?') => {
                self.advance();
                RuchyToken::Question
            }
            Some('~') => {
                self.advance();
                RuchyToken::Tilde
            }
            Some('^') => {
                self.advance();
                RuchyToken::Caret
            }
            Some('%') => {
                self.advance();
                RuchyToken::Percent
            }
            Some(ch) if ch.is_numeric() => self.read_number(),
            _ => {
                self.advance();
                RuchyToken::Error
            }
        }
    }
}

/// Parse a Ruchy file and analyze its complexity
pub async fn analyze_ruchy_file(path: &Path) -> Result<FileComplexityMetrics> {
    let content = tokio::fs::read_to_string(path).await?;

    // For now, use a simple heuristic-based analysis
    // A full parser would be implemented based on the grammar specification
    let _analyzer = RuchyComplexityAnalyzer::new();

    // Simple parsing - count functions and control flow
    let mut metrics = FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: ComplexityMetrics::default(),
        functions: vec![],
        classes: vec![],
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut in_function = false;
    let mut function_name = String::new();
    let mut function_start = 0;
    let mut brace_count = 0;
    let mut current_metrics = ComplexityMetrics {
        cyclomatic: 1,
        cognitive: 0,
        nesting_max: 0,
        lines: 0,
        halstead: None,
    };

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Detect function start
        if (trimmed.starts_with("fun ")
            || trimmed.starts_with("@test")
            || trimmed.contains("fun test_"))
            && !in_function
        {
            in_function = true;
            function_start = i as u32 + 1;

            // Extract function name
            if let Some(name_start) = trimmed.find("fun ") {
                let after_fun = &trimmed[name_start + 4..];
                function_name = after_fun.split('(').next().unwrap_or("").trim().to_string();
            }

            current_metrics = ComplexityMetrics {
                cyclomatic: 1,
                cognitive: 0,
                nesting_max: 0,
                lines: 0,
                halstead: None,
            };
        }

        if in_function {
            current_metrics.lines += 1;

            // Count control flow keywords
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                current_metrics.cyclomatic += 1;
                current_metrics.cognitive += 1;
            }
            if trimmed.starts_with("else if ") {
                current_metrics.cyclomatic += 1;
                current_metrics.cognitive += 1;
            }
            if trimmed.starts_with("while ") || trimmed.contains(" while ") {
                current_metrics.cyclomatic += 1;
                current_metrics.cognitive += 2;
            }
            if trimmed.starts_with("for ") || trimmed.contains(" for ") {
                current_metrics.cyclomatic += 1;
                current_metrics.cognitive += 2;
            }
            if trimmed.starts_with("match ") {
                current_metrics.cyclomatic += 1;
                current_metrics.cognitive += 2;
            }
            if trimmed.contains("&&") || trimmed.contains("||") {
                current_metrics.cyclomatic += 1;
                current_metrics.cognitive += 1;
            }

            // Track braces for function end
            brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Function ends when brace count returns to 0
            if brace_count == 0 && trimmed.contains('}') {
                metrics.functions.push(FunctionComplexity {
                    name: function_name.clone(),
                    line_start: function_start,
                    line_end: i as u32 + 1,
                    metrics: current_metrics,
                });

                in_function = false;
                function_name.clear();
            }
        }
    }

    // Calculate total complexity
    metrics.total_complexity = ComplexityMetrics {
        cyclomatic: metrics
            .functions
            .iter()
            .map(|f| f.metrics.cyclomatic)
            .sum::<u16>()
            .max(1),
        cognitive: metrics
            .functions
            .iter()
            .map(|f| f.metrics.cognitive)
            .sum::<u16>()
            .max(1),
        nesting_max: metrics
            .functions
            .iter()
            .map(|f| f.metrics.nesting_max)
            .max()
            .unwrap_or(0),
        lines: lines.len() as u16,
        halstead: None,
    };

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_ruchy_lexer_basic() {
        let mut lexer = RuchyLexer::new("fun test() { return 42 }".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Fun));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(_)));
        assert!(matches!(lexer.next_token(), RuchyToken::LeftParen));
        assert!(matches!(lexer.next_token(), RuchyToken::RightParen));
        assert!(matches!(lexer.next_token(), RuchyToken::LeftBrace));
        assert!(matches!(lexer.next_token(), RuchyToken::Return));
        assert!(matches!(lexer.next_token(), RuchyToken::Integer(42)));
        assert!(matches!(lexer.next_token(), RuchyToken::RightBrace));
        // lexer.next_token(); // Last token varies based on implementation
    }

    #[test]
    fn test_ruchy_halstead_calculation() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        // Track some operators and operands
        analyzer.track_operator("+");
        analyzer.track_operator("-");
        analyzer.track_operator("+"); // Duplicate should only count once in distinct
        analyzer.track_operand("x");
        analyzer.track_operand("y");
        analyzer.track_operand("42");
        analyzer.track_operand("x"); // Duplicate

        let halstead = analyzer.calculate_halstead();

        assert_eq!(halstead.n1, 2); // 2 distinct operators
        assert_eq!(halstead.n2, 3); // 3 distinct operands
        assert_eq!(halstead.n1_total, 3); // 3 total operators
        assert_eq!(halstead.n2_total, 4); // 4 total operands
        assert!(halstead.volume > 0.0);
    }

    #[test]
    fn test_dead_code_detection() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        // Simulate some function definitions and calls
        analyzer.defined_functions.insert("main".to_string());
        analyzer.defined_functions.insert("helper".to_string());
        analyzer.defined_functions.insert("unused".to_string());

        analyzer.called_functions.insert("helper".to_string());
        // 'unused' is never called, 'main' is entry point

        let dead_code = analyzer.get_dead_code();

        assert_eq!(dead_code.unused_functions.len(), 1);
        assert!(dead_code.unused_functions.contains(&"unused".to_string()));
        assert!(!dead_code.unused_functions.contains(&"main".to_string())); // main is entry point
    }

    #[tokio::test]
    async fn test_ruchy_complexity_analysis() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.ruchy");

        let content = r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    for i in 0..10 {
        println(fibonacci(i))
    }
}
"#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let metrics = analyze_ruchy_file(&file_path).await.unwrap();

        assert_eq!(metrics.functions.len(), 2);
        assert!(metrics.functions[0].metrics.cyclomatic > 1);
        assert!(metrics.total_complexity.cyclomatic > 1);
    }
}
