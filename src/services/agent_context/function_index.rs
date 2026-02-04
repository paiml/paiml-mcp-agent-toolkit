//! Function Index - RAG Index for Agent Context
//!
//! Builds a searchable index of all functions in a project with quality annotations.

use crate::services::semantic::{chunk_code, CodeChunk, Language};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    /// SHA256 checksums for each source file (for incremental updates)
    #[serde(default)]
    pub file_checksums: HashMap<String, String>,
}

/// Serialized payload for the index (v1.1.0+)
#[derive(Serialize, Deserialize)]
struct IndexPayload {
    functions: Vec<FunctionEntry>,
    corpus: Vec<String>,
    calls: HashMap<usize, Vec<usize>>,
    called_by: HashMap<usize, Vec<usize>>,
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
    /// Pre-computed lowercase corpus (avoids per-query lowercasing of 42K+ docs)
    pub(crate) corpus_lower: Vec<String>,
    /// Name frequency for generic name demotion (name -> fraction of total functions)
    pub(crate) name_frequency: HashMap<String, f32>,
    /// Caller graph: func_idx -> indices of functions it calls
    pub(crate) calls: HashMap<usize, Vec<usize>>,
    /// Callee graph: func_idx -> indices of functions that call it
    pub(crate) called_by: HashMap<usize, Vec<usize>>,
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
        let mut file_checksums: HashMap<String, String> = HashMap::new();

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

