#[cfg(test)]
mod helpers_tests {
    use crate::utils::helpers::*;
    use handlebars::Handlebars;
    use serde_json::json;

    #[test]
    fn test_snake_case_helper() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        let test_cases = vec![
            ("HelloWorld", "hello_world"),
            ("myVariableName", "my_variable_name"),
            ("HTML2PDF", "html2_pdf"),
            ("PDFGenerator", "pdfgenerator"),
            ("already_snake_case", "already_snake_case"),
            ("SCREAMING_SNAKE_CASE", "screaming__snake__case"),
            ("kebab-case-input", "kebab_case_input"),
            ("Mixed-Case_Input", "mixed__case__input"),
            ("123Numbers456", "123_numbers456"),
            ("A", "a"),
            ("ABC", "abc"),
        ];

        for (input, expected) in test_cases {
            let template = "{{snake_case name}}";
            let data = json!({"name": input});
            let result = handlebars.render_template(template, &data).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_kebab_case_helper() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("kebab_case", Box::new(kebab_case_helper));

        let test_cases = vec![
            ("HelloWorld", "hello-world"),
            ("myVariableName", "my-variable-name"),
            ("HTML2PDF", "html2-pdf"),
            ("PDFGenerator", "pdfgenerator"),
            ("already-kebab-case", "already-kebab-case"),
            ("SCREAMING_SNAKE_CASE", "screaming--snake--case"),
            ("snake_case_input", "snake-case-input"),
            ("Mixed-Case_Input", "mixed--case--input"),
            ("123Numbers456", "123-numbers456"),
            ("A", "a"),
            ("ABC", "abc"),
        ];

        for (input, expected) in test_cases {
            let template = "{{kebab_case name}}";
            let data = json!({"name": input});
            let result = handlebars.render_template(template, &data).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_pascal_case_helper() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("pascal_case", Box::new(pascal_case_helper));

        let test_cases = vec![
            ("hello_world", "HelloWorld"),
            ("my-variable-name", "MyVariableName"),
            ("html2pdf", "Html2pdf"),
            ("pdf_generator", "PdfGenerator"),
            ("AlreadyPascalCase", "AlreadyPascalCase"),
            ("SCREAMING_SNAKE_CASE", "SCREAMINGSNAKECASE"),
            ("mixed-Case_Input", "MixedCaseInput"),
            ("123numbers456", "123numbers456"),
            ("a", "A"),
            ("abc", "Abc"),
        ];

        for (input, expected) in test_cases {
            let template = "{{pascal_case name}}";
            let data = json!({"name": input});
            let result = handlebars.render_template(template, &data).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_current_year_helper() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("current_year", Box::new(current_year_helper));

        let template = "Copyright {{current_year}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data).unwrap();

        // Check that it returns a valid year
        let year: i32 = result.trim_start_matches("Copyright ").parse().unwrap();
        assert!((2024..=2030).contains(&year)); // Reasonable range
    }

    #[test]
    fn test_current_date_helper() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("current_date", Box::new(current_date_helper));

        let template = "Generated on {{current_date}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data).unwrap();

        // Check that it returns a date-like string
        assert!(result.starts_with("Generated on "));
        let date_part = result.trim_start_matches("Generated on ");
        assert!(date_part.contains('-')); // Should be in ISO format
    }

    #[test]
    fn test_helper_error_handling() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        // Test with missing parameter
        let template = "{{snake_case}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string_handling() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));
        handlebars.register_helper("kebab_case", Box::new(kebab_case_helper));
        handlebars.register_helper("pascal_case", Box::new(pascal_case_helper));

        let template = "snake: '{{snake_case name}}', kebab: '{{kebab_case name}}', pascal: '{{pascal_case name}}'";
        let data = json!({"name": ""});
        let result = handlebars.render_template(template, &data).unwrap();

        assert_eq!(result, "snake: '', kebab: '', pascal: ''");
    }

    #[test]
    fn test_helper_with_non_string_parameter() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        // Test with number parameter - should fail
        let template = "{{snake_case count}}";
        let data = json!({"count": 42});
        let result = handlebars.render_template(template, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_pascal_case_preserves_existing_capitalization() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("pascal_case", Box::new(pascal_case_helper));

        let template = "{{pascal_case name}}";

        // Already in PascalCase
        let data = json!({"name": "AlreadyPascalCase"});
        let result = handlebars.render_template(template, &data).unwrap();
        assert_eq!(result, "AlreadyPascalCase");

        // Mixed case with numbers
        let data = json!({"name": "version2_api"});
        let result = handlebars.render_template(template, &data).unwrap();
        assert_eq!(result, "Version2Api");
    }

    #[test]
    fn test_year_and_date_helpers_consistency() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("current_year", Box::new(current_year_helper));
        handlebars.register_helper("current_date", Box::new(current_date_helper));

        let template = "Year: {{current_year}}, Date: {{current_date}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data).unwrap();

        // Extract year from both helpers
        let parts: Vec<&str> = result.split(", Date: ").collect();
        let year_from_helper = parts[0].trim_start_matches("Year: ");
        let date_from_helper = parts[1];

        // The year in the date should match the year helper
        assert!(date_from_helper.starts_with(year_from_helper));
    }
}
