#![cfg_attr(coverage_nightly, coverage(off))]
//! Ruchy type definitions: tokens, AST nodes, type system, and data structures.

use std::collections::HashMap;
use std::sync::LazyLock;

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

// Static maps for O(1) keyword and token lookups
pub(super) static KEYWORD_MAP: LazyLock<HashMap<&'static str, RuchyToken>> = LazyLock::new(|| {
    use RuchyToken::{
        Actor, As, Async, Await, Break, Class, Const, Continue, Else, Enum, Export, False, For,
        From, Fun, If, Impl, Import, In, Let, Match, Mod, Pub, Receive, Return, Send, Spawn,
        Struct, Trait, True, Type, Use, Var, Where, While,
    };
    let mut map = HashMap::new();
    map.insert("fun", Fun);
    map.insert("if", If);
    map.insert("else", Else);
    map.insert("while", While);
    map.insert("for", For);
    map.insert("match", Match);
    map.insert("return", Return);
    map.insert("let", Let);
    map.insert("const", Const);
    map.insert("var", Var);
    map.insert("class", Class);
    map.insert("struct", Struct);
    map.insert("enum", Enum);
    map.insert("trait", Trait);
    map.insert("impl", Impl);
    map.insert("actor", Actor);
    map.insert("async", Async);
    map.insert("await", Await);
    map.insert("spawn", Spawn);
    map.insert("send", Send);
    map.insert("receive", Receive);
    map.insert("break", Break);
    map.insert("continue", Continue);
    map.insert("in", In);
    map.insert("as", As);
    map.insert("pub", Pub);
    map.insert("mod", Mod);
    map.insert("use", Use);
    map.insert("where", Where);
    map.insert("type", Type);
    map.insert("import", Import);
    map.insert("from", From);
    map.insert("export", Export);
    map.insert("true", True);
    map.insert("false", False);
    map
});

pub(super) static SINGLE_CHAR_TOKEN_MAP: LazyLock<HashMap<char, RuchyToken>> =
    LazyLock::new(|| {
        use RuchyToken::{
            Caret, Comma, Hash, LeftBrace, LeftBracket, LeftParen, Percent, Plus, Question,
            RightBrace, RightBracket, RightParen, Semicolon, Star, Tilde,
        };
        let mut map = HashMap::new();
        map.insert('+', Plus);
        map.insert('*', Star);
        map.insert('(', LeftParen);
        map.insert(')', RightParen);
        map.insert('{', LeftBrace);
        map.insert('}', RightBrace);
        map.insert('[', LeftBracket);
        map.insert(']', RightBracket);
        map.insert(';', Semicolon);
        map.insert(',', Comma);
        map.insert('?', Question);
        map.insert('~', Tilde);
        map.insert('^', Caret);
        map.insert('%', Percent);
        map.insert('#', Hash);
        map
    });

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
