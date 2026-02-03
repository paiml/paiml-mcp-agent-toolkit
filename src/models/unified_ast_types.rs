// Unified AST representation for cross-language code analysis
//
// This module provides a language-agnostic AST representation that enables
// consistent analysis across Rust, TypeScript/JavaScript, and Python codebases.
// Enhanced with formal verification metadata for proof-enriched ASTs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
pub enum ClassKind {
    Regular,
    Abstract,
    Interface,
    Trait,
    Enum,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarKind {
    Let,
    Const,
    Static,
    Field,
    Parameter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    Module,
    Named,
    Default,
    Namespace,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub enum ModuleKind {
    File,
    Namespace,
    Package,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroKind {
    ObjectLike,   // #define PI 3.14
    FunctionLike, // #define MAX(a,b) ((a)>(b)?(a):(b))
    Variadic,     // #define DEBUG(...) fprintf(stderr, __VA_ARGS__)
    Include,      // #include <stdio.h>
    Conditional,  // #ifdef, #ifndef, #if, #elif, #else, #endif
    Export,       // TypeScript export macro
    Decorator,    // TypeScript decorator
}

/// Proof annotation system for formal verification metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofAnnotation {
    #[serde(rename = "annotationId")]
    pub annotation_id: Uuid,

    #[serde(rename = "propertyProven")]
    pub property_proven: PropertyType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub specification_id: Option<String>,

    pub method: VerificationMethod,

    #[serde(rename = "toolName")]
    pub tool_name: String,

    #[serde(rename = "toolVersion")]
    pub tool_version: String,

    #[serde(rename = "confidenceLevel")]
    pub confidence_level: ConfidenceLevel,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,

    #[serde(rename = "evidenceType")]
    pub evidence_type: EvidenceType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_location: Option<String>,

    #[serde(rename = "dateVerified")]
    pub date_verified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PropertyType {
    MemorySafety,
    ThreadSafety,
    DataRaceFreeze,
    Termination,
    FunctionalCorrectness(String), // spec_id
    ResourceBounds {
        cpu: Option<u64>,
        memory: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ConfidenceLevel {
    Low = 1,    // Heuristic-based (e.g., pattern matching)
    Medium = 2, // Sound static analysis with assumptions
    High = 3,   // Machine-checkable proof
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationMethod {
    BorrowChecker,
    FormalProof { prover: String },
    StaticAnalysis { tool: String },
    ModelChecking { bounded: bool },
    AbstractInterpretation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    ImplicitTypeSystemGuarantee,
    ProofScriptReference {
        uri: String,
    },
    TheoremName {
        theorem: String,
        theory: Option<String>,
    },
    StaticAnalysisReport {
        report_id: String,
    },
    CertificateHash {
        hash: String,
        algorithm: String,
    },
}

/// Location system for precise code positioning
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub file_path: PathBuf,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePos(pub u32);

impl std::hash::Hash for Location {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Content-addressed hashing for deterministic cache keys
        self.file_path.hash(state);
        self.span.start.0.hash(state);
        // End position omitted for prefix matching scenarios
    }
}

impl Location {
    /// Creates a new location from a file path and byte positions.
    ///
    /// # Parameters
    ///
    /// * `file_path` - The path to the source file
    /// * `start` - Starting byte position in the file
    /// * `end` - Ending byte position in the file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{Location, BytePos, Span};
    /// use std::path::PathBuf;
    ///
    /// let location = Location::new(
    ///     PathBuf::from("src/main.rs"),
    ///     100,
    ///     150
    /// );
    ///
    /// assert_eq!(location.file_path, PathBuf::from("src/main.rs"));
    /// assert_eq!(location.span.start.0, 100);
    /// assert_eq!(location.span.end.0, 150);
    /// assert_eq!(location.span.len(), 50);
    /// ```
    #[must_use]
    pub fn new(file_path: PathBuf, start: u32, end: u32) -> Self {
        Self {
            file_path,
            span: Span {
                start: BytePos(start),
                end: BytePos(end),
            },
        }
    }

    /// Checks if this location completely contains another location.
    ///
    /// Two locations must be in the same file, and this location's span
    /// must completely encompass the other location's span.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::Location;
    /// use std::path::PathBuf;
    ///
    /// let file = PathBuf::from("test.rs");
    /// let outer = Location::new(file.clone(), 0, 100);
    /// let inner = Location::new(file.clone(), 10, 50);
    /// let separate = Location::new(PathBuf::from("other.rs"), 0, 100);
    ///
    /// assert!(outer.contains(&inner));
    /// assert!(!inner.contains(&outer));
    /// assert!(!outer.contains(&separate)); // Different files
    /// ```
    #[must_use]
    pub fn contains(&self, other: &Location) -> bool {
        self.file_path == other.file_path
            && self.span.start <= other.span.start
            && self.span.end >= other.span.end
    }

    /// Checks if this location overlaps with another location.
    ///
    /// Two locations overlap if they are in the same file and their
    /// byte ranges intersect (even partially).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{Location, BytePos, Span};
    /// use std::path::PathBuf;
    ///
    /// let file = PathBuf::from("test.rs");
    /// let loc1 = Location::new(file.clone(), 0, 50);
    /// let loc2 = Location::new(file.clone(), 25, 75); // Overlaps
    /// let loc3 = Location::new(file.clone(), 100, 150); // No overlap
    /// let loc4 = Location::new(PathBuf::from("other.rs"), 0, 100); // Different file
    ///
    /// assert!(loc1.overlaps(&loc2));
    /// assert!(loc2.overlaps(&loc1));
    /// assert!(!loc1.overlaps(&loc3));
    /// assert!(!loc1.overlaps(&loc4));
    /// ```
    #[must_use]
    pub fn overlaps(&self, other: &Location) -> bool {
        self.file_path == other.file_path
            && self.span.start < other.span.end
            && other.span.start < self.span.end
    }
}

impl Span {
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start: BytePos(start),
            end: BytePos(end),
        }
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.end.0 - self.start.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.0 >= self.end.0
    }

    #[must_use]
    pub fn contains(&self, pos: BytePos) -> bool {
        self.start <= pos && pos < self.end
    }
}

impl BytePos {
    #[must_use]
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    /// Creates a `BytePos` from a usize position
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::BytePos;
    ///
    /// let pos = BytePos::from_usize(42);
    /// assert_eq!(pos.to_usize(), 42);
    /// ```
    #[must_use]
    pub fn from_usize(pos: usize) -> Self {
        Self(pos as u32)
    }
}

/// Qualified name for symbol resolution
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QualifiedName {
    pub module_path: Vec<String>,
    pub name: String,
    pub disambiguator: Option<u32>, // For overloaded names
}

impl QualifiedName {
    /// Creates a new qualified name from module path and name components.
    ///
    /// # Parameters
    ///
    /// * `module_path` - Vector of module/namespace components
    /// * `name` - The final name component (function, type, etc.)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::QualifiedName;
    ///
    /// let qname = QualifiedName::new(
    ///     vec!["std".to_string(), "collections".to_string()],
    ///     "HashMap".to_string()
    /// );
    ///
    /// assert_eq!(qname.module_path, vec!["std", "collections"]);
    /// assert_eq!(qname.name, "HashMap");
    /// assert!(qname.disambiguator.is_none());
    /// assert_eq!(qname.to_qualified_string(), "std::collections::HashMap");
    /// ```
    #[must_use]
    pub fn new(module_path: Vec<String>, name: String) -> Self {
        Self {
            module_path,
            name,
            disambiguator: None,
        }
    }

    #[must_use]
    pub fn with_disambiguator(mut self, disambiguator: u32) -> Self {
        self.disambiguator = Some(disambiguator);
        self
    }

    /// Creates a qualified name from a string representation.
    ///
    /// Parses strings in the format "`module::submodule::Name`" where
    /// "::" separates module components from the final name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::QualifiedName;
    ///
    /// // Simple name without module path
    /// let simple = QualifiedName::from_string("main").expect("internal error");
    /// assert_eq!(simple.name, "main");
    /// assert!(simple.module_path.is_empty());
    ///
    /// // Fully qualified name
    /// let qualified = QualifiedName::from_string("std::collections::HashMap").expect("internal error");
    /// assert_eq!(qualified.module_path, vec!["std", "collections"]);
    /// assert_eq!(qualified.name, "HashMap");
    ///
    /// // Error case
    /// assert!(QualifiedName::from_string("").is_err());
    /// ```
    pub fn from_string(qualified_str: &str) -> Result<Self, &'static str> {
        if qualified_str.is_empty() {
            return Err("Empty qualified name");
        }

        let parts: Vec<&str> = qualified_str.split("::").collect();
        let name = (*parts.last().expect("internal error")).to_string();
        if name.is_empty() {
            return Err("Empty qualified name");
        }

        let module_path = parts[..parts.len() - 1]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        Ok(Self {
            module_path,
            name,
            disambiguator: None,
        })
    }

    /// Converts the qualified name back to its string representation.
    ///
    /// Creates a string in the format "`module::submodule::Name`", with
    /// optional disambiguator suffix "#N" for overloaded names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::QualifiedName;
    ///
    /// let qname = QualifiedName::new(
    ///     vec!["crate".to_string(), "module".to_string()],
    ///     "function".to_string()
    /// );
    /// assert_eq!(qname.to_qualified_string(), "crate::module::function");
    ///
    /// // With disambiguator
    /// let overloaded = qname.with_disambiguator(1);
    /// assert_eq!(overloaded.to_qualified_string(), "crate::module::function#1");
    ///
    /// // Simple name without modules
    /// let simple = QualifiedName::new(vec![], "main".to_string());
    /// assert_eq!(simple.to_qualified_string(), "main");
    /// ```
    #[must_use]
    pub fn to_qualified_string(&self) -> String {
        let mut result = self.module_path.join("::");
        if !result.is_empty() {
            result.push_str("::");
        }
        result.push_str(&self.name);
        if let Some(disambiguator) = self.disambiguator {
            result.push_str(&format!("#{disambiguator}"));
        }
        result
    }
}

impl std::str::FromStr for QualifiedName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_qualified_string())
    }
}

