/// A unified representation of a node in the polyglot AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNode {
    /// Unique identifier for this node
    pub id: String,

    /// The kind of node (class, method, etc.)
    pub kind: NodeKind,

    /// The name of the node
    pub name: String,

    /// The fully qualified name (including package/namespace)
    pub fqn: String,

    /// The source language of this node
    pub language: Language,

    /// The file path where this node is defined
    pub file_path: PathBuf,

    /// Line and column position in source
    pub position: SourcePosition,

    /// Node attributes (modifiers, visibility, etc.)
    pub attributes: HashMap<String, String>,

    /// For container nodes, child nodes
    pub children: Vec<String>, // IDs of child nodes

    /// For class members, the parent class/struct
    pub parent: Option<String>, // ID of parent node

    /// References to other nodes (inheritance, implementation, etc.)
    pub references: Vec<NodeReference>,

    /// Type information
    pub type_info: Option<TypeInfo>,

    /// Signature for methods/functions
    pub signature: Option<String>,

    /// Documentation/comments
    pub documentation: Option<String>,

    /// Original AST item this was created from (optional)
    #[serde(skip_serializing, skip_deserializing)]
    pub original_item: Option<AstItem>,

    /// Language-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Position in source code
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourcePosition {
    /// Starting line (1-based)
    pub start_line: usize,
    /// Starting column (1-based)
    pub start_col: usize,
    /// Ending line (1-based)
    pub end_line: usize,
    /// Ending column (1-based)
    pub end_col: usize,
}

/// A reference to another node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReference {
    /// Type of reference (inherits, implements, calls, etc.)
    pub kind: ReferenceKind,

    /// Target node ID
    pub target_id: String,

    /// Target name (may be used before resolving to ID)
    pub target_name: String,

    /// Target language (may be different than source node)
    pub target_language: Option<Language>,
}

/// Type of reference between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReferenceKind {
    /// Inheritance relationship (extends)
    Inherits,

    /// Implementation relationship (implements)
    Implements,

    /// Calls a method or function
    Calls,

    /// Uses a field, property or variable
    Uses,

    /// Creates an instance of a class
    Creates,

    /// Imports or requires
    Imports,

    /// Annotates or decorates
    Annotates,

    /// Generic dependency
    DependsOn,
}

/// Type information for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Base type name
    pub name: String,

    /// Fully qualified type name
    pub fqn: String,

    /// Type parameters (for generics)
    pub type_parameters: Vec<TypeInfo>,

    /// Is it a primitive type?
    pub is_primitive: bool,

    /// Is it a collection type?
    pub is_collection: bool,

    /// Is it nullable?
    pub is_nullable: bool,

    /// Original type string from source language
    pub original_type_string: String,
}
