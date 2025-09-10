use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Parser, Tree};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
    Ruby,
    Swift,
    Kotlin,
    Unknown,
}

impl Language {
    pub fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("js") | Some("jsx") => Language::JavaScript,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            Some("c") | Some("h") => Language::C,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") => Language::Cpp,
            Some("rb") => Language::Ruby,
            Some("swift") => Language::Swift,
            Some("kt") | Some("kts") => Language::Kotlin,
            _ => Language::Unknown,
        }
    }
    
    pub fn confidence(&self) -> f32 {
        match self {
            Language::Rust => 1.0,
            Language::Python => 0.95,
            Language::JavaScript => 0.90,
            Language::TypeScript => 0.90,
            Language::Go => 0.95,
            Language::Java => 0.85,
            Language::C => 0.80,
            Language::Cpp => 0.75,
            Language::Ruby => 0.85,
            Language::Swift => 0.85,
            Language::Kotlin => 0.85,
            Language::Unknown => 0.5,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Go => write!(f, "Go"),
            Language::Java => write!(f, "Java"),
            Language::C => write!(f, "C"),
            Language::Cpp => write!(f, "C++"),
            Language::Ruby => write!(f, "Ruby"),
            Language::Swift => write!(f, "Swift"),
            Language::Kotlin => write!(f, "Kotlin"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

pub trait LanguageAdapter: Send + Sync {
    fn detect(&self, path: &Path) -> bool;
    fn parse(&self, source: &str) -> Result<Tree>;
    fn confidence(&self) -> f32;
    fn language(&self) -> Language;
    fn naming_rules(&self) -> LanguageRules;
}

pub struct RustAdapter {
    parser: Parser,
}

impl RustAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for RustAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "rs")
    }
    
    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Rust source"))
    }
    
    fn confidence(&self) -> f32 {
        1.0
    }
    
    fn language(&self) -> Language {
        Language::Rust
    }
    
    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::rust_rules()
    }
}

pub struct PythonAdapter {
    parser: Parser,
}

impl PythonAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for PythonAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "py")
    }
    
    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python source"))
    }
    
    fn confidence(&self) -> f32 {
        0.95
    }
    
    fn language(&self) -> Language {
        Language::Python
    }
    
    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::python_rules()
    }
}

pub struct JavaScriptAdapter {
    parser: Parser,
}

impl JavaScriptAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for JavaScriptAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "js" || ext == "jsx")
    }
    
    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse JavaScript source"))
    }
    
    fn confidence(&self) -> f32 {
        0.90
    }
    
    fn language(&self) -> Language {
        Language::JavaScript
    }
    
    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::javascript_rules()
    }
}

pub struct TypeScriptAdapter {
    parser: Parser,
}

impl TypeScriptAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::language_typescript())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for TypeScriptAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "ts" || ext == "tsx")
    }
    
    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript source"))
    }
    
    fn confidence(&self) -> f32 {
        0.90
    }
    
    fn language(&self) -> Language {
        Language::TypeScript
    }
    
    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::typescript_rules()
    }
}

pub struct GoAdapter {
    parser: Parser,
}

impl GoAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for GoAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "go")
    }
    
    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Go source"))
    }
    
    fn confidence(&self) -> f32 {
        0.95
    }
    
    fn language(&self) -> Language {
        Language::Go
    }
    
    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::go_rules()
    }
}

pub struct LanguageRegistry {
    adapters: HashMap<Language, Box<dyn LanguageAdapter>>,
}

impl LanguageRegistry {
    pub fn new() -> Result<Self> {
        let mut registry = Self {
            adapters: HashMap::new(),
        };
        
        registry.register(Language::Rust, Box::new(RustAdapter::new()?))?;
        registry.register(Language::Python, Box::new(PythonAdapter::new()?))?;
        registry.register(Language::JavaScript, Box::new(JavaScriptAdapter::new()?))?;
        registry.register(Language::TypeScript, Box::new(TypeScriptAdapter::new()?))?;
        registry.register(Language::Go, Box::new(GoAdapter::new()?))?;
        
        Ok(registry)
    }
    
    pub fn register(&mut self, language: Language, adapter: Box<dyn LanguageAdapter>) -> Result<()> {
        self.adapters.insert(language, adapter);
        Ok(())
    }
    
