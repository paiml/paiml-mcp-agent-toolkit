impl Language {
    /// Convert enum variant to array index (Toyota Way: O(1) lookup)
    fn to_index(self) -> usize {
        self as usize
    }

    /// Get file extensions associated with this language (Toyota Way: ≤3 complexity)
    #[must_use]
    pub fn extensions(&self) -> &'static [&'static str] {
        LANGUAGE_INFO[(*self).to_index()].extensions
    }

    /// Get human-readable name for this language (Toyota Way: ≤3 complexity)  
    #[must_use]
    pub fn name(&self) -> &'static str {
        LANGUAGE_INFO[(*self).to_index()].name
    }

    /// Detect language from file extension
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        debug_assert!(!ext.is_empty(), "ext must not be empty");
        let ext = ext.to_lowercase();

        // Check all languages for matching extensions
        for &lang in &[
            Language::Rust,
            Language::C,
            Language::Cpp,
            Language::Go,
            Language::Zig,
            Language::Java,
            Language::Kotlin,
            Language::Scala,
            Language::Groovy,
            Language::Clojure,
            Language::CSharp,
            Language::FSharp,
            Language::VisualBasic,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Ruby,
            Language::PHP,
            Language::Perl,
            Language::Lua,
            Language::Haskell,
            Language::Elixir,
            Language::Erlang,
            Language::OCaml,
            Language::ReasonML,
            Language::Elm,
            Language::PureScript,
            Language::Lean,
            Language::Swift,
            Language::ObjectiveC,
            Language::Dart,
            Language::Bash,
            Language::Zsh,
            Language::Fish,
            Language::PowerShell,
            Language::SQL,
            Language::HCL,
            Language::YAML,
            Language::TOML,
            Language::JSON,
            Language::XML,
            Language::Markdown,
            Language::LaTeX,
            Language::AsciiDoc,
            Language::Makefile,
            Language::CMake,
            Language::Bazel,
            Language::Gradle,
            Language::Maven,
            Language::Solidity,
            Language::VHDL,
            Language::Verilog,
            Language::R,
            Language::Julia,
            Language::Matlab,
            Language::Assembly,
            Language::PTX,
        ] {
            if lang.extensions().contains(&ext.as_str()) {
                return lang;
            }
        }

        Language::Unknown
    }

    /// Detect language from file path
    #[must_use]
    pub fn from_path(path: &Path) -> Self {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
                "lakefile.lean" | "lean-toolchain" => return Language::Lean,
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

    /// Check if language has full AST parsing support
    #[must_use]
    pub fn has_ast_support(&self) -> bool {
        match self {
            // Languages with full AST support (existing implementations)
            Language::Rust
            | Language::TypeScript
            | Language::JavaScript
            | Language::Python
            | Language::C
            | Language::Cpp
            | Language::Kotlin
            | Language::Makefile => true,

            // Languages that can be analyzed with pattern matching
            Language::Go
            | Language::Java
            | Language::CSharp
            | Language::Swift
            | Language::Ruby
            | Language::PHP
            | Language::Bash
            | Language::SQL
            | Language::Lua
            | Language::Lean => true,

            // Configuration and markup languages (structure parsing)
            Language::JSON
            | Language::YAML
            | Language::TOML
            | Language::XML
            | Language::Markdown => true,

            // Others need basic support for now
            _ => false,
        }
    }

    /// Check if language supports complexity analysis
    #[must_use]
    pub fn supports_complexity(&self) -> bool {
        match self {
            // Programming languages that have control flow
            Language::Rust
            | Language::C
            | Language::Cpp
            | Language::Go
            | Language::Zig
            | Language::Java
            | Language::Kotlin
            | Language::Scala
            | Language::Groovy
            | Language::CSharp
            | Language::FSharp
            | Language::Python
            | Language::JavaScript
            | Language::TypeScript
            | Language::Ruby
            | Language::PHP
            | Language::Perl
            | Language::Haskell
            | Language::Elixir
            | Language::Swift
            | Language::ObjectiveC
            | Language::Dart
            | Language::Bash
            | Language::PowerShell
            | Language::R
            | Language::Julia
            | Language::Matlab
            | Language::Lua
            | Language::Lean => true,

            // Markup, config, and data languages don't have complexity
            _ => false,
        }
    }
}
