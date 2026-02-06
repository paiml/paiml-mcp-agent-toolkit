//! Utility handlers tests - Part 3: Extended property tests
//! Extracted for file health compliance (CB-040)

use super::*;

mod extended_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_calculate_pagerank_value_never_exceeds_one(
            incoming in 0usize..1000,
            outgoing in 0usize..1000
        ) {
            let result = calculate_pagerank_value(incoming, outgoing);
            prop_assert!(result <= 1.0);
            prop_assert!(result >= 0.0);
        }

        #[test]
        fn test_markdown_builder_header_levels(level in 1usize..6, text in "[a-zA-Z ]+") {
            let mut builder = MarkdownBuilder::new();
            builder.add_header(level, &text);

            let expected_hashes: String = (0..level).map(|_| '#').collect();
            prop_assert!(builder.content.starts_with(&expected_hashes));
        }

        #[test]
        fn test_markdown_builder_metric_format(
            label in "[a-zA-Z]+",
            value in 0i64..10000
        ) {
            let mut builder = MarkdownBuilder::new();
            builder.add_metric(&label, value);

            let expected_label = format!("**{}**", label);
            prop_assert!(builder.content.contains(&expected_label));
            prop_assert!(builder.content.contains(&value.to_string()));
        }

        #[test]
        fn test_markdown_builder_percentage_format(
            label in "[a-zA-Z]+",
            value in 0.0f64..100.0
        ) {
            let mut builder = MarkdownBuilder::new();
            builder.add_percentage_metric(&label, value);

            let expected_label = format!("**{}**", label);
            prop_assert!(builder.content.contains(&expected_label));
            prop_assert!(builder.content.contains('%'));
        }

        #[test]
        fn test_detect_or_use_toolchain_preserves_input(toolchain in "[a-z]+") {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let result = detect_or_use_toolchain(Some(toolchain.clone()), temp_dir.path());

            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), toolchain);
        }
    }
}