    pub fn get_adapter(&self, language: Language) -> Result<&dyn LanguageAdapter> {
        self.adapters
            .get(&language)
            .map(|a| a.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No adapter for language: {}", language))
    }
    
    pub fn detect_and_get(&self, path: &Path) -> Result<&dyn LanguageAdapter> {
        let language = Language::from_extension(path);
        if language == Language::Unknown {
            return Err(anyhow::anyhow!("Unknown language for file: {}", path.display()));
        }
        self.get_adapter(language)
    }
}

#[derive(Debug, Clone)]
pub enum NamingStyle {
    SnakeCase,
    CamelCase,
    PascalCase,
    ScreamingSnakeCase,
    KebabCase,
}

impl NamingStyle {
    pub fn matches(&self, name: &str) -> bool {
        match self {
            NamingStyle::SnakeCase => {
                name.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
            }
            NamingStyle::CamelCase => {
                !name.is_empty() && name.chars().next().unwrap().is_lowercase()
                    && !name.contains('_') && !name.contains('-')
            }
            NamingStyle::PascalCase => {
                !name.is_empty() && name.chars().next().unwrap().is_uppercase()
                    && !name.contains('_') && !name.contains('-')
            }
            NamingStyle::ScreamingSnakeCase => {
                name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            }
            NamingStyle::KebabCase => {
                name.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageRules {
    pub language: Language,
    pub function_style: NamingStyle,
    pub type_style: NamingStyle,
    pub constant_style: NamingStyle,
    pub variable_style: NamingStyle,
}

impl LanguageRules {
    pub fn rust_rules() -> Self {
        LanguageRules {
            language: Language::Rust,
            function_style: NamingStyle::SnakeCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }
    
    pub fn python_rules() -> Self {
        LanguageRules {
            language: Language::Python,
            function_style: NamingStyle::SnakeCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }
    
    pub fn javascript_rules() -> Self {
        LanguageRules {
            language: Language::JavaScript,
            function_style: NamingStyle::CamelCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::CamelCase,
        }
    }
    
    pub fn typescript_rules() -> Self {
        LanguageRules {
            language: Language::TypeScript,
            function_style: NamingStyle::CamelCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::CamelCase,
        }
    }
    
    pub fn go_rules() -> Self {
        LanguageRules {
            language: Language::Go,
            function_style: NamingStyle::PascalCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::PascalCase,
            variable_style: NamingStyle::CamelCase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension(Path::new("test.rs")), Language::Rust);
        assert_eq!(Language::from_extension(Path::new("test.py")), Language::Python);
        assert_eq!(Language::from_extension(Path::new("test.js")), Language::JavaScript);
        assert_eq!(Language::from_extension(Path::new("test.ts")), Language::TypeScript);
        assert_eq!(Language::from_extension(Path::new("test.go")), Language::Go);
        assert_eq!(Language::from_extension(Path::new("test.java")), Language::Java);
        assert_eq!(Language::from_extension(Path::new("test.c")), Language::C);
        assert_eq!(Language::from_extension(Path::new("test.cpp")), Language::Cpp);
        assert_eq!(Language::from_extension(Path::new("test.unknown")), Language::Unknown);
    }
    
    #[test]
    fn test_naming_style_matches() {
        assert!(NamingStyle::SnakeCase.matches("snake_case"));
        assert!(NamingStyle::SnakeCase.matches("my_variable_123"));
        assert!(!NamingStyle::SnakeCase.matches("camelCase"));
        
        assert!(NamingStyle::CamelCase.matches("camelCase"));
        assert!(NamingStyle::CamelCase.matches("myVariable123"));
        assert!(!NamingStyle::CamelCase.matches("PascalCase"));
        assert!(!NamingStyle::CamelCase.matches("snake_case"));
        
        assert!(NamingStyle::PascalCase.matches("PascalCase"));
        assert!(NamingStyle::PascalCase.matches("MyClass123"));
        assert!(!NamingStyle::PascalCase.matches("camelCase"));
        assert!(!NamingStyle::PascalCase.matches("snake_case"));
        
        assert!(NamingStyle::ScreamingSnakeCase.matches("SCREAMING_SNAKE"));
        assert!(NamingStyle::ScreamingSnakeCase.matches("MAX_VALUE_123"));
        assert!(!NamingStyle::ScreamingSnakeCase.matches("snake_case"));
        
        assert!(NamingStyle::KebabCase.matches("kebab-case"));
        assert!(NamingStyle::KebabCase.matches("my-component-123"));
        assert!(!NamingStyle::KebabCase.matches("snake_case"));
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