/// Relative location types for companion files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelativeLocation {
    Function {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        module: Option<String>,
    },
    Symbol {
        qualified_name: String, // e.g., "crate::module::Type::method"
    },
    Span {
        start: u32,
        end: u32,
    },
}

/// Type alias for proof mappings
pub type ProofMap = HashMap<Location, Vec<ProofAnnotation>>;

/// Node metadata union for language-specific data
#[repr(C)]
pub union NodeMetadata {
    pub complexity: u64,
    pub hash: u64,
    pub flags: u64,
    pub raw: u64,
}

// Safe default for union
impl Default for NodeMetadata {
    fn default() -> Self {
        Self { raw: 0 }
    }
}

// Manual Clone implementation for union
impl Clone for NodeMetadata {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for NodeMetadata {}

/// Unified AST node representation
///
/// This structure is carefully designed to be:
/// - Cache-line aligned (64 bytes + proof annotations)
/// - SIMD-friendly for vectorized operations
/// - Memory efficient with bit-packed fields
/// - Enhanced with formal verification metadata
#[repr(C, align(32))]
#[derive(Clone)]
pub struct UnifiedAstNode {
    // Core node data - 32 bytes aligned
    pub kind: AstKind,            // 2 bytes - language-agnostic
    pub lang: Language,           // 1 byte
    pub flags: NodeFlags,         // 1 byte
    pub parent: NodeKey,          // 4 bytes
    pub first_child: NodeKey,     // 4 bytes
    pub next_sibling: NodeKey,    // 4 bytes
    pub source_range: Range<u32>, // 8 bytes

