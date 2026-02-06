#![cfg_attr(coverage_nightly, coverage(off))]
//! Core types for AI-ready context generation.
//!
//! This module contains the fundamental data structures used for representing
//! project context, including AST items, file contexts, and project summaries.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectContext {
    pub project_type: String,
    pub files: Vec<FileContext>,
    pub summary: ProjectSummary,
    /// O(1) graph for symbol lookups and PageRank-based importance
    #[serde(skip)]
    pub graph: Option<crate::services::context_graph::ProjectContextGraph>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub total_files: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_enums: usize,
    pub total_traits: usize,
    pub total_impls: usize,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContext {
    pub path: String,
    pub language: String,
    pub items: Vec<AstItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_metrics: Option<crate::services::complexity::FileComplexityMetrics>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[derive(PartialEq)]
pub enum AstItem {
    Function {
        name: String,
        visibility: String,
        is_async: bool,
        line: usize,
    },
    Struct {
        name: String,
        visibility: String,
        fields_count: usize,
        derives: Vec<String>,
        line: usize,
    },
    Enum {
        name: String,
        visibility: String,
        variants_count: usize,
        line: usize,
    },
    Trait {
        name: String,
        visibility: String,
        line: usize,
    },
    Impl {
        type_name: String,
        trait_name: Option<String>,
        line: usize,
    },
    Module {
        name: String,
        visibility: String,
        line: usize,
    },
    Use {
        path: String,
        line: usize,
    },
    /// Import statement for language-specific module imports
    ///
    /// Supports various import patterns across different languages including
    /// Python, JavaScript, TypeScript, and other languages with module systems.
    ///
    /// # Examples
    ///
    /// ## Python Import Patterns
    ///
    /// ```
    /// use pmat::services::context::AstItem;
    ///
    /// // Simple import: import os
    /// let import1 = AstItem::Import {
    ///     module: "os".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 1,
    /// };
    /// assert_eq!(import1.display_name(), "os");
    ///
    /// // Import with alias: import numpy as np
    /// let import2 = AstItem::Import {
    ///     module: "numpy".to_string(),
    ///     items: vec![],
    ///     alias: Some("np".to_string()),
    ///     line: 2,
    /// };
    /// assert_eq!(import2.display_name(), "numpy");
    ///
    /// // From import: from typing import List, Dict
    /// let import3 = AstItem::Import {
    ///     module: "typing".to_string(),
    ///     items: vec!["List".to_string(), "Dict".to_string()],
    ///     alias: None,
    ///     line: 3,
    /// };
    /// assert_eq!(import3.display_name(), "typing");
    ///
    /// // Submodule import: import os.path
    /// let import4 = AstItem::Import {
    ///     module: "os.path".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 4,
    /// };
    /// assert_eq!(import4.display_name(), "os.path");
    ///
    /// // Wildcard import: from math import *
    /// let import5 = AstItem::Import {
    ///     module: "math".to_string(),
    ///     items: vec!["*".to_string()],
    ///     alias: None,
    ///     line: 5,
    /// };
    /// assert_eq!(import5.display_name(), "math");
    /// ```
    ///
    /// ## JavaScript/TypeScript Import Patterns
    ///
    /// ```
    /// use pmat::services::context::AstItem;
    ///
    /// // ES6 default import: import React from 'react'
    /// let import1 = AstItem::Import {
    ///     module: "react".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 1,
    /// };
    /// assert_eq!(import1.display_name(), "react");
    ///
    /// // Named imports: import { useState, useEffect } from 'react'
    /// let import2 = AstItem::Import {
    ///     module: "react".to_string(),
    ///     items: vec!["useState".to_string(), "useEffect".to_string()],
    ///     alias: None,
    ///     line: 2,
    /// };
    /// assert_eq!(import2.display_name(), "react");
    ///
    /// // Scoped package: import { Button } from '@mui/material'
    /// let import3 = AstItem::Import {
    ///     module: "@mui/material".to_string(),
    ///     items: vec!["Button".to_string()],
    ///     alias: None,
    ///     line: 3,
    /// };
    /// assert_eq!(import3.display_name(), "@mui/material");
    ///
    /// // Relative import: import { utils } from './utils'
    /// let import4 = AstItem::Import {
    ///     module: "./utils".to_string(),
    ///     items: vec!["utils".to_string()],
    ///     alias: None,
    ///     line: 4,
    /// };
    /// assert_eq!(import4.display_name(), "./utils");
    ///
    /// // Parent directory import: import Component from '../components/Button'
    /// let import5 = AstItem::Import {
    ///     module: "../components/Button".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 5,
    /// };
    /// assert_eq!(import5.display_name(), "../components/Button");
    /// ```
    ///
    /// ## Edge Cases and Special Patterns
    ///
    /// ```
    /// use pmat::services::context::AstItem;
    ///
    /// // Empty module (edge case)
    /// let import1 = AstItem::Import {
    ///     module: "".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 1,
    /// };
    /// assert_eq!(import1.display_name(), "");
    ///
    /// // Complex nested module path
    /// let import2 = AstItem::Import {
    ///     module: "matplotlib.pyplot.subplots".to_string(),
    ///     items: vec![],
    ///     alias: Some("plt".to_string()),
    ///     line: 2,
    /// };
    /// assert_eq!(import2.display_name(), "matplotlib.pyplot.subplots");
    ///
    /// // Multiple aliases don't affect display_name
    /// let import3 = AstItem::Import {
    ///     module: "pandas".to_string(),
    ///     items: vec!["DataFrame".to_string(), "Series".to_string()],
    ///     alias: Some("pd".to_string()),
    ///     line: 3,
    /// };
    /// assert_eq!(import3.display_name(), "pandas");
    /// ```
    Import {
        /// What is being imported (module, package, or specific items)
        module: String,
        /// Specific items imported from the module (if any)
        items: Vec<String>,
        /// Alias for the import (if any)
        alias: Option<String>,
        /// Line number
        line: usize,
    },
}

impl AstItem {
    #[must_use]
    pub fn display_name(&self) -> &str {
        match self {
            AstItem::Function { name, .. } => name,
            AstItem::Struct { name, .. } => name,
            AstItem::Enum { name, .. } => name,
            AstItem::Trait { name, .. } => name,
            AstItem::Impl { type_name, .. } => type_name,
            AstItem::Module { name, .. } => name,
            AstItem::Use { path, .. } => path,
            AstItem::Import { module, .. } => module,
        }
    }
}

/// Grouped items by type for formatting
pub(crate) struct GroupedItems<'a> {
    pub functions: Vec<&'a AstItem>,
    pub structs: Vec<&'a AstItem>,
    pub enums: Vec<&'a AstItem>,
    pub traits: Vec<&'a AstItem>,
    pub impls: Vec<&'a AstItem>,
    pub modules: Vec<&'a AstItem>,
}

impl<'a> GroupedItems<'a> {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            modules: Vec::new(),
        }
    }
}
