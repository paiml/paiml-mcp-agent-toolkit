    // TypeScript Language Tests

    #[test]
    fn test_typescript_simple_function() {
        let source = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "greet");
        assert_eq!(chunks[0].language, "typescript");
    }

    #[test]
    fn test_typescript_class() {
        let source = r#"
class MyClass {
    private value: number;

    constructor(value: number) {
        this.value = value;
    }

    getValue(): number {
        return this.value;
    }
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "MyClass");
    }

    #[test]
    fn test_typescript_interface() {
        let source = r#"
interface Person {
    name: string;
    age: number;
    greet(): void;
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "Person");
    }

    #[test]
    fn test_typescript_arrow_function() {
        let source = r#"
const add = (a: number, b: number): number => {
    return a + b;
};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
    }

    #[test]
    fn test_typescript_multiple_arrow_functions() {
        let source = r#"
const func1 = () => {};
const func2 = () => {};
let func3 = () => {};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 3);

        let names: Vec<&str> = chunks.iter().map(|c| c.chunk_name.as_str()).collect();
        assert!(names.contains(&"func1"));
        assert!(names.contains(&"func2"));
        assert!(names.contains(&"func3"));
    }

    #[test]
    fn test_typescript_function_with_jsdoc() {
        let source = r#"
/**
 * Multiplies two numbers
 * @param a - First number
 * @param b - Second number
 * @returns The product
 */
function multiply(a: number, b: number): number {
    return a * b;
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "multiply");
        assert!(chunks[0].content.contains("Multiplies two numbers"));
    }

    #[test]
    fn test_typescript_generic_class() {
        let source = r#"
class Container<T> {
    private item: T;

    constructor(item: T) {
        this.item = item;
    }

    get(): T {
        return this.item;
    }
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "Container");
    }

    #[test]
    fn test_typescript_async_function() {
        let source = r#"
async function fetchData(): Promise<string> {
    const response = await fetch('https://example.com');
    return response.text();
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "fetchData");
        assert!(chunks[0].content.contains("async function"));
    }

    #[test]
    fn test_typescript_export_function() {
        let source = r#"
export function exportedFunc(): void {
    console.log("exported");
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "exportedFunc");
    }

