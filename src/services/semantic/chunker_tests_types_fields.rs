    // CodeChunk Field Tests

    #[test]
    fn test_code_chunk_fields() {
        let source = "fn test_func() { let x = 1; }";
        let chunks = chunk_code(source, Language::Rust).unwrap();

        assert_eq!(chunks.len(), 1);
        let chunk = &chunks[0];

        assert!(chunk.file_path.is_empty());
        assert_eq!(chunk.chunk_type, ChunkType::Function);
        assert_eq!(chunk.chunk_name, "test_func");
        assert_eq!(chunk.language, "rust");
        assert!(chunk.start_line >= 1);
        assert!(chunk.end_line >= chunk.start_line);
        assert!(!chunk.content.is_empty());
        assert!(!chunk.content_checksum.is_empty());
        assert_eq!(chunk.content_checksum.len(), 64);
    }

    #[test]
    fn test_checksum_different_for_different_content() {
        let source1 = "fn func1() {}";
        let source2 = "fn func2() {}";

        let chunks1 = chunk_code(source1, Language::Rust).unwrap();
        let chunks2 = chunk_code(source2, Language::Rust).unwrap();

        assert_ne!(chunks1[0].content_checksum, chunks2[0].content_checksum);
    }

    // Language Enum Tests

    #[test]
    fn test_language_debug() {
        let lang = Language::Rust;
        let debug_str = format!("{:?}", lang);
        assert_eq!(debug_str, "Rust");
    }

    #[test]
    fn test_language_clone() {
        let lang = Language::TypeScript;
        let cloned = lang;
        assert_eq!(lang, cloned);
    }

    #[test]
    fn test_language_copy() {
        let lang = Language::Python;
        let copied = lang;
        assert_eq!(lang, copied);
    }

    #[test]
    fn test_all_language_variants() {
        let languages = [Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::C,
            Language::Cpp,
            Language::Go];

        for i in 0..languages.len() {
            for j in (i + 1)..languages.len() {
                assert_ne!(languages[i], languages[j]);
            }
        }
    }

    // ChunkType Enum Tests

    #[test]
    fn test_chunk_type_debug() {
        let chunk_type = ChunkType::Function;
        let debug_str = format!("{:?}", chunk_type);
        assert_eq!(debug_str, "Function");
    }

    #[test]
    fn test_chunk_type_clone() {
        let chunk_type = ChunkType::Class;
        let cloned = chunk_type.clone();
        assert_eq!(chunk_type, cloned);
    }

    #[test]
    fn test_all_chunk_type_variants() {
        let chunk_types = [ChunkType::Function,
            ChunkType::Class,
            ChunkType::Module,
            ChunkType::File];

        for i in 0..chunk_types.len() {
            for j in (i + 1)..chunk_types.len() {
                assert_ne!(chunk_types[i], chunk_types[j]);
            }
        }
    }

    // CodeChunk Struct Tests

    #[test]
    fn test_code_chunk_debug() {
        let chunk = CodeChunk {
            file_path: "test.rs".to_string(),
            chunk_type: ChunkType::Function,
            chunk_name: "test".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 3,
            content: "fn test() {}".to_string(),
            content_checksum: "abc123".to_string(),
        };

        let debug_str = format!("{:?}", chunk);
        assert!(debug_str.contains("CodeChunk"));
        assert!(debug_str.contains("test.rs"));
    }

    #[test]
    fn test_code_chunk_clone() {
        let chunk = CodeChunk {
            file_path: "test.rs".to_string(),
            chunk_type: ChunkType::Function,
            chunk_name: "test".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 3,
            content: "fn test() {}".to_string(),
            content_checksum: "abc123".to_string(),
        };

        let cloned = chunk.clone();
        assert_eq!(chunk.file_path, cloned.file_path);
        assert_eq!(chunk.chunk_type, cloned.chunk_type);
        assert_eq!(chunk.chunk_name, cloned.chunk_name);
    }

    // Complex Code Tests

    #[test]
    fn test_rust_complex_code() {
        let source = r#"
/// A struct with fields
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    /// Creates a new Point
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    /// Calculates distance from origin
    fn distance_from_origin(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}

mod geometry {
    /// Area.
    pub fn area(width: u32, height: u32) -> u32 {
        width * height
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        assert!(chunks.len() >= 3);

        let impl_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Class && c.chunk_name == "Point");
        assert!(impl_chunk.is_some());

        let module_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Module && c.chunk_name == "geometry");
        assert!(module_chunk.is_some());
    }

    #[test]
    fn test_typescript_complex_code() {
        let source = r#"
interface IService {
    start(): Promise<void>;
    stop(): void;
}

class Service implements IService {
    private running: boolean = false;

    async start(): Promise<void> {
        this.running = true;
    }

    stop(): void {
        this.running = false;
    }
}

const helper = (x: number): number => x * 2;

function processData(data: string[]): string[] {
    return data.map(d => d.toUpperCase());
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        assert!(chunks.len() >= 4);

        let names: Vec<&str> = chunks.iter().map(|c| c.chunk_name.as_str()).collect();
        assert!(names.contains(&"IService"));
        assert!(names.contains(&"Service"));
        assert!(names.contains(&"helper"));
        assert!(names.contains(&"processData"));
    }
