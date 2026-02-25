    // Python Language Tests (feature-gated)

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_simple_function() {
        let source = "def hello():\n    print(\"Hello, world!\")\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello");
        assert_eq!(chunks[0].language, "python");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_class() {
        let source = "class MyClass:\n    def __init__(self, value):\n        self.value = value\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "MyClass");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_multiple_functions() {
        let source = "def func1():\n    pass\n\ndef func2():\n    pass\n\ndef func3():\n    pass\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[cfg(not(feature = "python-ast"))]
    #[test]
    fn test_python_feature_disabled() {
        let source = "def test(): pass";
        let result = chunk_code(source, Language::Python);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("python-ast feature is disabled"));
    }

    // C Language Tests (feature-gated)

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_simple_function() {
        let source = "int add(int a, int b) {\n    return a + b;\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
        assert_eq!(chunks[0].language, "c");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_main_function() {
        let source = "int main(int argc, char *argv[]) {\n    return 0;\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "main");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_pointer_return_function() {
        let source = "char *get_string() {\n    return \"hello\";\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "get_string");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_multiple_functions() {
        let source = "void func1() {}\nint func2() { return 0; }\nfloat func3() { return 0.0; }\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[cfg(not(feature = "c-ast"))]
    #[test]
    fn test_c_feature_disabled() {
        let source = "int main() { return 0; }";
        let result = chunk_code(source, Language::C);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("c-ast feature is disabled"));
    }

    // C++ Language Tests (feature-gated)

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_simple_function() {
        let source = "int add(int a, int b) {\n    return a + b;\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
        assert_eq!(chunks[0].language, "cpp");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_class() {
        let source = "class MyClass {\npublic:\n    int value;\n};\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "MyClass");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_template_function() {
        let source = "template <typename T>\nT max(T a, T b) {\n    return (a > b) ? a : b;\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "max");
        assert!(chunks[0].content.contains("template"));
    }

    #[cfg(not(feature = "cpp-ast"))]
    #[test]
    fn test_cpp_feature_disabled() {
        let source = "int main() { return 0; }";
        let result = chunk_code(source, Language::Cpp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cpp-ast feature is disabled"));
    }

    // Go Language Tests (feature-gated)

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_simple_function() {
        let source = "package main\n\nfunc hello() {\n    fmt.Println(\"Hello\")\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello");
        assert_eq!(chunks[0].language, "go");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_method() {
        let source = "package main\n\ntype Person struct {\n    Name string\n}\n\nfunc (p Person) Greet() string {\n    return \"Hello, \" + p.Name\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        let method_chunk = chunks.iter().find(|c| c.chunk_name == "Greet");
        assert!(method_chunk.is_some());
        assert_eq!(method_chunk.unwrap().chunk_type, ChunkType::Function);
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_struct_type() {
        let source = "package main\n\ntype User struct {\n    ID   int\n    Name string\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "User");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_interface_type() {
        let source =
            "package main\n\ntype Reader interface {\n    Read(p []byte) (n int, err error)\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "Reader");
    }

    #[cfg(not(feature = "go-ast"))]
    #[test]
    fn test_go_feature_disabled() {
        let source = "package main\nfunc main() {}";
        let result = chunk_code(source, Language::Go);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("go-ast feature is disabled"));
    }
