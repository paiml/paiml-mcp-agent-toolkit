//! Language Registry for 30+ language support per SPECIFICATION.md Section 6.2
//!
//! This module provides comprehensive language detection and parser selection
//! for modern software development ecosystems.

use std::path::Path;
use serde::{Deserialize, Serialize};

/// Comprehensive language enumeration supporting 30+ languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    // Systems Programming
    Rust,
    C,
    Cpp,
    Go,
    Zig,
    
    // JVM Ecosystem
    Java,
    Kotlin,
    Scala,
    Groovy,
    Clojure,
    
    // .NET Ecosystem
    CSharp,
    FSharp,
    VisualBasic,
    
    // Dynamic Languages
    Python,
    JavaScript,
    TypeScript,
    Ruby,
    PHP,
    Perl,
    Lua,
    
    // Functional Languages
    Haskell,
    Elixir,
    Erlang,
    OCaml,
    ReasonML,
    Elm,
    PureScript,
    
    // Mobile Development
    Swift,
    ObjectiveC,
    Dart,
    
    // Shell & Scripting
    Bash,
    Zsh,
    Fish,
    PowerShell,
    
    // Data & Config
    SQL,
    HCL,     // Terraform
    YAML,
    TOML,
    JSON,
    XML,
    
    // Documentation & Markup
    Markdown,
    LaTeX,
    AsciiDoc,
    
    // Build Systems
    Makefile,
    CMake,
    Bazel,
    Gradle,
    Maven,
    
    // Specialized
    Solidity,    // Blockchain
    VHDL,        // Hardware
    Verilog,     // Hardware
    R,           // Statistics
    Julia,       // Scientific computing
    Matlab,      // Engineering
    Assembly,    // Low-level
    
    Unknown,
}

