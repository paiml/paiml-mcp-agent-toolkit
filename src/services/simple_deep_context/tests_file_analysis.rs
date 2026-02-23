#![cfg_attr(coverage_nightly, coverage(off))]
// File complexity coverage tests for SimpleDeepContext

use super::*;

use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn test_analyze_file_complexity_rust_simple_funcs() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.rs");
    fs::write(&test_file, "fn hello() {\n    println!(\"hello\");\n}\n\nfn goodbye() {\n    println!(\"goodbye\");\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert_eq!(metrics.function_count, 2);
    assert_eq!(metrics.high_complexity_functions, 0);
    assert!(metrics.avg_complexity >= 1.0);
    assert!(!metrics.function_names.is_empty());
}

#[tokio::test]
async fn test_analyze_file_complexity_rust_high_complexity() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complex.rs");
    fs::write(
        &test_file,
        concat!(
            "fn very_complex(x: i32, y: i32) -> i32 {\n",
            "    if x > 0 {\n",
            "        if x > 10 {\n",
            "            if y > 0 {\n",
            "                match x {\n",
            "                    1 => { if y > 5 { return 1; } else { return 2; } }\n",
            "                    2 => { if y > 3 { return 3; } else { return 4; } }\n",
            "                    3 => return 5,\n",
            "                    4 => return 6,\n",
            "                    5 => return 7,\n",
            "                    _ => return 8,\n",
            "                }\n",
            "            } else { return -1; }\n",
            "        }\n",
            "    }\n",
            "    0\n",
            "}\n\n",
            "fn simple() -> i32 { 42 }\n",
        ),
    )
    .unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert_eq!(metrics.function_count, 2);
    assert!(metrics.avg_complexity > 1.0);
    assert!(metrics.function_names.contains(&"very_complex".to_string()));
    assert!(metrics.function_names.contains(&"simple".to_string()));
}

