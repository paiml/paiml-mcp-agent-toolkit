/// Location system for precise code positioning
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub file_path: PathBuf,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePos(pub u32);

impl std::hash::Hash for Location {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Content-addressed hashing for deterministic cache keys
        self.file_path.hash(state);
        self.span.start.0.hash(state);
        // End position omitted for prefix matching scenarios
    }
}

impl Location {
    /// Creates a new location from a file path and byte positions.
    ///
    /// # Parameters
    ///
    /// * `file_path` - The path to the source file
    /// * `start` - Starting byte position in the file
    /// * `end` - Ending byte position in the file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{Location, BytePos, Span};
    /// use std::path::PathBuf;
    ///
    /// let location = Location::new(
    ///     PathBuf::from("src/main.rs"),
    ///     100,
    ///     150
    /// );
    ///
    /// assert_eq!(location.file_path, PathBuf::from("src/main.rs"));
    /// assert_eq!(location.span.start.0, 100);
    /// assert_eq!(location.span.end.0, 150);
    /// assert_eq!(location.span.len(), 50);
    /// ```
    #[must_use]
    pub fn new(file_path: PathBuf, start: u32, end: u32) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            file_path,
            span: Span {
                start: BytePos(start),
                end: BytePos(end),
            },
        }
    }

    /// Checks if this location completely contains another location.
    ///
    /// Two locations must be in the same file, and this location's span
    /// must completely encompass the other location's span.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::Location;
    /// use std::path::PathBuf;
    ///
    /// let file = PathBuf::from("test.rs");
    /// let outer = Location::new(file.clone(), 0, 100);
    /// let inner = Location::new(file.clone(), 10, 50);
    /// let separate = Location::new(PathBuf::from("other.rs"), 0, 100);
    ///
    /// assert!(outer.contains(&inner));
    /// assert!(!inner.contains(&outer));
    /// assert!(!outer.contains(&separate)); // Different files
    /// ```
    #[must_use]
    pub fn contains(&self, other: &Location) -> bool {
        self.file_path == other.file_path
            && self.span.start <= other.span.start
            && self.span.end >= other.span.end
    }

    /// Checks if this location overlaps with another location.
    ///
    /// Two locations overlap if they are in the same file and their
    /// byte ranges intersect (even partially).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{Location, BytePos, Span};
    /// use std::path::PathBuf;
    ///
    /// let file = PathBuf::from("test.rs");
    /// let loc1 = Location::new(file.clone(), 0, 50);
    /// let loc2 = Location::new(file.clone(), 25, 75); // Overlaps
    /// let loc3 = Location::new(file.clone(), 100, 150); // No overlap
    /// let loc4 = Location::new(PathBuf::from("other.rs"), 0, 100); // Different file
    ///
    /// assert!(loc1.overlaps(&loc2));
    /// assert!(loc2.overlaps(&loc1));
    /// assert!(!loc1.overlaps(&loc3));
    /// assert!(!loc1.overlaps(&loc4));
    /// ```
    #[must_use]
    pub fn overlaps(&self, other: &Location) -> bool {
        self.file_path == other.file_path
            && self.span.start < other.span.end
            && other.span.start < self.span.end
    }
}

impl Span {
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start: BytePos(start),
            end: BytePos(end),
        }
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.end.0 - self.start.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.0 >= self.end.0
    }

    #[must_use]
    pub fn contains(&self, pos: BytePos) -> bool {
        self.start <= pos && pos < self.end
    }
}

impl BytePos {
    #[must_use]
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    /// Creates a `BytePos` from a usize position
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::BytePos;
    ///
    /// let pos = BytePos::from_usize(42);
    /// assert_eq!(pos.to_usize(), 42);
    /// ```
    #[must_use]
    pub fn from_usize(pos: usize) -> Self {
        Self(pos as u32)
    }
}

