#![cfg(all(test, feature = "java-ast", feature = "integration-tests"))]

use anyhow::Result;
use pmat::services::ast::AstRegistry;
use pmat::services::file_classifier::FileClassifier;
use pmat::services::context::AstItem;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use tokio::test;

/// Creates a temporary Java file with the given content for integration testing
fn create_temp_java_file(content: &str, filename: &str) -> Result<(std::path::PathBuf, TempDir)> {
    let temp_dir = TempDir::new()?;
    let java_file_path = temp_dir.path().join(filename);

    let mut file = File::create(&java_file_path)?;
    file.write_all(content.as_bytes())?;
    drop(file); // Close the file

    Ok((java_file_path, temp_dir))
}

/// Basic integration test for Java language support
#[test]
async fn test_java_basic_integration() -> Result<()> {
    // Create a simple Java file
    let java_content = r#"
    package com.example.pmat.test;
    
    public class SimpleTest {
        private String message;
        
        public SimpleTest(String message) {
            this.message = message;
        }
        
        public void printMessage() {
            System.out.println(message);
        }
        
        public String getMessage() {
            return message;
        }
    }
    "#;
    
    let (java_file_path, _temp_dir) = create_temp_java_file(java_content, "SimpleTest.java")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Java strategy
    let strategy = registry.get_strategy("java")
        .ok_or_else(|| anyhow::anyhow!("Java strategy not found"))?;
    
    let result = strategy.analyze(&java_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "java");
    assert!(!result.items.is_empty(), "Should find AST items in Java file");
    
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
    
    // Check class visibility
    if let Some(AstItem::Struct { visibility, .. }) = class_items.first() {
        assert_eq!(visibility, "public", "Class should have public visibility");
    }
    
    Ok(())
}

/// Tests Java interface detection
#[test]
async fn test_java_interface_integration() -> Result<()> {
    // Create a Java file with an interface
    let java_content = r#"
    package com.example.pmat.test;
    
    public interface DataProcessor {
        void processData(String data);
        String getProcessedData();
    }
    
    public class DefaultProcessor implements DataProcessor {
        private String processedData;
        
        @Override
        public void processData(String data) {
            this.processedData = data.toUpperCase();
        }
        
        @Override
        public String getProcessedData() {
            return processedData;
        }
    }
    "#;
    
    let (java_file_path, _temp_dir) = create_temp_java_file(java_content, "DataProcessor.java")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Java strategy
    let strategy = registry.get_strategy("java")
        .ok_or_else(|| anyhow::anyhow!("Java strategy not found"))?;
    
    let result = strategy.analyze(&java_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "java");
    
    // Check that we found the interface
    let trait_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Trait { .. }))
        .collect();
    
    assert!(!trait_items.is_empty(), "Should find at least one interface");
    
    // Check interface name
    if let Some(AstItem::Trait { name, .. }) = trait_items.first() {
        assert!(name.contains("DataProcessor"), "Interface should be named DataProcessor");
    }
    
    // Check that we found the implementing class
    let class_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();
    
    assert!(!class_items.is_empty(), "Should find at least one class");
    
    // Check class name
    if let Some(AstItem::Struct { name, .. }) = class_items.first() {
        assert!(name.contains("DefaultProcessor"), "Class should be named DefaultProcessor");
    }
    
    Ok(())
}

/// Tests Java inheritance detection
#[test]
async fn test_java_inheritance_integration() -> Result<()> {
    // Create a Java file with inheritance
    let java_content = r#"
    package com.example.pmat.test;
    
    public class Animal {
        protected String name;
        
        public Animal(String name) {
            this.name = name;
        }
        
        public void makeSound() {
            System.out.println("Generic animal sound");
        }
    }
    
    public class Dog extends Animal {
        private String breed;
        
        public Dog(String name, String breed) {
            super(name);
            this.breed = breed;
        }
        
        @Override
        public void makeSound() {
            System.out.println("Woof!");
        }
        
        public String getBreed() {
            return breed;
        }
    }
    "#;
    
    let (java_file_path, _temp_dir) = create_temp_java_file(java_content, "Animal.java")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Java strategy
    let strategy = registry.get_strategy("java")
        .ok_or_else(|| anyhow::anyhow!("Java strategy not found"))?;
    
    let result = strategy.analyze(&java_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "java");
    
    // Check that we found both classes
    let class_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();
    
    assert!(class_items.len() >= 2, "Should find at least two classes");
    
    // Check methods
    let method_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();
    
    assert!(method_items.len() >= 3, "Should find at least three methods");
    
    // Find Animal class
    let has_animal_class = class_items.iter().any(|item| {
        if let AstItem::Struct { name, .. } = item {
            name.contains("Animal")
        } else {
            false
        }
    });
    
    assert!(has_animal_class, "Should find Animal class");
    
    // Find Dog class
    let has_dog_class = class_items.iter().any(|item| {
        if let AstItem::Struct { name, .. } = item {
            name.contains("Dog")
        } else {
            false
        }
    });
    
    assert!(has_dog_class, "Should find Dog class");
    
    Ok(())
}

