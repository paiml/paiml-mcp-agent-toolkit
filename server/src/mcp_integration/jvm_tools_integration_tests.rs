#[cfg(all(test, feature = "java-ast", feature = "scala-ast"))]
mod tests {
    use crate::mcp_integration::{java_tools::*, scala_tools::*};
    use crate::mcp_integration::McpTool;
    use crate::agents::registry::AgentRegistry;
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use std::path::PathBuf;

    async fn create_test_file(dir: &tempfile::TempDir, filename: &str, content: &str) -> PathBuf {
        let path = dir.path().join(filename);
        let mut file = File::create(&path).await.unwrap();
        file.write_all(content.as_bytes()).await.unwrap();
        path
    }

    #[tokio::test]
    async fn test_java_analysis_tool() {
        // Create a test Java file
        let dir = tempdir().unwrap();
        let java_content = r#"
            package com.example;
            
            public class TestClass {
                private int value;
                
                public TestClass(int value) {
                    this.value = value;
                }
                
                public int getValue() {
                    return this.value;
                }
                
                public void setValue(int value) {
                    this.value = value;
                }
                
                public int calculateComplex(int a, int b) {
                    int result = 0;
                    if (a > b) {
                        result = a * 2;
                        if (result > 100) {
                            result = 100;
                        }
                    } else {
                        result = b * 3;
                        for (int i = 0; i < a; i++) {
                            result += i;
                        }
                    }
                    return result;
                }
            }
            
            interface TestInterface {
                void doSomething();
            }
        "#;
        
        let java_path = create_test_file(&dir, "TestClass.java", java_content).await;
        
        // Create the Java analysis tool
        let agent_registry = Arc::new(AgentRegistry::new());
        let java_tool = JavaAnalysisTool::new(agent_registry.clone());
        
        // Test analysis without metrics
        let params = json!({
            "path": java_path.to_string_lossy().to_string(),
            "include_metrics": false,
            "include_ast": false
        });
        
        let result = java_tool.execute(params).await.unwrap();
        
        // Verify basic analysis
        assert_eq!(result["status"], "completed");
        assert_eq!(result["language"], "java");
        assert_eq!(result["summary"]["class_count"], 1);
        assert_eq!(result["summary"]["interface_count"], 1);
        assert!(result["summary"]["method_count"].as_u64().unwrap() >= 4);
        
        // Test with metrics and AST
        let params_with_metrics = json!({
            "path": java_path.to_string_lossy().to_string(),
            "include_metrics": true,
            "include_ast": true
        });
        
        let result_with_metrics = java_tool.execute(params_with_metrics).await.unwrap();
        
        // Verify metrics
        assert!(result_with_metrics["metrics"].is_object());
        assert!(result_with_metrics["metrics"]["total_complexity"].as_u64().unwrap() > 0);
        assert!(result_with_metrics["metrics"]["avg_complexity"].as_f64().unwrap() > 0.0);
        
        // Verify AST items
        assert!(result_with_metrics["items"].is_array());
        assert!(!result_with_metrics["items"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_scala_analysis_tool() {
        // Create a test Scala file
        let dir = tempdir().unwrap();
        let scala_content = r#"
            package com.example
            
            // A case class (functional pattern)
            case class Person(name: String, age: Int) 
            
            // A trait (similar to interface)
            trait Greeter {
              def greet(name: String): String
            }
            
            // A singleton object
            object HelloWorld extends Greeter {
              def main(args: Array[String]): Unit = {
                val person = Person("Alice", 30)
                println(greet(person.name))
              }
              
              def greet(name: String): String = s"Hello, $name!"
              
              // Higher-order function (functional pattern)
              def process(data: List[Int], transformer: Int => Int): List[Int] = {
                data.map(transformer)
              }
              
              // Pattern matching (functional pattern)
              def describe(x: Any): String = x match {
                case i: Int if i > 0 => "positive integer"
                case i: Int => "other integer"
                case s: String => s"string: $s"
                case p: Person => s"person: ${p.name}"
                case _ => "something else"
              }
              
              // For-comprehension (functional pattern)
              def complexComputation(values: List[Int]): List[Int] = {
                for {
                  v <- values
                  if v > 0
                  squared = v * v
                } yield squared + 1
              }
            }
            
            // Regular class (object-oriented pattern)
            class Calculator {
              var result: Int = 0  // Mutable state (imperative pattern)
              
              def add(x: Int): Unit = {
                result += x  // Side effect (imperative pattern)
              }
              
              def getResult(): Int = result
              
              def clear(): Unit = {
                result = 0
              }
            }
        "#;
        
        let scala_path = create_test_file(&dir, "HelloWorld.scala", scala_content).await;
        
        // Create the Scala analysis tool
        let agent_registry = Arc::new(AgentRegistry::new());
        let scala_tool = ScalaAnalysisTool::new(agent_registry.clone());
        
        // Test analysis without metrics
        let params = json!({
            "path": scala_path.to_string_lossy().to_string(),
            "include_metrics": false,
            "include_ast": false
        });
        
        let result = scala_tool.execute(params).await.unwrap();
        
        // Verify basic analysis
        assert_eq!(result["status"], "completed");
        assert_eq!(result["language"], "scala");
        assert_eq!(result["summary"]["case_class_count"], 1);
        assert_eq!(result["summary"]["trait_count"], 1);
        assert_eq!(result["summary"]["object_count"], 1);
        assert_eq!(result["summary"]["class_count"], 1);
        
        // Test with metrics and AST
        let params_with_metrics = json!({
            "path": scala_path.to_string_lossy().to_string(),
            "include_metrics": true,
            "include_ast": true
        });
        
        let result_with_metrics = scala_tool.execute(params_with_metrics).await.unwrap();
        
        // Verify metrics
        assert!(result_with_metrics["metrics"].is_object());
        assert!(result_with_metrics["metrics"]["total_complexity"].as_u64().unwrap() > 0);
        assert!(result_with_metrics["metrics"]["avg_complexity"].as_f64().unwrap() > 0.0);
        assert!(result_with_metrics["metrics"]["functional_percentage"].as_f64().unwrap() > 0.0);
        
        // Verify AST items
        assert!(result_with_metrics["items"].is_array());
        assert!(!result_with_metrics["items"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_java_mutation_tool() {
        let agent_registry = Arc::new(AgentRegistry::new());
        let java_mutation_tool = JavaMutationTool::new(agent_registry.clone());
        
        let params = json!({
            "project_path": "/tmp/test-project",
            "source_path": "/tmp/test-project/src/main/java/com/example",
            "test_command": "mvn test",
            "mutation_operators": ["arithmetic", "conditional"],
            "timeout": 15
        });
        
        let result = java_mutation_tool.execute(params).await.unwrap();
        
        // Since this is a mock implementation, just check that the expected fields are present
        assert_eq!(result["status"], "completed");
        assert_eq!(result["message"], "Java mutation testing completed");
        assert!(result["results"].is_object());
    }

    #[tokio::test]
    async fn test_scala_mutation_tool() {
        let agent_registry = Arc::new(AgentRegistry::new());
        let scala_mutation_tool = ScalaMutationTool::new(agent_registry.clone());
        
        let params = json!({
            "project_path": "/tmp/test-project",
            "source_path": "/tmp/test-project/src/main/scala/com/example",
            "test_command": "sbt test",
            "mutation_operators": ["arithmetic", "conditional", "functional"],
            "timeout": 20
        });
        
        let result = scala_mutation_tool.execute(params).await.unwrap();
        
        // Since this is a mock implementation, just check that the expected fields are present
        assert_eq!(result["status"], "completed");
        assert_eq!(result["message"], "Scala mutation testing completed");
        assert!(result["results"].is_object());
    }
}