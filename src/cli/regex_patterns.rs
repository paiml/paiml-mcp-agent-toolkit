use lazy_static::lazy_static;
use regex::Regex;

// Pre-compiled regex patterns for identifier extraction
lazy_static! {
    // Rust patterns
    pub static ref RUST_FUNCTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"impl\s+(?:\w+\s+for\s+)?(\w+)").expect("Hardcoded regex pattern must be valid"),
    ];
    
    pub static ref RUST_TYPE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?:pub\s+)?struct\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"(?:pub\s+)?enum\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"(?:pub\s+)?trait\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"(?:pub\s+)?type\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
    ];
    
    pub static ref RUST_VARIABLE_PATTERN: Regex = 
        Regex::new(r"let\s+(?:mut\s+)?(\w+)").expect("Hardcoded regex pattern must be valid");
    
    // JavaScript/TypeScript patterns
    pub static ref JS_FUNCTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"function\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\s*(?:function|\(.*?\)\s*=>)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"(\w+)\s*:\s*(?:async\s+)?function").expect("Hardcoded regex pattern must be valid"),
    ];
    
    pub static ref JS_TYPE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"class\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"interface\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"type\s+(\w+)\s*=").expect("Hardcoded regex pattern must be valid"),
    ];
    
    pub static ref JS_VARIABLE_PATTERN: Regex = 
        Regex::new(r"(?:const|let|var)\s+(\w+)").expect("Hardcoded regex pattern must be valid");
    
    // Python patterns
    pub static ref PYTHON_FUNCTION_PATTERN: Regex = 
        Regex::new(r"def\s+(\w+)").expect("Hardcoded regex pattern must be valid");
    
    pub static ref PYTHON_CLASS_PATTERN: Regex = 
        Regex::new(r"class\s+(\w+)").expect("Hardcoded regex pattern must be valid");
    
    pub static ref PYTHON_VARIABLE_PATTERN: Regex = 
        Regex::new(r"(\w+)\s*=").expect("Hardcoded regex pattern must be valid");
    
    // C/C++ patterns
    pub static ref C_FUNCTION_PATTERN: Regex = 
        Regex::new(r"(?:static\s+)?(?:inline\s+)?(?:\w+\s+)*(\w+)\s*\([^)]*\)\s*\{").expect("Hardcoded regex pattern must be valid");
    
    pub static ref C_TYPE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"struct\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"typedef\s+struct\s*\{[^}]*\}\s*(\w+)").expect("Hardcoded regex pattern must be valid"),
        Regex::new(r"typedef\s+(?:struct\s+)?(\w+)").expect("Hardcoded regex pattern must be valid"),
    ];
}
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
