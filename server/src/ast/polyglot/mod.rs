//! Polyglot AST module for cross-language analysis
//!
//! This module provides a unified representation of AST nodes across different
//! programming languages, enabling cross-language analysis and dependency tracking.
//! The polyglot AST framework is designed to map language-specific AST nodes into
//! a common representation that preserves semantic meaning while abstracting away
//! language-specific details.
//!
//! # Architecture
//!
//! The polyglot AST system consists of three main components:
//!
//! 1. **UnifiedNode**: A language-agnostic representation of code elements
//! 2. **LanguageMapper**: Translates language-specific ASTs to UnifiedNodes
//! 3. **CrossLanguageDependencies**: Tracks relationships between nodes in different languages
//!
//! # Example
//!
//! ```rust,no_run
//! use crate::ast::polyglot::{UnifiedNode, LanguageMapper, JavaMapper, TypeScriptMapper};
//! use std::path::Path;
//!
//! // Create language mappers
//! let java_mapper = JavaMapper::new();
//! let ts_mapper = TypeScriptMapper::new();
//!
//! // Analyze files from different languages
//! let java_file = Path::new("src/main/java/com/example/Model.java");
//! let ts_file = Path::new("src/frontend/models/Model.ts");
//!
//! // Map to unified representation
//! let java_nodes = java_mapper.map_file(java_file).await?;
//! let ts_nodes = ts_mapper.map_file(ts_file).await?;
//!
//! // Find relationships between nodes
//! let dependencies = CrossLanguageDependencies::detect(&java_nodes, &ts_nodes);
//! ```

use crate::services::context::AstItem;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod cross_language_dependencies;
pub mod language_mapper;
pub mod language_mapper_factory;
pub mod unified_node;
pub mod utils;

pub use cross_language_dependencies::CrossLanguageDependencies;
pub use language_mapper::{
    BaseLanguageMapper, CSharpMapper, JavaMapper, JavaScriptMapper, KotlinMapper, LanguageMapper,
    LanguageMapperFactory as LMFactory, RubyMapper, ScalaMapper, TypeScriptMapper,
};
pub use language_mapper_factory::{LanguageMapperFactory, StubMapper};
pub use unified_node::UnifiedNode;
pub use utils::PolyglotPathValidator;

/// Common language identifiers used throughout the polyglot AST system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Java,
    Kotlin,
    Scala,
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Go,
    Cpp,
    CSharp,
    Ruby,
    Swift,
    Php,
    Other(u32), // Numeric identifier for other languages
}

impl Language {
    /// Returns the name of the language as a string
    pub fn name(&self) -> &'static str {
        match self {
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Rust => "Rust",
            Language::Go => "Go",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Php => "PHP",
            Language::Other(_) => "Other",
        }
    }

    /// Returns the language from a file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "java" => Some(Language::Java),
            "kt" | "kts" => Some(Language::Kotlin),
            "scala" | "sc" => Some(Language::Scala),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "rs" => Some(Language::Rust),
            "go" => Some(Language::Go),
            "cpp" | "cc" | "cxx" | "c++" | "h" | "hpp" => Some(Language::Cpp),
            "cs" => Some(Language::CSharp),
            "rb" => Some(Language::Ruby),
            "swift" => Some(Language::Swift),
            "php" => Some(Language::Php),
            _ => None,
        }
    }

    /// Returns the language from a file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Returns common file extensions for this language
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Java => vec!["java"],
            Language::Kotlin => vec!["kt", "kts"],
            Language::Scala => vec!["scala", "sc"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::JavaScript => vec!["js", "jsx"],
            Language::Python => vec!["py"],
            Language::Rust => vec!["rs"],
            Language::Go => vec!["go"],
            Language::Cpp => vec!["cpp", "cc", "cxx", "c++", "h", "hpp"],
            Language::CSharp => vec!["cs"],
            Language::Ruby => vec!["rb"],
            Language::Swift => vec!["swift"],
            Language::Php => vec!["php"],
            Language::Other(_) => vec![],
        }
    }
}

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

