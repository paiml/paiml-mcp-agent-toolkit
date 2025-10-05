// Unit tests for Template system - TICKET-PMAT-5002

use super::*;

#[test]
fn test_template_creation() {
    let template = Template::from_string("Hello World");
    assert!(template.is_ok());
}

#[test]
fn test_template_render_simple() {
    let template = Template::from_string("Hello {{name}}!").unwrap();
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "World".to_string());

    let result = template.render(&vars).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_template_render_multiple_vars() {
    let template = Template::from_string("{{greeting}} {{name}}!").unwrap();
    let mut vars = HashMap::new();
    vars.insert("greeting".to_string(), "Hello".to_string());
    vars.insert("name".to_string(), "Alice".to_string());

    let result = template.render(&vars).unwrap();
    assert_eq!(result, "Hello Alice!");
}

#[test]
fn test_template_render_same_var_twice() {
    let template = Template::from_string("{{name}} and {{name}}").unwrap();
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "Bob".to_string());

    let result = template.render(&vars).unwrap();
    assert_eq!(result, "Bob and Bob");
}

#[test]
fn test_template_render_unreplaced_var_error() {
    let template = Template::from_string("Hello {{name}}!").unwrap();
    let vars = HashMap::new(); // Empty vars

    let result = template.render(&vars);
    assert!(result.is_err());
}

#[test]
fn test_template_registry_register() {
    let mut registry = TemplateRegistry::new();
    let template = Template::from_string("test").unwrap();

    registry.register("test".into(), template);
    assert!(registry.get("test").is_ok());
}

#[test]
fn test_template_registry_get_not_found() {
    let registry = TemplateRegistry::new();
    assert!(registry.get("nonexistent").is_err());
}

#[test]
fn test_template_registry_list() {
    let mut registry = TemplateRegistry::new();
    registry.register("t1".into(), Template::from_string("a").unwrap());
    registry.register("t2".into(), Template::from_string("b").unwrap());

    let list = registry.list();
    assert_eq!(list.len(), 2);
    assert!(list.contains(&"t1".to_string()));
    assert!(list.contains(&"t2".to_string()));
}

#[test]
fn test_template_pforge_yaml() {
    let template = Template::pforge_yaml();
    let mut vars = HashMap::new();
    vars.insert("project_name".to_string(), "test-agent".to_string());
    vars.insert("description".to_string(), "Test agent".to_string());
    vars.insert("author".to_string(), "Test Author".to_string());

    let result = template.render(&vars).unwrap();

    // Should be valid YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
    assert!(parsed.get("forge").is_some());
    assert_eq!(parsed["forge"]["name"].as_str(), Some("test-agent"));
}

#[test]
fn test_template_cargo_toml() {
    let template = Template::cargo_toml();
    let mut vars = HashMap::new();
    vars.insert("project_name".to_string(), "test-crate".to_string());
    vars.insert("author".to_string(), "Test Author".to_string());

    let result = template.render(&vars).unwrap();

    // Should contain package section
    assert!(result.contains("[package]"));
    assert!(result.contains("name = \"test-crate\""));
    assert!(result.contains("Test Author"));
}

#[test]
fn test_template_handler_rs() {
    let template = Template::handler_rs();
    let mut vars = HashMap::new();
    vars.insert("handler_name".to_string(), "Example".to_string());
    vars.insert("handler_description".to_string(), "Example handler".to_string());

    let result = template.render(&vars).unwrap();

    // Should contain struct and function definitions
    assert!(result.contains("pub struct ExampleInput"));
    assert!(result.contains("pub struct ExampleOutput"));
    assert!(result.contains("pub async fn execute"));
}

#[test]
fn test_template_readme_md() {
    let template = Template::readme_md();
    let mut vars = HashMap::new();
    vars.insert("project_name".to_string(), "my-project".to_string());
    vars.insert("description".to_string(), "My test project".to_string());
    vars.insert("author".to_string(), "Test Author".to_string());

    let result = template.render(&vars).unwrap();

    assert!(result.contains("# my-project"));
    assert!(result.contains("My test project"));
    assert!(result.contains("Test Author"));
}

#[test]
fn test_template_registry_with_pforge_templates() {
    let registry = TemplateRegistry::with_pforge_templates();

    assert!(registry.get("pforge.yaml").is_ok());
    assert!(registry.get("Cargo.toml").is_ok());
    assert!(registry.get("handler.rs").is_ok());
    assert!(registry.get("README.md").is_ok());

    let list = registry.list();
    assert_eq!(list.len(), 4);
}

#[test]
fn test_template_named() {
    let template = Template::new("test.txt", "content").unwrap();
    assert_eq!(template.name(), "test.txt");
}
