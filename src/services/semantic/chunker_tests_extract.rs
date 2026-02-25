    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::TypeScript.as_str(), "typescript");
        assert_eq!(Language::Python.as_str(), "python");
        assert_eq!(Language::C.as_str(), "c");
        assert_eq!(Language::Cpp.as_str(), "cpp");
        assert_eq!(Language::Go.as_str(), "go");
        assert_eq!(Language::Lua.as_str(), "lua");
    }

    #[test]
    fn test_chunk_type_as_str() {
        assert_eq!(ChunkType::Function.as_str(), "function");
        assert_eq!(ChunkType::Class.as_str(), "class");
        assert_eq!(ChunkType::Module.as_str(), "module");
        assert_eq!(ChunkType::File.as_str(), "file");
        assert_eq!(ChunkType::Struct.as_str(), "struct");
        assert_eq!(ChunkType::Enum.as_str(), "enum");
        assert_eq!(ChunkType::Trait.as_str(), "trait");
        assert_eq!(ChunkType::TypeAlias.as_str(), "type_alias");
        assert_eq!(ChunkType::Impl.as_str(), "impl");
    }

    #[test]
    fn test_extract_file_details_empty() {
        let result = extract_file_details("test.rs", "", Language::Rust).unwrap();
        assert_eq!(result.file, "test.rs");
        assert_eq!(result.language, "rust");
        assert!(result.imports.is_empty());
        assert!(result.cfg_test_line.is_none());
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_extract_rust_imports_and_visibility() {
        let source = r#"use std::collections::HashMap;
use super::*;

pub fn public_func() {}

fn private_func() {}

pub(crate) fn crate_func() {}
"#;
        let result = extract_file_details("lib.rs", source, Language::Rust).unwrap();

        assert_eq!(result.imports.len(), 2);
        assert!(result.imports[0].contains("std::collections::HashMap"));
        assert!(result.imports[1].contains("super::*"));

        assert_eq!(result.items.len(), 3);

        let public = result
            .items
            .iter()
            .find(|i| i.name == "public_func")
            .unwrap();
        assert_eq!(public.visibility, "pub");
        assert_eq!(public.item_type, "function");

        let private = result
            .items
            .iter()
            .find(|i| i.name == "private_func")
            .unwrap();
        assert_eq!(private.visibility, "");

        let crate_vis = result
            .items
            .iter()
            .find(|i| i.name == "crate_func")
            .unwrap();
        assert_eq!(crate_vis.visibility, "pub(crate)");
    }

    #[test]
    fn test_extract_rust_cfg_test_line() {
        let source = r#"use std::io;

pub fn production_code() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {}
}
"#;
        let result = extract_file_details("lib.rs", source, Language::Rust).unwrap();

        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].contains("std::io"));

        // cfg_test_line should be the line of #[cfg(test)]
        assert!(result.cfg_test_line.is_some());
        let test_line = result.cfg_test_line.unwrap();
        // #[cfg(test)] is on line 5
        assert_eq!(test_line, 5);
    }

    #[test]
    fn test_extract_rust_no_cfg_test() {
        let source = "pub fn foo() {}\nfn bar() {}\n";
        let result = extract_file_details("lib.rs", source, Language::Rust).unwrap();
        assert!(result.cfg_test_line.is_none());
    }

    #[test]
    fn test_extract_rust_struct_enum_trait() {
        let source = r#"pub struct MyStruct { x: i32 }

pub enum MyEnum { A, B }

pub trait MyTrait {
    fn required(&self);
}
"#;
        let result = extract_file_details("types.rs", source, Language::Rust).unwrap();

        let s = result.items.iter().find(|i| i.name == "MyStruct").unwrap();
        assert_eq!(s.item_type, "struct");
        assert_eq!(s.visibility, "pub");

        let e = result.items.iter().find(|i| i.name == "MyEnum").unwrap();
        assert_eq!(e.item_type, "enum");
        assert_eq!(e.visibility, "pub");

        let t = result.items.iter().find(|i| i.name == "MyTrait").unwrap();
        assert_eq!(t.item_type, "trait");
        assert_eq!(t.visibility, "pub");
    }

    #[test]
    fn test_extract_rust_impl_method_visibility() {
        let source = r#"struct Foo;

impl Foo {
    pub fn public_method(&self) {}
    fn private_method(&self) {}
}
"#;
        let result = extract_file_details("foo.rs", source, Language::Rust).unwrap();

        let pub_method = result
            .items
            .iter()
            .find(|i| i.name == "public_method")
            .unwrap();
        assert_eq!(pub_method.visibility, "pub");

        let priv_method = result
            .items
            .iter()
            .find(|i| i.name == "private_method")
            .unwrap();
        assert_eq!(priv_method.visibility, "");
    }

    #[test]
    fn test_extract_typescript_export_visibility() {
        let source = r#"import { foo } from './foo';

export function exportedFunc(): void {}

function privateFunc(): void {}

export const arrowExport = () => {};
"#;
        let result = extract_file_details("app.ts", source, Language::TypeScript).unwrap();

        assert_eq!(result.language, "typescript");
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].contains("foo"));
        assert!(result.cfg_test_line.is_none());

        let exported = result
            .items
            .iter()
            .find(|i| i.name == "exportedFunc")
            .unwrap();
        assert_eq!(exported.visibility, "export");

        let private = result
            .items
            .iter()
            .find(|i| i.name == "privateFunc")
            .unwrap();
        assert_eq!(private.visibility, "");

        let arrow = result
            .items
            .iter()
            .find(|i| i.name == "arrowExport")
            .unwrap();
        assert_eq!(arrow.visibility, "export");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_extract_go_visibility() {
        let source = r#"package main

import "fmt"

func ExportedFunc() {
    fmt.Println("hello")
}

func unexportedFunc() {}
"#;
        let result = extract_file_details("main.go", source, Language::Go).unwrap();

        assert_eq!(result.language, "go");
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].contains("fmt"));

        let exported = result
            .items
            .iter()
            .find(|i| i.name == "ExportedFunc")
            .unwrap();
        assert_eq!(exported.visibility, "pub");

        let unexported = result
            .items
            .iter()
            .find(|i| i.name == "unexportedFunc")
            .unwrap();
        assert_eq!(unexported.visibility, "");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_extract_python_imports() {
        let source = "import os\nfrom pathlib import Path\n\ndef my_func():\n    pass\n";
        let result = extract_file_details("app.py", source, Language::Python).unwrap();

        assert_eq!(result.language, "python");
        assert_eq!(result.imports.len(), 2);
        assert!(result.imports[0].contains("import os"));
        assert!(result.imports[1].contains("from pathlib"));
        assert!(result.cfg_test_line.is_none());
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].visibility, "");
    }

    #[test]
    fn test_extract_items_sorted_by_line() {
        let source = "fn c() {}\nfn a() {}\nfn b() {}\n";
        let result = extract_file_details("test.rs", source, Language::Rust).unwrap();

        assert_eq!(result.items.len(), 3);
        assert!(result.items[0].start_line <= result.items[1].start_line);
        assert!(result.items[1].start_line <= result.items[2].start_line);
    }

    #[test]
    fn test_extract_item_lines_field() {
        let source = "fn single_line() {}\n\nfn multi_line(\n    x: i32,\n    y: i32,\n) -> i32 {\n    x + y\n}\n";
        let result = extract_file_details("test.rs", source, Language::Rust).unwrap();

        let single = result
            .items
            .iter()
            .find(|i| i.name == "single_line")
            .unwrap();
        assert_eq!(single.lines, 1);

        let multi = result
            .items
            .iter()
            .find(|i| i.name == "multi_line")
            .unwrap();
        assert!(multi.lines > 1);
    }

    #[test]
    fn test_extract_rust_extern_crate() {
        let source = "extern crate alloc;\n\nfn foo() {}\n";
        let result = extract_file_details("lib.rs", source, Language::Rust).unwrap();
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].contains("extern crate alloc"));
    }

    #[test]
    fn test_file_extract_serializes_to_json() {
        let source = "use std::io;\n\npub fn hello() {}\n";
        let result = extract_file_details("test.rs", source, Language::Rust).unwrap();
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains("\"file\""));
        assert!(json.contains("\"language\""));
        assert!(json.contains("\"imports\""));
        assert!(json.contains("\"items\""));
        assert!(json.contains("\"visibility\""));
        // cfg_test_line should be absent (skip_serializing_if)
        assert!(!json.contains("cfg_test_line"));
    }

    #[test]
    fn test_file_extract_cfg_test_serializes() {
        let source = "fn foo() {}\n\n#[cfg(test)]\nmod tests {}\n";
        let result = extract_file_details("test.rs", source, Language::Rust).unwrap();
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("cfg_test_line"));
    }
