// Inline unit tests for BigOAnalyzer
// This file is include!()'d into big_o_analyzer.rs scope.
// NO use imports or #! inner attributes allowed.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let _analyzer = BigOAnalyzer::new();
        // Basic creation test - passes if compilation succeeds
    }

    #[test]
    fn test_complexity_analysis() {
        let analyzer = BigOAnalyzer::new();

        let rust_code = r#"
        fn bubble_sort(mut arr: Vec<i32>) -> Vec<i32> {
            for i in 0..arr.len() {
                for j in 0..arr.len() - 1 - i {
                    if arr[j] > arr[j + 1] {
                        arr.swap(j, j + 1);
                    }
                }
            }
            arr
        }
        "#;

        let result = analyzer.analyze_function_complexity("bubble_sort", rust_code, "rust");
        assert_eq!(result.time_complexity.class, BigOClass::Quadratic);
    }

    #[test]
    fn test_analyze_linear_function() {
        let analyzer = BigOAnalyzer::new();

        let linear_code = r#"
        fn sum(arr: &[i32]) -> i32 {
            let mut total = 0;
            for x in arr {
                total += x;
            }
            total
        }
        "#;

        let result = analyzer.analyze_function_complexity("sum", linear_code, "rust");
        assert_eq!(result.time_complexity.class, BigOClass::Linear);
    }

    #[test]
    fn test_analyze_constant_function() {
        let analyzer = BigOAnalyzer::new();

        let const_code = r#"
        fn get_first(arr: &[i32]) -> Option<i32> {
            arr.first().copied()
        }
        "#;

        let result = analyzer.analyze_function_complexity("get_first", const_code, "rust");
        assert_eq!(result.time_complexity.class, BigOClass::Constant);
    }

    #[test]
    fn test_big_o_class_display() {
        let constant = BigOClass::Constant;
        let linear = BigOClass::Linear;
        let quadratic = BigOClass::Quadratic;

        // Verify Display trait works
        assert_eq!(format!("{:?}", constant), "Constant");
        assert_eq!(format!("{:?}", linear), "Linear");
        assert_eq!(format!("{:?}", quadratic), "Quadratic");
    }
}

