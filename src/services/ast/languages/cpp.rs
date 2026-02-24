#![cfg_attr(coverage_nightly, coverage(off))]
//! C++ Language Support for PMAT
//!
//! This module provides C++-specific analysis capabilities using tree-sitter-cpp parser
//! for AST extraction and complexity analysis aligned with C++ best practices.

#[cfg(feature = "cpp-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "cpp-ast")]
use std::path::{Path, PathBuf};

/// C++ AST visitor that extracts C++-specific AST information
#[cfg(feature = "cpp-ast")]
pub struct CppAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    current_namespace: Vec<String>,
    current_class: Option<String>,
    #[allow(dead_code)]
    is_header: bool,
}

// CppAstVisitor: constructor, analyze_cpp_source, namespace/class extraction
include!("cpp_visitor_core.rs");

// CppAstVisitor: function/method/enum/typedef/template extraction
include!("cpp_visitor_extractors.rs");

// CppAstVisitor: helper/utility methods
include!("cpp_visitor_helpers.rs");

// CppComplexityAnalyzer struct and implementation
include!("cpp_complexity.rs");

// analyze_cpp_file async function
include!("cpp_analyze_file.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[cfg(feature = "cpp-ast")]
    use super::*;
    #[cfg(feature = "cpp-ast")]
    use std::path::Path;

    #[cfg(feature = "cpp-ast")]
    const SIMPLE_CPP_FUNCTION: &str = r#"
#include <iostream>

// Simple function
int add(int a, int b) {
    return a + b;
}

// Main function
int main() {
    int result = add(5, 3);
    std::cout << "Result: " << result << std::endl;
    return 0;
}
"#;

    #[cfg(feature = "cpp-ast")]
    const CPP_CLASS_EXAMPLE: &str = r#"
#include <string>
#include <iostream>

class Person {
private:
    std::string name;
    int age;

public:
    Person(std::string n, int a) : name(n), age(a) {}

    void setName(std::string n) {
        name = n;
    }

    std::string getName() const {
        return name;
    }

    int getAge() const {
        return age;
    }

    virtual void display() const {
        std::cout << "Name: " << name << ", Age: " << age << std::endl;
    }
};

class Student : public Person {
private:
    std::string studentId;

public:
    Student(std::string n, int a, std::string id) 
        : Person(n, a), studentId(id) {}

    std::string getStudentId() const {
        return studentId;
    }

    void display() const override {
        std::cout << "Student - Name: " << getName() 
                  << ", Age: " << getAge() 
                  << ", ID: " << studentId << std::endl;
    }
};
"#;

    #[cfg(feature = "cpp-ast")]
    const CPP_COMPLEX_EXAMPLE: &str = r#"
#include <iostream>
#include <vector>
#include <algorithm>

namespace example {

// Template class
template<typename T>
class Container {
private:
    std::vector<T> data;

public:
    void add(const T& item) {
        data.push_back(item);
    }

    bool contains(const T& item) const {
        return std::find(data.begin(), data.end(), item) != data.end();
    }
    
    size_t size() const {
        return data.size();
    }
    
    // Template method
    template<typename Func>
    void forEach(Func func) {
        std::for_each(data.begin(), data.end(), func);
    }
};

// Enum class
enum class Color {
    Red,
    Green,
    Blue
};

// Function with complex control flow
int complexFunction(int a, int b) {
    int result = 0;
    
    // Lambda expression
    auto calculate = [](int x, int y) -> int {
        if (x > y) {
            return x * 2;
        } else {
            return y * 2;
        }
    };
    
    if (a > b) {
        try {
            if (a > 10) {
                result = calculate(a, b);
            } else {
                result = a;
            }
        } catch (const std::exception& e) {
            std::cerr << "Error: " << e.what() << std::endl;
            return -1;
        }
    } else if (b > a) {
        // Range-based for loop
        std::vector<int> values(b);
        for (int i = 0; i < b; i++) {
            values[i] = i;
        }
        
        for (const auto& val : values) {
            result += val;
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

} // namespace example

int main() {
    example::Container<int> container;
    container.add(5);
    container.add(10);
    
    int x = 10;
    int y = 20;
    int result = example::complexFunction(x, y);
    
    // C++11 type alias
    using IntContainer = example::Container<int>;
    IntContainer another;
    
    return 0;
}
"#;

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_simple_cpp_function_analysis() {
        let visitor = CppAstVisitor::new(Path::new("test.cpp"));
        let items = visitor
            .analyze_cpp_source(SIMPLE_CPP_FUNCTION)
            .expect("Should parse C++ functions");

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

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_class_analysis() {
        let visitor = CppAstVisitor::new(Path::new("person.cpp"));
        let items = visitor
            .analyze_cpp_source(CPP_CLASS_EXAMPLE)
            .expect("Should parse C++ classes");

        // Check for class definitions
        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // Classes stored as Struct
            .collect();

        assert_eq!(class_items.len(), 2, "Should extract two classes");

        // Check class names
        let class_names: Vec<_> = class_items
            .iter()
            .filter_map(|item| match item {
                AstItem::Struct { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(
            class_names.contains(&"Person"),
            "Should extract 'Person' class"
        );
        assert!(
            class_names.contains(&"Student"),
            "Should extract 'Student' class"
        );

        // Check for methods
        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. })) // Methods stored as Function
            .collect();

        assert!(!method_items.is_empty(), "Should extract methods");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_complex_example() {
        let visitor = CppAstVisitor::new(Path::new("complex.cpp"));
        let items = visitor
            .analyze_cpp_source(CPP_COMPLEX_EXAMPLE)
            .expect("Should parse complex C++ code");

        // Check for template class
        let template_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // GenericTypes stored as Struct
            .collect();

        assert!(!template_items.is_empty(), "Should extract template class");

        // Check for enum
        let enum_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Enum { .. }))
            .collect();

        assert_eq!(enum_items.len(), 1, "Should extract one enum");

        if let AstItem::Enum { name, .. } = &enum_items[0] {
            assert_eq!(
                name, "example::Color",
                "Should extract correct enum name with namespace"
            );
        }

        // Check for type alias
        let alias_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // TypeAlias stored as Struct
            .collect();

        assert!(!alias_items.is_empty(), "Should extract type alias");

        // Check for complex function
        let complex_function: Vec<_> = items
            .iter()
            .filter(|item| match item {
                AstItem::Function { name, .. } => name.contains("complexFunction"),
                _ => false,
            })
            .collect();

        assert_eq!(complex_function.len(), 1, "Should extract complex function");

        // Check for namespace qualification
        if let AstItem::Function { name, .. } = &complex_function[0] {
            assert!(
                name.contains("example::"),
                "Should include namespace qualification"
            );
        }
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_complexity_analysis() {
        let mut analyzer = CppComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(CPP_COMPLEX_EXAMPLE)
            .expect("Should analyze C++ complexity");

        // Complex example should have significant complexity
        assert!(
            cyclomatic > 5,
            "Cyclomatic complexity should be significant"
        );
        assert!(cognitive > 5, "Cognitive complexity should be significant");
    }
}
