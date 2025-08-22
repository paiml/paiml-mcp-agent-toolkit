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

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use anyhow::Result;
use std::path::Path;

/// Ruchy language token types based on the lexer specification
#[derive(Debug, Clone, PartialEq)]
pub enum RuchyToken {
    // Keywords
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
    
    // Operators
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
    PipeForward,  // |>
    Arrow,        // ->
    FatArrow,     // =>
    
    // Delimiters
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
    
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    
    // Identifiers
    Identifier(String),
    
    // Special
    Annotation(String),  // @test, #[property], etc.
    Comment(String),
    Eof,
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
}

/// Ruchy complexity analyzer
pub struct RuchyComplexityAnalyzer {
    current_complexity: ComplexityMetrics,
    nesting_level: u8,
    functions: Vec<FunctionComplexity>,
    classes: Vec<crate::services::complexity::ClassComplexity>,
}

impl Default for RuchyComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RuchyComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            current_complexity: ComplexityMetrics::default(),
            nesting_level: 0,
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    /// Analyze a Ruchy AST node for complexity
    fn analyze_node(&mut self, node: &RuchyAst) {
        match node {
            RuchyAst::Function { name, body, line_start, line_end, .. } => {
                let prev_complexity = self.current_complexity;
                let prev_nesting = self.nesting_level;
                
                self.current_complexity = ComplexityMetrics {
                    cyclomatic: 1,  // Base complexity for function
                    cognitive: 0,
                    nesting_max: 0,
                    lines: 0,
                    halstead: None,
                };
                self.nesting_level = 0;
                
                self.analyze_node(body);
                
                self.functions.push(FunctionComplexity {
                    name: name.clone(),
                    line_start: *line_start,
                    line_end: *line_end,
                    metrics: self.current_complexity,
                });
                
                self.current_complexity = prev_complexity;
                self.nesting_level = prev_nesting;
            }
            
            RuchyAst::If { condition, then_branch, else_branch } => {
                self.current_complexity.cyclomatic += 1;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;
                
                self.nesting_level += 1;
                self.current_complexity.nesting_max = self.current_complexity.nesting_max.max(self.nesting_level);
                
                self.analyze_node(condition);
                self.analyze_node(then_branch);
                if let Some(else_br) = else_branch {
                    self.current_complexity.cyclomatic += 1;
                    self.analyze_node(else_br);
                }
                
                self.nesting_level -= 1;
            }
            
            RuchyAst::While { condition, body } => {
                self.current_complexity.cyclomatic += 1;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;
                
                self.nesting_level += 1;
                self.current_complexity.nesting_max = self.current_complexity.nesting_max.max(self.nesting_level);
                
                self.analyze_node(condition);
                self.analyze_node(body);
                
                self.nesting_level -= 1;
            }
            
            RuchyAst::For { body, .. } => {
                self.current_complexity.cyclomatic += 1;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;
                
                self.nesting_level += 1;
                self.current_complexity.nesting_max = self.current_complexity.nesting_max.max(self.nesting_level);
                
                self.analyze_node(body);
                
                self.nesting_level -= 1;
            }
            
            RuchyAst::Match { expr, arms } => {
                self.current_complexity.cyclomatic += arms.len() as u16;
                self.current_complexity.cognitive += 1 + self.nesting_level as u16;
                
                self.nesting_level += 1;
                self.current_complexity.nesting_max = self.current_complexity.nesting_max.max(self.nesting_level);
                
                self.analyze_node(expr);
                for (_, body) in arms {
                    self.analyze_node(body);
                }
                
                self.nesting_level -= 1;
            }
            
            RuchyAst::BinaryOp { left, op, right } => {
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
            
            RuchyAst::Class { methods, line_start, line_end, name, .. } => {
                let mut class_complexity = ComplexityMetrics::default();
                
                for method in methods {
                    self.analyze_node(method);
                    if let RuchyAst::Function { .. } = method {
                        if let Some(func) = self.functions.last() {
                            class_complexity.cyclomatic += func.metrics.cyclomatic;
                            class_complexity.cognitive += func.metrics.cognitive;
                            class_complexity.nesting_max = class_complexity.nesting_max.max(func.metrics.nesting_max);
                        }
                    }
                }
                
                self.classes.push(crate::services::complexity::ClassComplexity {
                    name: name.clone(),
                    line_start: *line_start,
                    line_end: *line_end,
                    metrics: class_complexity,
                    methods: vec![],
                });
            }
            
            RuchyAst::Call { function, args } => {
                self.analyze_node(function);
                for arg in args {
                    self.analyze_node(arg);
                }
            }
            
            _ => {
                // Other nodes don't affect complexity
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
            cyclomatic: self.functions.iter().map(|f| f.metrics.cyclomatic).sum::<u16>().max(1),
            cognitive: self.functions.iter().map(|f| f.metrics.cognitive).sum::<u16>().max(1),
            nesting_max: self.functions.iter().map(|f| f.metrics.nesting_max).max().unwrap_or(0),
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
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
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
                    "true" => RuchyToken::Bool(true),
                    "false" => RuchyToken::Bool(false),
                    _ => RuchyToken::Identifier(ident),
                }
            }
            Some(ch) if ch.is_numeric() => {
                self.read_number()
            }
            _ => {
                self.advance();
                RuchyToken::Eof
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
        if (trimmed.starts_with("fun ") || trimmed.starts_with("@test") || trimmed.contains("fun test_"))
            && !in_function {
                in_function = true;
                function_start = i as u32 + 1;
                
                // Extract function name
                if let Some(name_start) = trimmed.find("fun ") {
                    let after_fun = &trimmed[name_start + 4..];
                    function_name = after_fun.split('(')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
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
        cyclomatic: metrics.functions.iter().map(|f| f.metrics.cyclomatic).sum::<u16>().max(1),
        cognitive: metrics.functions.iter().map(|f| f.metrics.cognitive).sum::<u16>().max(1),
        nesting_max: metrics.functions.iter().map(|f| f.metrics.nesting_max).max().unwrap_or(0),
        lines: lines.len() as u16,
        halstead: None,
    };
    
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(lexer.next_token(), RuchyToken::Eof));
    }

    #[tokio::test]
    async fn test_ruchy_complexity_analysis() {
        use std::io::Write;
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