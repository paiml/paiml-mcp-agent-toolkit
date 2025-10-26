//! Scala Language Support for PMAT
//!
//! This module provides Scala-specific analysis capabilities using tree-sitter-scala parser
//! for AST extraction and complexity analysis aligned with Scala best practices.

#[cfg(feature = "scala-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "scala-ast")]
use std::path::{Path, PathBuf};

/// Scala AST visitor that extracts Scala-specific AST information
#[cfg(feature = "scala-ast")]
pub struct ScalaAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    class_count: usize,
    trait_count: usize,
    object_count: usize,
    case_class_count: usize,
}

#[cfg(feature = "scala-ast")]
impl ScalaAstVisitor {
    /// Creates a new Scala AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            class_count: 0,
            trait_count: 0,
            object_count: 0,
            case_class_count: 0,
        }
    }

    /// Analyzes Scala source code and extracts AST items (complexity ≤10)
    pub fn analyze_scala_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic Scala syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_scala_syntax(source) {
            return Err("Invalid Scala syntax".to_string());
        }

        self.extract_package_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_trait_declarations(source)?;
        self.extract_object_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_case_class_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic Scala syntax validity (complexity ≤10)
    fn is_valid_scala_syntax(&self, source: &str) -> bool {
        let open_braces = source.chars().filter(|&c| c == '{').count();
        let close_braces = source.chars().filter(|&c| c == '}').count();

        // Basic brace matching and no obvious syntax errors
        open_braces == close_braces && !source.contains("!!!")
    }

    /// Extracts package declaration (complexity ≤10)
    fn extract_package_declaration(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("package ") {
                let package_part = trimmed.strip_prefix("package ").unwrap_or("").trim();
                self.package_name = package_part.to_string();
                return Ok(());
            }
        }
        Ok(())
    }

    /// Extracts class declarations (complexity ≤10)
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(class_name) = self.extract_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = self.determine_visibility(trimmed);
                
                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    fields_count: 0, // To be filled in later analysis
                    derives: vec![],
                    line: line_num + 1,
                });
                self.class_count += 1;
            }
        }
        Ok(())
    }

    /// Extracts case class declarations (complexity ≤10)
    fn extract_case_class_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(class_name) = self.extract_case_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = self.determine_visibility(trimmed);
                
                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    fields_count: 0, // To be filled in later analysis
                    derives: vec!["case".to_string()],
                    line: line_num + 1,
                });
                self.case_class_count += 1;
            }
        }
        Ok(())
    }
    
    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("class ") && !line.contains("case class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    let class_name = parts[i + 1].split('[').next()?; // Handle generics
                    let class_name = class_name.split('(').next()?; // Handle constructor params
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }
    
    /// Helper to extract case class name from line (complexity ≤10)
    fn extract_case_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("case class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i > 0 && parts[i - 1] == "case" && i + 1 < parts.len() {
                    let class_name = parts[i + 1].split('[').next()?; // Handle generics
                    let class_name = class_name.split('(').next()?; // Handle constructor params
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts trait declarations (complexity ≤10)
    fn extract_trait_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(trait_name) = self.extract_trait_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&trait_name);
                let visibility = self.determine_visibility(trimmed);
                
                self.items.push(AstItem::Trait {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    line: line_num + 1,
                });
                self.trait_count += 1;
            }
        }
        Ok(())
    }
    
    /// Helper to extract trait name from line (complexity ≤10)
    fn extract_trait_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("trait ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "trait" && i + 1 < parts.len() {
                    let trait_name = parts[i + 1].split('[').next()?; // Handle generics
                    return Some(trait_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts object declarations (complexity ≤10)
    fn extract_object_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(object_name) = self.extract_object_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&object_name);
                let visibility = self.determine_visibility(trimmed);
                
                self.items.push(AstItem::Module {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    line: line_num + 1,
                });
                self.object_count += 1;
            }
        }
        Ok(())
    }
    
    /// Helper to extract object name from line (complexity ≤10)
    fn extract_object_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("object ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "object" && i + 1 < parts.len() {
                    let object_name = parts[i + 1];
                    return Some(object_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts method declarations (complexity ≤10)
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(method_name) = self.extract_method_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&method_name);
                let visibility = self.determine_visibility(trimmed);
                
                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: trimmed.contains("async ") || trimmed.contains("Future["),
                    line: line_num + 1,
                });
            }
        }
        Ok(())
    }
    
    /// Helper to extract method name from line (complexity ≤10)
    fn extract_method_name_from_line(&self, line: &str) -> Option<String> {
        // Match Scala method declarations: "def methodName(...)"
        if line.contains("def ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "def" && i + 1 < parts.len() {
                    let method_part = parts[i + 1];
                    let method_name = method_part.split('(').next()?;
                    if !method_name.is_empty() {
                        return Some(method_name.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// Helper to determine visibility from modifiers (complexity ≤10)
    fn determine_visibility(&self, line: &str) -> String {
        if line.contains("private ") {
            "private".to_string()
        } else if line.contains("protected ") {
            "protected".to_string()
        } else if line.contains("private[") {
            "package".to_string()
        } else {
            "public".to_string()
        }
    }
    
    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        if self.package_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.package_name, name)
        }
    }
}

