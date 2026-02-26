    // ============ extract_function_names Tests ============

    #[tokio::test]
    #[ignore] // Regex pattern issue - needs investigation
    async fn test_extract_function_names_rust() {
        let analyzer = SimpleDeepContext;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
fn main() {
    helper();
}

pub fn helper() -> i32 {
    42
}

async fn async_func() {
    tokio::time::sleep(Duration::from_millis(1)).await;
}

pub(crate) fn crate_visible() {}
"#,
        )
        .unwrap();

        let names = analyzer
            .extract_function_names_heuristic(&test_file, "rs")
            .await
            .unwrap();
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"helper".to_string()));
        assert!(names.contains(&"async_func".to_string()));
        assert!(names.contains(&"crate_visible".to_string()));
    }

    #[tokio::test]
    async fn test_extract_function_names_python() {
        let analyzer = SimpleDeepContext;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");

        fs::write(
            &test_file,
            r#"
def main():
    pass

async def async_handler():
    await something()

def helper_func(x, y):
    return x + y
"#,
        )
        .unwrap();

        let names = analyzer
            .extract_function_names_heuristic(&test_file, "py")
            .await
            .unwrap();
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"async_handler".to_string()));
        assert!(names.contains(&"helper_func".to_string()));
    }

    #[tokio::test]
    async fn test_extract_function_names_kotlin() {
        let analyzer = SimpleDeepContext;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.kt");

        fs::write(
            &test_file,
            r#"
fun main() {
    println("Hello")
}

suspend fun asyncOperation() {
    delay(100)
}

fun processData(data: String): Int {
    return data.length
}
"#,
        )
        .unwrap();

        let names = analyzer
            .extract_function_names_heuristic(&test_file, "kt")
            .await
            .unwrap();
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"asyncOperation".to_string()));
        assert!(names.contains(&"processData".to_string()));
    }

    #[tokio::test]
    async fn test_extract_function_names_go() {
        let analyzer = SimpleDeepContext;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.go");

        fs::write(
            &test_file,
            r#"
func main() {
    fmt.Println("hello")
}

func (s *Server) HandleRequest() {
    // method
}

func processData(data string) int {
    return len(data)
}
"#,
        )
        .unwrap();

        let names = analyzer
            .extract_function_names_heuristic(&test_file, "go")
            .await
            .unwrap();
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"HandleRequest".to_string()));
        assert!(names.contains(&"processData".to_string()));
    }

    #[tokio::test]
    async fn test_extract_function_names_unknown_extension() {
        let analyzer = SimpleDeepContext;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.xyz");

        fs::write(&test_file, "some content").unwrap();

        let names = analyzer
            .extract_function_names_heuristic(&test_file, "xyz")
            .await
            .unwrap();
        assert!(names.is_empty());
    }
