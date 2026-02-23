#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::super::filters::{passes_query_filter, passes_type_filter};
    use super::super::formatting::format_symbol_table_csv;
    use super::super::stats::{count_by_type, count_by_visibility};
    use super::super::types::SymbolInfo;
    use crate::cli::SymbolTypeFilter;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_passes_query_filter_case_insensitivity(
            name in "[a-zA-Z][a-zA-Z0-9_]{0,20}",
            query in "[a-zA-Z][a-zA-Z0-9_]{0,10}"
        ) {
            // If the name contains the query (case insensitive), the filter should pass
            let contains_case_insensitive = name.to_lowercase().contains(&query.to_lowercase());
            let result = passes_query_filter(&name, &Some(query));
            prop_assert_eq!(result, contains_case_insensitive);
        }

        #[test]
        fn test_passes_query_filter_none_always_passes(name in ".*") {
            prop_assert!(passes_query_filter(&name, &None));
        }

        #[test]
        fn test_count_by_type_sum_equals_total(
            count in 1usize..20,
        ) {
            let mut symbols = Vec::with_capacity(count);
            let kinds = ["function", "struct", "enum", "trait", "module"];

            for i in 0..count {
                symbols.push(SymbolInfo {
                    name: format!("symbol_{}", i),
                    kind: kinds[i % kinds.len()].to_string(),
                    file: std::path::PathBuf::from("test.rs"),
                    line: i,
                    visibility: "pub".to_string(),
                    is_async: false,
                });
            }

            let counts = count_by_type(&symbols);
            let total: usize = counts.values().sum();
            prop_assert_eq!(total, count);
        }

        #[test]
        fn test_count_by_visibility_sum_equals_total(
            count in 1usize..20,
        ) {
            let mut symbols = Vec::with_capacity(count);
            let visibilities = ["pub", "pub(crate)", "private", "pub(super)"];

            for i in 0..count {
                symbols.push(SymbolInfo {
                    name: format!("symbol_{}", i),
                    kind: "function".to_string(),
                    file: std::path::PathBuf::from("test.rs"),
                    line: i,
                    visibility: visibilities[i % visibilities.len()].to_string(),
                    is_async: false,
                });
            }

            let counts = count_by_visibility(&symbols);
            let total: usize = counts.values().sum();
            prop_assert_eq!(total, count);
        }

        #[test]
        fn test_passes_type_filter_all_or_none_always_passes(kind in "[a-z]+") {
            prop_assert!(passes_type_filter(&kind, &None));
            prop_assert!(passes_type_filter(&kind, &Some(SymbolTypeFilter::All)));
        }

        #[test]
        fn test_csv_output_has_correct_line_count(count in 0usize..50) {
            let mut symbols = Vec::with_capacity(count);

            for i in 0..count {
                symbols.push(SymbolInfo {
                    name: format!("sym_{}", i),
                    kind: "function".to_string(),
                    file: std::path::PathBuf::from("test.rs"),
                    line: i,
                    visibility: "pub".to_string(),
                    is_async: false,
                });
            }

            let csv = format_symbol_table_csv(&symbols);
            let line_count = csv.lines().count();
            // Header + one line per symbol
            prop_assert_eq!(line_count, count + 1);
        }
    }
}
