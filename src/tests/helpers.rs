#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod helpers_tests {
    use crate::utils::helpers::{register_helpers, to_pascal_case, to_snake_case};
    use minijinja::context;

    fn make_env() -> minijinja::Environment<'static> {
        let mut env = minijinja::Environment::new();
        register_helpers(&mut env);
        env
    }

    fn render(env: &minijinja::Environment, template: &str, name: &str) -> String {
        let tmpl = env.template_from_str(template).unwrap();
        tmpl.render(context! { name => name }).unwrap()
    }

    #[test]
    fn test_snake_case_helper() {
        let env = make_env();

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
            let result = render(&env, "{{ name|snake_case }}", input);
            assert_eq!(result, expected, "Failed for input: {input}");
        }
    }

    #[test]
    fn test_kebab_case_helper() {
        let env = make_env();

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
            let result = render(&env, "{{ name|kebab_case }}", input);
            assert_eq!(result, expected, "Failed for input: {input}");
        }
    }

    #[test]
    fn test_pascal_case_helper() {
        let env = make_env();

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
            let result = render(&env, "{{ name|pascal_case }}", input);
            assert_eq!(result, expected, "Failed for input: {input}");
        }
    }

    #[test]
    fn test_current_year_helper() {
        let env = make_env();
        let tmpl = env
            .template_from_str("Copyright {{ current_year() }}")
            .unwrap();
        let result = tmpl.render(context! {}).unwrap();

        let year: i32 = result.trim_start_matches("Copyright ").parse().unwrap();
        assert!((2024..=2030).contains(&year));
    }

    #[test]
    fn test_current_date_helper() {
        let env = make_env();
        let tmpl = env
            .template_from_str("Generated on {{ current_date() }}")
            .unwrap();
        let result = tmpl.render(context! {}).unwrap();

        assert!(result.starts_with("Generated on "));
        let date_part = result.trim_start_matches("Generated on ");
        assert!(date_part.contains('-'));
    }

    #[test]
    fn test_empty_string_handling() {
        let env = make_env();
        let tmpl = env
            .template_from_str("snake: '{{ name|snake_case }}', kebab: '{{ name|kebab_case }}', pascal: '{{ name|pascal_case }}'")
            .unwrap();
        let result = tmpl.render(context! { name => "" }).unwrap();
        assert_eq!(result, "snake: '', kebab: '', pascal: ''");
    }

    #[test]
    fn test_pascal_case_preserves_existing_capitalization() {
        let env = make_env();

        let result = render(&env, "{{ name|pascal_case }}", "AlreadyPascalCase");
        assert_eq!(result, "AlreadyPascalCase");

        let result = render(&env, "{{ name|pascal_case }}", "version2_api");
        assert_eq!(result, "Version2Api");
    }

    #[test]
    fn test_year_and_date_helpers_consistency() {
        let env = make_env();
        let tmpl = env
            .template_from_str("Year: {{ current_year() }}, Date: {{ current_date() }}")
            .unwrap();
        let result = tmpl.render(context! {}).unwrap();

        let parts: Vec<&str> = result.split(", Date: ").collect();
        let year_from_helper = parts[0].trim_start_matches("Year: ");
        let date_from_helper = parts[1];

        assert!(date_from_helper.starts_with(year_from_helper));
    }

    #[test]
    fn test_to_snake_case_direct() {
        assert_eq!(to_snake_case("MyProjectName"), "my_project_name");
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_to_pascal_case_direct() {
        assert_eq!(to_pascal_case("my_project"), "MyProject");
        assert_eq!(to_pascal_case(""), "");
    }
}