/// Tests Java annotations detection
#[test]
async fn test_java_annotations_integration() -> Result<()> {
    // Create a Java file with annotations
    let java_content = r#"
    package com.example.pmat.test;
    
    import java.util.List;
    import java.util.ArrayList;
    
    @SuppressWarnings("unchecked")
    public class AnnotatedClass {
        
        @Deprecated
        public void oldMethod() {
            System.out.println("This method is deprecated");
        }
        
        @Override
        public String toString() {
            return "AnnotatedClass instance";
        }
        
        @SuppressWarnings({"unchecked", "rawtypes"})
        public List getItems() {
            return new ArrayList();
        }
    }
    "#;
    
    let (java_file_path, _temp_dir) = create_temp_java_file(java_content, "AnnotatedClass.java")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Java strategy
    let strategy = registry.get_strategy("java")
        .ok_or_else(|| anyhow::anyhow!("Java strategy not found"))?;
    
    let result = strategy.analyze(&java_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "java");
    
    // Check that we found the class
    let class_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();
    
    assert!(!class_items.is_empty(), "Should find at least one class");
    
    // Check methods
    let method_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();
    
    assert!(method_items.len() >= 3, "Should find at least three methods");
    
    // Check methods are found with appropriate names
    let has_old_method = method_items.iter().any(|item| {
        if let AstItem::Function { name, .. } = item {
            name.contains("oldMethod")
        } else {
            false
        }
    });
    
    assert!(has_old_method, "Should find oldMethod");
    
    let has_to_string = method_items.iter().any(|item| {
        if let AstItem::Function { name, .. } = item {
            name.contains("toString")
        } else {
            false
        }
    });
    
    assert!(has_to_string, "Should find toString method");
    
    let has_get_items = method_items.iter().any(|item| {
        if let AstItem::Function { name, .. } = item {
            name.contains("getItems")
        } else {
            false
        }
    });
    
    assert!(has_get_items, "Should find getItems method");
    
    Ok(())
}

/// Comprehensive integration test for Java language support
#[test]
async fn test_java_comprehensive_integration() -> Result<()> {
    // Create a more complex Java file with multiple features
    let java_content = r#"
    package com.example.pmat.comprehensive;
    
    import java.util.List;
    import java.util.ArrayList;
    import java.util.function.Consumer;
    
    /**
     * A comprehensive Java class demonstrating various language features.
     */
    public class ComprehensiveExample<T> {
        // Static constants
        private static final int MAX_SIZE = 100;
        
        // Fields
        private List<T> items;
        private int currentSize;
        
        // Constructor
        public ComprehensiveExample() {
            this.items = new ArrayList<>();
            this.currentSize = 0;
        }
        
        // Generic method
        public <R> R process(T item, Function<T, R> processor) {
            return processor.apply(item);
        }
        
        // Method with lambda parameter
        public void forEach(Consumer<T> action) {
            for (T item : items) {
                action.accept(item);
            }
        }
        
        // Synchronized method
        public synchronized void addItem(T item) {
            if (currentSize < MAX_SIZE) {
                items.add(item);
                currentSize++;
            }
        }
        
        // Method with try-catch
        public T getItem(int index) {
            try {
                return items.get(index);
            } catch (IndexOutOfBoundsException e) {
                System.err.println("Index out of bounds: " + index);
                return null;
            }
        }
        
        // Nested class
        public static class Builder<T> {
            private ComprehensiveExample<T> instance;
            
            public Builder() {
                instance = new ComprehensiveExample<>();
            }
            
            public Builder<T> addItem(T item) {
                instance.addItem(item);
                return this;
            }
            
            public ComprehensiveExample<T> build() {
                return instance;
            }
        }
        
        // Functional interface
        @FunctionalInterface
        public interface Function<T, R> {
            R apply(T t);
        }
    }
    "#;
    
    let (java_file_path, _temp_dir) = create_temp_java_file(java_content, "ComprehensiveExample.java")?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Get the Java strategy
    let strategy = registry.get_strategy("java")
        .ok_or_else(|| anyhow::anyhow!("Java strategy not found"))?;
    
    let result = strategy.analyze(&java_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "java");
    assert!(!result.items.is_empty(), "Should find AST items in Java file");
    
    // Check classes and nested classes
    let class_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();
    
    assert!(class_items.len() >= 2, "Should find at least two classes (main + nested)");
    
    // Check interfaces
    let trait_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Trait { .. }))
        .collect();
    
    assert!(!trait_items.is_empty(), "Should find at least one interface");
    
    // Check methods
    let method_items: Vec<_> = result.items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();
    
    assert!(method_items.len() >= 5, "Should find at least five methods");
    
    Ok(())
}

/// Tests registry initialization and Java strategy availability
#[test]
async fn test_java_registry_integration() -> Result<()> {
    // Create registry
    let registry = AstRegistry::new();
    
    // Check that Java strategy is available
    let java_strategy = registry.get_strategy("java");
    assert!(java_strategy.is_some(), "Java strategy should be available");
    
    // Check strategy name
    let strategy = java_strategy.unwrap();
    assert_eq!(strategy.language_name(), "Java");
    
    // Check supported extensions
    let extensions = strategy.supported_extensions();
    assert!(extensions.contains(&"java"), "Java extension should be supported");
    
    Ok(())
}