/// Qualified name for symbol resolution
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QualifiedName {
    pub module_path: Vec<String>,
    pub name: String,
    pub disambiguator: Option<u32>, // For overloaded names
}

impl QualifiedName {
    /// Creates a new qualified name from module path and name components.
    ///
    /// # Parameters
    ///
    /// * `module_path` - Vector of module/namespace components
    /// * `name` - The final name component (function, type, etc.)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::QualifiedName;
    ///
    /// let qname = QualifiedName::new(
    ///     vec!["std".to_string(), "collections".to_string()],
    ///     "HashMap".to_string()
    /// );
    ///
    /// assert_eq!(qname.module_path, vec!["std", "collections"]);
    /// assert_eq!(qname.name, "HashMap");
    /// assert!(qname.disambiguator.is_none());
    /// assert_eq!(qname.to_qualified_string(), "std::collections::HashMap");
    /// ```
    #[must_use]
    pub fn new(module_path: Vec<String>, name: String) -> Self {
        debug_assert!(!module_path.is_empty(), "module_path must not be empty");
        Self {
            module_path,
            name,
            disambiguator: None,
        }
    }

    #[must_use]
    pub fn with_disambiguator(mut self, disambiguator: u32) -> Self {
        self.disambiguator = Some(disambiguator);
        self
    }

    /// Creates a qualified name from a string representation.
    ///
    /// Parses strings in the format "`module::submodule::Name`" where
    /// "::" separates module components from the final name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::QualifiedName;
    ///
    /// // Simple name without module path
    /// let simple = QualifiedName::from_string("main").expect("internal error");
    /// assert_eq!(simple.name, "main");
    /// assert!(simple.module_path.is_empty());
    ///
    /// // Fully qualified name
    /// let qualified = QualifiedName::from_string("std::collections::HashMap").expect("internal error");
    /// assert_eq!(qualified.module_path, vec!["std", "collections"]);
    /// assert_eq!(qualified.name, "HashMap");
    ///
    /// // Error case
    /// assert!(QualifiedName::from_string("").is_err());
    /// ```
    pub fn from_string(qualified_str: &str) -> Result<Self, &'static str> {
        debug_assert!(!qualified_str.is_empty(), "qualified_str must not be empty");
        if qualified_str.is_empty() {
            return Err("Empty qualified name");
        }

        let parts: Vec<&str> = qualified_str.split("::").collect();
        let name = (*parts.last().expect("internal error")).to_string();
        if name.is_empty() {
            return Err("Empty qualified name");
        }

        let module_path = parts[..parts.len() - 1]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        Ok(Self {
            module_path,
            name,
            disambiguator: None,
        })
    }

    /// Converts the qualified name back to its string representation.
    ///
    /// Creates a string in the format "`module::submodule::Name`", with
    /// optional disambiguator suffix "#N" for overloaded names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::QualifiedName;
    ///
    /// let qname = QualifiedName::new(
    ///     vec!["crate".to_string(), "module".to_string()],
    ///     "function".to_string()
    /// );
    /// assert_eq!(qname.to_qualified_string(), "crate::module::function");
    ///
    /// // With disambiguator
    /// let overloaded = qname.with_disambiguator(1);
    /// assert_eq!(overloaded.to_qualified_string(), "crate::module::function#1");
    ///
    /// // Simple name without modules
    /// let simple = QualifiedName::new(vec![], "main".to_string());
    /// assert_eq!(simple.to_qualified_string(), "main");
    /// ```
    #[must_use]
    pub fn to_qualified_string(&self) -> String {
        let mut result = self.module_path.join("::");
        if !result.is_empty() {
            result.push_str("::");
        }
        result.push_str(&self.name);
        if let Some(disambiguator) = self.disambiguator {
            result.push_str(&format!("#{disambiguator}"));
        }
        result
    }
}

impl std::str::FromStr for QualifiedName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug_assert!(!s.is_empty(), "s must not be empty");
        Self::from_string(s)
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_qualified_string())
    }
}
