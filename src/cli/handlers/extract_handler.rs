#![cfg_attr(coverage_nightly, coverage(off))]
//! Extract handler: parse a single file with tree-sitter and dump function boundaries as JSON.
//! No index required — direct AST extraction (#215).

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::Path;

use crate::services::semantic::chunker::{self, ChunkType, Language};

#[derive(Serialize)]
struct ExtractItem {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
    start_line: usize,
    end_line: usize,
    lines: usize,
}

/// Handle `pmat extract --list <FILE>`
pub async fn handle_extract_list(file_path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(file_path)
        .with_context(|| format!("Cannot read {}", file_path.display()))?;

    let language = detect_chunker_language(file_path)?;
    let chunks = chunker::chunk_code(&source, language)
        .map_err(|e| anyhow::anyhow!("tree-sitter parse failed: {e}"))?;

    let mut items: Vec<ExtractItem> = chunks
        .into_iter()
        .filter(|c| c.chunk_type != ChunkType::File)
        .map(|c| ExtractItem {
            name: c.chunk_name,
            item_type: chunk_type_label(&c.chunk_type).to_string(),
            start_line: c.start_line,
            end_line: c.end_line,
            lines: c.end_line.saturating_sub(c.start_line) + 1,
        })
        .collect();

    items.sort_by_key(|i| i.start_line);

    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}

fn chunk_type_label(ct: &ChunkType) -> &'static str {
    match ct {
        ChunkType::Function => "function",
        ChunkType::Class => "class",
        ChunkType::Module => "module",
        ChunkType::File => "file",
        ChunkType::Struct => "struct",
        ChunkType::Enum => "enum",
        ChunkType::Trait => "trait",
        ChunkType::TypeAlias => "type_alias",
        ChunkType::Impl => "impl",
    }
}

fn detect_chunker_language(path: &Path) -> Result<Language> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => Ok(Language::Rust),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => Ok(Language::TypeScript),
        "py" | "pyi" => Ok(Language::Python),
        "c" | "h" => Ok(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Ok(Language::Cpp),
        "go" => Ok(Language::Go),
        "lua" => Ok(Language::Lua),
        _ => bail!("Unsupported file extension '.{ext}' for extract. Supported: rs, ts, py, c, cpp, go, lua"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_chunker_language() {
        assert_eq!(
            detect_chunker_language(&PathBuf::from("foo.rs")).unwrap(),
            Language::Rust
        );
        assert_eq!(
            detect_chunker_language(&PathBuf::from("bar.py")).unwrap(),
            Language::Python
        );
        assert_eq!(
            detect_chunker_language(&PathBuf::from("baz.ts")).unwrap(),
            Language::TypeScript
        );
        assert_eq!(
            detect_chunker_language(&PathBuf::from("qux.go")).unwrap(),
            Language::Go
        );
        assert!(detect_chunker_language(&PathBuf::from("nope.xyz")).is_err());
    }

    #[test]
    fn test_chunk_type_label() {
        assert_eq!(chunk_type_label(&ChunkType::Function), "function");
        assert_eq!(chunk_type_label(&ChunkType::Struct), "struct");
        assert_eq!(chunk_type_label(&ChunkType::Enum), "enum");
        assert_eq!(chunk_type_label(&ChunkType::Trait), "trait");
        assert_eq!(chunk_type_label(&ChunkType::Impl), "impl");
    }
}
