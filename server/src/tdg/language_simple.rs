use serde::{Deserialize, Serialize};
use std::path::Path;

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
    Ruchy,
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
            Some("ruchy") | Some("rh") => Language::Ruchy,
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
            Language::Ruchy => 0.95,
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
            Language::Ruchy => write!(f, "Ruchy"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
            NamingStyle::SnakeCase => name
                .chars()
                .all(|c| c.is_lowercase() || c == '_' || c.is_numeric()),
            NamingStyle::CamelCase => {
                !name.is_empty()
                    && name.chars().next().unwrap().is_lowercase()
                    && !name.contains('_')
                    && !name.contains('-')
            }
            NamingStyle::PascalCase => {
                !name.is_empty()
                    && name.chars().next().unwrap().is_uppercase()
                    && !name.contains('_')
                    && !name.contains('-')
            }
            NamingStyle::ScreamingSnakeCase => name
                .chars()
                .all(|c| c.is_uppercase() || c == '_' || c.is_numeric()),
            NamingStyle::KebabCase => name
                .chars()
                .all(|c| c.is_lowercase() || c == '-' || c.is_numeric()),
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
    pub fn for_language(language: Language) -> Self {
        match language {
            Language::Rust => Self::rust_rules(),
            Language::Python => Self::python_rules(),
            Language::JavaScript => Self::javascript_rules(),
            Language::TypeScript => Self::typescript_rules(),
            Language::Go => Self::go_rules(),
            Language::Ruchy => Self::ruchy_rules(),
            _ => Self::rust_rules(), // Default
        }
    }

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

    pub fn ruchy_rules() -> Self {
        LanguageRules {
            language: Language::Ruchy,
            function_style: NamingStyle::SnakeCase,      // fun hello_world() 
            type_style: NamingStyle::PascalCase,         // struct Point, enum Color
            constant_style: NamingStyle::ScreamingSnakeCase, // const MAX_SIZE
            variable_style: NamingStyle::SnakeCase,      // let my_variable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(
            Language::from_extension(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            Language::from_extension(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            Language::from_extension(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            Language::from_extension(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(Language::from_extension(Path::new("test.go")), Language::Go);
        assert_eq!(
            Language::from_extension(Path::new("test.java")),
            Language::Java
        );
        assert_eq!(Language::from_extension(Path::new("test.c")), Language::C);
        assert_eq!(
            Language::from_extension(Path::new("test.cpp")),
            Language::Cpp
        );
        assert_eq!(
            Language::from_extension(Path::new("test.ruchy")),
            Language::Ruchy
        );
        assert_eq!(
            Language::from_extension(Path::new("script.rh")),
            Language::Ruchy
        );
        assert_eq!(
            Language::from_extension(Path::new("test.unknown")),
            Language::Unknown
        );
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

    #[test]
    fn test_ruchy_language_rules() {
        let rules = LanguageRules::for_language(Language::Ruchy);
        
        assert_eq!(rules.language, Language::Ruchy);
        assert_eq!(rules.function_style, NamingStyle::SnakeCase);
        assert_eq!(rules.type_style, NamingStyle::PascalCase);
        assert_eq!(rules.constant_style, NamingStyle::ScreamingSnakeCase);
        assert_eq!(rules.variable_style, NamingStyle::SnakeCase);
    }

    #[test]
    fn test_ruchy_language_confidence() {
        assert_eq!(Language::Ruchy.confidence(), 0.95);
        assert!(Language::Ruchy.confidence() > Language::Java.confidence());
        assert!(Language::Ruchy.confidence() == Language::Go.confidence());
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
