// RED Phase: Write failing tests first

// NOTE (Sprint 47): Use assetsearch (../../assetsearch) for MCP-based semantic search.
// All tests in this file marked #[ignore] pending migration to assetsearch.

// PMAT-SEARCH-001: AST-Aware Code Chunker
// Test count: 20 tests (5 per language + 5 edge cases)

use pmat::services::semantic::chunker::*;

// ============================================================================
// Rust Language Tests (4 tests)
// ============================================================================

#[test]
fn test_chunk_rust_functions() {
    let source = r#"
        /// Calculate sum
        fn add(a: i32, b: i32) -> i32 { a + b }

        /// Calculate product
        fn multiply(a: i32, b: i32) -> i32 { a * b }
    "#;

    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].chunk_type, ChunkType::Function);
    assert_eq!(chunks[0].chunk_name, "add");
    assert!(chunks[0].content.contains("Calculate sum"));
    assert_eq!(chunks[1].chunk_type, ChunkType::Function);
    assert_eq!(chunks[1].chunk_name, "multiply");
}

#[test]
fn test_chunk_rust_impl_blocks() {
    let source = r#"
        struct Calculator;

        impl Calculator {
            fn add(&self, a: i32, b: i32) -> i32 { a + b }
            fn sub(&self, a: i32, b: i32) -> i32 { a - b }
        }
    "#;

    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert!(chunks.len() >= 1);
    // Should extract impl block or individual methods
    let has_impl_or_methods = chunks.iter().any(|c| {
        c.chunk_type == ChunkType::Class || c.chunk_type == ChunkType::Function
    });
    assert!(has_impl_or_methods);
}

#[test]
fn test_chunk_rust_modules() {
    let source = r#"
        mod calculator {
            pub fn add(a: i32, b: i32) -> i32 { a + b }
        }
    "#;

    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert!(chunks.len() >= 1);
    // Should extract module or functions within
    let has_content = chunks.iter().any(|c| {
        c.chunk_type == ChunkType::Module || c.chunk_type == ChunkType::Function
    });
    assert!(has_content);
}

#[test]
fn test_chunk_rust_with_docstrings() {
    let source = r#"
        /// This function adds two numbers
        /// # Examples
        /// ```
        /// let result = add(2, 3);
        /// assert_eq!(result, 5);
        /// ```
        fn add(a: i32, b: i32) -> i32 { a + b }
    "#;

    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("This function adds two numbers"));
    assert!(chunks[0].content.contains("# Examples"));
}

// ============================================================================
// TypeScript Language Tests (4 tests)
// ============================================================================

#[test]
fn test_chunk_typescript_class() {
    let source = r#"
        class Calculator {
            add(a: number, b: number): number { return a + b; }
            multiply(a: number, b: number): number { return a * b; }
        }
    "#;

    let chunks = chunk_code(source, Language::TypeScript).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_type, ChunkType::Class);
    assert_eq!(chunks[0].chunk_name, "Calculator");
}

#[test]
fn test_chunk_typescript_functions() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b;
        }

        const multiply = (a: number, b: number): number => a * b;
    "#;

    let chunks = chunk_code(source, Language::TypeScript).unwrap();
    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().any(|c| c.chunk_name == "add"));
    assert!(chunks.iter().any(|c| c.chunk_name == "multiply"));
}

#[test]
fn test_chunk_typescript_interface() {
    let source = r#"
        interface Calculator {
            add(a: number, b: number): number;
            multiply(a: number, b: number): number;
        }
    "#;

    let chunks = chunk_code(source, Language::TypeScript).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_type, ChunkType::Class); // Interfaces treated as class-like
    assert_eq!(chunks[0].chunk_name, "Calculator");
}

#[test]
fn test_chunk_typescript_with_jsdoc() {
    let source = r#"
        /**
         * Adds two numbers together
         * @param a First number
         * @param b Second number
         * @returns Sum of a and b
         */
        function add(a: number, b: number): number {
            return a + b;
        }
    "#;

    let chunks = chunk_code(source, Language::TypeScript).unwrap();
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("Adds two numbers together"));
    assert!(chunks[0].content.contains("@param"));
}

// ============================================================================
// Python Language Tests (4 tests)
// ============================================================================

#[test]
fn test_chunk_python_functions() {
    let source = r#"
def add(a, b):
    """Calculate sum"""
    return a + b

def multiply(a, b):
    """Calculate product"""
    return a * b
    "#;

    let chunks = chunk_code(source, Language::Python).unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].chunk_type, ChunkType::Function);
    assert_eq!(chunks[0].chunk_name, "add");
    assert!(chunks[0].content.contains("Calculate sum"));
}

#[test]
fn test_chunk_python_class() {
    let source = r#"
class Calculator:
    """A simple calculator class"""

    def add(self, a, b):
        return a + b

    def multiply(self, a, b):
        return a * b
    "#;

    let chunks = chunk_code(source, Language::Python).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_type, ChunkType::Class);
    assert_eq!(chunks[0].chunk_name, "Calculator");
    assert!(chunks[0].content.contains("simple calculator"));
}

#[test]
fn test_chunk_python_with_decorators() {
    let source = r#"
@property
def value(self):
    """Get the value"""
    return self._value

@staticmethod
def create():
    """Factory method"""
    return Calculator()
    "#;

    let chunks = chunk_code(source, Language::Python).unwrap();
    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().any(|c| c.chunk_name == "value"));
    assert!(chunks.iter().any(|c| c.chunk_name == "create"));
}