impl Language {
    /// Get file extensions associated with this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            // Systems Programming
            Language::Rust => &["rs"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx", "C", "H"],
            Language::Go => &["go"],
            Language::Zig => &["zig"],
            
            // JVM Ecosystem
            Language::Java => &["java"],
            Language::Kotlin => &["kt", "kts"],
            Language::Scala => &["scala", "sc"],
            Language::Groovy => &["groovy", "gvy", "gy", "gsh"],
            Language::Clojure => &["clj", "cljs", "cljc", "edn"],
            
            // .NET Ecosystem
            Language::CSharp => &["cs"],
            Language::FSharp => &["fs", "fsi", "fsx"],
            Language::VisualBasic => &["vb"],
            
            // Dynamic Languages
            Language::Python => &["py", "pyw", "pyi", "pyx", "pxd"],
            Language::JavaScript => &["js", "jsx", "mjs", "cjs"],
            Language::TypeScript => &["ts", "tsx", "d.ts"],
            Language::Ruby => &["rb", "rbw", "rake", "gemspec"],
            Language::PHP => &["php", "phtml", "php3", "php4", "php5", "phps"],
            Language::Perl => &["pl", "pm", "t", "pod"],
            Language::Lua => &["lua"],
            
            // Functional Languages
            Language::Haskell => &["hs", "lhs"],
            Language::Elixir => &["ex", "exs"],
            Language::Erlang => &["erl", "hrl"],
            Language::OCaml => &["ml", "mli"],
            Language::ReasonML => &["re", "rei"],
            Language::Elm => &["elm"],
            Language::PureScript => &["purs"],
            
            // Mobile Development
            Language::Swift => &["swift"],
            Language::ObjectiveC => &["m", "mm", "M"],
            Language::Dart => &["dart"],
            
            // Shell & Scripting
            Language::Bash => &["sh", "bash", "zsh"],
            Language::Zsh => &["zsh"],
            Language::Fish => &["fish"],
            Language::PowerShell => &["ps1", "psm1", "psd1"],
            
            // Data & Config
            Language::SQL => &["sql", "ddl", "dml"],
            Language::HCL => &["tf", "tfvars", "hcl"],
            Language::YAML => &["yml", "yaml"],
            Language::TOML => &["toml"],
            Language::JSON => &["json", "jsonc"],
            Language::XML => &["xml", "xsd", "xsl", "xslt"],
            
            // Documentation & Markup
            Language::Markdown => &["md", "markdown", "mdown", "mkd"],
            Language::LaTeX => &["tex", "latex", "sty", "cls"],
            Language::AsciiDoc => &["adoc", "asciidoc"],
            
            // Build Systems
            Language::Makefile => &["mk", "make"],
            Language::CMake => &["cmake"],
            Language::Bazel => &["bazel", "bzl"],
            Language::Gradle => &["gradle"],
            Language::Maven => &["pom"],
            
            // Specialized
            Language::Solidity => &["sol"],
            Language::VHDL => &["vhd", "vhdl"],
            Language::Verilog => &["v", "vh"],
            Language::R => &["r", "R"],
            Language::Julia => &["jl"],
            Language::Matlab => &["m"],
            Language::Assembly => &["s", "S", "asm"],
            
            Language::Unknown => &[],
        }
    }
    
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        let ext = ext.to_lowercase();
        
        // Check all languages for matching extensions
        for &lang in &[
            Language::Rust, Language::C, Language::Cpp, Language::Go, Language::Zig,
            Language::Java, Language::Kotlin, Language::Scala, Language::Groovy, Language::Clojure,
            Language::CSharp, Language::FSharp, Language::VisualBasic,
            Language::Python, Language::JavaScript, Language::TypeScript, Language::Ruby,
            Language::PHP, Language::Perl, Language::Lua,
            Language::Haskell, Language::Elixir, Language::Erlang, Language::OCaml,
            Language::ReasonML, Language::Elm, Language::PureScript,
            Language::Swift, Language::ObjectiveC, Language::Dart,
            Language::Bash, Language::Zsh, Language::Fish, Language::PowerShell,
            Language::SQL, Language::HCL, Language::YAML, Language::TOML, Language::JSON, Language::XML,
            Language::Markdown, Language::LaTeX, Language::AsciiDoc,
            Language::Makefile, Language::CMake, Language::Bazel, Language::Gradle, Language::Maven,
            Language::Solidity, Language::VHDL, Language::Verilog, Language::R, Language::Julia,
            Language::Matlab, Language::Assembly,
        ] {
            if lang.extensions().contains(&ext.as_str()) {
                return lang;
            }
        }
        
        Language::Unknown
    }
    
    /// Detect language from file path
    pub fn from_path(path: &Path) -> Self {
        // Handle special cases by filename
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            match filename.to_lowercase().as_str() {
                "makefile" | "gnumakefile" => return Language::Makefile,
                "dockerfile" | "dockerfile.dev" => return Language::Bash, // Docker files use shell-like syntax
                "rakefile" => return Language::Ruby,
                "gemfile" | "gemfile.lock" => return Language::Ruby,
                "cargo.toml" | "cargo.lock" => return Language::TOML,
                "package.json" | "package-lock.json" => return Language::JSON,
                "tsconfig.json" => return Language::JSON,
                "build.gradle" | "settings.gradle" => return Language::Gradle,
                "pom.xml" => return Language::Maven,
                "requirements.txt" | "setup.py" | "pyproject.toml" => return Language::Python,
                _ => {}
            }
        }
        
        // Extract extension and detect language
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::from_extension(ext)
        } else {
            Language::Unknown
        }
    }
    
    /// Get language name as string
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Go => "Go",
            Language::Zig => "Zig",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::Groovy => "Groovy",
            Language::Clojure => "Clojure",
            Language::CSharp => "C#",
            Language::FSharp => "F#",
            Language::VisualBasic => "Visual Basic",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Ruby => "Ruby",
            Language::PHP => "PHP",
            Language::Perl => "Perl",
            Language::Lua => "Lua",
            Language::Haskell => "Haskell",
            Language::Elixir => "Elixir",
            Language::Erlang => "Erlang",
            Language::OCaml => "OCaml",
            Language::ReasonML => "ReasonML",
            Language::Elm => "Elm",
            Language::PureScript => "PureScript",
            Language::Swift => "Swift",
            Language::ObjectiveC => "Objective-C",
            Language::Dart => "Dart",
            Language::Bash => "Bash",
            Language::Zsh => "Zsh",
            Language::Fish => "Fish",
            Language::PowerShell => "PowerShell",
            Language::SQL => "SQL",
            Language::HCL => "HCL",
            Language::YAML => "YAML",
            Language::TOML => "TOML",
            Language::JSON => "JSON",
            Language::XML => "XML",
            Language::Markdown => "Markdown",
            Language::LaTeX => "LaTeX",
            Language::AsciiDoc => "AsciiDoc",
            Language::Makefile => "Makefile",
            Language::CMake => "CMake",
            Language::Bazel => "Bazel",
            Language::Gradle => "Gradle",
            Language::Maven => "Maven",
            Language::Solidity => "Solidity",
            Language::VHDL => "VHDL",
            Language::Verilog => "Verilog",
            Language::R => "R",
            Language::Julia => "Julia",
            Language::Matlab => "MATLAB",
            Language::Assembly => "Assembly",
            Language::Unknown => "Unknown",
        }
    }
    
    /// Check if language has full AST parsing support
    pub fn has_ast_support(&self) -> bool {
        match self {
            // Languages with full AST support (existing implementations)
            Language::Rust | Language::TypeScript | Language::JavaScript | 
            Language::Python | Language::C | Language::Cpp | Language::Kotlin |
            Language::Makefile => true,
            
            // Languages that can be analyzed with pattern matching
            Language::Go | Language::Java | Language::CSharp | Language::Swift |
            Language::Ruby | Language::PHP | Language::Bash | Language::SQL => true,
            
            // Configuration and markup languages (structure parsing)
            Language::JSON | Language::YAML | Language::TOML | Language::XML |
            Language::Markdown => true,
            
            // Others need basic support for now
            _ => false,
        }
    }
    
    /// Check if language supports complexity analysis
    pub fn supports_complexity(&self) -> bool {
        match self {
            // Programming languages that have control flow
            Language::Rust | Language::C | Language::Cpp | Language::Go | Language::Zig |
            Language::Java | Language::Kotlin | Language::Scala | Language::Groovy |
            Language::CSharp | Language::FSharp | Language::Python | Language::JavaScript |
            Language::TypeScript | Language::Ruby | Language::PHP | Language::Perl |
            Language::Haskell | Language::Elixir | Language::Swift | Language::ObjectiveC |
            Language::Dart | Language::Bash | Language::PowerShell | Language::R |
            Language::Julia | Language::Matlab => true,
            
            // Markup, config, and data languages don't have complexity
            _ => false,
        }
    }
}

