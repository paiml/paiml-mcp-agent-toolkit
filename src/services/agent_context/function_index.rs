//! Function Index - RAG Index for Agent Context
//!
//! Builds a searchable index of all functions in a project with quality annotations.

use crate::services::semantic::{chunk_code, CodeChunk, Language};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Quality metrics for a function
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// TDG score (lower is better, 0-10 scale)
    pub tdg_score: f32,
    /// TDG grade (A, B, C, D, F)
    pub tdg_grade: String,
    /// Cyclomatic complexity
    pub complexity: u32,
    /// Cognitive complexity
    pub cognitive_complexity: u32,
    /// Big-O runtime estimate
    pub big_o: String,
    /// SATD marker count (TODO, FIXME, HACK)
    pub satd_count: u32,
    /// Lines of code
    pub loc: u32,
}

/// A function entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    /// File path relative to project root
    pub file_path: String,
    /// Function name
    pub function_name: String,
    /// Full function signature
    pub signature: String,
    /// Documentation comment (if any)
    pub doc_comment: Option<String>,
    /// Full function source code
    pub source: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
    /// Programming language
    pub language: String,
    /// Quality metrics
    pub quality: QualityMetrics,
    /// Content checksum for incremental updates
    pub checksum: String,
}

/// Index manifest with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManifest {
    /// Version of the index format
    pub version: String,
    /// Timestamp when index was built
    pub built_at: String,
    /// Project root path
    pub project_root: String,
    /// Number of functions indexed
    pub function_count: usize,
    /// Number of files processed
    pub file_count: usize,
    /// Languages detected
    pub languages: Vec<String>,
    /// Average TDG score
    pub avg_tdg_score: f32,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total functions indexed
    pub total_functions: usize,
    /// Functions by language
    pub by_language: HashMap<String, usize>,
    /// Functions by TDG grade
    pub by_grade: HashMap<String, usize>,
    /// Average complexity
    pub avg_complexity: f32,
    /// Index size in bytes
    pub index_size_bytes: u64,
}

/// Agent Context Index - searchable function database
#[derive(Clone)]
pub struct AgentContextIndex {
    /// All indexed functions
    pub(crate) functions: Vec<FunctionEntry>,
    /// Function name -> indices for O(1) lookup
    pub(crate) name_index: HashMap<String, Vec<usize>>,
    /// File path -> indices
    pub(crate) file_index: HashMap<String, Vec<usize>>,
    /// Document corpus for BM25-style search
    pub(crate) corpus: Vec<String>,
    /// Project root
    pub(crate) project_root: PathBuf,
    /// Manifest
    pub(crate) manifest: IndexManifest,
}

impl AgentContextIndex {
    /// Build index from project directory
    ///
    /// # Arguments
    /// * `project_path` - Root directory to index
    ///
    /// # Returns
    /// Built index ready for queries
    pub fn build(project_path: &Path) -> Result<Self, String> {
        let project_root = project_path
            .canonicalize()
            .map_err(|e| format!("Invalid project path: {e}"))?;

        let mut functions = Vec::new();
        let mut file_count = 0;
        let mut languages_seen = HashMap::new();

        // Walk the project directory
        for entry in WalkDir::new(&project_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_ignored_dir(e.path()))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Detect language from extension
            let language = match detect_language(path) {
                Some(lang) => lang,
                None => continue,
            };

            // Read file content
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary/unreadable files
            };

            // Extract functions using AST chunker
            let chunks = match chunk_code(&content, language) {
                Ok(c) => c,
                Err(_) => continue, // Skip parse errors
            };

            let relative_path = path
                .strip_prefix(&project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let lang_str = format!("{language:?}");
            *languages_seen.entry(lang_str.clone()).or_insert(0) += 1;

            for chunk in chunks {
                // Only index functions (not classes/modules for now)
                if chunk.chunk_type != crate::services::semantic::ChunkType::Function {
                    continue;
                }

                // Extract quality metrics
                let quality = extract_quality_metrics(&chunk, &content);

                // Extract signature (first line of function)
                let signature = chunk
                    .content
                    .lines()
                    .next()
                    .unwrap_or(&chunk.chunk_name)
                    .to_string();

                // Extract doc comment (lines starting with /// or /** before function)
                let doc_comment = extract_doc_comment(&content, chunk.start_line);

                let entry = FunctionEntry {
                    file_path: relative_path.clone(),
                    function_name: chunk.chunk_name.clone(),
                    signature,
                    doc_comment,
                    source: chunk.content.clone(),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    language: lang_str.clone(),
                    quality,
                    checksum: chunk.content_checksum,
                };

                functions.push(entry);
            }

            file_count += 1;
        }

