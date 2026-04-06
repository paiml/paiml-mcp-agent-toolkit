#![cfg_attr(coverage_nightly, coverage(off))]

use super::helpers::*;
use super::types::*;
use crate::services::semantic::chunk_code;
use ignore::WalkBuilder;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Check if file content has module-level `coverage(off)` annotation.
///
/// Checks first 5 lines for the annotation (it's always at the top).
fn has_coverage_off(content: &str) -> bool {
    debug_assert!(!content.is_empty(), "content must not be empty");
    // Module-level inner attributes (#![...]) can appear anywhere before the first
    // code item, often after long doc comments (line 6-200+). Scan all #! lines.
    content.lines().any(|line| {
        let t = line.trim();
        t.starts_with("#!")
            && (t.contains("cfg_attr(coverage_nightly, coverage(off))")
                || t.contains("cfg_attr(coverage_nightly,coverage(off))"))
    })
}

/// Load cached coverage_off_files from SQLite metadata.
fn load_coverage_off_files(conn: &rusqlite::Connection) -> HashSet<String> {
    debug_assert!(true, "contract: load_coverage_off_files");
    let json: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'coverage_off_files'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    serde_json::from_str(&json).unwrap_or_default()
}

impl AgentContextIndex {
    /// Build index from project directory
    ///
    /// # Arguments
    /// * `project_path` - Root directory to index
    ///
    /// # Returns
    /// Built index ready for queries
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn build(project_path: &Path) -> Result<Self, String> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let project_root = project_path
            .canonicalize()
            .map_err(|e| format!("Invalid project path: {e}"))?;

        let mut functions = Vec::with_capacity(20_000);
        let mut file_count = 0;
        let mut languages_seen = HashMap::new();
        let mut file_checksums: HashMap<String, String> = HashMap::with_capacity(4_000);
        let mut coverage_off_files = HashSet::new();
        // Reusable read buffer — avoids allocating a new String per file (~33 MB saved)
        let mut read_buf = String::with_capacity(32 * 1024);

        // Load compile_commands.json for C/C++ include path discovery
        let _compile_commands = load_compile_commands(&project_root);

        // Walk the project directory respecting .gitignore (fixes issue #146)
        for entry in WalkBuilder::new(&project_root)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .filter_entry(|e| !is_ignored_dir(e.path()))
            .build()
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

            // Read file content into reusable buffer
            read_buf.clear();
            let content = match std::fs::File::open(path).and_then(|mut f| {
                use std::io::Read;
                f.read_to_string(&mut read_buf)
            }) {
                Ok(_) => read_buf.as_str(),
                Err(_) => continue, // Skip binary/unreadable files
            };

            let relative_path = path
                .strip_prefix(&project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Compute SHA256 checksum for incremental updates
            let checksum = compute_file_sha256(content);
            file_checksums.insert(relative_path.clone(), checksum);

            // Detect module-level coverage(off) — cached for O(1) query-time lookup
            if has_coverage_off(content) {
                coverage_off_files.insert(relative_path.clone());
            }

            // Extract functions using AST chunker
            let chunks = match chunk_code(content, language) {
                Ok(c) => c,
                Err(_) => continue, // Skip parse errors
            };

            let lang_str = format!("{language:?}");
            *languages_seen.entry(lang_str.clone()).or_insert(0) += 1;

            for mut chunk in chunks {
                // Index functions, structs, enums, traits, type aliases (issue #150)
                use crate::services::semantic::ChunkType;
                let definition_type = match &chunk.chunk_type {
                    ChunkType::Function => DefinitionType::Function,
                    ChunkType::Struct => DefinitionType::Struct,
                    ChunkType::Enum => DefinitionType::Enum,
                    ChunkType::Trait => DefinitionType::Trait,
                    ChunkType::TypeAlias => DefinitionType::TypeAlias,
                    _ => continue, // Skip classes, modules, files, impl blocks
                };

                // Skip test functions and test files (#159: reduce index bloat)
                if is_test_chunk(&chunk.chunk_name, &relative_path) {
                    continue;
                }

                // Extract quality metrics (borrows chunk)
                let quality = extract_quality_metrics(&chunk, content);

                // Extract signature (first line of definition) — must borrow before move
                let signature = chunk
                    .content
                    .lines()
                    .next()
                    .unwrap_or(&chunk.chunk_name)
                    .to_string();

                // Extract doc comment (lines starting with /// or /** before definition)
                let doc_comment = extract_doc_comment(content, chunk.start_line);

                // Take ownership of chunk fields (avoids .clone() — saves ~12.5 MB peak)
                let entry = FunctionEntry {
                    file_path: relative_path.clone(),
                    function_name: std::mem::take(&mut chunk.chunk_name),
                    signature,
                    definition_type,
                    doc_comment,
                    source: std::mem::take(&mut chunk.content),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    language: lang_str.clone(),
                    quality,
                    checksum: std::mem::take(&mut chunk.content_checksum),
                    // Annotations populated after all definitions collected
                    commit_count: 0,
                    churn_score: 0.0,
                    clone_count: 0,
                    pattern_diversity: 0.0,
                    fault_annotations: Vec::new(),
                    linked_definition: None,
                };

                functions.push(entry);
            }

            file_count += 1;
            if file_count % 500 == 0 {
                eprint!("\r  Indexing... {} files", file_count);
            }
        }
        if file_count >= 500 {
            eprintln!("\r  Indexed {} files", file_count);
        }

