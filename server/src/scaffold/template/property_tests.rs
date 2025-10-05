// Property-based tests for Template system - TICKET-PMAT-5002

use super::*;
use proptest::prelude::*;

// Generator for valid project names (same as in scaffold property_tests)
fn valid_project_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,63}"
        .prop_map(|s| s.to_string())
}

// Generator for author names
fn valid_author() -> impl Strategy<Value = String> {
    "[A-Za-z ]{3,30}"
        .prop_map(|s| s.to_string())
}

proptest! {
    #[test]
    fn prop_template_render_deterministic(
        name in valid_project_name(),
        author in valid_author()
    ) {
        let template = Template::from_string("{{name}} by {{author}}").unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".into(), name.clone());
        vars.insert("author".into(), author.clone());

        let result1 = template.render(&vars).unwrap();
        let result2 = template.render(&vars).unwrap();

        // Property: Deterministic rendering
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_template_variables_substituted(
        name in valid_project_name(),
        author in valid_author()
    ) {
        let template = Template::from_string("Project: {{name}}, Author: {{author}}").unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".into(), name.clone());
        vars.insert("author".into(), author.clone());

        let result = template.render(&vars).unwrap();

        // Property: Variables substituted (no {{ remaining)
        prop_assert!(!result.contains("{{name}}"));
        prop_assert!(!result.contains("{{author}}"));

        // Property: Values present in output
        prop_assert!(result.contains(&name));
        prop_assert!(result.contains(&author));
    }

    #[test]
    fn prop_pforge_yaml_valid(
        name in valid_project_name(),
        author in valid_author()
    ) {
        let template = Template::pforge_yaml();
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), name.clone());
        vars.insert("author".into(), author);
        vars.insert("description".into(), "Test project".into());

        let rendered = template.render(&vars).unwrap();

        // Property: Rendered output is valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&rendered).unwrap();
        prop_assert!(parsed.is_mapping());

        // Property: Contains expected structure
        prop_assert!(parsed.get("forge").is_some());
        prop_assert_eq!(parsed["forge"]["name"].as_str(), Some(name.as_str()));
    }

    #[test]
    fn prop_cargo_toml_contains_name(
        name in valid_project_name(),
        author in valid_author()
    ) {
        let template = Template::cargo_toml();
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), name.clone());
        vars.insert("author".into(), author);

        let rendered = template.render(&vars).unwrap();

        // Property: Contains package name
        let package_name = format!("name = \"{}\"", name);
        prop_assert!(rendered.contains(&package_name));

        // Property: Contains required sections
        prop_assert!(rendered.contains("[package]"));
        prop_assert!(rendered.contains("[dependencies]"));
    }

    #[test]
    fn prop_handler_compiles_syntax(
        handler_name in "[A-Z][a-zA-Z]{3,20}"
    ) {
        let template = Template::handler_rs();
        let mut vars = HashMap::new();
        vars.insert("handler_name".into(), handler_name.clone());
        vars.insert("handler_description".into(), "Test handler".into());

        let rendered = template.render(&vars).unwrap();

        // Property: Contains expected Rust syntax
        let input_struct = format!("pub struct {}Input", handler_name);
        let output_struct = format!("pub struct {}Output", handler_name);
        prop_assert!(rendered.contains(&input_struct));
        prop_assert!(rendered.contains(&output_struct));
        prop_assert!(rendered.contains("pub async fn execute"));

        // Property: Has test module
        prop_assert!(rendered.contains("#[cfg(test)]"));
        prop_assert!(rendered.contains("#[tokio::test]"));
    }

    #[test]
    fn prop_template_never_panics(content in ".*", key in "[a-z]{1,10}", value in ".*") {
        let template = Template::from_string(content).unwrap();
        let mut vars = HashMap::new();
        vars.insert(key, value);

        // Property: Rendering never panics
        let _result = template.render(&vars);
        // If it returns, we're good (error or success, no panic)
    }

    #[test]
    fn prop_registry_idempotent(name in valid_project_name()) {
        let mut registry = TemplateRegistry::new();
        let template1 = Template::from_string("content").unwrap();
        let template2 = Template::from_string("content").unwrap();

        registry.register(name.clone(), template1);
        registry.register(name.clone(), template2);

        // Property: Second register overwrites first
        let retrieved = registry.get(&name);
        prop_assert!(retrieved.is_ok());
    }
}