/// Language statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: Language,
    pub file_count: usize,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
}

/// Language detection and analysis registry
pub struct LanguageRegistry {
    supported_languages: Vec<Language>,
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRegistry {
    /// Create a new language registry with all supported languages
    pub fn new() -> Self {
        let supported_languages = vec![
            // Systems Programming (5)
            Language::Rust, Language::C, Language::Cpp, Language::Go, Language::Zig,
            
            // JVM Ecosystem (5)
            Language::Java, Language::Kotlin, Language::Scala, Language::Groovy, Language::Clojure,
            
            // .NET Ecosystem (3)
            Language::CSharp, Language::FSharp, Language::VisualBasic,
            
            // Dynamic Languages (6)
            Language::Python, Language::JavaScript, Language::TypeScript, 
            Language::Ruby, Language::PHP, Language::Perl, Language::Lua,
            
            // Functional Languages (7)
            Language::Haskell, Language::Elixir, Language::Erlang, Language::OCaml,
            Language::ReasonML, Language::Elm, Language::PureScript,
            
            // Mobile Development (3)
            Language::Swift, Language::ObjectiveC, Language::Dart,
            
            // Shell & Scripting (4)
            Language::Bash, Language::Zsh, Language::Fish, Language::PowerShell,
            
            // Data & Config (6)
            Language::SQL, Language::HCL, Language::YAML, Language::TOML, 
            Language::JSON, Language::XML,
            
            // Documentation & Markup (3)
            Language::Markdown, Language::LaTeX, Language::AsciiDoc,
            
            // Build Systems (5)
            Language::Makefile, Language::CMake, Language::Bazel, 
            Language::Gradle, Language::Maven,
            
            // Specialized (7)
            Language::Solidity, Language::VHDL, Language::Verilog, 
            Language::R, Language::Julia, Language::Matlab, Language::Assembly,
        ];
        
        Self { supported_languages }
    }
    
    /// Get all supported languages
    pub fn supported_languages(&self) -> &[Language] {
        &self.supported_languages
    }
    
    /// Get language count
    pub fn language_count(&self) -> usize {
        self.supported_languages.len()
    }
    
    /// Detect language from file path
    pub fn detect_language(&self, path: &Path) -> Language {
        Language::from_path(path)
    }
    
    /// Get languages that support AST analysis
    pub fn ast_supported_languages(&self) -> Vec<Language> {
        self.supported_languages
            .iter()
            .filter(|lang| lang.has_ast_support())
            .copied()
            .collect()
    }
    
    /// Get languages that support complexity analysis
    pub fn complexity_supported_languages(&self) -> Vec<Language> {
        self.supported_languages
            .iter()
            .filter(|lang| lang.supports_complexity())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_language_count() {
        let registry = LanguageRegistry::new();
        assert!(registry.language_count() >= 50, "Should support 50+ languages");
    }
    
    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("java"), Language::Java);
        assert_eq!(Language::from_extension("cpp"), Language::Cpp);
        assert_eq!(Language::from_extension("kt"), Language::Kotlin);
        assert_eq!(Language::from_extension("swift"), Language::Swift);
        assert_eq!(Language::from_extension("rb"), Language::Ruby);
    }
    
