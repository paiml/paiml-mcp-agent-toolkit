// Tests for DependencyGraphBuilder
// Extracted from builder.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_source_file() {
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.rs")));
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.py")));
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.ts")));
        assert!(!DependencyGraphBuilder::is_source_file(Path::new(
            "test.txt"
        )));
        assert!(!DependencyGraphBuilder::is_source_file(Path::new(
            "README.md"
        )));
    }

    #[test]
    fn test_extract_function_names() {
        assert_eq!(
            DependencyGraphBuilder::extract_function_name("pub fn test_func() {"),
            Some("test_func")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_function_name("fn private_func(arg: i32) {"),
            Some("private_func")
        );
    }

    /// extract_type_name strips `pub`/keyword prefix, trailing generics `<...>`
    /// and braces `{...}`.
    #[test]
    fn test_extract_type_name_covers_generics_and_body() {
        assert_eq!(
            DependencyGraphBuilder::extract_type_name("pub struct Foo {", "struct"),
            Some("Foo")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_type_name("struct Bar<T> { x: T }", "struct"),
            Some("Bar")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_type_name("enum Status", "enum"),
            Some("Status")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_type_name("   ", "struct"),
            None
        );
    }

    /// extract_ts_name strips `export`/`const`/`function`, `(...)` args, and `= ...`.
    #[test]
    fn test_extract_ts_name_covers_all_prefixes() {
        assert_eq!(
            DependencyGraphBuilder::extract_ts_name("export function foo() {}"),
            Some("foo")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_ts_name("function bar(arg: number) {}"),
            Some("bar")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_ts_name("export const baz = 42"),
            Some("baz")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_ts_name("const qux = () => 1"),
            Some("qux")
        );
        assert_eq!(DependencyGraphBuilder::extract_ts_name(""), None);
    }

    /// extract_ts_class_name strips `export`/`class` and trailing `{...}`.
    #[test]
    fn test_extract_ts_class_name_covers_branches() {
        assert_eq!(
            DependencyGraphBuilder::extract_ts_class_name("export class PubClass {}"),
            Some("PubClass")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_ts_class_name("class Bare"),
            Some("Bare")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_ts_class_name("class Foo{body}"),
            Some("Foo")
        );
        assert_eq!(DependencyGraphBuilder::extract_ts_class_name(""), None);
    }

    #[test]
    fn test_extract_python_names() {
        assert_eq!(
            DependencyGraphBuilder::extract_python_function_name("def test_func():"),
            Some("test_func")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_python_class_name("class TestClass(BaseClass):"),
            Some("TestClass")
        );
    }

    #[test]
    fn test_builder_creation() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.symbol_table.is_empty());
        assert_eq!(builder.graph.node_count(), 0);
        assert_eq!(builder.graph.edge_count(), 0);
    }

    /// Test that analyze_file cache hit returns correct node_id
    /// Validates unwrap at line 159-162
    #[test]
    fn test_analyze_file_cache_hit() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // First analysis - should create new node
        let node_id_1 = builder.analyze_file(&test_file).unwrap();
        assert_eq!(builder.graph.node_count(), 1);
        assert!(builder.node_map.contains_key(&test_file));
        assert!(builder.processed_hashes.contains_key(&test_file));

        // Second analysis with same content - should return cached node_id
        let node_id_2 = builder.analyze_file(&test_file).unwrap();
        assert_eq!(node_id_1, node_id_2, "Cache hit should return same node_id");
        assert_eq!(
            builder.graph.node_count(),
            1,
            "Should not create duplicate node"
        );
    }

    /// Test that analyze_file updates node when content changes
    /// Validates unwrap at line 184-187
    #[test]
    fn test_analyze_file_content_change() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // First analysis
        let node_id_1 = builder.analyze_file(&test_file).unwrap();
        let original_loc = builder.graph.node_weight(node_id_1).unwrap().loc;

        // Modify file (different content = different hash)
        fs::write(&test_file, "fn main() {}\nfn helper() {}\n").unwrap();

        // Second analysis - should update existing node
        let node_id_2 = builder.analyze_file(&test_file).unwrap();
        assert_eq!(
            node_id_1, node_id_2,
            "Should reuse same node_id for same path"
        );
        assert_eq!(
            builder.graph.node_count(),
            1,
            "Should still have only 1 node"
        );

        // Verify node was updated (LOC should increase)
        let updated_loc = builder.graph.node_weight(node_id_2).unwrap().loc;
        assert!(
            updated_loc > original_loc,
            "Node should be updated with new LOC"
        );
    }

    /// Test that node_map and processed_hashes stay synchronized
    /// Validates invariant that both maps are updated together (lines 191-195)
    #[test]
    fn test_node_map_hash_map_synchronization() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn test() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // Analyze file
        builder.analyze_file(&test_file).unwrap();

        // Both maps should contain the file
        assert!(
            builder.node_map.contains_key(&test_file),
            "node_map should contain analyzed file"
        );
        assert!(
            builder.processed_hashes.contains_key(&test_file),
            "processed_hashes should contain analyzed file"
        );

        // Both maps should have same size
        assert_eq!(
            builder.node_map.len(),
            builder.processed_hashes.len(),
            "node_map and processed_hashes must stay synchronized"
        );
    }

    /// Parse Python: `def` (public), `def _` (private), `class`, and skipped lines.
    #[test]
    fn test_parse_python_symbols_covers_all_branches() {
        let builder = DependencyGraphBuilder::new();
        let content = "\
def public_func():
    pass
def _private_func():
    pass
class MyClass(Base):
    pass
# a comment
import os
";
        let symbols = builder.parse_python_symbols(content).unwrap();
        assert_eq!(symbols.len(), 3);

        assert_eq!(symbols[0].name, "public_func");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[0].visibility, Visibility::Public);
        assert_eq!(symbols[0].line, 0);

        assert_eq!(symbols[1].name, "_private_func");
        assert_eq!(symbols[1].visibility, Visibility::Private);
        assert_eq!(symbols[1].line, 2);

        assert_eq!(symbols[2].name, "MyClass");
        assert_eq!(symbols[2].kind, SymbolKind::Struct);
        assert_eq!(symbols[2].visibility, Visibility::Public);
        assert_eq!(symbols[2].line, 4);
    }

    #[test]
    fn test_parse_python_symbols_empty_content() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.parse_python_symbols("").unwrap().is_empty());
    }

    /// Parse TS/JS: `export function`, `export const`, `function`, `const`, `export class`.
    #[test]
    fn test_parse_typescript_symbols_covers_all_branches() {
        let builder = DependencyGraphBuilder::new();
        let content = "\
export function pubFn() {}
export const pubConst = 1;
function privFn() {}
const privConst = 2;
export class PubClass {}
// skip
let other = 5;
";
        let symbols = builder.parse_typescript_symbols(content).unwrap();
        assert_eq!(symbols.len(), 5);

        assert_eq!(symbols[0].name, "pubFn");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[0].visibility, Visibility::Public);

        assert_eq!(symbols[1].name, "pubConst");
        assert_eq!(symbols[1].visibility, Visibility::Public);

        assert_eq!(symbols[2].name, "privFn");
        assert_eq!(symbols[2].visibility, Visibility::Private);

        assert_eq!(symbols[3].name, "privConst");
        assert_eq!(symbols[3].visibility, Visibility::Private);

        assert_eq!(symbols[4].name, "PubClass");
        assert_eq!(symbols[4].kind, SymbolKind::Struct);
        assert_eq!(symbols[4].visibility, Visibility::Public);
    }

    #[test]
    fn test_parse_typescript_symbols_empty_content() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.parse_typescript_symbols("").unwrap().is_empty());
    }

    /// Parse Rust: `use x;` (picked), `use y` (no semicolon, skipped),
    /// `fn` (skipped). Covers strip_prefix/strip_suffix path.
    #[test]
    fn test_parse_rust_imports_covers_branches() {
        let builder = DependencyGraphBuilder::new();
        let content = "\
use std::path::Path;
use crate::graph::Node;
  use indented::Import;
use missing_semicolon
fn main() {}
// comment
";
        let imports = builder.parse_rust_imports(content).unwrap();
        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0], "std::path::Path");
        assert_eq!(imports[1], "crate::graph::Node");
        assert_eq!(imports[2], "indented::Import");
    }

    #[test]
    fn test_parse_rust_imports_empty_content() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.parse_rust_imports("").unwrap().is_empty());
    }

    /// Parse Python: `import` and `from` both picked;
    /// other lines (def, comment, code) skipped.
    #[test]
    fn test_parse_python_imports_covers_branches() {
        let builder = DependencyGraphBuilder::new();
        let content = "\
import os
from pathlib import Path
  from indented import Foo
def main():
    pass
# a comment
x = 1
";
        let imports = builder.parse_python_imports(content).unwrap();
        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0], "import os");
        assert_eq!(imports[1], "from pathlib import Path");
        assert_eq!(imports[2], "from indented import Foo");
    }

    #[test]
    fn test_parse_python_imports_empty_content() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.parse_python_imports("").unwrap().is_empty());
    }

    /// Parse TS/JS: `import ... from '...'` (picked, quote-stripped),
    /// `import` with no `from` (skipped), `const X = require('...')` (picked),
    /// `const` without require (skipped), other lines skipped.
    #[test]
    fn test_parse_typescript_imports_covers_branches() {
        let builder = DependencyGraphBuilder::new();
        let content = "\
import { x } from 'lodash';
import \"side-effect-only\";
const fs = require('fs');
const unused = 42;
let other = 5;
// comment
";
        let imports = builder.parse_typescript_imports(content).unwrap();
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0], "lodash");
        assert_eq!(imports[1], "fs");
    }

    #[test]
    fn test_parse_typescript_imports_empty_content() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.parse_typescript_imports("").unwrap().is_empty());
    }

    /// Test first-time file analysis creates both node and hash entry
    /// Validates initialization path (lines 189-195)
    #[test]
    fn test_first_time_analysis() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("fresh.rs");
        fs::write(&test_file, "pub fn new_function() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // Fresh builder should have empty maps
        assert_eq!(builder.node_map.len(), 0);
        assert_eq!(builder.processed_hashes.len(), 0);

        // First analysis
        let node_id = builder.analyze_file(&test_file).unwrap();

        // Should create entries in both maps
        assert_eq!(builder.node_map.len(), 1);
        assert_eq!(builder.processed_hashes.len(), 1);
        assert!(builder.node_map.contains_key(&test_file));
        assert!(builder.processed_hashes.contains_key(&test_file));

        // Should create node in graph
        assert_eq!(builder.graph.node_count(), 1);
        assert!(builder.graph.node_weight(node_id).is_some());
    }
}