/// Configuration for polyglot AST analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotConfig {
    /// List of languages to include in analysis
    pub languages: Vec<Language>,

    /// Whether to detect cross-language relationships
    pub detect_relationships: bool,

    /// Maximum depth for relationship analysis (inheritance, implementation, etc.)
    pub relationship_depth: usize,

    /// Whether to include language-specific details
    pub include_language_specific: bool,
}

impl Default for PolyglotConfig {
    fn default() -> Self {
        Self {
            languages: vec![
                Language::Java,
                Language::Kotlin,
                Language::Scala,
                Language::TypeScript,
                Language::JavaScript,
            ],
            detect_relationships: true,
            relationship_depth: 3,
            include_language_specific: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("java"), Some(Language::Java));
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_language_from_path() {
        let java_path = Path::new("/path/to/MyClass.java");
        let scala_path = Path::new("/path/to/MyClass.scala");
        let unknown_path = Path::new("/path/to/file.unknown");

        assert_eq!(Language::from_path(java_path), Some(Language::Java));
        assert_eq!(Language::from_path(scala_path), Some(Language::Scala));
        assert_eq!(Language::from_path(unknown_path), None);
    }

    #[test]
    fn test_node_kind_from_ast_item_kind() {
        assert_eq!(NodeKind::from_ast_item_kind("class"), NodeKind::Class);
        assert_eq!(NodeKind::from_ast_item_kind("method"), NodeKind::Method);
        assert_eq!(NodeKind::from_ast_item_kind("unknown"), NodeKind::Unknown);
    }

    #[test]
    fn test_polyglot_config_default() {
        let config = PolyglotConfig::default();

        assert!(config.languages.contains(&Language::Java));
        assert!(config.languages.contains(&Language::Scala));
        assert!(config.languages.contains(&Language::TypeScript));
        assert!(config.detect_relationships);
        assert_eq!(config.relationship_depth, 3);
    }

    #[test]
    fn test_node_kind_comprehensive_string_conversion() {
        // Test conversion from strings to NodeKind
        assert_eq!(NodeKind::from_ast_item_kind("function"), NodeKind::Function);
        assert_eq!(NodeKind::from_ast_item_kind("struct"), NodeKind::Struct);
        assert_eq!(NodeKind::from_ast_item_kind("enum"), NodeKind::Enum);
        assert_eq!(NodeKind::from_ast_item_kind("trait"), NodeKind::Trait);
        assert_eq!(
            NodeKind::from_ast_item_kind("implements"),
            NodeKind::Implements
        );
        assert_eq!(NodeKind::from_ast_item_kind("import"), NodeKind::Import);
        assert_eq!(NodeKind::from_ast_item_kind("module"), NodeKind::Module);
        assert_eq!(NodeKind::from_ast_item_kind("type"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typedef"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typealias"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("macro"), NodeKind::Macro);
        assert_eq!(NodeKind::from_ast_item_kind("variable"), NodeKind::Variable);

        // Test NodeKind to string conversion
        assert_eq!(NodeKind::Function.as_str(), "function");
        assert_eq!(NodeKind::Struct.as_str(), "struct");
        assert_eq!(NodeKind::Enum.as_str(), "enum");
        assert_eq!(NodeKind::Trait.as_str(), "trait");
        assert_eq!(NodeKind::Implements.as_str(), "implements");
        assert_eq!(NodeKind::Import.as_str(), "import");
        assert_eq!(NodeKind::Module.as_str(), "module");
        assert_eq!(NodeKind::Type.as_str(), "type");
        assert_eq!(NodeKind::Macro.as_str(), "macro");
        assert_eq!(NodeKind::Variable.as_str(), "variable");
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ==========================================================================
    // Language enum tests - complete coverage
    // ==========================================================================

    #[test]
    fn test_language_name_all_variants() {
        assert_eq!(Language::Java.name(), "Java");
        assert_eq!(Language::Kotlin.name(), "Kotlin");
        assert_eq!(Language::Scala.name(), "Scala");
        assert_eq!(Language::TypeScript.name(), "TypeScript");
        assert_eq!(Language::JavaScript.name(), "JavaScript");
        assert_eq!(Language::Python.name(), "Python");
        assert_eq!(Language::Rust.name(), "Rust");
        assert_eq!(Language::Go.name(), "Go");
        assert_eq!(Language::Cpp.name(), "C++");
        assert_eq!(Language::CSharp.name(), "C#");
        assert_eq!(Language::Ruby.name(), "Ruby");
        assert_eq!(Language::Swift.name(), "Swift");
        assert_eq!(Language::Php.name(), "PHP");
        assert_eq!(Language::Other(42).name(), "Other");
        assert_eq!(Language::Other(0).name(), "Other");
    }

    #[test]
    fn test_language_from_extension_all_variants() {
        // Java
        assert_eq!(Language::from_extension("java"), Some(Language::Java));

        // Kotlin
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("kts"), Some(Language::Kotlin));

        // Scala
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("sc"), Some(Language::Scala));

        // TypeScript
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::TypeScript));

