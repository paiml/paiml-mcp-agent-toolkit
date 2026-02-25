// CB-800 Scala Best Practices tests
// Included from tests_part2.rs via include!() - shares parent module scope

#[test]
fn test_cb800_detects_mutable_collection() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("App.scala"),
        "val cache = mutable.HashMap[String, Int]()\nval items = mutable.Buffer[Int]()",
    )
    .unwrap();

    let violations = detect_cb800_mutable_collection(temp.path());
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].pattern_id, "CB-800");
    assert!(violations[0].description.contains("mutable.HashMap"));
}

#[test]
fn test_cb800_allows_import_of_mutable() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "import scala.collection.mutable.Map\nval x = Map(\"a\" -> 1)",
    )
    .unwrap();

    let violations = detect_cb800_mutable_collection(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb801_detects_null_literal() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "val x: String = null\nval y = if (x == null) \"default\" else x",
    )
    .unwrap();

    let violations = detect_cb801_null_usage(temp.path());
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].pattern_id, "CB-801");
}

#[test]
fn test_cb801_allows_java_interop_null() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "@Nullable val x: String = null",
    )
    .unwrap();

    let violations = detect_cb801_null_usage(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb801_no_false_positive_on_nullable_identifier() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "val nullable = true\nval isNullable = false",
    )
    .unwrap();

    let violations = detect_cb801_null_usage(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb802_detects_wildcard_import() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "import com.example.models._\nimport org.apache.spark.sql.*",
    )
    .unwrap();

    let violations = detect_cb802_wildcard_import(temp.path());
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].pattern_id, "CB-802");
}

#[test]
fn test_cb802_allows_stdlib_wildcard() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "import scala.collection.immutable._\nimport java.util._",
    )
    .unwrap();

    let violations = detect_cb802_wildcard_import(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb803_detects_return_statement() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "def foo(x: Int): Int = {\n  if (x > 0) return x\n  x * -1\n}",
    )
    .unwrap();

    let violations = detect_cb803_return_statement(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-803");
}

#[test]
fn test_cb804_detects_var_declaration() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "var count = 0\nprivate var state = \"init\"",
    )
    .unwrap();

    let violations = detect_cb804_var_declaration(temp.path());
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].pattern_id, "CB-804");
}

#[test]
fn test_cb804_no_false_positive_on_val() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "val count = 0\nprivate val state = \"init\"",
    )
    .unwrap();

    let violations = detect_cb804_var_declaration(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb805_detects_blocking_in_future() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "import scala.concurrent.Future\nval f = Future {\n  Thread.sleep(1000)\n  42\n}",
    )
    .unwrap();

    let violations = detect_cb805_blocking_in_future(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-805");
    assert!(violations[0].description.contains("Thread.sleep"));
}

#[test]
fn test_cb805_no_false_positive_outside_future() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("App.scala"),
        "def main(): Unit = {\n  Thread.sleep(1000)\n}",
    )
    .unwrap();

    let violations = detect_cb805_blocking_in_future(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_scala_test_file_detection() {
    use std::path::Path;
    assert!(is_scala_test_file(Path::new(
        "src/test/scala/AppTest.scala"
    )));
    assert!(is_scala_test_file(Path::new("AppSpec.scala")));
    assert!(is_scala_test_file(Path::new("TestHelper.scala")));
    assert!(!is_scala_test_file(Path::new("src/main/scala/App.scala")));
}

#[test]
fn test_scala_production_lines() {
    let content = "// comment\nval x = 1\n/* block */\nval y = 2\n\nval z = 3 // inline";
    let lines = compute_scala_production_lines(content);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], (2, "val x = 1".to_string()));
    assert_eq!(lines[1], (4, "val y = 2".to_string()));
    assert_eq!(lines[2], (6, "val z = 3".to_string()));
}

#[test]
fn test_walkdir_scala_files() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("App.scala"), "object App").unwrap();
    fs::write(temp.path().join("build.sc"), "// mill build").unwrap();
    fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

    let files = walkdir_scala_files(temp.path());
    assert_eq!(files.len(), 2);
}

#[test]
fn test_scala_skips_test_files() {
    let temp = TempDir::new().unwrap();
    let test_dir = temp.path().join("test");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(
        test_dir.join("AppTest.scala"),
        "var x = null\nimport foo._\nreturn 42",
    )
    .unwrap();

    // All detectors should skip test files
    assert_eq!(detect_cb800_mutable_collection(temp.path()).len(), 0);
    assert_eq!(detect_cb801_null_usage(temp.path()).len(), 0);
    assert_eq!(detect_cb802_wildcard_import(temp.path()).len(), 0);
    assert_eq!(detect_cb803_return_statement(temp.path()).len(), 0);
    assert_eq!(detect_cb804_var_declaration(temp.path()).len(), 0);
}
