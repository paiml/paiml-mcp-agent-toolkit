/// Language detection for mutation testing
///
/// Detects programming language from file extensions to enable multi-language
/// mutation testing support.
///
/// Sprint 63 - Multi-Language Mutation Testing Support

use std::path::Path;

/// Supported programming languages for mutation testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Rust programming language
    Rust,
    /// Python programming language
    Python,
    /// TypeScript programming language
    TypeScript,
    /// JavaScript programming language
    JavaScript,
    /// Go programming language
    Go,
    /// C++ programming language
    Cpp,
    /// Unsupported or unknown language
    Unsupported,
}

impl Language {
    /// Detect language from file extension
    ///
    /// # Arguments
    /// * `path` - Path to the source file
    ///
    /// # Returns
    /// The detected language, or `Language::Unsupported` if not recognized
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::services::mutation::language_detector::Language;
    ///
    /// let lang = Language::from_extension(Path::new("foo.rs"));
    /// assert_eq!(lang, Language::Rust);
    ///
    /// let lang = Language::from_extension(Path::new("bar.py"));
    /// assert_eq!(lang, Language::Python);
    /// ```
    pub fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("ts") => Language::TypeScript,
            Some("tsx") => Language::TypeScript,
            Some("js") => Language::JavaScript,
            Some("jsx") => Language::JavaScript,
            Some("go") => Language::Go,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hxx") | Some("h") => {
                Language::Cpp
            }
            _ => Language::Unsupported,
        }
    }

    /// Get the human-readable name of the language
    ///
    /// # Returns
    /// The language name as a string
    ///
    /// # Examples
    /// ```
    /// use pmat::services::mutation::language_detector::Language;
    ///
    /// assert_eq!(Language::Rust.name(), "Rust");
    /// assert_eq!(Language::Python.name(), "Python");
    /// assert_eq!(Language::TypeScript.name(), "TypeScript");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Go => "Go",
            Language::Cpp => "C++",
            Language::Unsupported => "Unsupported",
        }
    }

    /// Check if the language is currently supported for mutation testing
    ///
    /// # Returns
    /// `true` if mutation operators exist for this language
    ///
    /// # Examples
    /// ```
    /// use pmat::services::mutation::language_detector::Language;
    ///
    /// assert!(Language::Rust.is_supported());
    /// assert!(!Language::Unsupported.is_supported());
    /// ```
    pub fn is_supported(&self) -> bool {
        !matches!(self, Language::Unsupported)
    }

    /// Get file extensions for this language
    ///
    /// # Returns
    /// A vector of file extensions (without the dot)
    ///
    /// # Examples
    /// ```
    /// use pmat::services::mutation::language_detector::Language;
    ///
    /// assert_eq!(Language::Rust.extensions(), vec!["rs"]);
    /// assert_eq!(Language::Python.extensions(), vec!["py"]);
    /// assert!(Language::TypeScript.extensions().contains(&"ts"));
    /// ```
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Rust => vec!["rs"],
            Language::Python => vec!["py"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::JavaScript => vec!["js", "jsx"],
            Language::Go => vec!["go"],
            Language::Cpp => vec!["cpp", "cc", "cxx", "hpp", "hxx", "h"],
            Language::Unsupported => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rust_detection() {
        assert_eq!(
            Language::from_extension(Path::new("foo.rs")),
            Language::Rust
        );
        assert_eq!(
            Language::from_extension(Path::new("/path/to/main.rs")),
            Language::Rust
        );
    }

    #[test]
    fn test_python_detection() {
        assert_eq!(
            Language::from_extension(Path::new("foo.py")),
            Language::Python
        );
        assert_eq!(
            Language::from_extension(Path::new("/path/to/script.py")),
            Language::Python
        );
    }

    #[test]
    fn test_typescript_detection() {
        assert_eq!(
            Language::from_extension(Path::new("foo.ts")),
            Language::TypeScript
        );
        assert_eq!(
            Language::from_extension(Path::new("Component.tsx")),
            Language::TypeScript
        );
    }

    #[test]
    fn test_javascript_detection() {
        assert_eq!(
            Language::from_extension(Path::new("foo.js")),
            Language::JavaScript
        );
        assert_eq!(
            Language::from_extension(Path::new("Component.jsx")),
            Language::JavaScript
        );
    }

    #[test]
    fn test_go_detection() {
        assert_eq!(
            Language::from_extension(Path::new("main.go")),
            Language::Go
        );
    }

    #[test]
    fn test_cpp_detection() {
        assert_eq!(
            Language::from_extension(Path::new("main.cpp")),
            Language::Cpp
        );
        assert_eq!(
            Language::from_extension(Path::new("main.cc")),
            Language::Cpp
        );
        assert_eq!(
            Language::from_extension(Path::new("main.cxx")),
            Language::Cpp
        );
        assert_eq!(
            Language::from_extension(Path::new("header.hpp")),
            Language::Cpp
        );
        assert_eq!(
            Language::from_extension(Path::new("header.hxx")),
            Language::Cpp
        );
        assert_eq!(
            Language::from_extension(Path::new("header.h")),
            Language::Cpp
        );
    }

    #[test]
    fn test_unsupported_detection() {
        assert_eq!(
            Language::from_extension(Path::new("foo.txt")),
            Language::Unsupported
        );
        assert_eq!(
            Language::from_extension(Path::new("foo.md")),
            Language::Unsupported
        );
        assert_eq!(
            Language::from_extension(Path::new("README")),
            Language::Unsupported
        );
    }

    #[test]
    fn test_language_names() {
        assert_eq!(Language::Rust.name(), "Rust");
        assert_eq!(Language::Python.name(), "Python");
        assert_eq!(Language::TypeScript.name(), "TypeScript");
        assert_eq!(Language::JavaScript.name(), "JavaScript");
        assert_eq!(Language::Go.name(), "Go");
        assert_eq!(Language::Cpp.name(), "C++");
        assert_eq!(Language::Unsupported.name(), "Unsupported");
    }

    #[test]
    fn test_is_supported() {
        assert!(Language::Rust.is_supported());
        assert!(Language::Python.is_supported());
        assert!(Language::TypeScript.is_supported());
        assert!(Language::JavaScript.is_supported());
        assert!(Language::Go.is_supported());
        assert!(Language::Cpp.is_supported());
        assert!(!Language::Unsupported.is_supported());
    }

    #[test]
    fn test_extensions() {
        assert_eq!(Language::Rust.extensions(), vec!["rs"]);
        assert_eq!(Language::Python.extensions(), vec!["py"]);
        assert_eq!(Language::TypeScript.extensions(), vec!["ts", "tsx"]);
        assert_eq!(Language::JavaScript.extensions(), vec!["js", "jsx"]);
        assert_eq!(Language::Go.extensions(), vec!["go"]);
        assert_eq!(
            Language::Cpp.extensions(),
            vec!["cpp", "cc", "cxx", "hpp", "hxx", "h"]
        );
        assert_eq!(Language::Unsupported.extensions(), Vec::<&str>::new());
    }

    #[test]
    fn test_case_sensitivity() {
        // Extensions should be case-sensitive (lowercase expected)
        assert_eq!(
            Language::from_extension(Path::new("foo.RS")),
            Language::Unsupported
        );
        assert_eq!(
            Language::from_extension(Path::new("foo.PY")),
            Language::Unsupported
        );
    }
}
