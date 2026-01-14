#![cfg(all(test, feature = "scala-ast", feature = "integration-tests"))]

use anyhow::Result;
use pmat::services::ast::AstRegistry;
use pmat::services::file_classifier::FileClassifier;
use pmat::services::context::AstItem;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use tokio::test;

/// Creates a temporary Scala file with the given content for integration testing
fn create_temp_scala_file(content: &str, filename: &str) -> Result<(std::path::PathBuf, TempDir)> {
    let temp_dir = TempDir::new()?;
    let scala_file_path = temp_dir.path().join(filename);

    let mut file = File::create(&scala_file_path)?;
    file.write_all(content.as_bytes())?;
    drop(file); // Close the file

    Ok((scala_file_path, temp_dir))
}

/// Basic integration test for Scala language support
#[test]
async fn test_scala_basic_integration() -> Result<()> {
    // Create a simple Scala file
    let scala_content = r#"
    package com.example.pmat.test
    
    class SimpleTest(message: String) {
        def printMessage(): Unit = {
            println(message)
        }
        
        def getMessage(): String = {
            message
        }
    }
    "#;
    
    let (scala_file_path, _temp_dir) = create_temp_scala_file(scala_content, "SimpleTest.scala")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Scala strategy
    let strategy = registry.get_strategy("scala")
        .ok_or_else(|| anyhow::anyhow!("Scala strategy not found"))?;
    
    let result = strategy.analyze(&scala_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "scala");
    assert!(!result.items.is_empty(), "Should find AST items in Scala file");
    
    // Check that we found the class
    let class_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();
    
    assert!(!class_items.is_empty(), "Should find at least one class");
    
    // Check that we found at least one method
    let method_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();
    
    assert!(!method_items.is_empty(), "Should find at least one method");
    
    Ok(())
}

/// Tests Scala case class detection
#[test]
async fn test_scala_case_class_integration() -> Result<()> {
    // Create a Scala file with a case class
    let scala_content = r#"
    package com.example.pmat.test
    
    case class Person(name: String, age: Int) {
        def isAdult: Boolean = age >= 18
    }
    
    object Person {
        def create(name: String): Person = new Person(name, 0)
    }
    "#;
    
    let (scala_file_path, _temp_dir) = create_temp_scala_file(scala_content, "Person.scala")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Scala strategy
    let strategy = registry.get_strategy("scala")
        .ok_or_else(|| anyhow::anyhow!("Scala strategy not found"))?;
    
    let result = strategy.analyze(&scala_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "scala");
    
    // Check that we found the case class
    let case_class_items: Vec<_> = result.items.iter()
        .filter(|item| {
            if let AstItem::Struct { derives, .. } = item {
                derives.contains(&"case".to_string())
            } else {
                false
            }
        })
        .collect();
    
    assert!(!case_class_items.is_empty(), "Should find at least one case class");
    
    // Check that we found the companion object
    let object_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Module { .. }))
        .collect();
    
    assert!(!object_items.is_empty(), "Should find at least one object");
    
    Ok(())
}

/// Tests Scala trait detection
#[test]
async fn test_scala_trait_integration() -> Result<()> {
    // Create a Scala file with a trait
    let scala_content = r#"
    package com.example.pmat.test
    
    trait Shape {
        def area(): Double
        def perimeter(): Double
    }
    
    class Circle(radius: Double) extends Shape {
        def area(): Double = math.Pi * radius * radius
        def perimeter(): Double = 2 * math.Pi * radius
    }
    "#;
    
    let (scala_file_path, _temp_dir) = create_temp_scala_file(scala_content, "Shape.scala")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Scala strategy
    let strategy = registry.get_strategy("scala")
        .ok_or_else(|| anyhow::anyhow!("Scala strategy not found"))?;
    
    let result = strategy.analyze(&scala_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "scala");
    
    // Check that we found the trait
    let trait_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Trait { .. }))
        .collect();
    
    assert!(!trait_items.is_empty(), "Should find at least one trait");
    
    // Check trait name
    if let Some(AstItem::Trait { name, .. }) = trait_items.first() {
        assert!(name.contains("Shape"), "Trait should be named Shape");
    }
    
    // Check that we found the implementing class
    let class_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();
    
    assert!(!class_items.is_empty(), "Should find at least one class");
    
    Ok(())
}