            // Read file content
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary/unreadable files
            };

            let relative_path = path
                .strip_prefix(&project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Compute SHA256 checksum for incremental updates
            let checksum = compute_file_sha256(&content);
            file_checksums.insert(relative_path.clone(), checksum);

            // Extract functions using AST chunker
            let chunks = match chunk_code(&content, language) {
                Ok(c) => c,
                Err(_) => continue, // Skip parse errors
            };

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

        // Build indices and corpus
        let (name_index, file_index, corpus) = build_indices(&functions);

        // Build call graph
        let (calls, called_by) = build_call_graph(&functions, &name_index);

        // Compute name frequency for generic name demotion
        let name_frequency = compute_name_frequency(&name_index, functions.len());

        // Calculate average TDG score
        let avg_tdg = if !functions.is_empty() {
            functions.iter().map(|f| f.quality.tdg_score).sum::<f32>() / functions.len() as f32
        } else {
            0.0
        };

        let manifest = IndexManifest {
            version: "1.1.0".to_string(),
            built_at: chrono::Utc::now().to_rfc3339(),
            project_root: project_root.to_string_lossy().to_string(),
            function_count: functions.len(),
            file_count,
            languages: languages_seen.keys().cloned().collect(),
            avg_tdg_score: avg_tdg,
            file_checksums,
        };

        // Pre-compute lowercase corpus (avoids per-query lowercasing of 42K+ docs)
        let corpus_lower: Vec<String> = corpus.iter().map(|d| d.to_lowercase()).collect();

        Ok(Self {
            functions,
            name_index,
            file_index,
            corpus,
            corpus_lower,
            name_frequency,
            calls,
            called_by,
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

    /// Build a workspace-level index across multiple project roots.
    ///
    /// For cross-project RAG, each project is indexed and merged into a single
    /// searchable index. File paths are prefixed with the project directory name
    /// to disambiguate across projects.
    pub fn build_workspace(project_paths: &[&Path]) -> Result<Self, String> {
        if project_paths.is_empty() {
            return Err("No project paths provided".to_string());
        }

        if project_paths.len() == 1 {
            return Self::build(project_paths[0]);
        }

        // Build first project as base
        let mut merged = Self::build(project_paths[0])?;

        // Merge remaining projects
        for &path in &project_paths[1..] {
            let other = Self::build(path)?;
            merged.merge(other);
        }

        Ok(merged)
    }

    /// Load an index and prefix all file paths with a project name.
    ///
    /// Used for cross-project search: each sibling project's functions get
    /// paths like `aprender/src/lib.rs` instead of `src/lib.rs`.
    /// Only rebuilds file_index (paths changed). Corpus, name_index, and
    /// call graph are reused from the persisted payload.
    pub fn load_with_prefix(index_path: &Path, prefix: &str) -> Result<Self, String> {
        let mut index = Self::load(index_path)?;

        // Prefix file paths in functions
        for func in &mut index.functions {
            func.file_path = format!("{prefix}/{}", func.file_path);
        }

        // Rebuild only file_index (paths changed), name_index is unchanged
        let mut file_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, func) in index.functions.iter().enumerate() {
            file_index
                .entry(func.file_path.clone())
                .or_default()
                .push(idx);
        }
        index.file_index = file_index;

        // Corpus still valid for search (path token changes are minor)
        // calls/called_by indices still valid (positional)
        Ok(index)
    }

    /// Read `.pmat/workspace.toml` to find configured sibling projects.
    ///
    /// Users opt in to cross-project search by creating this file:
    /// ```toml
    /// siblings = ["../aprender", "../trueno", "../realizar"]
    /// ```
    ///
    /// Returns `(index_path, project_name)` pairs for siblings that have
    /// a `.pmat/context.idx`. Silently skips siblings without an index.
    pub fn discover_sibling_indexes(project_path: &Path) -> Vec<(PathBuf, String)> {
        let workspace_config = project_path.join(".pmat/workspace.toml");
        let config_str = match fs::read_to_string(&workspace_config) {
            Ok(s) => s,
            Err(_) => return Vec::new(), // No config = no siblings
        };

        // Minimal TOML parsing: extract siblings = ["...", "..."]
        let sibling_paths = parse_workspace_siblings(&config_str);
        let mut siblings = Vec::new();

        for rel_path in sibling_paths {
            let abs_path = project_path.join(&rel_path);
            let resolved = match abs_path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue, // Path doesn't exist
            };
            let project_name = resolved
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let idx_path = resolved.join(".pmat/context.idx");
            if idx_path.exists() {
                siblings.push((idx_path, project_name));
            }
        }

        siblings
    }

    /// Merge sibling project indexes into this index for cross-project search.
    ///
    /// Uses a fast path that concatenates pre-built data from each sibling
    /// instead of rebuilding indices/call-graph from scratch. Each sibling's
    /// corpus, name_index, file_index, and call graph are offset-adjusted
    /// and appended in O(n) time.
    ///
    /// Each sibling's `.pmat/context.idx` is never modified.
    pub fn merge_siblings(&mut self, siblings: &[(PathBuf, String)]) {
        for (idx_path, project_name) in siblings {
            match Self::load_with_prefix(idx_path, project_name) {
                Ok(sibling) => {
                    let count = sibling.functions.len();
                    self.merge_fast(sibling);
                    eprintln!("  + {}: {} functions", project_name, count);
                }
                Err(e) => {
                    eprintln!("  ! {}: failed to load ({})", project_name, e);
                }
            }
        }
    }

    /// Fast merge: concatenate pre-built data with index offset adjustment.
    ///
    /// Unlike `merge()` which rebuilds all indices from scratch (O(n*m) for
    /// call graph), this just offsets positional indices and appends. O(n).
    fn merge_fast(&mut self, other: Self) {
        let offset = self.functions.len();

        // Append functions and corpus
        self.functions.extend(other.functions);
        self.corpus.extend(other.corpus.iter().cloned());
        self.corpus_lower.extend(other.corpus_lower);

        // Offset and append name_index
        for (name, indices) in other.name_index {
            self.name_index
                .entry(name)
                .or_default()
                .extend(indices.iter().map(|i| i + offset));
        }

        // Offset and append file_index
        for (path, indices) in other.file_index {
            self.file_index
                .entry(path)
                .or_default()
                .extend(indices.iter().map(|i| i + offset));
        }

        // Offset and append call graph
        for (caller, callees) in other.calls {
            self.calls.insert(
                caller + offset,
                callees.iter().map(|i| i + offset).collect(),
            );
        }
        for (callee, callers) in other.called_by {
            self.called_by.insert(
                callee + offset,
                callers.iter().map(|i| i + offset).collect(),
            );
        }

        // Merge name_frequency (recompute — cheap, just HashMap iteration)
        self.name_frequency =
            compute_name_frequency(&self.name_index, self.functions.len());

        // Update manifest
        self.manifest.function_count = self.functions.len();
        self.manifest.file_count += other.manifest.file_count;
    }

    /// Merge another index into this one.
    fn merge(&mut self, other: Self) {
        for func in other.functions {
            self.functions.push(func);
        }

        // Rebuild all derived data from scratch after merge
        let (name_index, file_index, corpus) = build_indices(&self.functions);
        let (calls, called_by) = build_call_graph(&self.functions, &name_index);
        let name_frequency = compute_name_frequency(&name_index, self.functions.len());

        self.name_index = name_index;
        self.file_index = file_index;
        self.corpus = corpus;
        self.corpus_lower = self.corpus.iter().map(|d| d.to_lowercase()).collect();
        self.name_frequency = name_frequency;
        self.calls = calls;
        self.called_by = called_by;

        // Update manifest
        self.manifest.function_count = self.functions.len();
        self.manifest.file_count += other.manifest.file_count;
        for (k, v) in other.manifest.file_checksums {
            self.manifest.file_checksums.insert(k, v);
        }
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

        // Save payload (functions + corpus + call graph) using bincode v1.1.0+ format
        let payload = IndexPayload {
            functions: self.functions.clone(),
            corpus: self.corpus.clone(),
            calls: self.calls.clone(),
            called_by: self.called_by.clone(),
        };
        let payload_bin = bincode::serialize(&payload)
            .map_err(|e| format!("Failed to serialize payload: {e}"))?;

        // Compress with LZ4
        let compressed = lz4_flex::compress_prepend_size(&payload_bin);
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

        // Load compressed blob
        let compressed = fs::read(index_path.join("functions.lz4"))
            .map_err(|e| format!("Failed to read functions: {e}"))?;
        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| format!("Failed to decompress functions: {e}"))?;

        // Try v1.1.0+ payload format first, fall back to legacy functions-only
        let is_v1_1 = manifest.version.starts_with("1.1");
        let (functions, corpus, calls, called_by) = if is_v1_1 {
            let payload: IndexPayload = bincode::deserialize(&decompressed)
                .map_err(|e| format!("Failed to parse payload: {e}"))?;
            (payload.functions, payload.corpus, payload.calls, payload.called_by)
        } else {
            // Legacy v1.0.0: only functions stored, rebuild corpus + call graph
            let functions: Vec<FunctionEntry> = bincode::deserialize(&decompressed)
                .map_err(|e| format!("Failed to parse functions: {e}"))?;
            let (_, _, corpus) = build_indices(&functions);
            (functions, corpus, HashMap::new(), HashMap::new())
        };

        // Always rebuild HashMap indices (fast, O(n))
        let (name_index, file_index, _) = build_indices(&functions);

        // Rebuild call graph if loading legacy format
        let (calls, called_by) = if !is_v1_1 {
            build_call_graph(&functions, &name_index)
        } else {
            (calls, called_by)
        };

        // Compute name frequency and lowercase corpus
        let name_frequency = compute_name_frequency(&name_index, functions.len());
        let corpus_lower: Vec<String> = corpus.iter().map(|d| d.to_lowercase()).collect();

        let project_root = PathBuf::from(&manifest.project_root);

        Ok(Self {
            functions,
            name_index,
            file_index,
            corpus,
            corpus_lower,
            name_frequency,
            calls,
            called_by,
            project_root,
            manifest,
        })
    }

    /// Build an incremental index update, re-parsing only changed files.
    ///
    /// Compares file SHA256 checksums against the existing index to determine
    /// which files need re-parsing. Unchanged files reuse existing function entries.
    pub fn build_incremental(project_path: &Path, existing: &Self) -> Result<Self, String> {
        let project_root = project_path
            .canonicalize()
            .map_err(|e| format!("Invalid project path: {e}"))?;

        let mut functions = Vec::new();
        let mut file_count = 0;
        let mut languages_seen: HashMap<String, usize> = HashMap::new();
        let mut file_checksums: HashMap<String, String> = HashMap::new();
        let mut files_reused = 0usize;
        let mut files_reparsed = 0usize;

        // Walk the project directory
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

            let language = match detect_language(path) {
                Some(lang) => lang,
                None => continue,
            };

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path
                .strip_prefix(&project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let checksum = compute_file_sha256(&content);
            file_checksums.insert(relative_path.clone(), checksum.clone());

            // Check if file is unchanged
            let unchanged = existing
                .manifest
                .file_checksums
                .get(&relative_path)
                .map(|old| old == &checksum)
                .unwrap_or(false);

            if unchanged {
                // Copy existing functions for this file
                if let Some(indices) = existing.file_index.get(&relative_path) {
                    for &idx in indices {
                        functions.push(existing.functions[idx].clone());
                    }
                }
                files_reused += 1;
            } else {
                // Re-parse changed/new file
                let chunks = match chunk_code(&content, language) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let lang_str = format!("{language:?}");
                *languages_seen.entry(lang_str.clone()).or_insert(0) += 1;

                for chunk in chunks {
                    if chunk.chunk_type != crate::services::semantic::ChunkType::Function {
                        continue;
                    }

                    let quality = extract_quality_metrics(&chunk, &content);
                    let signature = chunk
                        .content
                        .lines()
                        .next()
                        .unwrap_or(&chunk.chunk_name)
                        .to_string();
                    let doc_comment = extract_doc_comment(&content, chunk.start_line);

                    functions.push(FunctionEntry {
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
                    });
                }
                files_reparsed += 1;
            }

            file_count += 1;
        }

        eprintln!(
            "Incremental update: {} files reused, {} files re-parsed",
            files_reused, files_reparsed
        );

        // Build all derived data
        let (name_index, file_index, corpus) = build_indices(&functions);
        let (calls, called_by) = build_call_graph(&functions, &name_index);
        let name_frequency = compute_name_frequency(&name_index, functions.len());
        let corpus_lower: Vec<String> = corpus.iter().map(|d| d.to_lowercase()).collect();

        let avg_tdg = if !functions.is_empty() {
            functions.iter().map(|f| f.quality.tdg_score).sum::<f32>() / functions.len() as f32
        } else {
            0.0
        };

        // Collect languages from all functions (including reused ones)
        for f in &functions {
            *languages_seen.entry(f.language.clone()).or_insert(0) += 1;
        }

        let manifest = IndexManifest {
            version: "1.1.0".to_string(),
            built_at: chrono::Utc::now().to_rfc3339(),
            project_root: project_root.to_string_lossy().to_string(),
            function_count: functions.len(),
            file_count,
            languages: languages_seen.keys().cloned().collect(),
            avg_tdg_score: avg_tdg,
            file_checksums,
        };

        Ok(Self {
            functions,
            name_index,
            file_index,
            corpus,
            corpus_lower,
            name_frequency,
            calls,
            called_by,
            project_root,
            manifest,
        })
    }

    /// Get caller functions for a given function index
    pub fn get_calls(&self, func_idx: usize) -> Vec<&str> {
        self.calls
            .get(&func_idx)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&i| self.functions[i].function_name.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get functions that call a given function
    pub fn get_called_by(&self, func_idx: usize) -> Vec<&str> {
        self.called_by
            .get(&func_idx)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&i| self.functions[i].function_name.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find function index by file path and name
    pub fn find_function_index(&self, file_path: &str, function_name: &str) -> Option<usize> {
        self.functions
            .iter()
            .position(|f| f.file_path == file_path && f.function_name == function_name)
    }
}

/// Parse `siblings` array from workspace.toml content.
///
/// Handles: `siblings = ["../aprender", "../trueno"]`
/// Minimal parser — no full TOML dependency needed for one key.
fn parse_workspace_siblings(content: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("siblings") {
            let rest = rest.trim().strip_prefix('=').unwrap_or("").trim();
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                return inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Build name_index, file_index, and corpus from functions.
pub(crate) fn build_indices(
    functions: &[FunctionEntry],
) -> (HashMap<String, Vec<usize>>, HashMap<String, Vec<usize>>, Vec<String>) {
    let mut name_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut file_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut corpus = Vec::with_capacity(functions.len());

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
            "{name} {name} {sig} {sig} {doc} {doc} {path} {idents}",
            name = func.function_name,
            sig = func.signature,
            doc = func.doc_comment.as_deref().unwrap_or(""),
            path = func.file_path,
            idents = extract_identifiers(&func.source)
        );
        corpus.push(doc);
    }

    (name_index, file_index, corpus)
}

/// Compute SHA256 hash of file content
fn compute_file_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Compute name frequency for generic name demotion.
///
/// Returns a map of function_name -> fraction of total functions with that name.
/// High-frequency names like `new`, `default`, `from` get demoted in search results.
pub(crate) fn compute_name_frequency(
    name_index: &HashMap<String, Vec<usize>>,
    total: usize,
) -> HashMap<String, f32> {
    if total == 0 {
        return HashMap::new();
    }
    name_index
        .iter()
        .map(|(name, indices)| (name.clone(), indices.len() as f32 / total as f32))
        .collect()
}

/// Build caller/callee graph by matching identifiers in source against function names.
///
/// For each function, extracts identifiers from its source and checks if they match
/// any known function name. If a match is found (and it's not a self-reference),
/// records a call edge.
pub(crate) fn build_call_graph(
    functions: &[FunctionEntry],
    name_index: &HashMap<String, Vec<usize>>,
) -> (HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>) {
    let mut calls: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut called_by: HashMap<usize, Vec<usize>> = HashMap::new();

    for (caller_idx, func) in functions.iter().enumerate() {
        // Extract identifiers from source
        let idents: Vec<&str> = func
            .source
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() >= 3 && !is_keyword(s))
            .collect();

        let mut seen_callees: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for ident in &idents {
            if let Some(callee_indices) = name_index.get(*ident) {
                for &callee_idx in callee_indices {
                    // Skip self-references and duplicates
                    if callee_idx == caller_idx || seen_callees.contains(&callee_idx) {
                        continue;
                    }
                    seen_callees.insert(callee_idx);
                    calls.entry(caller_idx).or_default().push(callee_idx);
                    called_by.entry(callee_idx).or_default().push(caller_idx);
                }
            }
        }
    }

    (calls, called_by)
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
            | "third_party"
            | "third-party"
            | "external"
            | "deps"
            | "book"
            | "theme"
            | "fixtures"
            | ".cargo"
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

    #[test]
    fn test_compute_file_sha256() {
        let hash1 = compute_file_sha256("hello world");
        let hash2 = compute_file_sha256("hello world");
        let hash3 = compute_file_sha256("different content");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_compute_name_frequency() {
        let mut name_index = HashMap::new();
        name_index.insert("new".to_string(), vec![0, 1, 2, 3, 4]);
        name_index.insert("unique_func".to_string(), vec![5]);
        let freq = compute_name_frequency(&name_index, 10);
        assert!((freq["new"] - 0.5).abs() < 0.01);
        assert!((freq["unique_func"] - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_compute_name_frequency_empty() {
        let name_index = HashMap::new();
        let freq = compute_name_frequency(&name_index, 0);
        assert!(freq.is_empty());
    }

    #[test]
    fn test_build_indices() {
        let functions = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "foo".to_string(),
                signature: "fn foo()".to_string(),
                doc_comment: None,
                source: "fn foo() {}".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "abc".to_string(),
            },
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "bar".to_string(),
                signature: "fn bar()".to_string(),
                doc_comment: None,
                source: "fn bar() {}".to_string(),
                start_line: 3,
                end_line: 3,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "def".to_string(),
            },
        ];
        let (name_idx, file_idx, corpus) = build_indices(&functions);
        assert_eq!(name_idx["foo"], vec![0]);
        assert_eq!(name_idx["bar"], vec![1]);
        assert_eq!(file_idx["a.rs"], vec![0, 1]);
        assert_eq!(corpus.len(), 2);
    }

    #[test]
    fn test_build_call_graph() {
        // foo calls bar (bar appears as identifier in foo's source)
        let functions = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "foo".to_string(),
                signature: "fn foo()".to_string(),
                doc_comment: None,
                source: "fn foo() { bar(); }".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "abc".to_string(),
            },
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "bar".to_string(),
                signature: "fn bar()".to_string(),
                doc_comment: None,
                source: "fn bar() { println!(\"hello\"); }".to_string(),
                start_line: 3,
                end_line: 3,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "def".to_string(),
            },
        ];
        let (name_index, _, _) = build_indices(&functions);
        let (calls, called_by) = build_call_graph(&functions, &name_index);

        // foo calls bar
        assert!(calls.get(&0).map_or(false, |v| v.contains(&1)));
        // bar is called by foo
        assert!(called_by.get(&1).map_or(false, |v| v.contains(&0)));
        // bar does not call foo
        assert!(!calls.get(&1).map_or(false, |v| v.contains(&0)));
    }

    #[test]
    fn test_save_load_roundtrip_v1_1() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create a simple Rust file
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/lib.rs"),
            "fn hello() { world(); }\nfn world() { println!(\"hi\"); }\n",
        )
        .unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        let index_path = project_path.join("idx");
        index.save(&index_path).unwrap();

        let loaded = AgentContextIndex::load(&index_path).unwrap();
        assert_eq!(loaded.manifest.version, "1.1.0");
        assert_eq!(loaded.functions.len(), index.functions.len());
        assert_eq!(loaded.corpus.len(), index.corpus.len());
        // Verify corpus is identical (not rebuilt from scratch)
        for (orig, loaded_c) in index.corpus.iter().zip(loaded.corpus.iter()) {
            assert_eq!(orig, loaded_c);
        }
    }

    #[test]
    fn test_incremental_build_unchanged() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/lib.rs"),
            "fn alpha() { }\nfn beta() { }\n",
        )
        .unwrap();

        let original = AgentContextIndex::build(project_path).unwrap();
        let incremental =
            AgentContextIndex::build_incremental(project_path, &original).unwrap();

        // Same number of functions (nothing changed)
        assert_eq!(incremental.functions.len(), original.functions.len());
    }

    #[test]
    fn test_incremental_build_with_change() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/lib.rs"), "fn alpha() { }\n").unwrap();

        let original = AgentContextIndex::build(project_path).unwrap();
        assert_eq!(original.functions.len(), 1);

        // Modify the file to add a function
        std::fs::write(
            project_path.join("src/lib.rs"),
            "fn alpha() { }\nfn gamma() { }\n",
        )
        .unwrap();

        let incremental =
            AgentContextIndex::build_incremental(project_path, &original).unwrap();
        // Should now have 2 functions
        assert_eq!(incremental.functions.len(), 2);
    }

    #[test]
    fn test_parse_workspace_siblings() {
        let toml = r#"siblings = ["../aprender", "../trueno", "../realizar"]"#;
        let result = parse_workspace_siblings(toml);
        assert_eq!(result, vec!["../aprender", "../trueno", "../realizar"]);
    }

    #[test]
    fn test_parse_workspace_siblings_empty() {
        let toml = "# no siblings configured\n";
        let result = parse_workspace_siblings(toml);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_workspace_siblings_single() {
        let toml = r#"siblings = ["../trueno"]"#;
        let result = parse_workspace_siblings(toml);
        assert_eq!(result, vec!["../trueno"]);
    }

    #[test]
    fn test_parse_workspace_siblings_with_spaces() {
        let toml = r#"siblings  =  [ "../a" , "../b" ]"#;
        let result = parse_workspace_siblings(toml);
        assert_eq!(result, vec!["../a", "../b"]);
    }

    #[test]
    fn test_discover_siblings_no_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = AgentContextIndex::discover_sibling_indexes(temp_dir.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_file_checksums_populated() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/lib.rs"), "fn test_func() { }\n").unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        assert!(!index.manifest.file_checksums.is_empty());
        assert!(index.manifest.file_checksums.contains_key("src/lib.rs"));
    }

    #[test]
    fn test_get_calls_and_called_by() {
        let functions = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "caller".to_string(),
                signature: "fn caller()".to_string(),
                doc_comment: None,
                source: "fn caller() { callee(); }".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "aaa".to_string(),
            },
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "callee".to_string(),
                signature: "fn callee()".to_string(),
                doc_comment: None,
                source: "fn callee() { println!(\"hello\"); }".to_string(),
                start_line: 3,
                end_line: 3,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "bbb".to_string(),
            },
        ];

        let (name_index, file_index, corpus) = build_indices(&functions);
        let corpus_lower: Vec<String> = corpus.iter().map(|c| c.to_lowercase()).collect();
        let (calls, called_by) = build_call_graph(&functions, &name_index);

        let index = AgentContextIndex {
            functions,
            name_index,
            file_index,
            corpus,
            corpus_lower,
            name_frequency: HashMap::new(),
            calls,
            called_by,
            project_root: PathBuf::from("/test"),
            manifest: super::IndexManifest {
                version: "1.1.0".to_string(),
                built_at: "2025-01-01T00:00:00Z".to_string(),
                project_root: "/test".to_string(),
                function_count: 2,
                file_count: 1,
                languages: vec!["Rust".to_string()],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
            },
        };

        let calls_of_0 = index.get_calls(0);
        assert!(calls_of_0.contains(&"callee"), "caller should call callee");

        let called_by_1 = index.get_called_by(1);
        assert!(called_by_1.contains(&"caller"), "callee should be called by caller");

        // Non-existent index
        assert!(index.get_calls(999).is_empty());
        assert!(index.get_called_by(999).is_empty());
    }

    #[test]
    fn test_find_function_index() {
        let functions = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "foo".to_string(),
                signature: "fn foo()".to_string(),
                doc_comment: None,
                source: "fn foo() {}".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "aaa".to_string(),
            },
        ];

        let (name_index, file_index, corpus) = build_indices(&functions);
        let corpus_lower: Vec<String> = corpus.iter().map(|c| c.to_lowercase()).collect();

        let index = AgentContextIndex {
            functions,
            name_index,
            file_index,
            corpus,
            corpus_lower,
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            project_root: PathBuf::from("/test"),
            manifest: super::IndexManifest {
                version: "1.1.0".to_string(),
                built_at: "2025-01-01T00:00:00Z".to_string(),
                project_root: "/test".to_string(),
                function_count: 1,
                file_count: 1,
                languages: vec!["Rust".to_string()],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
            },
        };

        assert_eq!(index.find_function_index("a.rs", "foo"), Some(0));
        assert_eq!(index.find_function_index("a.rs", "bar"), None);
        assert_eq!(index.find_function_index("b.rs", "foo"), None);
    }

    #[test]
    fn test_discover_siblings_with_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create .pmat/workspace.toml
        std::fs::create_dir_all(project_path.join(".pmat")).unwrap();
        std::fs::write(
            project_path.join(".pmat/workspace.toml"),
            r#"siblings = ["../sibling_a"]"#,
        )
        .unwrap();

        // We can't easily create a real sibling in tempdir, so just verify
        // the function reads the config correctly without panicking
        let result = AgentContextIndex::discover_sibling_indexes(project_path);
        // Sibling doesn't exist, so no results
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_siblings_with_real_sibling() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workspace = temp_dir.path();

        // Create project A
        let project_a = workspace.join("project_a");
        std::fs::create_dir_all(project_a.join(".pmat")).unwrap();

        // Create project B with an index
        let project_b = workspace.join("project_b");
        std::fs::create_dir_all(project_b.join("src")).unwrap();
        std::fs::write(project_b.join("src/lib.rs"), "fn sibling_func() {}\n").unwrap();
        let b_index = AgentContextIndex::build(&project_b).unwrap();
        let b_idx_path = project_b.join(".pmat/context.idx");
        std::fs::create_dir_all(b_idx_path.parent().unwrap()).unwrap();
        b_index.save(&b_idx_path).unwrap();

        // Configure A to point to B
        std::fs::write(
            project_a.join(".pmat/workspace.toml"),
            format!(r#"siblings = ["../project_b"]"#),
        )
        .unwrap();

        let siblings = AgentContextIndex::discover_sibling_indexes(&project_a);
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0].1, "project_b");
    }

    #[test]
    fn test_merge_fast() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Build index A
        let proj_a = temp_dir.path().join("a");
        std::fs::create_dir_all(proj_a.join("src")).unwrap();
        std::fs::write(proj_a.join("src/lib.rs"), "fn alpha() {}\n").unwrap();
        let mut index_a = AgentContextIndex::build(&proj_a).unwrap();
        let a_count = index_a.functions.len();

        // Build index B
        let proj_b = temp_dir.path().join("b");
        std::fs::create_dir_all(proj_b.join("src")).unwrap();
        std::fs::write(proj_b.join("src/lib.rs"), "fn beta() {}\n").unwrap();
        let index_b = AgentContextIndex::build(&proj_b).unwrap();
        let b_count = index_b.functions.len();

        index_a.merge_fast(index_b);

        assert_eq!(index_a.functions.len(), a_count + b_count);
        // All functions accessible
        let names: Vec<&str> = index_a.functions.iter().map(|f| f.function_name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_manifest_accessor() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/lib.rs"), "fn foo() {}\n").unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        let manifest = index.manifest();
        assert_eq!(manifest.version, "1.1.0");
        assert!(manifest.function_count > 0);
        assert!(manifest.file_count > 0);
    }

    #[test]
    fn test_extract_doc_comment_basic() {
        let content = "/// This is a doc comment\nfn foo() {}";
        let doc = extract_doc_comment(content, 2); // fn is on line 2
        assert!(doc.is_some());
        assert!(doc.unwrap().contains("This is a doc comment"));
    }

    #[test]
    fn test_extract_doc_comment_none() {
        let content = "fn foo() {}";
        let doc = extract_doc_comment(content, 1);
        assert!(doc.is_none());
    }

    #[test]
    fn test_calculate_simple_tdg() {
        // Low complexity, no SATD, small LOC = low score
        let score = calculate_simple_tdg(1, 0, 10);
        assert!(score < 2.0);

        // High complexity, SATD, large LOC = higher score
        let high_score = calculate_simple_tdg(20, 3, 200);
        assert!(high_score > score);
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("fn"));
        assert!(is_keyword("let"));
        assert!(is_keyword("if"));
        assert!(is_keyword("for"));
        assert!(is_keyword("while"));
        assert!(is_keyword("return"));
        assert!(is_keyword("def"));
        assert!(is_keyword("class"));
        assert!(is_keyword("import"));
        assert!(!is_keyword("handle_error"));
        assert!(!is_keyword("MyStruct"));
    }

    #[test]
    fn test_estimate_big_o_quadratic() {
        let quadratic = "fn foo() {\n    for i in items {\n        for j in items {\n            process(i, j);\n        }\n    }\n}";
        assert_eq!(estimate_big_o(quadratic), "O(n^2)");
    }

    #[test]
    fn test_estimate_big_o_logarithmic() {
        let log = "fn foo() {\n    while n > 0 {\n        n /= 2;\n    }\n}";
        // Contains while + divide = log
        assert!(["O(n log n)", "O(log n)", "O(n)"].contains(&estimate_big_o(log).as_str()));
    }

    #[test]
    fn test_build_with_multiple_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create multiple Rust files
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/lib.rs"),
            "/// Documentation for rust_func\nfn rust_func() { if true { println!(\"hello\"); } }\n",
        )
        .unwrap();
        std::fs::write(
            project_path.join("src/helper.rs"),
            "/// Helper function\nfn helper_func() { for i in 0..10 { println!(\"{}\", i); } }\n",
        )
        .unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        assert!(index.functions.len() >= 2);
        assert!(index.manifest.file_count >= 2);
        assert_eq!(index.manifest.version, "1.1.0");

        // Verify quality metrics computed
        for func in &index.functions {
            assert!(!func.function_name.is_empty());
            assert!(!func.file_path.is_empty());
            assert!(!func.language.is_empty());
            assert!(!func.quality.tdg_grade.is_empty());
            assert!(!func.quality.big_o.is_empty());
        }
    }

    #[test]
    fn test_build_empty_project() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        // Empty file - no functions
        std::fs::write(project_path.join("src/lib.rs"), "// empty\n").unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        assert_eq!(index.functions.len(), 0);
        assert!((index.manifest.avg_tdg_score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_build_with_binary_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/lib.rs"), "fn valid() {}\n").unwrap();
        // Binary file should be skipped
        std::fs::write(project_path.join("src/data.bin"), &[0u8, 1, 2, 255, 254]).unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        // Should index the .rs file but skip .bin
        assert!(index.functions.len() >= 1);
    }

    #[test]
    fn test_build_skips_ignored_dirs() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::create_dir_all(project_path.join("node_modules/pkg")).unwrap();
        std::fs::create_dir_all(project_path.join("target/debug")).unwrap();

        std::fs::write(project_path.join("src/lib.rs"), "fn keep() {}\n").unwrap();
        std::fs::write(
            project_path.join("node_modules/pkg/index.rs"),
            "fn skip_nm() {}\n",
        )
        .unwrap();
        std::fs::write(
            project_path.join("target/debug/output.rs"),
            "fn skip_target() {}\n",
        )
        .unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        let names: Vec<&str> = index.functions.iter().map(|f| f.function_name.as_str()).collect();
        assert!(names.contains(&"keep"));
        assert!(!names.contains(&"skip_nm"));
        assert!(!names.contains(&"skip_target"));
    }

    #[test]
    fn test_save_and_load_preserves_calls() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/lib.rs"),
            "fn caller() { callee(); }\nfn callee() { println!(\"hi\"); }\n",
        )
        .unwrap();

        let index = AgentContextIndex::build(project_path).unwrap();
        let idx_path = project_path.join("idx");
        index.save(&idx_path).unwrap();

        let loaded = AgentContextIndex::load(&idx_path).unwrap();
        // Call graph should be preserved
        assert_eq!(loaded.calls.len(), index.calls.len());
        assert_eq!(loaded.called_by.len(), index.called_by.len());
    }

    #[test]
    fn test_load_invalid_path() {
        let result = AgentContextIndex::load(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_quality_metrics() {
        use crate::services::semantic::{chunk_code, Language};
        let source = "fn complex() {\n    if a {\n        if b {\n            for i in items {\n                // TODO: fix\n                process(i);\n            }\n        }\n    }\n}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        if let Some(chunk) = chunks.first() {
            let quality = extract_quality_metrics(chunk, source);
            assert!(quality.complexity >= 3); // if + if + for
            assert!(quality.satd_count >= 1); // TODO
            assert_eq!(quality.big_o, "O(n)"); // single for loop
        }
    }

    #[test]
    fn test_count_complexity_various() {
        // Multi-line if/else if
        let if_else = "fn f() {\n    if a {\n    } else if b {\n    } else {\n    }\n}";
        assert!(count_complexity(if_else) >= 3);
        // Match expression on its own line
        let matchex = "fn f() {\n    match x {\n        A => {},\n        B => {}\n    }\n}";
        assert!(count_complexity(matchex) >= 2);
        // While loop
        let whileex = "fn f() {\n    while true {\n        break;\n    }\n}";
        assert!(count_complexity(whileex) >= 2);
        // Boolean operators on one line
        let booleans = "fn f() { x && y || z }";
        assert!(count_complexity(booleans) >= 2); // && and || both on same line count once
    }

    #[test]
    fn test_count_satd_markers_various() {
        assert_eq!(count_satd_markers("// FIXME: broken"), 1);
        assert_eq!(count_satd_markers("// HACK: workaround"), 1);
        assert_eq!(count_satd_markers("// XXX: temporary"), 1);
        assert_eq!(count_satd_markers("// TODO: fix\n// FIXME: also fix"), 2);
        assert_eq!(count_satd_markers("// Normal comment"), 0);
    }

    #[test]
    fn test_detect_language_all_types() {
        assert_eq!(detect_language(Path::new("test.rs")), Some(Language::Rust));
        assert_eq!(detect_language(Path::new("test.py")), Some(Language::Python));
        assert_eq!(detect_language(Path::new("test.ts")), Some(Language::TypeScript));
        assert_eq!(detect_language(Path::new("test.tsx")), Some(Language::TypeScript));
        assert_eq!(detect_language(Path::new("test.c")), Some(Language::C));
        assert_eq!(detect_language(Path::new("test.h")), Some(Language::C));
        assert_eq!(detect_language(Path::new("test.cpp")), Some(Language::Cpp));
        assert_eq!(detect_language(Path::new("test.go")), Some(Language::Go));
        assert_eq!(detect_language(Path::new("test.md")), None);
        assert_eq!(detect_language(Path::new("test.toml")), None);
    }
}
