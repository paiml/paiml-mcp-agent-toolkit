#![allow(dead_code)]

use std::hash::Hash;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ShellAst {
    Script {
        constants: Vec<(String, usize)>,
        functions: Vec<Function>,
        main: Vec<Statement>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Statement {
    Assignment {
        var: String,
        value: Expression,
    },
    LocalAssignment {
        var: String,
        value: Expression,
    },
    Command {
        cmd: String,
        args: Vec<Expression>,
    },
    Conditional {
        test: Test,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    Case {
        expr: Expression,
        patterns: Vec<(String, Vec<Statement>)>,
    },
    Exit {
        code: i32,
    },
    Return {
        code: i32,
    },
    Comment {
        text: String,
    },
    If {
        condition: String,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    SetTrap {
        command: String,
        signals: Vec<String>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Expression {
    Literal(String),
    Variable(String),
    CommandSubstitution { command: String, args: Vec<String> },
    Concat(Vec<Expression>),
    StringInterpolation { parts: Vec<InterpolationPart> },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InterpolationPart {
    Literal(String),
    Variable(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Test {
    FileExists(String),
    DirectoryExists(String),
    StringEquals(String, String),
    StringNotEquals(String, String),
    CommandSuccess(String, Vec<String>),
    Not(Box<Test>),
}

impl ShellAst {
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        let serialized = format!("{:?}", self);
        hasher.update(serialized.as_bytes());
        hasher.finalize().into()
    }
}