    // Semantic data - 32 bytes
    pub semantic_hash: u64,     // 8 bytes - content hash
    pub structural_hash: u64,   // 8 bytes - structure hash
    pub name_vector: u64,       // 8 bytes - packed name embedding
    pub metadata: NodeMetadata, // 8 bytes - union type

    // Proof annotations - sparse allocation for performance
    pub proof_annotations: Option<Vec<ProofAnnotation>>,
}

impl UnifiedAstNode {
    /// Creates a new unified AST node with the specified kind and language.
    ///
    /// Initializes all fields to default values. The node is created with:
    /// - No parent, children, or siblings (all keys set to 0)
    /// - Empty source range (0..0)
    /// - Zero semantic and structural hashes
    /// - Default flags and metadata
    /// - No proof annotations
    ///
    /// # Performance
    ///
    /// - Memory: 64-128 bytes (cache-line aligned)
    /// - Time: O(1) initialization
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, FunctionKind, Language, ClassKind
    /// };
    ///
    /// let func_node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    ///
    /// assert!(func_node.is_function());
    /// assert_eq!(func_node.lang, Language::Rust);
    /// assert_eq!(func_node.parent, 0);
    /// assert_eq!(func_node.source_range, 0..0);
    /// assert!(!func_node.has_proof_annotations());
    ///
    /// let class_node = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::TypeScript
    /// );
    ///
    /// assert!(class_node.is_type_definition());
    /// assert!(!class_node.is_function());
    /// ```
    #[must_use]
    pub fn new(kind: AstKind, lang: Language) -> Self {
        Self {
            kind,
            lang,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..0,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: NodeMetadata::default(),
            proof_annotations: None,
        }
    }

    /// Checks if this node represents a function-like construct.
    ///
    /// Returns true for any node with `AstKind::Function`, regardless
    /// of the specific function kind (regular, method, constructor, etc.).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, FunctionKind, ClassKind, Language
    /// };
    ///
    /// let function = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// assert!(function.is_function());
    ///
    /// let method = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Method),
    ///     Language::TypeScript
    /// );
    /// assert!(method.is_function());
    ///
    /// let class = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::Rust
    /// );
    /// assert!(!class.is_function());
    /// ```
    #[must_use]
    pub fn is_function(&self) -> bool {
        matches!(self.kind, AstKind::Function(_))
    }

    /// Checks if this node represents a type definition.
    ///
    /// Returns true for classes, type aliases, modules, and other
    /// type-defining constructs across all supported languages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, ClassKind, TypeKind,
    ///     ModuleKind, FunctionKind, Language
    /// };
    ///
    /// let struct_node = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::Rust
    /// );
    /// assert!(struct_node.is_type_definition());
    ///
    /// let interface = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Interface),
    ///     Language::TypeScript
    /// );
    /// assert!(interface.is_type_definition());
    ///
    /// let type_alias = UnifiedAstNode::new(
    ///     AstKind::Type(TypeKind::Alias),
    ///     Language::Rust
    /// );
    /// assert!(type_alias.is_type_definition());
    ///
    /// let module = UnifiedAstNode::new(
    ///     AstKind::Module(ModuleKind::File),
    ///     Language::Python
    /// );
    /// assert!(module.is_type_definition());
    ///
    /// let function = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// assert!(!function.is_type_definition());
    /// ```
    #[must_use]
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self.kind,
            AstKind::Class(_) | AstKind::Type(_) | AstKind::Module(_)
        )
    }

    /// Get the complexity score for this node
    #[must_use]
    pub fn complexity(&self) -> u32 {
        // SAFETY: Accessing the complexity field of the metadata union.
        // This is safe because:
        // 1. The metadata union is properly initialized in all constructors
        // 2. We only read the lower 32 bits which are always valid u32 values
        // 3. The bit mask (0xFFFFFFFF) ensures we only access the complexity portion
        unsafe { (self.metadata.complexity & 0xFFFFFFFF) as u32 }
    }

    /// Set the complexity score for this node
    pub fn set_complexity(&mut self, complexity: u32) {
        self.metadata.complexity = u64::from(complexity);
    }

    /// Adds a formal verification proof annotation to this node.
    ///
    /// Proof annotations provide metadata about formally verified properties
    /// of the code represented by this AST node. Multiple annotations can
    /// be added to track different verified properties.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, FunctionKind, Language,
    ///     ProofAnnotation, PropertyType, VerificationMethod,
    ///     ConfidenceLevel, EvidenceType
    /// };
    /// use uuid::Uuid;
    /// use chrono::Utc;
    ///
    /// let mut node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    ///
    /// assert!(!node.has_proof_annotations());
    ///
    /// let annotation = ProofAnnotation {
    ///     annotation_id: Uuid::new_v4(),
    ///     property_proven: PropertyType::MemorySafety,
    ///     specification_id: Some("memory_safety_spec_v1".to_string()),
    ///     method: VerificationMethod::BorrowChecker,
    ///     tool_name: "rustc".to_string(),
    ///     tool_version: "1.70.0".to_string(),
    ///     confidence_level: ConfidenceLevel::High,
    ///     assumptions: vec![],
    ///     evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
    ///     evidence_location: None,
    ///     date_verified: Utc::now(),
    /// };
    ///
    /// node.add_proof_annotation(annotation);
    ///
    /// assert!(node.has_proof_annotations());
    /// assert_eq!(node.proof_annotations().len(), 1);
    /// ```
    pub fn add_proof_annotation(&mut self, annotation: ProofAnnotation) {
        match &mut self.proof_annotations {
            Some(annotations) => annotations.push(annotation),
            None => self.proof_annotations = Some(vec![annotation]),
        }
    }

    /// Get all proof annotations for this node
    #[must_use]
    pub fn proof_annotations(&self) -> &[ProofAnnotation] {
        self.proof_annotations.as_deref().unwrap_or(&[])
    }

    /// Check if this node has proof annotations
    #[must_use]
    pub fn has_proof_annotations(&self) -> bool {
        self.proof_annotations
            .as_ref()
            .is_some_and(|annotations| !annotations.is_empty())
    }

    /// Get location for this node (requires file path context)
    #[must_use]
    pub fn location(&self, file_path: &Path) -> Location {
        Location {
            file_path: file_path.to_path_buf(),
            span: Span {
                start: BytePos(self.source_range.start),
                end: BytePos(self.source_range.end),
            },
        }
    }
}