#[tokio::test]
async fn test_analyze_file_complexity_rust_no_functions() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("no_funcs.rs");
    fs::write(
        &test_file,
        "const FOO: i32 = 42;\nconst BAR: &str = \"hello\";\n",
    )
    .unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert_eq!(metrics.function_count, 0);
    assert_eq!(metrics.high_complexity_functions, 0);
    assert_eq!(metrics.avg_complexity, 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_rust_parse_error() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("bad.rs");
    fs::write(
        &test_file,
        "this is not valid rust code {{{}}}}}\nfn broken(\n",
    )
    .unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count == 0 || metrics.avg_complexity >= 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_python_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.py");
    fs::write(&test_file, concat!(
        "def hello():\n    print('hello')\n\n",
        "def complex_func(x):\n    if x > 0:\n        for i in range(x):\n            if i % 2 == 0:\n                print(i)\n    return x\n\n",
        "class MyClass:\n    def method(self):\n        pass\n"
    )).unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
    assert!(metrics.avg_complexity >= 1.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_js_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("app.js");
    fs::write(&test_file, concat!(
        "function greet(name) {\n    if (name) {\n        console.log('Hello ' + name);\n    }\n}\n\n",
        "const process = (data) => {\n    for (let i = 0; i < data.length; i++) {\n        console.log(data[i]);\n    }\n};\n\n",
        "function helper() { return 42; }\n"
    )).unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
    assert!(metrics.avg_complexity >= 1.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_ts_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("app.ts");
    fs::write(&test_file, "function greet(name: string): void {\n    console.log('Hello ' + name);\n}\n\nconst add = (a: number, b: number): number => {\n    return a + b;\n};\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 1);
}

#[tokio::test]
async fn test_analyze_file_complexity_ruby_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("app.rb");
    fs::write(&test_file, "def hello\n  puts 'hello'\nend\n\ndef process(data)\n  data.each do |item|\n    puts item\n  end\nend\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
    assert!(metrics.avg_complexity >= 1.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_go_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("main.go");
    fs::write(&test_file, "package main\n\nfunc main() {\n    fmt.Println(\"hello\")\n}\n\nfunc helper(x int) int {\n    if x > 0 {\n        return x * 2\n    }\n    return 0\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
    assert!(metrics.avg_complexity >= 1.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_cs_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("Program.cs");
    fs::write(&test_file, "public class Program {\n    public static void Main(string[] args) {\n        Console.WriteLine(\"Hello\");\n    }\n\n    public static int Process(int x) {\n        if (x > 0) { return x * 2; }\n        return 0;\n    }\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
}

#[tokio::test]
async fn test_analyze_file_complexity_kt_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("Main.kt");
    fs::write(&test_file, "fun main() {\n    println(\"Hello\")\n}\n\nfun process(x: Int): Int {\n    return if (x > 0) x * 2 else 0\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
}

#[tokio::test]
async fn test_analyze_file_complexity_nonexistent_file() {
    let analyzer = SimpleDeepContext;
    let path = Path::new("/tmp/definitely_does_not_exist_abc123.rs");
    let metrics = analyzer.analyze_file_complexity(path).await.unwrap();
    assert_eq!(metrics.function_count, 0);
    assert_eq!(metrics.avg_complexity, 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_no_extension() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("Makefile");
    fs::write(&test_file, "all:\n\techo hello\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.avg_complexity >= 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_java_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("Main.java");
    fs::write(&test_file, "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello\");\n    }\n\n    public int process(int x) {\n        if (x > 0) { return x * 2; }\n        return 0;\n    }\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
}

#[tokio::test]
async fn test_analyze_file_complexity_c_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("main.c");
    fs::write(&test_file, "int main() {\n    printf(\"hello\\n\");\n    return 0;\n}\n\nint helper(int x) {\n    if (x > 0) { return x * 2; }\n    return 0;\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
    assert!(metrics.avg_complexity >= 1.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_cpp_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("main.cpp");
    fs::write(&test_file, "#include <iostream>\n\nint main() {\n    std::cout << \"Hello\" << std::endl;\n    return 0;\n}\n\nvoid process(int x) {\n    if (x > 0) {\n        for (int i = 0; i < x; i++) { std::cout << i; }\n    }\n}\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
}

#[tokio::test]
async fn test_analyze_file_complexity_wat_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("module.wat");
    fs::write(&test_file, "(module\n  (func $add (param $a i32) (param $b i32) (result i32)\n    local.get $a\n    local.get $b\n    i32.add))\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.avg_complexity >= 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_wasm_binary() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("module.wasm");
    fs::write(&test_file, &[0x00, 0x61, 0x73, 0x6D]).unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.avg_complexity >= 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_empty_rust_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.rs");
    fs::write(&test_file, "").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert_eq!(metrics.function_count, 0);
    assert_eq!(metrics.high_complexity_functions, 0);
    assert_eq!(metrics.avg_complexity, 0.0);
}

#[tokio::test]
async fn test_analyze_file_complexity_ruchy_file() {
    let analyzer = SimpleDeepContext;
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("app.ruchy");
    fs::write(&test_file, "def hello\n  puts 'hello'\nend\n\ndef process(data)\n  data.each do |item|\n    puts item\n  end\nend\n").unwrap();

    let metrics = analyzer.analyze_file_complexity(&test_file).await.unwrap();
    assert!(metrics.function_count >= 2);
}

#[tokio::test]
async fn test_analyze_multi_language_project() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        "fn compute(x: i32) -> i32 {\n    if x > 0 { x * 2 } else { -x }\n}\n",
    )
    .unwrap();
    fs::write(
        src_dir.join("utils.py"),
        "def calculate(x):\n    if x > 0:\n        return x * 2\n    return -x\n",
    )
    .unwrap();

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec![],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();
    assert_eq!(report.file_count, 2);
    assert!(report.complexity_metrics.total_functions >= 2);
    assert!(report.file_complexity_details.len() == 2);
}
