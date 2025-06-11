use lazy_static::lazy_static;
use regex::Regex;

// Pre-compiled regex patterns for identifier extraction
lazy_static! {
    // Rust patterns
    pub static ref RUST_FUNCTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap(),
        Regex::new(r"impl\s+(?:\w+\s+for\s+)?(\w+)").unwrap(),
    ];
    
    pub static ref RUST_TYPE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?:pub\s+)?struct\s+(\w+)").unwrap(),
        Regex::new(r"(?:pub\s+)?enum\s+(\w+)").unwrap(),
        Regex::new(r"(?:pub\s+)?trait\s+(\w+)").unwrap(),
        Regex::new(r"(?:pub\s+)?type\s+(\w+)").unwrap(),
    ];
    
    pub static ref RUST_VARIABLE_PATTERN: Regex = 
        Regex::new(r"let\s+(?:mut\s+)?(\w+)").unwrap();
    
    // JavaScript/TypeScript patterns
    pub static ref JS_FUNCTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"function\s+(\w+)").unwrap(),
        Regex::new(r"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\s*(?:function|\(.*?\)\s*=>)").unwrap(),
        Regex::new(r"(\w+)\s*:\s*(?:async\s+)?function").unwrap(),
    ];
    
    pub static ref JS_TYPE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"class\s+(\w+)").unwrap(),
        Regex::new(r"interface\s+(\w+)").unwrap(),
        Regex::new(r"type\s+(\w+)\s*=").unwrap(),
    ];
    
    pub static ref JS_VARIABLE_PATTERN: Regex = 
        Regex::new(r"(?:const|let|var)\s+(\w+)").unwrap();
    
    // Python patterns
    pub static ref PYTHON_FUNCTION_PATTERN: Regex = 
        Regex::new(r"def\s+(\w+)").unwrap();
    
    pub static ref PYTHON_CLASS_PATTERN: Regex = 
        Regex::new(r"class\s+(\w+)").unwrap();
    
    pub static ref PYTHON_VARIABLE_PATTERN: Regex = 
        Regex::new(r"(\w+)\s*=").unwrap();
    
    // C/C++ patterns
    pub static ref C_FUNCTION_PATTERN: Regex = 
        Regex::new(r"(?:static\s+)?(?:inline\s+)?(?:\w+\s+)*(\w+)\s*\([^)]*\)\s*\{").unwrap();
    
    pub static ref C_TYPE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"struct\s+(\w+)").unwrap(),
        Regex::new(r"typedef\s+struct\s*\{[^}]*\}\s*(\w+)").unwrap(),
        Regex::new(r"typedef\s+(?:struct\s+)?(\w+)").unwrap(),
    ];
}