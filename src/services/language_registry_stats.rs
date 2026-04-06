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
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let supported_languages = vec![
            // Systems Programming (5)
            Language::Rust,
            Language::C,
            Language::Cpp,
            Language::Go,
            Language::Zig,
            // JVM Ecosystem (5)
            Language::Java,
            Language::Kotlin,
            Language::Scala,
            Language::Groovy,
            Language::Clojure,
            // .NET Ecosystem (3)
            Language::CSharp,
            Language::FSharp,
            Language::VisualBasic,
            // Dynamic Languages (6)
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Ruby,
            Language::PHP,
            Language::Perl,
            Language::Lua,
            // Functional Languages (7)
            Language::Haskell,
            Language::Elixir,
            Language::Erlang,
            Language::OCaml,
            Language::ReasonML,
            Language::Elm,
            Language::PureScript,
            // Proof Assistants (1)
            Language::Lean,
            // Mobile Development (3)
            Language::Swift,
            Language::ObjectiveC,
            Language::Dart,
            // Shell & Scripting (4)
            Language::Bash,
            Language::Zsh,
            Language::Fish,
            Language::PowerShell,
            // Data & Config (6)
            Language::SQL,
            Language::HCL,
            Language::YAML,
            Language::TOML,
            Language::JSON,
            Language::XML,
            // Documentation & Markup (3)
            Language::Markdown,
            Language::LaTeX,
            Language::AsciiDoc,
            // Build Systems (5)
            Language::Makefile,
            Language::CMake,
            Language::Bazel,
            Language::Gradle,
            Language::Maven,
            // Specialized (7)
            Language::Solidity,
            Language::VHDL,
            Language::Verilog,
            Language::R,
            Language::Julia,
            Language::Matlab,
            Language::Assembly,
        ];

        Self {
            supported_languages,
        }
    }

    /// Get all supported languages
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn supported_languages(&self) -> &[Language] {
        &self.supported_languages
    }

    /// Get language count
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn language_count(&self) -> usize {
        self.supported_languages.len()
    }

    /// Detect language from file path
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_language(&self, path: &Path) -> Language {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Language::from_path(path)
    }

    /// Get languages that support AST analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn ast_supported_languages(&self) -> Vec<Language> {
        self.supported_languages
            .iter()
            .filter(|lang| lang.has_ast_support())
            .copied()
            .collect()
    }

    /// Get languages that support complexity analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn complexity_supported_languages(&self) -> Vec<Language> {
        self.supported_languages
            .iter()
            .filter(|lang| lang.supports_complexity())
            .copied()
            .collect()
    }
}
