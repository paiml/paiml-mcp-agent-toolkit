#![cfg_attr(coverage_nightly, coverage(off))]
//! Core type aliases, constants, and fundamental types for the unified AST.

use serde::{Deserialize, Serialize};

/// Unique identifier for AST nodes
pub type NodeKey = u32;

/// Invalid node key constant
pub const INVALID_NODE_KEY: NodeKey = u32::MAX;

/// Language identifier
/// Apply Kaizen - Add support for additional project file types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Language {
    Rust = 0,
    TypeScript = 1,
    JavaScript = 2,
    Python = 3,
    // Kaizen improvement - Add project documentation and configuration languages
    Markdown = 4,
    Makefile = 5,
    Toml = 6,
    Yaml = 7,
    Json = 8,
    Shell = 9,
    C = 10,
    Cpp = 11,
    Cython = 12,
    Kotlin = 13,
    AssemblyScript = 14,
    WebAssembly = 15,
    Lua = 16,
    Sql = 17,
    Scala = 18,
}

/// Node flags for quick filtering and AST node categorization
///
/// Provides efficient bitwise operations for marking AST nodes with language-specific
/// attributes like async/await, visibility modifiers, and semantic properties.
///
/// # Examples
///
/// Basic flag manipulation:
/// ```rust,no_run
/// use pmat::models::unified_ast::NodeFlags;
///
/// let mut flags = NodeFlags::new();
/// assert!(!flags.has(NodeFlags::ASYNC));
///
/// flags.set(NodeFlags::ASYNC);
/// assert!(flags.has(NodeFlags::ASYNC));
///
/// flags.unset(NodeFlags::ASYNC);
/// assert!(!flags.has(NodeFlags::ASYNC));
/// ```
///
/// Combining multiple flags:
/// ```rust,no_run
/// use pmat::models::unified_ast::NodeFlags;
///
/// let mut flags = NodeFlags::new();
/// flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
///
/// assert!(flags.has(NodeFlags::ASYNC));
/// assert!(flags.has(NodeFlags::EXPORTED));
/// assert!(!flags.has(NodeFlags::PRIVATE));
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct NodeFlags(u8);

impl NodeFlags {
    pub const ASYNC: u8 = 0b0000_0001;
    pub const GENERATOR: u8 = 0b0000_0010;
    pub const ABSTRACT: u8 = 0b0000_0100;
    pub const STATIC: u8 = 0b0000_1000;
    pub const CONST: u8 = 0b0001_0000;
    pub const EXPORTED: u8 = 0b0010_0000;
    pub const PRIVATE: u8 = 0b0100_0000;
    pub const DEPRECATED: u8 = 0b1000_0000;

    // Node type flags - these indicate what kind of AST node this is
    pub const FUNCTION: u8 = 0b0000_0001;
    pub const STRUCT: u8 = 0b0000_0010;
    pub const CLASS: u8 = 0b0000_0100;
    pub const ENUM: u8 = 0b0000_1000;
    pub const TRAIT: u8 = 0b0001_0000;
    pub const INTERFACE: u8 = 0b0010_0000;
    pub const IMPORT: u8 = 0b0100_0000;
    pub const CONTROL_FLOW: u8 = 0b1000_0000;
    pub const TYPE_ALIAS: u8 = 0b0000_0001;
    pub const IMPL: u8 = 0b0000_0010;

    // C-specific flags (using a second byte in future if needed)
    pub const INLINE: u8 = 0b00000001; // inline function
    pub const VOLATILE: u8 = 0b00000010; // volatile variable
    pub const RESTRICT: u8 = 0b00000100; // restrict pointer
    pub const EXTERN: u8 = 0b00001000; // extern linkage

    // C++-specific flags (can overlap with C flags as they're language-specific)
    pub const VIRTUAL: u8 = 0b00000001; // virtual function
    pub const OVERRIDE: u8 = 0b00000010; // override specifier
    pub const FINAL: u8 = 0b00000100; // final specifier
    pub const MUTABLE: u8 = 0b00001000; // mutable member
    pub const CONSTEXPR: u8 = 0b00010000; // constexpr
    pub const NOEXCEPT: u8 = 0b00100000; // noexcept

    /// Creates a new `NodeFlags` instance with no flags set
    ///
    /// # Examples
    /// ```rust
    /// use pmat::models::unified_ast::NodeFlags;
    ///
    /// let flags = NodeFlags::new();
    /// assert!(!flags.has(NodeFlags::ASYNC));
    /// assert!(!flags.has(NodeFlags::EXPORTED));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self(0)
    }

    /// Sets the specified flag(s) using bitwise OR
    ///
    /// # Examples
    /// ```rust
    /// use pmat::models::unified_ast::NodeFlags;
    ///
    /// let mut flags = NodeFlags::new();
    /// flags.set(NodeFlags::ASYNC);
    /// assert!(flags.has(NodeFlags::ASYNC));
    ///
    /// // Set multiple flags at once
    /// flags.set(NodeFlags::EXPORTED | NodeFlags::CONST);
    /// assert!(flags.has(NodeFlags::EXPORTED));
    /// assert!(flags.has(NodeFlags::CONST));
    /// ```
    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }

    /// Unsets the specified flag(s) using bitwise AND NOT
    ///
    /// # Examples
    /// ```rust
    /// use pmat::models::unified_ast::NodeFlags;
    ///
    /// let mut flags = NodeFlags::new();
    /// flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
    /// assert!(flags.has(NodeFlags::ASYNC));
    /// assert!(flags.has(NodeFlags::EXPORTED));
    ///
    /// flags.unset(NodeFlags::ASYNC);
    /// assert!(!flags.has(NodeFlags::ASYNC));
    /// assert!(flags.has(NodeFlags::EXPORTED)); // Other flags preserved
    /// ```
    pub fn unset(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    /// Checks if any of the specified flag(s) are set
    ///
    /// # Examples
    /// ```rust
    /// use pmat::models::unified_ast::NodeFlags;
    ///
    /// let mut flags = NodeFlags::new();
    /// flags.set(NodeFlags::ASYNC);
    ///
    /// assert!(flags.has(NodeFlags::ASYNC));
    /// assert!(!flags.has(NodeFlags::EXPORTED));
    ///
    /// // Check multiple flags (returns true if ANY are set)
    /// assert!(flags.has(NodeFlags::ASYNC | NodeFlags::EXPORTED));
    /// ```
    #[must_use]
    pub fn has(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}