#[test]
fn test_chunk_python_nested_functions() {
    let source = r#"
def outer(x):
    """Outer function"""
    def inner(y):
        """Inner function"""
        return x + y
    return inner
    "#;

    let chunks = chunk_code(source, Language::Python).unwrap();
    // Should extract outer function (may or may not extract inner)
    assert!(chunks.len() >= 1);
    assert!(chunks.iter().any(|c| c.chunk_name == "outer"));
}

// ============================================================================
// C/C++ Language Tests (4 tests)
// ============================================================================

#[test]
fn test_chunk_c_functions() {
    let source = r#"
        // Calculate sum
        int add(int a, int b) {
            return a + b;
        }

        // Calculate product
        int multiply(int a, int b) {
            return a * b;
        }
    "#;

    let chunks = chunk_code(source, Language::C).unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].chunk_type, ChunkType::Function);
    assert_eq!(chunks[0].chunk_name, "add");
}

#[test]
fn test_chunk_cpp_class() {
    let source = r#"
        class Calculator {
        public:
            int add(int a, int b) { return a + b; }
            int multiply(int a, int b) { return a * b; }
        };
    "#;

    let chunks = chunk_code(source, Language::Cpp).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_type, ChunkType::Class);
    assert_eq!(chunks[0].chunk_name, "Calculator");
}

#[test]
fn test_chunk_cpp_template() {
    let source = r#"
        template<typename T>
        T add(T a, T b) {
            return a + b;
        }
    "#;

    let chunks = chunk_code(source, Language::Cpp).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_name, "add");
}

#[test]
fn test_chunk_c_with_comments() {
    let source = r#"
        /**
         * Adds two integers
         * @param a First integer
         * @param b Second integer
         * @return Sum of a and b
         */
        int add(int a, int b) {
            return a + b;
        }
    "#;

    let chunks = chunk_code(source, Language::C).unwrap();
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("Adds two integers"));
}

// ============================================================================
// Go Language Tests (4 tests)
// ============================================================================

#[test]
fn test_chunk_go_functions() {
    let source = r#"
        // Add calculates sum
        func Add(a, b int) int {
            return a + b
        }

        // Multiply calculates product
        func Multiply(a, b int) int {
            return a * b
        }
    "#;

    let chunks = chunk_code(source, Language::Go).unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].chunk_type, ChunkType::Function);
    assert_eq!(chunks[0].chunk_name, "Add");
    assert!(chunks[0].content.contains("calculates sum"));
}

#[test]
fn test_chunk_go_struct() {
    let source = r#"
        type Calculator struct {
            value int
        }

        func (c *Calculator) Add(a, b int) int {
            return a + b
        }
    "#;

    let chunks = chunk_code(source, Language::Go).unwrap();
    // Should extract struct and/or methods
    assert!(chunks.len() >= 1);
    let has_struct_or_method = chunks.iter().any(|c| {
        c.chunk_name == "Calculator" || c.chunk_name == "Add"
    });
    assert!(has_struct_or_method);
}

#[test]
fn test_chunk_go_interface() {
    let source = r#"
        // Calculator interface for arithmetic operations
        type Calculator interface {
            Add(a, b int) int
            Multiply(a, b int) int
        }
    "#;

    let chunks = chunk_code(source, Language::Go).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_name, "Calculator");
}

#[test]
fn test_chunk_go_package() {
    let source = r#"
        package calculator

        // Add calculates sum
        func Add(a, b int) int {
            return a + b
        }
    "#;

    let chunks = chunk_code(source, Language::Go).unwrap();
    assert!(chunks.len() >= 1);
    assert!(chunks.iter().any(|c| c.chunk_name == "Add"));
}

// ============================================================================
// Edge Case Tests (5 tests)
// ============================================================================

#[test]
fn test_chunk_empty_file() {
    let source = "";
    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_whitespace_only() {
    let source = "   \n\n   \t\t   \n   ";
    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_comments_only() {
    let source = r#"
        // This is a comment
        /* This is a block comment */
        /// This is a doc comment
    "#;
    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_checksum_deterministic() {
    let source = "fn foo() { }";
    let chunks1 = chunk_code(source, Language::Rust).unwrap();
    let chunks2 = chunk_code(source, Language::Rust).unwrap();

    assert_eq!(chunks1.len(), chunks2.len());
    assert_eq!(chunks1[0].content_checksum, chunks2[0].content_checksum);
}

#[test]
fn test_chunk_metadata_complete() {
    let source = r#"
        /// Test function
        fn test_func() {
            println!("test");
        }
    "#;

    let chunks = chunk_code(source, Language::Rust).unwrap();
    assert_eq!(chunks.len(), 1);

    let chunk = &chunks[0];
    assert_eq!(chunk.chunk_type, ChunkType::Function);
    assert_eq!(chunk.chunk_name, "test_func");
    assert_eq!(chunk.language, "rust");
    assert!(chunk.start_line > 0);
    assert!(chunk.end_line >= chunk.start_line);
    assert!(!chunk.content.is_empty());
    assert!(!chunk.content_checksum.is_empty());
    assert_eq!(chunk.content_checksum.len(), 64); // SHA256 hex length
}
