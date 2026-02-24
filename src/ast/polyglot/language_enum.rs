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
