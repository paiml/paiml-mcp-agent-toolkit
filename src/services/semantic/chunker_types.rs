/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    C,
    Cpp,
    Go,
    Lua,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Go => "go",
            Language::Lua => "lua",
        }
    }
}

/// Type of code chunk
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChunkType {
    Function,
    Class,
    Module,
    File,
    /// Rust struct definition
    Struct,
    /// Rust enum definition
    Enum,
    /// Rust trait definition
    Trait,
    /// Rust type alias
    TypeAlias,
    /// Rust impl block
    Impl,
    /// Rust #[cfg(test)] module
    TestModule,
}

impl ChunkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkType::Function => "function",
            ChunkType::Class => "class",
            ChunkType::Module => "module",
            ChunkType::File => "file",
            ChunkType::Struct => "struct",
            ChunkType::Enum => "enum",
            ChunkType::Trait => "trait",
            ChunkType::TypeAlias => "type_alias",
            ChunkType::Impl => "impl",
            ChunkType::TestModule => "test_module",
        }
    }
}

/// A semantic code chunk with metadata
#[derive(Debug, Clone)]
pub struct CodeChunk {
    /// Full file path (empty in tests)
    pub file_path: String,
    /// Type of chunk (function, class, module, file)
    pub chunk_type: ChunkType,
    /// Identifier name (function name, class name, etc.)
    pub chunk_name: String,
    /// Programming language
    pub language: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (inclusive)
    pub end_line: usize,
    /// Full source code including docstrings
    pub content: String,
    /// SHA256 checksum of content for incremental updates
    pub content_checksum: String,
}

/// Extract code chunks from source code
///
/// # Arguments
/// * `source` - Source code string
/// * `language` - Programming language
///
/// # Returns
/// Vector of code chunks (functions, classes, modules)
pub fn chunk_code(source: &str, language: Language) -> Result<Vec<CodeChunk>, String> {
    // Handle empty input
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }

    match language {
        #[cfg(feature = "rust-ast")]
        Language::Rust => chunk_rust_file(source),
        #[cfg(feature = "typescript-ast")]
        Language::TypeScript => chunk_typescript_file(source),
        #[cfg(feature = "python-ast")]
        Language::Python => chunk_python_file(source),
        #[cfg(feature = "c-ast")]
        Language::C => chunk_c_file(source),
        #[cfg(feature = "c-ast")]
        Language::Cpp => chunk_cpp_file(source),
        #[cfg(feature = "tree-sitter")]
        Language::Go => chunk_go_file(source),
        #[cfg(feature = "tree-sitter")]
        Language::Lua => chunk_lua_file(source),
        #[allow(unreachable_patterns)]
        _ => Err(format!(
            "language {:?} not enabled; enable the corresponding feature",
            language.as_str()
        )),
    }
}
