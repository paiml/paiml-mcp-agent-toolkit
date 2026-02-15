#![cfg_attr(coverage_nightly, coverage(off))]
//! Extract handler: parse a single file with tree-sitter and dump function boundaries as JSON.
//! No index required — direct AST extraction (#215).

use anyhow::{bail, Context, Result};
use std::path::Path;

use crate::services::semantic::chunker::{self, Language};

/// Handle `pmat extract --list <FILE>`
pub async fn handle_extract_list(file_path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(file_path)
        .with_context(|| format!("Cannot read {}", file_path.display()))?;

    let language = detect_chunker_language(file_path)?;
    let result =
        chunker::extract_file_details(&file_path.display().to_string(), &source, language)
            .map_err(|e| anyhow::anyhow!("tree-sitter parse failed: {e}"))?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
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
}