        // Build indices
        let mut name_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut file_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut corpus = Vec::new();

        for (idx, func) in functions.iter().enumerate() {
            name_index
                .entry(func.function_name.clone())
                .or_default()
                .push(idx);
            file_index
                .entry(func.file_path.clone())
                .or_default()
                .push(idx);

            // Build corpus document: name + signature + doc + content
            let doc = format!(
                "{} {} {} {}",
                func.function_name,
                func.signature,
                func.doc_comment.as_deref().unwrap_or(""),
                extract_identifiers(&func.source)
            );
            corpus.push(doc);
        }

        // Calculate average TDG score
        let avg_tdg = if !functions.is_empty() {
            functions.iter().map(|f| f.quality.tdg_score).sum::<f32>() / functions.len() as f32
        } else {
            0.0
        };

        let manifest = IndexManifest {
            version: "1.0.0".to_string(),
            built_at: chrono::Utc::now().to_rfc3339(),
            project_root: project_root.to_string_lossy().to_string(),
            function_count: functions.len(),
            file_count,
            languages: languages_seen.keys().cloned().collect(),
            avg_tdg_score: avg_tdg,
        };

        Ok(Self {
            functions,
            name_index,
            file_index,
            corpus,
            project_root,
            manifest,
        })
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        let mut by_language: HashMap<String, usize> = HashMap::new();
        let mut by_grade: HashMap<String, usize> = HashMap::new();
        let mut total_complexity: u32 = 0;

        for func in &self.functions {
            *by_language.entry(func.language.clone()).or_default() += 1;
            *by_grade.entry(func.quality.tdg_grade.clone()).or_default() += 1;
            total_complexity += func.quality.complexity;
        }

        let avg_complexity = if !self.functions.is_empty() {
            total_complexity as f32 / self.functions.len() as f32
        } else {
            0.0
        };

        IndexStats {
            total_functions: self.functions.len(),
            by_language,
            by_grade,
            avg_complexity,
            index_size_bytes: 0, // TODO: Calculate actual size
        }
    }

    /// Get manifest
    pub fn manifest(&self) -> &IndexManifest {
        &self.manifest
    }

    /// Get function by exact name
    pub fn get_by_name(&self, name: &str) -> Vec<&FunctionEntry> {
        self.name_index
            .get(name)
            .map(|indices| indices.iter().map(|&i| &self.functions[i]).collect())
            .unwrap_or_default()
    }

    /// Get functions in a file
    pub fn get_by_file(&self, file_path: &str) -> Vec<&FunctionEntry> {
        self.file_index
            .get(file_path)
            .map(|indices| indices.iter().map(|&i| &self.functions[i]).collect())
            .unwrap_or_default()
    }

    /// Get all functions (for iteration)
    pub fn all_functions(&self) -> &[FunctionEntry] {
        &self.functions
    }

    /// Get corpus for search
    pub fn corpus(&self) -> &[String] {
        &self.corpus
    }

    /// Get project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Save index to directory
    pub fn save(&self, index_path: &Path) -> Result<(), String> {
        fs::create_dir_all(index_path)
            .map_err(|e| format!("Failed to create index directory: {e}"))?;

        // Save manifest
        let manifest_json = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
        fs::write(index_path.join("manifest.json"), manifest_json)
            .map_err(|e| format!("Failed to write manifest: {e}"))?;

        // Save functions
        let functions_json = serde_json::to_string(&self.functions)
            .map_err(|e| format!("Failed to serialize functions: {e}"))?;

        // Compress with LZ4
        let compressed = lz4_flex::compress_prepend_size(functions_json.as_bytes());
        fs::write(index_path.join("functions.lz4"), compressed)
            .map_err(|e| format!("Failed to write functions: {e}"))?;

        Ok(())
    }

    /// Load index from directory
    pub fn load(index_path: &Path) -> Result<Self, String> {
        // Load manifest
        let manifest_str = fs::read_to_string(index_path.join("manifest.json"))
            .map_err(|e| format!("Failed to read manifest: {e}"))?;
        let manifest: IndexManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("Failed to parse manifest: {e}"))?;

        // Load functions
        let compressed = fs::read(index_path.join("functions.lz4"))
            .map_err(|e| format!("Failed to read functions: {e}"))?;
        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| format!("Failed to decompress functions: {e}"))?;
        let functions: Vec<FunctionEntry> = serde_json::from_slice(&decompressed)
            .map_err(|e| format!("Failed to parse functions: {e}"))?;

        // Rebuild indices
        let mut name_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut file_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut corpus = Vec::new();

        for (idx, func) in functions.iter().enumerate() {
            name_index
                .entry(func.function_name.clone())
                .or_default()
                .push(idx);
            file_index
                .entry(func.file_path.clone())
                .or_default()
                .push(idx);

            let doc = format!(
                "{} {} {} {}",
                func.function_name,
                func.signature,
                func.doc_comment.as_deref().unwrap_or(""),
                extract_identifiers(&func.source)
            );
            corpus.push(doc);
        }

        let project_root = PathBuf::from(&manifest.project_root);

        Ok(Self {
            functions,
            name_index,
            file_index,
            corpus,
            project_root,
            manifest,
        })
    }
}