/// Tests Scala object (singleton) detection
#[test]
async fn test_scala_object_integration() -> Result<()> {
    // Create a Scala file with objects
    let scala_content = r#"
    package com.example.pmat.test
    
    object Constants {
        val Pi: Double = 3.14159
        val E: Double = 2.71828
    }
    
    object MathUtils {
        def square(x: Double): Double = x * x
        def cube(x: Double): Double = x * x * x
    }
    "#;
    
    let (scala_file_path, _temp_dir) = create_temp_scala_file(scala_content, "Constants.scala")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Scala strategy
    let strategy = registry.get_strategy("scala")
        .ok_or_else(|| anyhow::anyhow!("Scala strategy not found"))?;
    
    let result = strategy.analyze(&scala_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "scala");
    
    // Check that we found the objects
    let object_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Module { .. }))
        .collect();
    
    assert!(object_items.len() >= 2, "Should find at least two objects");
    
    // Check object names
    let has_constants = object_items.iter().any(|item| {
        if let AstItem::Module { name, .. } = item {
            name.contains("Constants")
        } else {
            false
        }
    });
    
    let has_math_utils = object_items.iter().any(|item| {
        if let AstItem::Module { name, .. } = item {
            name.contains("MathUtils")
        } else {
            false
        }
    });
    
    assert!(has_constants, "Should find Constants object");
    assert!(has_math_utils, "Should find MathUtils object");
    
    Ok(())
}

/// Comprehensive integration test for Scala language support
#[test]
async fn test_scala_comprehensive_integration() -> Result<()> {
    // Create a more complex Scala file with multiple features
    let scala_content = r#"
    package com.example.pmat.comprehensive
    
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
    
    let (scala_file_path, _temp_dir) = create_temp_scala_file(scala_content, "Comprehensive.scala")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Scala strategy
    let strategy = registry.get_strategy("scala")
        .ok_or_else(|| anyhow::anyhow!("Scala strategy not found"))?;
    
    let result = strategy.analyze(&scala_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "scala");
    assert!(!result.items.is_empty(), "Should find AST items in Scala file");
    
    // Count different types of items
    let trait_count = result.items.iter()
        .filter(|item| matches!(item, AstItem::Trait { .. }))
        .count();
        
    let case_class_count = result.items.iter()
        .filter(|item| {
            if let AstItem::Struct { derives, .. } = item {
                derives.contains(&"case".to_string())
            } else {
                false
            }
        })
        .count();
        
    let object_count = result.items.iter()
        .filter(|item| matches!(item, AstItem::Module { .. }))
        .count();
        
    let function_count = result.items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .count();
    
    // Assert that we found each type of item
    assert!(trait_count > 0, "Should find at least one trait");
    assert!(case_class_count > 0, "Should find at least one case class");
    assert!(object_count > 0, "Should find at least one object");
    assert!(function_count > 0, "Should find at least one function");
    
    // Check for specific items we expect to find
    let has_functor_trait = result.items.iter().any(|item| {
        if let AstItem::Trait { name, .. } = item {
            name.contains("Functor")
        } else {
            false
        }
    });
    
    let has_user_case_class = result.items.iter().any(|item| {
        if let AstItem::Struct { name, derives, .. } = item {
            name.contains("User") && derives.contains(&"case".to_string())
        } else {
            false
        }
    });
    
    let has_user_service_object = result.items.iter().any(|item| {
        if let AstItem::Module { name, .. } = item {
            name.contains("UserService")
        } else {
            false
        }
    });
    
    let has_data_processor_class = result.items.iter().any(|item| {
        if let AstItem::Struct { name, .. } = item {
            name.contains("DataProcessor")
        } else {
            false
        }
    });
    
    assert!(has_functor_trait, "Should find Functor trait");
    assert!(has_user_case_class, "Should find User case class");
    assert!(has_user_service_object, "Should find UserService object");
    assert!(has_data_processor_class, "Should find DataProcessor class");
    
    Ok(())
}

/// Tests registry initialization and Scala strategy availability
#[test]
async fn test_scala_registry_integration() -> Result<()> {
    // Create registry
    let registry = AstRegistry::new();
    
    // Check that Scala strategy is available
    let scala_strategy = registry.get_strategy("scala");
    assert!(scala_strategy.is_some(), "Scala strategy should be available");
    
    // Check strategy name
    let strategy = scala_strategy.unwrap();
    assert_eq!(strategy.language_name(), "Scala");
    
    // Check supported extensions
    let extensions = strategy.supported_extensions();
    assert!(extensions.contains(&"scala"), "Scala extension should be supported");
    assert!(extensions.contains(&"sc"), "Scala script extension should be supported");
    
    Ok(())
}