        // JavaScript
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("jsx"), Some(Language::JavaScript));

        // Python
        assert_eq!(Language::from_extension("py"), Some(Language::Python));

        // Rust
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));

        // Go
        assert_eq!(Language::from_extension("go"), Some(Language::Go));

        // C++
        assert_eq!(Language::from_extension("cpp"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("cc"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("cxx"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("c++"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("h"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("hpp"), Some(Language::Cpp));

        // C#
        assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));

        // Ruby
        assert_eq!(Language::from_extension("rb"), Some(Language::Ruby));

        // Swift
        assert_eq!(Language::from_extension("swift"), Some(Language::Swift));

        // PHP
        assert_eq!(Language::from_extension("php"), Some(Language::Php));

        // Unknown extensions
        assert_eq!(Language::from_extension("txt"), None);
        assert_eq!(Language::from_extension("md"), None);
        assert_eq!(Language::from_extension(""), None);
    }

    #[test]
    fn test_language_from_extension_case_insensitive() {
        assert_eq!(Language::from_extension("JAVA"), Some(Language::Java));
        assert_eq!(Language::from_extension("Java"), Some(Language::Java));
        assert_eq!(Language::from_extension("TS"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("PY"), Some(Language::Python));
        assert_eq!(Language::from_extension("RS"), Some(Language::Rust));
    }

    #[test]
    fn test_language_from_path_all_extensions() {
        assert_eq!(
            Language::from_path(Path::new("Test.java")),
            Some(Language::Java)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.kt")),
            Some(Language::Kotlin)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.scala")),
            Some(Language::Scala)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.js")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.py")),
            Some(Language::Python)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.go")),
            Some(Language::Go)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.cpp")),
            Some(Language::Cpp)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.cs")),
            Some(Language::CSharp)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.rb")),
            Some(Language::Ruby)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.swift")),
            Some(Language::Swift)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.php")),
            Some(Language::Php)
        );
    }

    #[test]
    fn test_language_from_path_edge_cases() {
        // No extension
        assert_eq!(Language::from_path(Path::new("Makefile")), None);
        assert_eq!(Language::from_path(Path::new("README")), None);

        // Hidden files
        assert_eq!(Language::from_path(Path::new(".gitignore")), None);

        // Deep paths
        assert_eq!(
            Language::from_path(Path::new("/a/b/c/d/e.java")),
            Some(Language::Java)
        );

        // Relative paths
        assert_eq!(
            Language::from_path(Path::new("./src/main.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(Path::new("../lib.py")),
            Some(Language::Python)
        );
    }

    #[test]
    fn test_language_file_extensions_all_variants() {
        assert_eq!(Language::Java.file_extensions(), vec!["java"]);
        assert_eq!(Language::Kotlin.file_extensions(), vec!["kt", "kts"]);
        assert_eq!(Language::Scala.file_extensions(), vec!["scala", "sc"]);
        assert_eq!(Language::TypeScript.file_extensions(), vec!["ts", "tsx"]);
        assert_eq!(Language::JavaScript.file_extensions(), vec!["js", "jsx"]);
        assert_eq!(Language::Python.file_extensions(), vec!["py"]);
        assert_eq!(Language::Rust.file_extensions(), vec!["rs"]);
        assert_eq!(Language::Go.file_extensions(), vec!["go"]);
        assert_eq!(
            Language::Cpp.file_extensions(),
            vec!["cpp", "cc", "cxx", "c++", "h", "hpp"]
        );
        assert_eq!(Language::CSharp.file_extensions(), vec!["cs"]);
        assert_eq!(Language::Ruby.file_extensions(), vec!["rb"]);
        assert_eq!(Language::Swift.file_extensions(), vec!["swift"]);
        assert_eq!(Language::Php.file_extensions(), vec!["php"]);
        assert!(Language::Other(0).file_extensions().is_empty());
        assert!(Language::Other(100).file_extensions().is_empty());
    }

    #[test]
    fn test_language_equality_and_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Language::Java);
        set.insert(Language::Kotlin);
        set.insert(Language::Other(1));
        set.insert(Language::Other(2));

        assert!(set.contains(&Language::Java));
        assert!(set.contains(&Language::Kotlin));
        assert!(set.contains(&Language::Other(1)));
        assert!(set.contains(&Language::Other(2)));
        assert!(!set.contains(&Language::Python));
        assert!(!set.contains(&Language::Other(3)));
    }

    #[test]
    fn test_language_clone_and_copy() {
        let lang = Language::TypeScript;
        let cloned = lang.clone();
        let copied = lang;

        assert_eq!(lang, cloned);
        assert_eq!(lang, copied);
    }

    #[test]
    fn test_language_debug() {
        let debug_str = format!("{:?}", Language::Java);
        assert_eq!(debug_str, "Java");

        let debug_other = format!("{:?}", Language::Other(42));
        assert_eq!(debug_other, "Other(42)");
    }

    // ==========================================================================
    // NodeKind enum tests - complete coverage
    // ==========================================================================

    #[test]
    fn test_node_kind_from_ast_item_all_variants() {
        let function_item = AstItem::Function {
            name: "test".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&function_item), NodeKind::Function);

        let struct_item = AstItem::Struct {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&struct_item), NodeKind::Struct);

        let enum_item = AstItem::Enum {
            name: "MyEnum".to_string(),
            visibility: "pub".to_string(),
            variants_count: 3,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&enum_item), NodeKind::Enum);

        let trait_item = AstItem::Trait {
            name: "MyTrait".to_string(),
            visibility: "pub".to_string(),
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&trait_item), NodeKind::Trait);

        let impl_item = AstItem::Impl {
            type_name: "MyType".to_string(),
            trait_name: None,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&impl_item), NodeKind::Implements);

        let use_item = AstItem::Use {
            path: "std::io".to_string(),
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&use_item), NodeKind::Uses);

        let module_item = AstItem::Module {
            name: "my_module".to_string(),
            visibility: "pub".to_string(),
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&module_item), NodeKind::Module);

        let import_item = AstItem::Import {
            module: "external".to_string(),
            items: vec![],
            alias: None,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&import_item), NodeKind::Import);
    }

    #[test]
    fn test_node_kind_from_ast_item_kind_all_variants() {
        // Declarations
        assert_eq!(NodeKind::from_ast_item_kind("package"), NodeKind::Package);
        assert_eq!(NodeKind::from_ast_item_kind("import"), NodeKind::Import);
        assert_eq!(NodeKind::from_ast_item_kind("module"), NodeKind::Module);
        assert_eq!(
            NodeKind::from_ast_item_kind("namespace"),
            NodeKind::Namespace
        );

        // Types
        assert_eq!(NodeKind::from_ast_item_kind("class"), NodeKind::Class);
        assert_eq!(
            NodeKind::from_ast_item_kind("interface"),
            NodeKind::Interface
        );
        assert_eq!(NodeKind::from_ast_item_kind("trait"), NodeKind::Trait);
        assert_eq!(NodeKind::from_ast_item_kind("enum"), NodeKind::Enum);
        assert_eq!(NodeKind::from_ast_item_kind("struct"), NodeKind::Struct);
        assert_eq!(NodeKind::from_ast_item_kind("union"), NodeKind::Union);
        assert_eq!(NodeKind::from_ast_item_kind("type"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typealias"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typedef"), NodeKind::Type);

        // Type variations
        assert_eq!(NodeKind::from_ast_item_kind("record"), NodeKind::Record);
        assert_eq!(
            NodeKind::from_ast_item_kind("caseclass"),
            NodeKind::CaseClass
        );
        assert_eq!(
            NodeKind::from_ast_item_kind("abstracttype"),
            NodeKind::AbstractType
        );

        // Functions & methods
        assert_eq!(NodeKind::from_ast_item_kind("method"), NodeKind::Method);
        assert_eq!(NodeKind::from_ast_item_kind("function"), NodeKind::Function);
        assert_eq!(
            NodeKind::from_ast_item_kind("constructor"),
            NodeKind::Constructor
        );
        assert_eq!(NodeKind::from_ast_item_kind("lambda"), NodeKind::Lambda);
        assert_eq!(NodeKind::from_ast_item_kind("closure"), NodeKind::Closure);

        // Variables
        assert_eq!(NodeKind::from_ast_item_kind("field"), NodeKind::Field);
        assert_eq!(NodeKind::from_ast_item_kind("property"), NodeKind::Property);
        assert_eq!(
            NodeKind::from_ast_item_kind("localvariable"),
            NodeKind::LocalVariable
        );
        assert_eq!(
            NodeKind::from_ast_item_kind("parameter"),
            NodeKind::Parameter
        );
        assert_eq!(NodeKind::from_ast_item_kind("variable"), NodeKind::Variable);

        // Other elements
        assert_eq!(
            NodeKind::from_ast_item_kind("annotation"),
            NodeKind::Annotation
        );
        assert_eq!(
            NodeKind::from_ast_item_kind("decorator"),
            NodeKind::Decorator
        );
        assert_eq!(NodeKind::from_ast_item_kind("comment"), NodeKind::Comment);
        assert_eq!(NodeKind::from_ast_item_kind("macro"), NodeKind::Macro);

        // Relationships
        assert_eq!(NodeKind::from_ast_item_kind("inherits"), NodeKind::Inherits);
        assert_eq!(
            NodeKind::from_ast_item_kind("implements"),
            NodeKind::Implements
        );
        assert_eq!(NodeKind::from_ast_item_kind("uses"), NodeKind::Uses);

        // Unknown
        assert_eq!(NodeKind::from_ast_item_kind("unknown"), NodeKind::Unknown);
        assert_eq!(
            NodeKind::from_ast_item_kind("not_a_real_kind"),
            NodeKind::Unknown
        );
        assert_eq!(NodeKind::from_ast_item_kind(""), NodeKind::Unknown);
    }

    #[test]
    fn test_node_kind_as_str_all_variants() {
        assert_eq!(NodeKind::Package.as_str(), "package");
        assert_eq!(NodeKind::Import.as_str(), "import");
        assert_eq!(NodeKind::Module.as_str(), "module");
        assert_eq!(NodeKind::Namespace.as_str(), "namespace");
        assert_eq!(NodeKind::Class.as_str(), "class");
        assert_eq!(NodeKind::Interface.as_str(), "interface");
        assert_eq!(NodeKind::Trait.as_str(), "trait");
        assert_eq!(NodeKind::Enum.as_str(), "enum");
        assert_eq!(NodeKind::Struct.as_str(), "struct");
        assert_eq!(NodeKind::Union.as_str(), "union");
        assert_eq!(NodeKind::Type.as_str(), "type");
        assert_eq!(NodeKind::Record.as_str(), "record");
        assert_eq!(NodeKind::CaseClass.as_str(), "caseClass");
        assert_eq!(NodeKind::AbstractType.as_str(), "abstractType");
        assert_eq!(NodeKind::Method.as_str(), "method");
        assert_eq!(NodeKind::Function.as_str(), "function");
        assert_eq!(NodeKind::Constructor.as_str(), "constructor");
        assert_eq!(NodeKind::Lambda.as_str(), "lambda");
        assert_eq!(NodeKind::Closure.as_str(), "closure");
        assert_eq!(NodeKind::Field.as_str(), "field");
        assert_eq!(NodeKind::Property.as_str(), "property");
        assert_eq!(NodeKind::LocalVariable.as_str(), "localVariable");
        assert_eq!(NodeKind::Parameter.as_str(), "parameter");
        assert_eq!(NodeKind::Variable.as_str(), "variable");
        assert_eq!(NodeKind::Annotation.as_str(), "annotation");
        assert_eq!(NodeKind::Decorator.as_str(), "decorator");
        assert_eq!(NodeKind::Comment.as_str(), "comment");
        assert_eq!(NodeKind::Macro.as_str(), "macro");
        assert_eq!(NodeKind::Inherits.as_str(), "inherits");
        assert_eq!(NodeKind::Implements.as_str(), "implements");
        assert_eq!(NodeKind::Uses.as_str(), "uses");
        assert_eq!(
            NodeKind::LanguageSpecific(0).as_str(),
            "languageSpecific"
        );
        assert_eq!(
            NodeKind::LanguageSpecific(100).as_str(),
            "languageSpecific"
        );
        assert_eq!(NodeKind::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_node_kind_equality_and_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(NodeKind::Function);
        set.insert(NodeKind::Method);
        set.insert(NodeKind::LanguageSpecific(1));
        set.insert(NodeKind::LanguageSpecific(2));

        assert!(set.contains(&NodeKind::Function));
        assert!(set.contains(&NodeKind::Method));
        assert!(set.contains(&NodeKind::LanguageSpecific(1)));
        assert!(set.contains(&NodeKind::LanguageSpecific(2)));
        assert!(!set.contains(&NodeKind::Class));
        assert!(!set.contains(&NodeKind::LanguageSpecific(3)));
    }

    #[test]
    fn test_node_kind_clone_and_copy() {
        let kind = NodeKind::Function;
        let cloned = kind.clone();
        let copied = kind;

        assert_eq!(kind, cloned);
        assert_eq!(kind, copied);
    }

    #[test]
    fn test_node_kind_debug() {
        let debug_str = format!("{:?}", NodeKind::Function);
        assert_eq!(debug_str, "Function");

        let debug_specific = format!("{:?}", NodeKind::LanguageSpecific(42));
        assert_eq!(debug_specific, "LanguageSpecific(42)");
    }

    // ==========================================================================
    // PolyglotConfig tests
    // ==========================================================================

    #[test]
    fn test_polyglot_config_default_values() {
        let config = PolyglotConfig::default();

        assert_eq!(config.languages.len(), 5);
        assert!(config.languages.contains(&Language::Java));
        assert!(config.languages.contains(&Language::Kotlin));
        assert!(config.languages.contains(&Language::Scala));
        assert!(config.languages.contains(&Language::TypeScript));
        assert!(config.languages.contains(&Language::JavaScript));
        assert!(config.detect_relationships);
        assert_eq!(config.relationship_depth, 3);
        assert!(config.include_language_specific);
    }

    #[test]
    fn test_polyglot_config_custom() {
        let config = PolyglotConfig {
            languages: vec![Language::Rust, Language::Python],
            detect_relationships: false,
            relationship_depth: 5,
            include_language_specific: false,
        };

        assert_eq!(config.languages.len(), 2);
        assert!(config.languages.contains(&Language::Rust));
        assert!(config.languages.contains(&Language::Python));
        assert!(!config.detect_relationships);
        assert_eq!(config.relationship_depth, 5);
        assert!(!config.include_language_specific);
    }

    #[test]
    fn test_polyglot_config_clone() {
        let config = PolyglotConfig::default();
        let cloned = config.clone();

        assert_eq!(config.languages, cloned.languages);
        assert_eq!(config.detect_relationships, cloned.detect_relationships);
        assert_eq!(config.relationship_depth, cloned.relationship_depth);
        assert_eq!(
            config.include_language_specific,
            cloned.include_language_specific
        );
    }

    #[test]
    fn test_polyglot_config_debug() {
        let config = PolyglotConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("PolyglotConfig"));
        assert!(debug_str.contains("languages"));
        assert!(debug_str.contains("detect_relationships"));
    }

    #[test]
    fn test_polyglot_config_serialize_deserialize() {
        let config = PolyglotConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PolyglotConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.languages, deserialized.languages);
        assert_eq!(
            config.detect_relationships,
            deserialized.detect_relationships
        );
        assert_eq!(config.relationship_depth, deserialized.relationship_depth);
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn test_language_serialize_deserialize() {
        let languages = vec![
            Language::Java,
            Language::TypeScript,
            Language::Other(42),
        ];

        for lang in languages {
            let json = serde_json::to_string(&lang).unwrap();
            let deserialized: Language = serde_json::from_str(&json).unwrap();
            assert_eq!(lang, deserialized);
        }
    }

    #[test]
    fn test_node_kind_serialize_deserialize() {
        let kinds = vec![
            NodeKind::Function,
            NodeKind::Class,
            NodeKind::LanguageSpecific(42),
            NodeKind::Unknown,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let deserialized: NodeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, deserialized);
        }
    }

    // ==========================================================================
    // Edge cases and boundary tests
    // ==========================================================================

    #[test]
    fn test_language_from_extension_with_dots() {
        // Extensions should not include the dot
        assert_eq!(Language::from_extension(".java"), None);
        assert_eq!(Language::from_extension("..."), None);
    }

    #[test]
    fn test_language_from_path_no_extension() {
        assert_eq!(Language::from_path(Path::new("noextension")), None);
        assert_eq!(Language::from_path(Path::new("/path/to/noextension")), None);
    }

    #[test]
    fn test_language_other_variant_equality() {
        // Different numeric values should not be equal
        assert_ne!(Language::Other(1), Language::Other(2));
        assert_eq!(Language::Other(100), Language::Other(100));
    }

    #[test]
    fn test_node_kind_language_specific_variant() {
        // Different numeric values should not be equal
        assert_ne!(NodeKind::LanguageSpecific(1), NodeKind::LanguageSpecific(2));
        assert_eq!(
            NodeKind::LanguageSpecific(100),
            NodeKind::LanguageSpecific(100)
        );
    }

    #[test]
    fn test_language_from_ast_item_kind_case_insensitive() {
        // Should handle case insensitivity
        assert_eq!(NodeKind::from_ast_item_kind("FUNCTION"), NodeKind::Function);
        assert_eq!(NodeKind::from_ast_item_kind("Function"), NodeKind::Function);
        assert_eq!(NodeKind::from_ast_item_kind("CLASS"), NodeKind::Class);
        assert_eq!(NodeKind::from_ast_item_kind("Class"), NodeKind::Class);
    }
}