/// Check if directory should be ignored
fn is_ignored_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    matches!(
        name,
        "target"
            | "node_modules"
            | ".git"
            | ".pmat"
            | "__pycache__"
            | "venv"
            | ".venv"
            | "dist"
            | "build"
            | ".next"
            | ".cache"
            | "vendor"
    )
}

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" | "js" | "jsx" => Some(Language::TypeScript),
        "py" => Some(Language::Python),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
        "go" => Some(Language::Go),
        _ => None,
    }
}

/// Extract quality metrics from a code chunk
fn extract_quality_metrics(chunk: &CodeChunk, _full_content: &str) -> QualityMetrics {
    let loc = chunk.content.lines().count() as u32;

    // Count control flow complexity (simple heuristic)
    let complexity = count_complexity(&chunk.content);

    // Count SATD markers
    let satd_count = count_satd_markers(&chunk.content);

    // Estimate Big-O from control flow
    let big_o = estimate_big_o(&chunk.content);

    // Calculate TDG score (simplified - real implementation uses full TDG)
    let tdg_score = calculate_simple_tdg(complexity, satd_count, loc);
    let tdg_grade = score_to_grade(tdg_score);

    QualityMetrics {
        tdg_score,
        tdg_grade,
        complexity,
        cognitive_complexity: complexity, // Simplified: use same as cyclomatic
        big_o,
        satd_count,
        loc,
    }
}

/// Count cyclomatic complexity (simplified)
fn count_complexity(source: &str) -> u32 {
    let mut complexity = 1u32; // Base complexity

    // Count decision points
    for line in source.lines() {
        let trimmed = line.trim();

        // Control flow keywords
        if trimmed.starts_with("if ")
            || trimmed.starts_with("else if ")
            || trimmed.contains(" if ")
            || trimmed.starts_with("match ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("loop ")
            || trimmed.contains("&&")
            || trimmed.contains("||")
            || trimmed.contains("? ")
        {
            complexity += 1;
        }

        // Match arms
        if trimmed.contains("=>") && !trimmed.starts_with("//") {
            complexity += 1;
        }
    }

    complexity
}

/// Count SATD markers
fn count_satd_markers(source: &str) -> u32 {
    let upper = source.to_uppercase();
    let mut count = 0;

    for marker in ["TODO", "FIXME", "HACK", "XXX", "BUG", "OPTIMIZE"] {
        count += upper.matches(marker).count() as u32;
    }

    count
}

/// Estimate Big-O from control flow
fn estimate_big_o(source: &str) -> String {
    let mut current_nesting = 0;
    let mut max_nesting = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("loop ")
        {
            current_nesting += 1;
            max_nesting = max_nesting.max(current_nesting);
        }

        if trimmed == "}" && current_nesting > 0 {
            current_nesting -= 1;
        }
    }

    match max_nesting {
        0 => "O(1)".to_string(),
        1 => "O(n)".to_string(),
        2 => "O(n^2)".to_string(),
        3 => "O(n^3)".to_string(),
        n => format!("O(n^{n})"),
    }
}