// Manual Debug implementation for UnifiedAstNode
impl std::fmt::Debug for UnifiedAstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedAstNode")
            .field("kind", &self.kind)
            .field("lang", &self.lang)
            .field("flags", &self.flags)
            .field("parent", &self.parent)
            .field("first_child", &self.first_child)
            .field("next_sibling", &self.next_sibling)
            .field("source_range", &self.source_range)
            .field("semantic_hash", &self.semantic_hash)
            .field("structural_hash", &self.structural_hash)
            .field("name_vector", &self.name_vector)
            // SAFETY: Accessing the raw field of the metadata union for debug output.
            // This is safe because:
            // 1. We only read the raw bytes for display purposes
            // 2. The metadata union is properly initialized
            // 3. Reading raw bytes does not violate memory safety
            .field("metadata_raw", &unsafe { self.metadata.raw })
            .field("proof_annotations", &self.proof_annotations)
            .finish()
    }
}

/// Column-oriented storage for SIMD operations
pub struct ColumnStore<T> {
    data: Vec<T>,
    #[allow(dead_code)]
    capacity: usize,
}

impl<T: Clone> ColumnStore<T> {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) -> NodeKey {
        let key = self.data.len() as NodeKey;
        self.data.push(item);
        key
    }

    #[must_use]
    pub fn get(&self, key: NodeKey) -> Option<&T> {
        self.data.get(key as usize)
    }

    pub fn get_mut(&mut self, key: NodeKey) -> Option<&mut T> {
        self.data.get_mut(key as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// AST DAG structure for efficient traversal and analysis
pub struct AstDag {
    /// Columnar storage for SIMD operations
    pub nodes: ColumnStore<UnifiedAstNode>,

    /// Language-specific parsers for multi-language support
    pub parsers: LanguageParsers,

    /// Incremental update tracking
    pub dirty_nodes: roaring::RoaringBitmap,

    /// Generation counter for cache invalidation
    pub generation: std::sync::atomic::AtomicU32,
}

impl Default for AstDag {
    fn default() -> Self {
        Self::new()
    }
}

impl AstDag {
    /// Creates a new AST DAG with default configuration.
    ///
    /// Initializes an empty DAG with:
    /// - Column store with 10,000 initial node capacity
    /// - Empty roaring bitmap for dirty node tracking
    /// - Generation counter starting at 0
    /// - Default language parsers
    ///
    /// # Performance Characteristics
    ///
    /// - Memory: ~40MB initial allocation for node storage
    /// - Insertion: O(1) amortized with occasional reallocation
    /// - Lookup: O(1) by node key
    /// - Dirty tracking: O(1) insertion/removal with compressed bitmaps
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     AstDag, UnifiedAstNode, AstKind, FunctionKind, Language
    /// };
    ///
    /// let mut dag = AstDag::new();
    ///
    /// // Initially empty
    /// assert_eq!(dag.nodes.len(), 0);
    /// assert!(dag.nodes.is_empty());
    /// assert_eq!(dag.generation(), 0);
    ///
    /// // Add a node
    /// let node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// let key = dag.add_node(node);
    ///
    /// assert_eq!(dag.nodes.len(), 1);
    /// assert_eq!(dag.generation(), 1);
    /// assert!(dag.dirty_nodes().any(|k| k == key));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: ColumnStore::new(10000), // Initial capacity
            parsers: LanguageParsers::default(),
            dirty_nodes: roaring::RoaringBitmap::new(),
            generation: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Adds a new node to the DAG and returns its unique key.
    ///
    /// The node is automatically marked as dirty and the generation
    /// counter is incremented for cache invalidation.
    ///
    /// # Performance
    ///
    /// - Time: O(1) amortized, O(n) worst case during reallocation
    /// - Space: Constant overhead per node
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     AstDag, UnifiedAstNode, AstKind, FunctionKind, ClassKind, Language
    /// };
    ///
    /// let mut dag = AstDag::new();
    /// let initial_gen = dag.generation();
    ///
    /// // Add multiple nodes
    /// let func_node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// let class_node = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::Rust
    /// );
    ///
    /// let func_key = dag.add_node(func_node);
    /// let class_key = dag.add_node(class_node);
    ///
    /// // Keys are unique and sequential
    /// assert_ne!(func_key, class_key);
    /// assert_eq!(dag.nodes.len(), 2);
    ///
    /// // Generation incremented for each addition
    /// assert_eq!(dag.generation(), initial_gen + 2);
    ///
    /// // Both nodes are dirty
    /// let dirty: Vec<_> = dag.dirty_nodes().collect();
    /// assert_eq!(dirty.len(), 2);
    /// assert!(dirty.contains(&func_key));
    /// assert!(dirty.contains(&class_key));
    ///
    /// // Nodes can be retrieved by key
    /// assert!(dag.nodes.get(func_key).expect("internal error").is_function());
    /// assert!(dag.nodes.get(class_key).expect("internal error").is_type_definition());
    /// ```
    pub fn add_node(&mut self, node: UnifiedAstNode) -> NodeKey {
        let key = self.nodes.push(node);
        self.dirty_nodes.insert(key);
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        key
    }

    /// Marks a node as clean (processed) by removing it from the dirty set.
    ///
    /// This is typically called after processing a node for incremental
    /// analysis to avoid reprocessing unchanged nodes.
    ///
    /// # Performance
    ///
    /// - Time: O(1) for roaring bitmap removal
    /// - Space: No additional allocation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     AstDag, UnifiedAstNode, AstKind, FunctionKind, Language
    /// };
    ///
    /// let mut dag = AstDag::new();
    ///
    /// let node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// let key = dag.add_node(node);
    ///
    /// // Node starts dirty
    /// assert!(dag.dirty_nodes().any(|k| k == key));
    ///
    /// // Mark as processed
    /// dag.mark_clean(key);
    ///
    /// // No longer in dirty set
    /// assert!(!dag.dirty_nodes().any(|k| k == key));
    ///
    /// // Node still exists in the DAG
    /// assert!(dag.nodes.get(key).is_some());
    /// ```
    pub fn mark_clean(&mut self, key: NodeKey) {
        self.dirty_nodes.remove(key);
    }

    /// Returns an iterator over all dirty (unprocessed) node keys.
    ///
    /// Dirty nodes are those that have been added or modified since
    /// the last processing cycle. This enables efficient incremental
    /// analysis by only processing changed nodes.
    ///
    /// # Performance
    ///
    /// - Time: O(1) to create iterator, O(k) to iterate where k = dirty count
    /// - Space: No additional allocation (streaming iterator)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     AstDag, UnifiedAstNode, AstKind, FunctionKind, Language
    /// };
    ///
    /// let mut dag = AstDag::new();
    ///
    /// // Initially no dirty nodes
    /// assert_eq!(dag.dirty_nodes().count(), 0);
    ///
    /// // Add some nodes
    /// let keys: Vec<_> = (0..3).map(|_| {
    ///     dag.add_node(UnifiedAstNode::new(
    ///         AstKind::Function(FunctionKind::Regular),
    ///         Language::Rust
    ///     ))
    /// }).collect();
    ///
    /// // All nodes are dirty
    /// assert_eq!(dag.dirty_nodes().count(), 3);
    ///
    /// // Process some nodes
    /// dag.mark_clean(keys[0]);
    /// dag.mark_clean(keys[2]);
    ///
    /// // Only unprocessed nodes remain dirty
    /// let dirty: Vec<_> = dag.dirty_nodes().collect();
    /// assert_eq!(dirty.len(), 1);
    /// assert_eq!(dirty[0], keys[1]);
    /// ```
    pub fn dirty_nodes(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.dirty_nodes.iter()
    }

    /// Returns the current generation number for cache invalidation.
    ///
    /// The generation number is incremented each time a node is added
    /// to the DAG, providing a monotonic cache invalidation key.
    ///
    /// # Performance
    ///
    /// - Time: O(1) atomic load with relaxed ordering
    /// - Thread-safe: Can be called from multiple threads
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     AstDag, UnifiedAstNode, AstKind, FunctionKind, Language
    /// };
    ///
    /// let mut dag = AstDag::new();
    ///
    /// // Starts at generation 0
    /// assert_eq!(dag.generation(), 0);
    ///
    /// // Generation increments with each node addition
    /// dag.add_node(UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// ));
    /// assert_eq!(dag.generation(), 1);
    ///
    /// dag.add_node(UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Method),
    ///     Language::TypeScript
    /// ));
    /// assert_eq!(dag.generation(), 2);
    ///
    /// // Marking clean does not change generation
    /// dag.mark_clean(0);
    /// assert_eq!(dag.generation(), 2);
    /// ```
    pub fn generation(&self) -> u32 {
        self.generation.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Placeholder for language-specific parsers
#[derive(Default)]
pub struct LanguageParsers {
    // TRACKED: Add actual parser implementations
}

#[cfg(test)]
mod unified_ast_types_tests {
    use super::*;

    // ============================================================================
    // Language Tests
    // ============================================================================

    #[test]
    fn test_language_variants() {
        assert_eq!(Language::Rust as u8, 0);
        assert_eq!(Language::TypeScript as u8, 1);
        assert_eq!(Language::JavaScript as u8, 2);
        assert_eq!(Language::Python as u8, 3);
        assert_eq!(Language::Markdown as u8, 4);
        assert_eq!(Language::WebAssembly as u8, 15);
    }

    #[test]
    fn test_language_equality() {
        assert_eq!(Language::Rust, Language::Rust);
        assert_ne!(Language::Rust, Language::Python);
    }

    #[test]
    fn test_language_serialization() {
        let lang = Language::TypeScript;
        let json = serde_json::to_string(&lang).unwrap();
        let deserialized: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(lang, deserialized);
    }

    // ============================================================================
    // NodeFlags Tests
    // ============================================================================

    #[test]
    fn test_node_flags_new() {
        let flags = NodeFlags::new();
        assert!(!flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));
    }

    #[test]
    fn test_node_flags_set() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_set_multiple() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));
    }

    #[test]
    fn test_node_flags_unset() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
        flags.unset(NodeFlags::ASYNC);
        assert!(!flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_default() {
        let flags = NodeFlags::default();
        assert!(!flags.has(NodeFlags::ASYNC));
    }

    #[test]
    fn test_node_flags_all_flags() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        flags.set(NodeFlags::GENERATOR);
        flags.set(NodeFlags::ABSTRACT);
        flags.set(NodeFlags::STATIC);
        flags.set(NodeFlags::CONST);
        flags.set(NodeFlags::EXPORTED);
        flags.set(NodeFlags::PRIVATE);
        flags.set(NodeFlags::DEPRECATED);

        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::GENERATOR));
        assert!(flags.has(NodeFlags::ABSTRACT));
        assert!(flags.has(NodeFlags::STATIC));
        assert!(flags.has(NodeFlags::CONST));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(flags.has(NodeFlags::PRIVATE));
        assert!(flags.has(NodeFlags::DEPRECATED));
    }

    // ============================================================================
    // FunctionKind Tests
    // ============================================================================

    #[test]
    fn test_function_kind_variants() {
        assert_eq!(FunctionKind::Regular, FunctionKind::Regular);
        assert_ne!(FunctionKind::Regular, FunctionKind::Method);
        assert_ne!(FunctionKind::Constructor, FunctionKind::Destructor);
    }

    #[test]
    fn test_function_kind_serialization() {
        let kind = FunctionKind::Lambda;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: FunctionKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    // ============================================================================
    // ClassKind Tests
    // ============================================================================

    #[test]
    fn test_class_kind_variants() {
        assert_eq!(ClassKind::Regular, ClassKind::Regular);
        assert_ne!(ClassKind::Interface, ClassKind::Trait);
        assert_ne!(ClassKind::Struct, ClassKind::Enum);
    }

    #[test]
    fn test_class_kind_serialization() {
        let kind = ClassKind::Abstract;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: ClassKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    // ============================================================================
    // BytePos Tests
    // ============================================================================

    #[test]
    fn test_byte_pos_creation() {
        let pos = BytePos(100);
        assert_eq!(pos.0, 100);
    }

    #[test]
    fn test_byte_pos_to_usize() {
        let pos = BytePos(42);
        assert_eq!(pos.to_usize(), 42);
    }

    #[test]
    fn test_byte_pos_from_usize() {
        let pos = BytePos::from_usize(256);
        assert_eq!(pos.0, 256);
    }

    #[test]
    fn test_byte_pos_ordering() {
        let pos1 = BytePos(10);
        let pos2 = BytePos(20);
        assert!(pos1 < pos2);
        assert!(pos2 > pos1);
        assert!(pos1 <= pos1);
        assert!(pos1 >= pos1);
    }

    #[test]
    fn test_byte_pos_equality() {
        let pos1 = BytePos(50);
        let pos2 = BytePos(50);
        assert_eq!(pos1, pos2);
    }

    // ============================================================================
    // Span Tests
    // ============================================================================

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 50);
        assert_eq!(span.start.0, 10);
        assert_eq!(span.end.0, 50);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(10, 50);
        assert_eq!(span.len(), 40);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(10, 10);
        assert!(empty.is_empty());

        let non_empty = Span::new(10, 20);
        assert!(!non_empty.is_empty());

        let invalid = Span::new(20, 10); // end < start
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 50);
        assert!(span.contains(BytePos(10)));
        assert!(span.contains(BytePos(25)));
        assert!(span.contains(BytePos(49)));
        assert!(!span.contains(BytePos(50)));
        assert!(!span.contains(BytePos(9)));
    }

    // ============================================================================
    // Location Tests
    // ============================================================================

    #[test]
    fn test_location_new() {
        let loc = Location::new(PathBuf::from("test.rs"), 100, 200);
        assert_eq!(loc.file_path, PathBuf::from("test.rs"));
        assert_eq!(loc.span.start.0, 100);
        assert_eq!(loc.span.end.0, 200);
    }

    #[test]
    fn test_location_contains() {
        let outer = Location::new(PathBuf::from("test.rs"), 0, 100);
        let inner = Location::new(PathBuf::from("test.rs"), 10, 50);
        let other_file = Location::new(PathBuf::from("other.rs"), 10, 50);

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
        assert!(!outer.contains(&other_file));
    }

    #[test]
    fn test_location_overlaps() {
        let loc1 = Location::new(PathBuf::from("test.rs"), 0, 50);
        let loc2 = Location::new(PathBuf::from("test.rs"), 25, 75);
        let loc3 = Location::new(PathBuf::from("test.rs"), 100, 150);
        let other_file = Location::new(PathBuf::from("other.rs"), 0, 100);

        assert!(loc1.overlaps(&loc2));
        assert!(loc2.overlaps(&loc1));
        assert!(!loc1.overlaps(&loc3));
        assert!(!loc1.overlaps(&other_file));
    }

    // ============================================================================
    // QualifiedName Tests
    // ============================================================================

    #[test]
    fn test_qualified_name_new() {
        let qname = QualifiedName::new(
            vec!["std".to_string(), "collections".to_string()],
            "HashMap".to_string(),
        );
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
        assert!(qname.disambiguator.is_none());
    }

    #[test]
    fn test_qualified_name_with_disambiguator() {
        let qname = QualifiedName::new(vec![], "function".to_string())
            .with_disambiguator(1);
        assert_eq!(qname.disambiguator, Some(1));
    }

    #[test]
    fn test_qualified_name_from_string() {
        let qname = QualifiedName::from_string("std::collections::HashMap").unwrap();
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
    }

    #[test]
    fn test_qualified_name_from_string_simple() {
        let qname = QualifiedName::from_string("main").unwrap();
        assert!(qname.module_path.is_empty());
        assert_eq!(qname.name, "main");
    }

    #[test]
    fn test_qualified_name_from_string_empty() {
        let result = QualifiedName::from_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_qualified_name_to_qualified_string() {
        let qname = QualifiedName::new(
            vec!["crate".to_string(), "module".to_string()],
            "function".to_string(),
        );
        assert_eq!(qname.to_qualified_string(), "crate::module::function");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_with_disambiguator() {
        let qname = QualifiedName::new(vec!["mod".to_string()], "func".to_string())
            .with_disambiguator(2);
        assert_eq!(qname.to_qualified_string(), "mod::func#2");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_simple() {
        let qname = QualifiedName::new(vec![], "main".to_string());
        assert_eq!(qname.to_qualified_string(), "main");
    }

    #[test]
    fn test_qualified_name_from_str() {
        let qname: QualifiedName = "std::io::Error".parse().unwrap();
        assert_eq!(qname.module_path, vec!["std", "io"]);
        assert_eq!(qname.name, "Error");
    }

    #[test]
    fn test_qualified_name_display() {
        let qname = QualifiedName::new(
            vec!["std".to_string()],
            "Vec".to_string(),
        );
        assert_eq!(format!("{}", qname), "std::Vec");
    }

    // ============================================================================
    // AstKind Tests
    // ============================================================================

    #[test]
    fn test_ast_kind_variants() {
        let func = AstKind::Function(FunctionKind::Regular);
        let class = AstKind::Class(ClassKind::Struct);
        assert_ne!(func, class);
    }

    #[test]
    fn test_ast_kind_serialization() {
        let kind = AstKind::Function(FunctionKind::Method);
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    // ============================================================================
    // ConfidenceLevel Tests
    // ============================================================================

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
    }

    #[test]
    fn test_confidence_level_values() {
        assert_eq!(ConfidenceLevel::Low as u8, 1);
        assert_eq!(ConfidenceLevel::Medium as u8, 2);
        assert_eq!(ConfidenceLevel::High as u8, 3);
    }

    // ============================================================================
    // ColumnStore Tests
    // ============================================================================

    #[test]
    fn test_column_store_new() {
        let store: ColumnStore<i32> = ColumnStore::new(100);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_column_store_push() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        let key0 = store.push(42);
        let key1 = store.push(100);

        assert_eq!(key0, 0);
        assert_eq!(key1, 1);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_column_store_get() {
        let mut store: ColumnStore<String> = ColumnStore::new(10);
        store.push("hello".to_string());
        store.push("world".to_string());

        assert_eq!(store.get(0), Some(&"hello".to_string()));
        assert_eq!(store.get(1), Some(&"world".to_string()));
        assert_eq!(store.get(2), None);
    }

    #[test]
    fn test_column_store_get_mut() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        store.push(10);

        if let Some(val) = store.get_mut(0) {
            *val = 20;
        }

        assert_eq!(store.get(0), Some(&20));
    }

    #[test]
    fn test_column_store_iter() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        store.push(1);
        store.push(2);
        store.push(3);

        let sum: i32 = store.iter().sum();
        assert_eq!(sum, 6);
    }

    // ============================================================================
    // UnifiedAstNode Tests
    // ============================================================================

    #[test]
    fn test_unified_ast_node_new() {
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(node.is_function());
        assert!(!node.is_type_definition());
        assert_eq!(node.lang, Language::Rust);
        assert_eq!(node.parent, 0);
        assert_eq!(node.source_range, 0..0);
    }

    #[test]
    fn test_unified_ast_node_is_function() {
        let func = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Method),
            Language::TypeScript,
        );
        assert!(func.is_function());

        let class = UnifiedAstNode::new(
            AstKind::Class(ClassKind::Regular),
            Language::TypeScript,
        );
        assert!(!class.is_function());
    }

    #[test]
    fn test_unified_ast_node_is_type_definition() {
        let class = UnifiedAstNode::new(
            AstKind::Class(ClassKind::Interface),
            Language::TypeScript,
        );
        assert!(class.is_type_definition());

        let module = UnifiedAstNode::new(
            AstKind::Module(ModuleKind::File),
            Language::Python,
        );
        assert!(module.is_type_definition());

        let func = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(!func.is_type_definition());
    }

    #[test]
    fn test_unified_ast_node_complexity() {
        let mut node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );

        assert_eq!(node.complexity(), 0);

        node.set_complexity(25);
        assert_eq!(node.complexity(), 25);
    }

    #[test]
    fn test_unified_ast_node_proof_annotations() {
        let mut node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );

        assert!(!node.has_proof_annotations());
        assert!(node.proof_annotations().is_empty());

        let annotation = ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: "rustc".to_string(),
            tool_version: "1.70.0".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        };

        node.add_proof_annotation(annotation);

        assert!(node.has_proof_annotations());
        assert_eq!(node.proof_annotations().len(), 1);
    }

    #[test]
    fn test_unified_ast_node_location() {
        let mut node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        node.source_range = 100..200;

        let loc = node.location(Path::new("src/main.rs"));
        assert_eq!(loc.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(loc.span.start.0, 100);
        assert_eq!(loc.span.end.0, 200);
    }

    // ============================================================================
    // AstDag Tests
    // ============================================================================

    #[test]
    fn test_ast_dag_new() {
        let dag = AstDag::new();
        assert!(dag.nodes.is_empty());
        assert_eq!(dag.generation(), 0);
    }

    #[test]
    fn test_ast_dag_default() {
        let dag = AstDag::default();
        assert!(dag.nodes.is_empty());
    }

    #[test]
    fn test_ast_dag_add_node() {
        let mut dag = AstDag::new();
        let initial_gen = dag.generation();

        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        let key = dag.add_node(node);

        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.generation(), initial_gen + 1);
        assert!(dag.nodes.get(key).is_some());
    }

    #[test]
    fn test_ast_dag_mark_clean() {
        let mut dag = AstDag::new();
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        let key = dag.add_node(node);

        // Initially dirty
        assert!(dag.dirty_nodes().any(|k| k == key));

        dag.mark_clean(key);

        // No longer dirty
        assert!(!dag.dirty_nodes().any(|k| k == key));
    }

    #[test]
    fn test_ast_dag_dirty_nodes() {
        let mut dag = AstDag::new();

        let keys: Vec<_> = (0..3).map(|_| {
            dag.add_node(UnifiedAstNode::new(
                AstKind::Function(FunctionKind::Regular),
                Language::Rust,
            ))
        }).collect();

        assert_eq!(dag.dirty_nodes().count(), 3);

        dag.mark_clean(keys[1]);

        assert_eq!(dag.dirty_nodes().count(), 2);
    }

    // ============================================================================
    // NodeMetadata Tests
    // ============================================================================

    #[test]
    fn test_node_metadata_default() {
        let metadata = NodeMetadata::default();
        unsafe {
            assert_eq!(metadata.raw, 0);
        }
    }

    #[test]
    fn test_node_metadata_clone() {
        let metadata = NodeMetadata { complexity: 42 };
        let cloned = metadata;
        unsafe {
            assert_eq!(metadata.complexity, cloned.complexity);
        }
    }

    // ============================================================================
    // LanguageParsers Tests
    // ============================================================================

    #[test]
    fn test_language_parsers_default() {
        let _parsers = LanguageParsers::default();
        // Just verify it can be instantiated
    }

    // ============================================================================
    // PropertyType Tests
    // ============================================================================

    #[test]
    fn test_property_type_equality() {
        assert_eq!(PropertyType::MemorySafety, PropertyType::MemorySafety);
        assert_ne!(PropertyType::MemorySafety, PropertyType::ThreadSafety);
    }

    #[test]
    fn test_property_type_serialization() {
        let prop = PropertyType::Termination;
        let json = serde_json::to_string(&prop).unwrap();
        let deserialized: PropertyType = serde_json::from_str(&json).unwrap();
        assert_eq!(prop, deserialized);
    }

    // ============================================================================
    // VerificationMethod Tests
    // ============================================================================

    #[test]
    fn test_verification_method_equality() {
        assert_eq!(
            VerificationMethod::BorrowChecker,
            VerificationMethod::BorrowChecker
        );
    }

    #[test]
    fn test_verification_method_serialization() {
        let method = VerificationMethod::BorrowChecker;
        let json = serde_json::to_string(&method).unwrap();
        let deserialized: VerificationMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(method, deserialized);
    }

    // ============================================================================
    // EvidenceType Tests
    // ============================================================================

    #[test]
    fn test_evidence_type_equality() {
        assert_eq!(
            EvidenceType::ImplicitTypeSystemGuarantee,
            EvidenceType::ImplicitTypeSystemGuarantee
        );
    }

    #[test]
    fn test_evidence_type_serialization() {
        let evidence = EvidenceType::ProofScriptReference {
            uri: "proof.v".to_string(),
        };
        let json = serde_json::to_string(&evidence).unwrap();
        let deserialized: EvidenceType = serde_json::from_str(&json).unwrap();
        assert_eq!(evidence, deserialized);
    }

    // ============================================================================
    // RelativeLocation Tests
    // ============================================================================

    #[test]
    fn test_relative_location_function() {
        let loc = RelativeLocation::Function {
            name: "main".to_string(),
            module: Some("app".to_string()),
        };

        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: RelativeLocation = serde_json::from_str(&json).unwrap();
        if let RelativeLocation::Function { name, module } = deserialized {
            assert_eq!(name, "main");
            assert_eq!(module, Some("app".to_string()));
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn test_relative_location_span() {
        let loc = RelativeLocation::Span { start: 100, end: 200 };
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: RelativeLocation = serde_json::from_str(&json).unwrap();
        if let RelativeLocation::Span { start, end } = deserialized {
            assert_eq!(start, 100);
            assert_eq!(end, 200);
        } else {
            panic!("Wrong variant");
        }
    }
}

