#![cfg_attr(coverage_nightly, coverage(off))]
//! Language-agnostic AST node kind enumerations.

use serde::{Deserialize, Serialize};

/// Language-agnostic AST node kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum AstKind {
    // Universal constructs
    Function(FunctionKind),
    Class(ClassKind),
    Variable(VarKind),
    Import(ImportKind),
    Expression(ExprKind),
    Statement(StmtKind),
    Type(TypeKind),
    Module(ModuleKind),
    Macro(MacroKind), // C-specific preprocessor macros
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Function kind.
pub enum FunctionKind {
    Regular,
    Method,
    Constructor,
    Getter,
    Setter,
    Lambda,
    Closure,
    Destructor, // C++ destructor
    Operator,   // C++ operator overload
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Class kind.
pub enum ClassKind {
    Regular,
    Abstract,
    Interface,
    Trait,
    Enum,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Var kind.
pub enum VarKind {
    Let,
    Const,
    Static,
    Field,
    Parameter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Import kind.
pub enum ImportKind {
    Module,
    Named,
    Default,
    Namespace,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Expr kind.
pub enum ExprKind {
    Call,
    Member,
    Binary,
    Unary,
    Literal,
    Identifier,
    Array,
    Object,
    New,         // C++ new expression
    Delete,      // C++ delete expression
    Lambda,      // C++ lambda expression
    Conditional, // TypeScript conditional expression (?:)
    This,        // TypeScript this expression
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Stmt kind.
pub enum StmtKind {
    Block,
    If,
    For,
    While,
    Return,
    Throw,
    Try,
    Switch,
    Goto,     // C-specific
    Label,    // C-specific
    DoWhile,  // C-specific
    ForEach,  // C++ range-based for
    Catch,    // C++ catch clause
    Break,    // break statement
    Continue, // continue statement
    Case,     // case statement in switch
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Type kind.
pub enum TypeKind {
    Primitive,
    Array,
    Tuple,
    Union,
    Intersection,
    Generic,
    Function,
    Object,
    Pointer,     // C-specific
    Struct,      // C-specific (distinct from Object)
    Enum,        // C-specific enum (distinct from Rust enum)
    Typedef,     // C-specific
    Class,       // C++ class
    Template,    // C++ template
    Namespace,   // C++ namespace
    Alias,       // C++ using alias
    Interface,   // TypeScript interface
    Module,      // TypeScript module
    Annotation,  // TypeScript type annotation
    Mapped,      // TypeScript mapped type
    Conditional, // TypeScript conditional type
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Module kind.
pub enum ModuleKind {
    File,
    Namespace,
    Package,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Macro kind.
pub enum MacroKind {
    ObjectLike,   // #define PI 3.14
    FunctionLike, // #define MAX(a,b) ((a)>(b)?(a):(b))
    Variadic,     // #define DEBUG(...) fprintf(stderr, __VA_ARGS__)
    Include,      // #include <stdio.h>
    Conditional,  // #ifdef, #ifndef, #if, #elif, #else, #endif
    Export,       // TypeScript export macro
    Decorator,    // TypeScript decorator
}