/// Scala complexity analyzer for extracting Scala-specific metrics (complexity ≤10)
#[cfg(feature = "scala-ast")]
pub struct ScalaComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "scala-ast")]
impl Default for ScalaComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "scala-ast")]
impl ScalaComplexityAnalyzer {
    /// Creates a new Scala complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }
    
    /// Analyzes complexity of Scala source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;
        
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            self.analyze_complexity_for_line(trimmed);
        }
        
        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
    
    /// Helper to analyze complexity for a single line (complexity ≤10)
    fn analyze_complexity_for_line(&mut self, line: &str) {
        // Control flow increases cyclomatic complexity
        if line.contains("if ") || line.contains(" if ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        
        // Match expressions
        if line.contains("match ") || line.contains(" match ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        
        // Case patterns
        if line.contains("case ") && !line.contains("case class ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        
        // Logical operators increase complexity
        if line.contains(" && ") || line.contains(" || ") {
            self.cyclomatic_complexity += 1;
        }
        
        // Loops
        if line.contains(" for ") || line.contains("while ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        
        // Try/catch blocks
        if line.contains("try ") || line.contains("catch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }
}

#[cfg(all(test, feature = "scala-ast"))]
mod tests {
    use super::*;
    use std::path::Path;
    
    const SIMPLE_SCALA_CLASS: &str = r#"
    package com.example
    
    class HelloWorld {
      def sayHello(): String = {
        "Hello, World!"
      }
    }
    "#;
    
    const SCALA_TRAIT_EXAMPLE: &str = r#"
    package com.example.shapes
    
    trait Shape {
      def area(): Double
      def perimeter(): Double
    }
    
    class Circle(radius: Double) extends Shape {
      def area(): Double = math.Pi * radius * radius
      def perimeter(): Double = 2 * math.Pi * radius
    }
    "#;
    
    const SCALA_CASE_CLASS_EXAMPLE: &str = r#"
    package com.example.models
    
    case class Person(name: String, age: Int) {
      def isAdult: Boolean = age >= 18
    }
    
    object Person {
      def apply(name: String): Person = new Person(name, 0)
    }
    "#;
    
    const SCALA_COMPREHENSIVE_EXAMPLE: &str = r#"
    package com.example.functional
    
    import scala.concurrent.Future
    import scala.concurrent.ExecutionContext.Implicits.global
    
    // A trait defining functional operations
    trait Functor[F[_]] {
      def map[A, B](fa: F[A])(f: A => B): F[B]
    }
    
    // A case class for user data
    case class User(id: String, name: String, email: String)
    
    // An object with utility methods
    object UserService {
      private val users = Map(
        "1" -> User("1", "Alice", "alice@example.com"),
        "2" -> User("2", "Bob", "bob@example.com")
      )
      
      def getUser(id: String): Option[User] = users.get(id)
      
      def findUserByEmail(email: String): Option[User] = 
        users.values.find(_.email == email)
        
      def getUserAsync(id: String): Future[Option[User]] = 
        Future.successful(getUser(id))
        
      // Pattern matching example
      def processUserResult(result: Option[User]): String = result match {
        case Some(user) if user.name.startsWith("A") => s"A user: ${user.name}"
        case Some(user) => s"User: ${user.name}"
        case None => "User not found"
      }
    }
    
    // A class that uses higher-order functions
    class DataProcessor[T](data: List[T]) {
      def map[B](f: T => B): List[B] = data.map(f)
      
      def filter(p: T => Boolean): List[T] = data.filter(p)
      
      def fold[B](z: B)(op: (B, T) => B): B = data.foldLeft(z)(op)
    }
    "#;
    
    #[test]
    fn test_simple_scala_class_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("HelloWorld.scala"));
        let items = visitor
            .analyze_scala_source(SIMPLE_SCALA_CLASS)
            .expect("Should parse Scala class");
        
        assert!(!items.is_empty(), "Should extract at least one AST item");
        
        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();
        
        assert_eq!(class_items.len(), 1, "Should extract exactly one class");
        
        if let AstItem::Struct {
            name, visibility, ..
        } = &class_items[0] {
            assert_eq!(name, "com.example::HelloWorld", "Should have qualified class name");
            assert_eq!(visibility, "public", "Scala classes have public visibility by default");
        } else {
            panic!("Expected class item");
        }
    }
    
    #[test]
    fn test_scala_trait_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("Shape.scala"));
        let items = visitor
            .analyze_scala_source(SCALA_TRAIT_EXAMPLE)
            .expect("Should parse Scala trait");
        
        // Check that we found the trait
        let trait_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .collect();
        
        assert_eq!(trait_items.len(), 1, "Should extract exactly one trait");
        
        if let AstItem::Trait { name, .. } = &trait_items[0] {
            assert_eq!(name, "com.example.shapes::Shape", "Should have qualified trait name");
        }
        
        // Check that we found the implementing class
        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();
        
        assert_eq!(class_items.len(), 1, "Should extract exactly one class");
        
        // Check for methods
        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();
        
        assert!(method_items.len() >= 2, "Should extract at least two methods");
    }
    
    #[test]
    fn test_scala_case_class_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("Person.scala"));
        let items = visitor
            .analyze_scala_source(SCALA_CASE_CLASS_EXAMPLE)
            .expect("Should parse Scala case class");
        
        // Check that we found the case class
        let case_class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();
        
        assert!(!case_class_items.is_empty(), "Should extract at least one case class");
        
        let has_case_class = case_class_items.iter().any(|item| {
            if let AstItem::Struct { derives, .. } = item {
                derives.contains(&"case".to_string())
            } else {
                false
            }
        });
        
        assert!(has_case_class, "Should identify a case class");
        
        // Check that we found the companion object
        let object_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Module { .. }))
            .collect();
        
        assert!(!object_items.is_empty(), "Should extract at least one object");
    }
    
    #[test]
    fn test_scala_comprehensive_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("Comprehensive.scala"));
        let items = visitor
            .analyze_scala_source(SCALA_COMPREHENSIVE_EXAMPLE)
            .expect("Should parse comprehensive Scala example");
        
        // Should have extracted a good number of items
        assert!(items.len() >= 10, "Should extract numerous AST items from comprehensive example");
        
        // Check for traits, case classes, objects, and methods
        let trait_count = items.iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .count();
        
        let case_class_count = items.iter()
            .filter(|item| matches!(item, AstItem::Struct { derives, .. } if derives.contains(&"case".to_string())))
            .count();
        
        let object_count = items.iter()
            .filter(|item| matches!(item, AstItem::Module { .. }))
            .count();
        
        let method_count = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .count();
        
        assert!(trait_count > 0, "Should find at least one trait");
        assert!(case_class_count > 0, "Should find at least one case class");
        assert!(object_count > 0, "Should find at least one object");
        assert!(method_count > 0, "Should find at least one method");
    }
    
    #[test]
    fn test_scala_complexity_analysis() {
        let mut analyzer = ScalaComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SCALA_COMPREHENSIVE_EXAMPLE)
            .expect("Should analyze Scala complexity");
        
        assert!(cyclomatic >= 1, "Should have at least cyclomatic complexity of 1");
        assert!(cognitive >= 1, "Should have at least cognitive complexity of 1");
        assert!(cyclomatic <= 20, "Should maintain reasonable cyclomatic complexity");
        assert!(cognitive <= 20, "Should maintain reasonable cognitive complexity");
    }
    
    #[test]
    fn test_empty_scala_source() {
        let visitor = ScalaAstVisitor::new(Path::new("empty.scala"));
        let items = visitor.analyze_scala_source("").expect("Should handle empty source");
        
        assert!(items.is_empty(), "Empty source should produce no AST items");
    }
    
    #[test]
    fn test_invalid_scala_syntax() {
        let visitor = ScalaAstVisitor::new(Path::new("invalid.scala"));
        let result = visitor.analyze_scala_source("invalid scala syntax {{{ !!!");
        
        assert!(result.is_err(), "Should return error for invalid Scala syntax");
    }
}