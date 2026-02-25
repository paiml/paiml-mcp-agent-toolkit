// C language analysis tests: test fixtures and unit tests for CAstVisitor and CComplexityAnalyzer.
// This file is included into c.rs and shares its module scope (no `use` imports here).

#[cfg(feature = "c-ast")]
const SIMPLE_C_FUNCTION: &str = r#"
#include <stdio.h>

// Simple function
int add(int a, int b) {
    return a + b;
}

// Main function
int main() {
    int result = add(5, 3);
    printf("Result: %d\n", result);
    return 0;
}
"#;

#[cfg(feature = "c-ast")]
const C_STRUCT_EXAMPLE: &str = r#"
#include <stdio.h>
#include <stdlib.h>

// Define a simple structure
struct Point {
    int x;
    int y;
};

// Function to create a new point
struct Point* createPoint(int x, int y) {
    struct Point* p = (struct Point*)malloc(sizeof(struct Point));
    p->x = x;
    p->y = y;
    return p;
}

// Calculate distance (simplified)
int distance(struct Point* p1, struct Point* p2) {
    int dx = p2->x - p1->x;
    int dy = p2->y - p1->y;
    return dx*dx + dy*dy;
}
"#;

#[cfg(feature = "c-ast")]
const C_COMPLEX_EXAMPLE: &str = r#"
#include <stdio.h>

// Global variables
int globalValue = 10;
static int privateValue = 20;

// Typedef example
typedef unsigned long size_t;
typedef struct {
    int value;
} Container;

// Enum example
enum Color {
    RED,
    GREEN,
    BLUE
};

// Function with complex control flow
int complexFunction(int a, int b) {
    int result = 0;

    if (a > b) {
        if (a > 10) {
            result = a * 2;
        } else {
            result = a;
        }
    } else if (b > a) {
        for (int i = 0; i < b; i++) {
            result += i;
            if (result > 100) {
                break;
            }
        }
    } else {
        switch (a) {
            case 0:
                result = 0;
                break;
            case 1:
                result = 1;
                break;
            default:
                result = a + b;
                break;
        }
    }

    return result;
}

// Main function
int main() {
    int x = 10;
    int y = 20;
    int result = complexFunction(x, y);
    printf("Result: %d\n", result);
    return 0;
}
"#;

#[cfg(feature = "c-ast")]
#[test]
fn test_simple_c_function_analysis() {
    let visitor = CAstVisitor::new(Path::new("test.c"));
    let items = visitor
        .analyze_c_source(SIMPLE_C_FUNCTION)
        .expect("Should parse C functions");

    assert!(!items.is_empty(), "Should extract at least one AST item");

    let function_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();

    assert_eq!(function_items.len(), 2, "Should extract two functions");

    // Check function names
    let func_names: Vec<_> = function_items
        .iter()
        .filter_map(|item| match item {
            AstItem::Function { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert!(func_names.contains(&"add"), "Should extract 'add' function");
    assert!(
        func_names.contains(&"main"),
        "Should extract 'main' function"
    );
}

#[cfg(feature = "c-ast")]
#[test]
fn test_c_struct_analysis() {
    let visitor = CAstVisitor::new(Path::new("point.c"));
    let items = visitor
        .analyze_c_source(C_STRUCT_EXAMPLE)
        .expect("Should parse C struct");

    // Check for struct definition
    let struct_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();

    assert_eq!(struct_items.len(), 1, "Should extract one struct");

    if let AstItem::Struct { name, .. } = &struct_items[0] {
        assert_eq!(name, "Point", "Should extract correct struct name");
    }

    // Check for functions
    let function_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();

    assert_eq!(function_items.len(), 2, "Should extract two functions");
}

#[cfg(feature = "c-ast")]
#[test]
fn test_c_complex_example() {
    let visitor = CAstVisitor::new(Path::new("complex.c"));
    let items = visitor
        .analyze_c_source(C_COMPLEX_EXAMPLE)
        .expect("Should parse complex C code");

    // Check for global variables
    let var_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, AstItem::Struct { .. })) // Variables stored as Struct
        .collect();

    assert!(!var_items.is_empty(), "Should extract global variables");

    // Check for enum
    let enum_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, AstItem::Enum { .. }))
        .collect();

    assert_eq!(enum_items.len(), 1, "Should extract one enum");

    if let AstItem::Enum { name, .. } = &enum_items[0] {
        assert_eq!(name, "Color", "Should extract correct enum name");
    }

    // Check for typedef
    let typedef_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, AstItem::Struct { .. })) // TypeAliases stored as Struct
        .collect();

    assert!(!typedef_items.is_empty(), "Should extract typedefs");
}

#[cfg(feature = "c-ast")]
#[test]
fn test_c_complexity_analysis() {
    let mut analyzer = CComplexityAnalyzer::new();
    let (cyclomatic, cognitive) = analyzer
        .analyze_complexity(C_COMPLEX_EXAMPLE)
        .expect("Should analyze C complexity");

    // Complex example should have significant complexity
    assert!(
        cyclomatic > 5,
        "Cyclomatic complexity should be significant"
    );
    assert!(cognitive > 5, "Cognitive complexity should be significant");
}
