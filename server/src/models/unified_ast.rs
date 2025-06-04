//! Unified AST representation for cross-language code analysis
//!
//! This module provides a language-agnostic AST representation that enables
//! consistent analysis across Rust, TypeScript/JavaScript, and Python codebases.
//! Enhanced with formal verification metadata for proof-enriched ASTs.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Language {
    Rust = 0,
    TypeScript = 1,
    JavaScript = 2,
    Python = 3,
}

/// Node flags for quick filtering
#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct NodeFlags(u8);

impl NodeFlags {
    pub const ASYNC: u8 = 0b00000001;
    pub const GENERATOR: u8 = 0b00000010;
    pub const ABSTRACT: u8 = 0b00000100;
    pub const STATIC: u8 = 0b00001000;
    pub const CONST: u8 = 0b00010000;
    pub const EXPORTED: u8 = 0b00100000;
    pub const PRIVATE: u8 = 0b01000000;
    pub const DEPRECATED: u8 = 0b10000000;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }

    pub fn unset(&mut self, flag: u8) {
        self.0 &= !flag;
    }

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleKind {
    File,
    Namespace,
    Package,
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
    pub fn new(file_path: PathBuf, start: u32, end: u32) -> Self {
        Self {
            file_path,
            span: Span {
                start: BytePos(start),
                end: BytePos(end),
            },
        }
    }

    pub fn contains(&self, other: &Location) -> bool {
        self.file_path == other.file_path
            && self.span.start <= other.span.start
            && self.span.end >= other.span.end
    }

    pub fn overlaps(&self, other: &Location) -> bool {
        self.file_path == other.file_path
            && self.span.start < other.span.end
            && self.span.end > other.span.start
    }
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start: BytePos(start),
            end: BytePos(end),
        }
    }

    pub fn len(&self) -> u32 {
        self.end.0 - self.start.0
    }

    pub fn is_empty(&self) -> bool {
        self.start.0 >= self.end.0
    }

    pub fn contains(&self, pos: BytePos) -> bool {
        self.start <= pos && pos < self.end
    }
}

impl BytePos {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

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
    pub fn new(module_path: Vec<String>, name: String) -> Self {
        Self {
            module_path,
            name,
            disambiguator: None,
        }
    }

    pub fn with_disambiguator(mut self, disambiguator: u32) -> Self {
        self.disambiguator = Some(disambiguator);
        self
    }

    /// Create a qualified name from a string like "crate::module::Type::method"
    pub fn from_string(qualified_str: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = qualified_str.split("::").collect();
        if parts.is_empty() {
            return Err("Empty qualified name");
        }

        let name = parts.last().unwrap().to_string();
        let module_path = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            module_path,
            name,
            disambiguator: None,
        })
    }

    /// Convert to fully qualified string representation
    pub fn to_qualified_string(&self) -> String {
        let mut result = self.module_path.join("::");
        if !result.is_empty() {
            result.push_str("::");
        }
        result.push_str(&self.name);
        if let Some(disambiguator) = self.disambiguator {
            result.push_str(&format!("#{}", disambiguator));
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

    /// Check if this node represents a function-like construct
    pub fn is_function(&self) -> bool {
        matches!(self.kind, AstKind::Function(_))
    }

    /// Check if this node represents a type definition
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self.kind,
            AstKind::Class(_) | AstKind::Type(_) | AstKind::Module(_)
        )
    }

    /// Get the complexity score for this node
    pub fn complexity(&self) -> u32 {
        unsafe { (self.metadata.complexity & 0xFFFFFFFF) as u32 }
    }

    /// Set the complexity score for this node
    pub fn set_complexity(&mut self, complexity: u32) {
        self.metadata.complexity = complexity as u64;
    }

    /// Add a proof annotation to this node
    pub fn add_proof_annotation(&mut self, annotation: ProofAnnotation) {
        match &mut self.proof_annotations {
            Some(annotations) => annotations.push(annotation),
            None => self.proof_annotations = Some(vec![annotation]),
        }
    }

    /// Get all proof annotations for this node
    pub fn proof_annotations(&self) -> &[ProofAnnotation] {
        self.proof_annotations.as_deref().unwrap_or(&[])
    }

    /// Check if this node has proof annotations
    pub fn has_proof_annotations(&self) -> bool {
        self.proof_annotations
            .as_ref()
            .is_some_and(|annotations| !annotations.is_empty())
    }

    /// Get location for this node (requires file path context)
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

    pub fn get(&self, key: NodeKey) -> Option<&T> {
        self.data.get(key as usize)
    }

    pub fn get_mut(&mut self, key: NodeKey) -> Option<&mut T> {
        self.data.get_mut(key as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// AST DAG structure for efficient traversal and analysis
pub struct AstDag {
    /// Columnar storage for SIMD operations
    pub nodes: ColumnStore<UnifiedAstNode>,

    /// Language-specific parsers (placeholder for now)
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
    pub fn new() -> Self {
        Self {
            nodes: ColumnStore::new(10000), // Initial capacity
            parsers: LanguageParsers::default(),
            dirty_nodes: roaring::RoaringBitmap::new(),
            generation: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Add a new node to the DAG
    pub fn add_node(&mut self, node: UnifiedAstNode) -> NodeKey {
        let key = self.nodes.push(node);
        self.dirty_nodes.insert(key);
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        key
    }

    /// Mark a node as clean (processed)
    pub fn mark_clean(&mut self, key: NodeKey) {
        self.dirty_nodes.remove(key);
    }

    /// Get all dirty nodes for incremental processing
    pub fn dirty_nodes(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.dirty_nodes.iter()
    }

    /// Get the current generation number
    pub fn generation(&self) -> u32 {
        self.generation.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Placeholder for language-specific parsers
#[derive(Default)]
pub struct LanguageParsers {
    // TODO: Add actual parser implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_size() {
        // Ensure our node structure is within expected bounds
        // With proof annotations, the size is larger than the original 64 bytes
        let size = std::mem::size_of::<UnifiedAstNode>();
        assert!(
            size <= 128,
            "Node size {} exceeds maximum expected size of 128 bytes",
            size
        );
        // Structure should be at least 64 bytes for the core data
        assert!(
            size >= 64,
            "Node size {} is smaller than minimum expected size of 64 bytes",
            size
        );
    }

    #[test]
    fn test_node_alignment() {
        // Ensure proper alignment for SIMD operations
        assert_eq!(std::mem::align_of::<UnifiedAstNode>(), 32);
    }

    #[test]
    fn test_node_flags() {
        let mut flags = NodeFlags::new();

        flags.set(NodeFlags::ASYNC);
        flags.set(NodeFlags::EXPORTED);

        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));

        flags.unset(NodeFlags::ASYNC);
        assert!(!flags.has(NodeFlags::ASYNC));
    }

    #[test]
    fn test_ast_dag() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        let key = dag.add_node(node);

        assert_eq!(dag.nodes.len(), 1);
        assert!(dag.dirty_nodes.contains(key));

        dag.mark_clean(key);
        assert!(!dag.dirty_nodes.contains(key));
    }
}
