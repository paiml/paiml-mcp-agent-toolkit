/// Common node types across languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    // Declarations
    Package,
    Import,
    Module,
    Namespace,

    // Types
    Class,
    Interface,
    Trait,
    Enum,
    Struct,
    Union,
    Type, // Type alias or typedef

    // Type variations
    Record,       // Java record, Kotlin data class, TypeScript interface
    CaseClass,    // Scala case class
    AbstractType, // Abstract class/interface/trait

    // Functions & methods
    Method,
    Function,
    Constructor,
    Lambda,
    Closure,

    // Variables
    Field,
    Property,
    LocalVariable,
    Parameter,
    Variable, // Generic variable declaration

    // Other elements
    Annotation,
    Decorator,
    Comment,
    Macro, // Macro definition

    // Relationships
    Inherits,
    Implements,
    Uses,

    // For any language-specific constructs
    LanguageSpecific(u32), // Numeric identifier for language-specific nodes
    Unknown,
}

impl NodeKind {
    /// Convert from AstItem enum
    pub fn from_ast_item(item: &AstItem) -> Self {
        match item {
            AstItem::Function { .. } => NodeKind::Function,
            AstItem::Struct { .. } => NodeKind::Struct,
            AstItem::Enum { .. } => NodeKind::Enum,
            AstItem::Trait { .. } => NodeKind::Trait,
            AstItem::Impl { .. } => NodeKind::Implements,
            AstItem::Use { .. } => NodeKind::Uses,
            AstItem::Module { .. } => NodeKind::Module,
            AstItem::Import { .. } => NodeKind::Import,
        }
    }

    /// Convert from a string item kind
    pub fn from_ast_item_kind(kind: &str) -> Self {
        match kind.to_lowercase().as_str() {
            "package" => NodeKind::Package,
            "import" => NodeKind::Import,
            "module" => NodeKind::Module,
            "namespace" => NodeKind::Namespace,

            "class" => NodeKind::Class,
            "interface" => NodeKind::Interface,
            "trait" => NodeKind::Trait,
            "enum" => NodeKind::Enum,
            "struct" => NodeKind::Struct,
            "union" => NodeKind::Union,
            "type" | "typealias" | "typedef" => NodeKind::Type,

            "record" => NodeKind::Record,
            "caseclass" => NodeKind::CaseClass,
            "abstracttype" => NodeKind::AbstractType,

            "method" => NodeKind::Method,
            "function" => NodeKind::Function,
            "constructor" => NodeKind::Constructor,
            "lambda" => NodeKind::Lambda,
            "closure" => NodeKind::Closure,

            "field" => NodeKind::Field,
            "property" => NodeKind::Property,
            "localvariable" => NodeKind::LocalVariable,
            "parameter" => NodeKind::Parameter,
            "variable" => NodeKind::Variable,

            "annotation" => NodeKind::Annotation,
            "decorator" => NodeKind::Decorator,
            "comment" => NodeKind::Comment,
            "macro" => NodeKind::Macro,

            "inherits" => NodeKind::Inherits,
            "implements" => NodeKind::Implements,
            "uses" => NodeKind::Uses,

            _ => NodeKind::Unknown,
        }
    }

    /// Returns a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Package => "package",
            NodeKind::Import => "import",
            NodeKind::Module => "module",
            NodeKind::Namespace => "namespace",

            NodeKind::Class => "class",
            NodeKind::Interface => "interface",
            NodeKind::Trait => "trait",
            NodeKind::Enum => "enum",
            NodeKind::Struct => "struct",
            NodeKind::Union => "union",
            NodeKind::Type => "type",

            NodeKind::Record => "record",
            NodeKind::CaseClass => "caseClass",
            NodeKind::AbstractType => "abstractType",

            NodeKind::Method => "method",
            NodeKind::Function => "function",
            NodeKind::Constructor => "constructor",
            NodeKind::Lambda => "lambda",
            NodeKind::Closure => "closure",

            NodeKind::Field => "field",
            NodeKind::Property => "property",
            NodeKind::LocalVariable => "localVariable",
            NodeKind::Parameter => "parameter",
            NodeKind::Variable => "variable",

            NodeKind::Annotation => "annotation",
            NodeKind::Decorator => "decorator",
            NodeKind::Comment => "comment",
            NodeKind::Macro => "macro",

            NodeKind::Inherits => "inherits",
            NodeKind::Implements => "implements",
            NodeKind::Uses => "uses",

            NodeKind::LanguageSpecific(_) => "languageSpecific",
            NodeKind::Unknown => "unknown",
        }
    }
}