/// Calculate simplified TDG score
fn calculate_simple_tdg(complexity: u32, satd_count: u32, loc: u32) -> f32 {
    let mut score = 0.0f32;

    // Complexity penalty (0-4 points)
    score += (complexity as f32 / 10.0).min(4.0);

    // SATD penalty (0-2 points)
    score += (satd_count as f32).min(2.0);

    // LOC penalty (0-2 points for > 50 lines)
    if loc > 50 {
        score += ((loc - 50) as f32 / 50.0).min(2.0);
    }

    score.min(10.0)
}

/// Convert TDG score to letter grade
fn score_to_grade(score: f32) -> String {
    match score {
        s if s < 2.0 => "A".to_string(),
        s if s < 4.0 => "B".to_string(),
        s if s < 6.0 => "C".to_string(),
        s if s < 8.0 => "D".to_string(),
        _ => "F".to_string(),
    }
}

/// Extract doc comment from source
fn extract_doc_comment(content: &str, start_line: usize) -> Option<String> {
    if start_line <= 1 {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut doc_lines = Vec::new();

    // Look backwards from function start for doc comments
    for i in (0..start_line - 1).rev() {
        let line = lines.get(i)?.trim();

        if line.starts_with("///") || line.starts_with("//!") {
            doc_lines.push(line.trim_start_matches("///").trim_start_matches("//!").trim());
        } else if line.starts_with("/**") || line.starts_with("/*") {
            // Block comment
            break;
        } else if line.starts_with('*') {
            // Inside block comment
            doc_lines.push(line.trim_start_matches('*').trim());
        } else if line.is_empty() || line.starts_with("#[") || line.starts_with('@') {
            // Empty line or attribute - continue
            continue;
        } else {
            break;
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        doc_lines.reverse();
        Some(doc_lines.join(" "))
    }
}

/// Extract identifiers from source for better search
fn extract_identifiers(source: &str) -> String {
    let mut identifiers = Vec::new();

    for word in source.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let trimmed = word.trim();
        if trimmed.len() >= 3 && !is_keyword(trimmed) {
            identifiers.push(trimmed.to_lowercase());
        }
    }

    identifiers.join(" ")
}

/// Check if word is a language keyword
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "const"
            | "static"
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "return"
            | "break"
            | "continue"
            | "async"
            | "await"
            | "true"
            | "false"
            | "self"
            | "Self"
            | "super"
            | "crate"
            | "where"
            | "move"
            | "ref"
            | "dyn"
            | "box"
            | "in"
            | "as"
            | "unsafe"
            | "extern"
            | "macro"
            | "function"
            | "class"
            | "def"
            | "import"
            | "from"
            | "try"
            | "catch"
            | "throw"
            | "new"
            | "this"
            | "var"
            | "void"
            | "int"
            | "str"
            | "bool"
            | "None"
            | "null"
            | "undefined"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(
            detect_language(Path::new("test.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            detect_language(Path::new("test.py")),
            Some(Language::Python)
        );
        assert_eq!(detect_language(Path::new("test.txt")), None);
    }

    #[test]
    fn test_count_complexity() {
        let simple = "fn foo() { return 1; }";
        assert_eq!(count_complexity(simple), 1);

        let with_if = "fn foo() { if x { return 1; } return 2; }";
        assert_eq!(count_complexity(with_if), 2);
    }

    #[test]
    fn test_count_satd_markers() {
        let clean = "fn foo() { return 1; }";
        assert_eq!(count_satd_markers(clean), 0);

        let with_todo = "fn foo() { // TODO: fix this\n return 1; }";
        assert_eq!(count_satd_markers(with_todo), 1);
    }

    #[test]
    fn test_estimate_big_o() {
        let constant = "fn foo() { return 1; }";
        assert_eq!(estimate_big_o(constant), "O(1)");

        let linear = "fn foo() {\n    for i in items {\n        process(i);\n    }\n}";
        assert_eq!(estimate_big_o(linear), "O(n)");
    }

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(0.5), "A");
        assert_eq!(score_to_grade(2.5), "B");
        assert_eq!(score_to_grade(5.0), "C");
        assert_eq!(score_to_grade(7.0), "D");
        assert_eq!(score_to_grade(9.0), "F");
    }

    #[test]
    fn test_is_ignored_dir() {
        assert!(is_ignored_dir(Path::new("target")));
        assert!(is_ignored_dir(Path::new("node_modules")));
        assert!(!is_ignored_dir(Path::new("src")));
    }
}
