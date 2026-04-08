#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use super::{MakefileRule, Severity, Violation};
use crate::services::makefile_linter::ast::{
    AssignmentOp, MakefileAst, MakefileNodeKind, NodeData, SourceSpan,
};
use std::collections::HashSet;

// --- Phony rules: MinPhonyRule, PhonyDeclaredRule ---
include!("checkmake_phony.rs");

// --- Recipe rules: MaxBodyLengthRule, TimestampExpandedRule ---
include!("checkmake_recipe.rs");

// --- Variable analysis: UndefinedVariableRule, VariableScanner, helpers ---
include!("checkmake_variables.rs");

// --- Portability rule: PortabilityRule ---
include!("checkmake_portability.rs");

// --- Tests ---
include!("checkmake_tests.rs");