        // Build indices and corpus
        let indices = build_indices(&functions);

        // Build call graph
        let (calls, called_by) = build_call_graph(&functions, &indices.name_index);

        // Compute graph metrics (PageRank, centrality)
        let graph_metrics = compute_graph_metrics(functions.len(), &calls, &called_by);

        // Compute name frequency for generic name demotion
        let name_frequency = compute_name_frequency(&indices.name_index, functions.len());

        // Populate cached annotations (churn, duplicates, entropy, faults)
        populate_cached_annotations(&mut functions, &indices.file_index, &project_root);

        // Link C/C++ declarations to definitions (header → implementation)
        link_declarations_to_definitions(&mut functions);

        // Calculate average TDG score
        let avg_tdg = if !functions.is_empty() {
            functions.iter().map(|f| f.quality.tdg_score).sum::<f32>() / functions.len() as f32
        } else {
            0.0
        };

        let manifest = IndexManifest {
            version: "1.4.0".to_string(), // v1.4.0: call graph exclusion, test filtering, lazy corpus_lower
            built_at: chrono::Utc::now().to_rfc3339(),
            project_root: project_root.to_string_lossy().to_string(),
            function_count: functions.len(),
            file_count,
            languages: languages_seen.keys().cloned().collect(),
            avg_tdg_score: avg_tdg,
            file_checksums,
            last_incremental_changes: 0, // Full build, not incremental
        };

        // Pre-compute lowercase corpus (avoids per-query lowercasing of 42K+ docs)
        let corpus_lower: Vec<String> = indices.corpus.iter().map(|d| d.to_lowercase()).collect();

        Ok(Self {
            functions,
            name_index: indices.name_index,
            file_index: indices.file_index,
            corpus: indices.corpus,
            corpus_lower,
            name_frequency,
            calls,
            called_by,
            graph_metrics,
            project_root,
            manifest,
            db_path: None, // Set after save()
            coverage_off_files,
        })
    }

    /// Get index statistics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            // Estimate index size: functions vec + name_index map + file_index map
            index_size_bytes: (std::mem::size_of_val(&self.functions)
                + self.functions.len() * std::mem::size_of::<FunctionEntry>()
                + self.name_index.len() * 64  // Approximate string + vec overhead
                + self.file_index.len() * 64) as u64,
        }
    }

    /// Get manifest
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn manifest(&self) -> &IndexManifest {
        &self.manifest
    }

    /// Get function by exact name
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_by_name(&self, name: &str) -> Vec<&FunctionEntry> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.name_index
            .get(name)
            .map(|indices| indices.iter().map(|&i| &self.functions[i]).collect())
            .unwrap_or_default()
    }

    /// Get functions in a file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_by_file(&self, file_path: &str) -> Vec<&FunctionEntry> {
        debug_assert!(!file_path.is_empty(), "file_path must not be empty");
        self.file_index
            .get(file_path)
            .map(|indices| indices.iter().map(|&i| &self.functions[i]).collect())
            .unwrap_or_default()
    }

    /// Get all functions (for iteration)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn all_functions(&self) -> &[FunctionEntry] {
        &self.functions
    }

    /// Get corpus for search
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn corpus(&self) -> &[String] {
        &self.corpus
    }

    /// Get project root
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

include!("build_workspace.rs");
include!("build_persistence.rs");
include!("build_incremental.rs");
include!("build_accessors.rs");
include!("build_helpers.rs");