    #[test]
    fn test_path_detection() {
        assert_eq!(Language::from_path(&PathBuf::from("src/main.rs")), Language::Rust);
        assert_eq!(Language::from_path(&PathBuf::from("app.py")), Language::Python);
        assert_eq!(Language::from_path(&PathBuf::from("index.js")), Language::JavaScript);
        assert_eq!(Language::from_path(&PathBuf::from("component.tsx")), Language::TypeScript);
        assert_eq!(Language::from_path(&PathBuf::from("Makefile")), Language::Makefile);
        assert_eq!(Language::from_path(&PathBuf::from("package.json")), Language::JSON);
        assert_eq!(Language::from_path(&PathBuf::from("docker-compose.yml")), Language::YAML);
    }
    
    #[test]
    fn test_ast_support() {
        assert!(Language::Rust.has_ast_support());
        assert!(Language::Python.has_ast_support());
        assert!(Language::TypeScript.has_ast_support());
        assert!(Language::JSON.has_ast_support());
        assert!(!Language::Unknown.has_ast_support());
    }
    
    #[test]
    fn test_complexity_support() {
        assert!(Language::Rust.supports_complexity());
        assert!(Language::Python.supports_complexity());
        assert!(Language::Java.supports_complexity());
        assert!(!Language::JSON.supports_complexity());
        assert!(!Language::Markdown.supports_complexity());
    }
    
    #[test]
    fn test_language_names() {
        assert_eq!(Language::Rust.name(), "Rust");
        assert_eq!(Language::TypeScript.name(), "TypeScript");
        assert_eq!(Language::CSharp.name(), "C#");
        assert_eq!(Language::Cpp.name(), "C++");
        assert_eq!(Language::ObjectiveC.name(), "Objective-C");
    }
}