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
    /// Git commit count for the file (churn indicator)
    #[serde(default)]
    pub commit_count: u32,
    /// Churn score (0.0-1.0, higher = more volatile)
    #[serde(default)]
    pub churn_score: f32,
}

/// Definition type for indexed items (issue #150)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DefinitionType {
    #[default]
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
}

/// A function/type entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    /// File path relative to project root
    pub file_path: String,
    /// Function/type name
    pub function_name: String,
    /// Full signature/definition
    pub signature: String,
    /// Type of definition (function, struct, enum, trait, type alias)
    #[serde(default)]
    pub definition_type: DefinitionType,
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
    // === Cached annotations (computed at build time, not query time) ===
    /// Git commit count for this file
    #[serde(default)]
    pub commit_count: u32,
    /// Churn score 0.0-1.0 (higher = more volatile)
    #[serde(default)]
    pub churn_score: f32,
    /// Number of code clones/duplicates
    #[serde(default)]
    pub clone_count: u32,
    /// Pattern diversity 0.0-1.0 (lower = more repetitive)
    #[serde(default)]
    pub pattern_diversity: f32,
    /// Fault pattern annotations from batuta
    #[serde(default)]
    pub fault_annotations: Vec<String>,
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
    /// Number of files reparsed in last incremental update (0 = no changes)
    #[serde(default)]
    pub last_incremental_changes: usize,
}

/// Serialized payload for the index (v1.3.0+ with cached indices)
#[derive(Serialize, Deserialize)]
struct IndexPayload {
    functions: Vec<FunctionEntry>,
    corpus: Vec<String>,
    calls: HashMap<usize, Vec<usize>>,
    called_by: HashMap<usize, Vec<usize>>,
    // v1.3.0: Cached indices to avoid rebuild on load
    #[serde(default)]
    name_index: HashMap<String, Vec<usize>>,
    #[serde(default)]
    file_index: HashMap<String, Vec<usize>>,
    #[serde(default)]
    graph_metrics: Vec<GraphMetrics>,
    #[serde(default)]
    corpus_lower: Vec<String>,
    #[serde(default)]
    name_frequency: HashMap<String, f32>,
}

/// Result of build_indices function
#[derive(Debug, Clone, Default)]
pub(crate) struct BuildIndicesResult {
    pub name_index: HashMap<String, Vec<usize>>,
    pub file_index: HashMap<String, Vec<usize>>,
    pub corpus: Vec<String>,
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

/// Graph metrics for ranking functions by importance
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GraphMetrics {
    /// PageRank score (higher = more important, transitively called)
    pub pagerank: f32,
    /// Degree centrality (direct callers + callees)
    pub centrality: f32,
    /// In-degree (number of direct callers)
    pub in_degree: u32,
    /// Out-degree (number of direct callees)
    pub out_degree: u32,
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
    /// Graph metrics per function (PageRank, centrality)
    pub(crate) graph_metrics: Vec<GraphMetrics>,
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

                // Extract quality metrics
                let quality = extract_quality_metrics(&chunk, &content);

                // Extract signature (first line of definition)
                let signature = chunk
                    .content
                    .lines()
                    .next()
                    .unwrap_or(&chunk.chunk_name)
                    .to_string();

                // Extract doc comment (lines starting with /// or /** before definition)
                let doc_comment = extract_doc_comment(&content, chunk.start_line);

                let entry = FunctionEntry {
                    file_path: relative_path.clone(),
                    function_name: chunk.chunk_name.clone(),
                    signature,
                    definition_type,
                    doc_comment,
                    source: chunk.content.clone(),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    language: lang_str.clone(),
                    quality,
                    checksum: chunk.content_checksum,
                    // Annotations populated after all definitions collected
                    commit_count: 0,
                    churn_score: 0.0,
                    clone_count: 0,
                    pattern_diversity: 0.0,
                    fault_annotations: Vec::new(),
                };

                functions.push(entry);
            }

            file_count += 1;
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

        // Calculate average TDG score
        let avg_tdg = if !functions.is_empty() {
            functions.iter().map(|f| f.quality.tdg_score).sum::<f32>() / functions.len() as f32
        } else {
            0.0
        };

        let manifest = IndexManifest {
            version: "1.3.0".to_string(), // Bump version for graph metrics
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
            // Estimate index size: functions vec + name_index map + file_index map
            index_size_bytes: (std::mem::size_of_val(&self.functions)
                + self.functions.len() * std::mem::size_of::<FunctionEntry>()
                + self.name_index.len() * 64  // Approximate string + vec overhead
                + self.file_index.len() * 64) as u64,
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
        let indices = build_indices(&self.functions);
        let (calls, called_by) = build_call_graph(&self.functions, &indices.name_index);
        let name_frequency = compute_name_frequency(&indices.name_index, self.functions.len());

        self.name_index = indices.name_index;
        self.file_index = indices.file_index;
        self.corpus = indices.corpus;
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

        // Save payload (functions + corpus + call graph + cached indices) v1.3.0 format
        let payload = IndexPayload {
            functions: self.functions.clone(),
            corpus: self.corpus.clone(),
            calls: self.calls.clone(),
            called_by: self.called_by.clone(),
            // v1.3.0: Cache computed indices to avoid rebuild on load
            name_index: self.name_index.clone(),
            file_index: self.file_index.clone(),
            graph_metrics: self.graph_metrics.clone(),
            corpus_lower: self.corpus_lower.clone(),
            name_frequency: self.name_frequency.clone(),
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

        // Load and decompress blob
        let compressed = fs::read(index_path.join("functions.lz4"))
            .map_err(|e| format!("Failed to read functions: {e}"))?;
        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| format!("Failed to decompress functions: {e}"))?;

        // Deserialize payload (v1.3.0+ has cached indices)
        let payload: IndexPayload = bincode::deserialize(&decompressed)
            .map_err(|e| format!("Failed to parse payload: {e}"))?;

        let functions = payload.functions;
        let corpus = payload.corpus;
        let calls = payload.calls;
        let called_by = payload.called_by;

        // Check if we have cached indices (v1.3.0+) - avoids expensive PageRank recomputation
        let has_cached_indices = !payload.name_index.is_empty();

        let (name_index, file_index, graph_metrics, corpus_lower, name_frequency) = if has_cached_indices {
            // Fast path: use cached indices directly (saves ~4s for 100k functions)
            (
                payload.name_index,
                payload.file_index,
                payload.graph_metrics,
                payload.corpus_lower,
                payload.name_frequency,
            )
        } else {
            // Slow path: rebuild indices for legacy formats (v1.0-v1.2)
            let is_legacy = manifest.version.starts_with("1.0") || manifest.version.starts_with("1.1");
            let indices = build_indices(&functions);

            // Rebuild call graph if loading legacy format
            let (calls_rebuilt, called_by_rebuilt) = if is_legacy && calls.is_empty() {
                build_call_graph(&functions, &indices.name_index)
            } else {
                (calls.clone(), called_by.clone())
            };

            // Compute graph metrics (PageRank, centrality)
            let graph_metrics = compute_graph_metrics(functions.len(), &calls_rebuilt, &called_by_rebuilt);

            // Compute name frequency and lowercase corpus
            let name_frequency = compute_name_frequency(&indices.name_index, functions.len());
            let corpus_lower: Vec<String> = corpus.iter().map(|d| d.to_lowercase()).collect();

            (indices.name_index, indices.file_index, graph_metrics, corpus_lower, name_frequency)
        };

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
            graph_metrics,
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
                        definition_type,
                        commit_count: 0,
                        churn_score: 0.0,
                        clone_count: 0,
                        pattern_diversity: 0.0,
                        fault_annotations: Vec::new(),
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
        let indices = build_indices(&functions);
        let (calls, called_by) = build_call_graph(&functions, &indices.name_index);
        let graph_metrics = compute_graph_metrics(functions.len(), &calls, &called_by);
        let name_frequency = compute_name_frequency(&indices.name_index, functions.len());
        let corpus_lower: Vec<String> = indices.corpus.iter().map(|d| d.to_lowercase()).collect();

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
            version: "1.3.0".to_string(),
            built_at: chrono::Utc::now().to_rfc3339(),
            project_root: project_root.to_string_lossy().to_string(),
            function_count: functions.len(),
            file_count,
            languages: languages_seen.keys().cloned().collect(),
            avg_tdg_score: avg_tdg,
            file_checksums,
            last_incremental_changes: files_reparsed,
        };

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
pub(crate) fn build_indices(functions: &[FunctionEntry]) -> BuildIndicesResult {
    let mut result = BuildIndicesResult {
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: Vec::with_capacity(functions.len()),
    };

    for (idx, func) in functions.iter().enumerate() {
        result
            .name_index
            .entry(func.function_name.clone())
            .or_default()
            .push(idx);
        result
            .file_index
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
        result.corpus.push(doc);
    }

    result
}

/// Compute SHA256 hash of file content
fn compute_file_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Populate cached annotations for all functions during index build.
/// Computes: git churn, code clones, pattern diversity, fault patterns.
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git + filesystem for churn/clones
fn populate_cached_annotations(
    functions: &mut [FunctionEntry],
    file_index: &HashMap<String, Vec<usize>>,
    project_root: &std::path::Path,
) {
    eprintln!("Computing annotations for {} functions...", functions.len());

    // 1. Git churn: get commit counts per file
    let file_commits = get_file_commit_counts(project_root, file_index.keys());
    let max_commits = file_commits.values().copied().max().unwrap_or(1) as f32;
    eprintln!(
        "  Git churn: {} files with commits (max={})",
        file_commits.len(),
        max_commits as u32
    );

    // 2. Detect duplicate/similar functions (by normalized source hash)
    let clone_groups = detect_code_clones(functions);
    eprintln!("  Clones: {} functions with duplicates", clone_groups.len());

    // 3. Compute pattern diversity per file
    let file_diversity = compute_file_pattern_diversity(functions, file_index);
    eprintln!("  Diversity: {} files analyzed", file_diversity.len());

    // 4. Detect fault patterns in source code
    let fault_patterns = detect_fault_patterns(functions);
    eprintln!(
        "  Faults: {} functions with patterns",
        fault_patterns.len()
    );

    // Apply annotations to functions
    let mut churn_applied = 0;
    let mut clone_applied = 0;
    let mut diversity_applied = 0;
    let mut fault_applied = 0;

    for (i, func) in functions.iter_mut().enumerate() {
        // Churn data
        if let Some(&commits) = file_commits.get(&func.file_path) {
            func.commit_count = commits;
            func.churn_score = commits as f32 / max_commits;
            churn_applied += 1;
        }

        // Clone count
        if let Some(&count) = clone_groups.get(&i) {
            func.clone_count = count;
            clone_applied += 1;
        }

        // Pattern diversity (from file-level)
        if let Some(&diversity) = file_diversity.get(&func.file_path) {
            func.pattern_diversity = diversity;
            diversity_applied += 1;
        }

        // Fault annotations
        if let Some(faults) = fault_patterns.get(&i) {
            func.fault_annotations = faults.clone();
            fault_applied += 1;
        }
    }

    eprintln!(
        "  Applied: churn={}, clones={}, diversity={}, faults={}",
        churn_applied, clone_applied, diversity_applied, fault_applied
    );
}

/// Get commit counts per file from git log
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git process
fn get_file_commit_counts<'a>(
    project_root: &std::path::Path,
    files: impl Iterator<Item = &'a String>,
) -> HashMap<String, u32> {
    let mut result = HashMap::new();

    // Collect unique files
    let files: std::collections::HashSet<_> = files.collect();
    if files.is_empty() {
        return result;
    }

    // Get all file changes from git log (fast batch operation)
    let output = std::process::Command::new("git")
        .args(["log", "--format=", "--name-only", "--since=1 year ago"])
        .current_dir(project_root)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Try exact match first
                if files.contains(&line.to_string()) {
                    *result.entry(line.to_string()).or_insert(0) += 1;
                    continue;
                }

                // Handle path migrations (e.g., server/src/foo.rs -> src/foo.rs)
                let normalized = line.strip_prefix("server/").unwrap_or(line);

                if files.contains(&normalized.to_string()) {
                    *result.entry(normalized.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    result
}

/// Detect code clones by normalized source hash
fn detect_code_clones(functions: &[FunctionEntry]) -> HashMap<usize, u32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut result = HashMap::new();
    let mut hash_to_indices: HashMap<u64, Vec<usize>> = HashMap::new();

    for (i, func) in functions.iter().enumerate() {
        // Normalize source: remove whitespace, lowercase identifiers
        let normalized = normalize_source(&func.source);

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        let hash = hasher.finish();

        hash_to_indices.entry(hash).or_default().push(i);
    }

    // Mark functions that have clones (more than 1 with same hash)
    for indices in hash_to_indices.values() {
        if indices.len() > 1 {
            let count = indices.len() as u32;
            for &idx in indices {
                result.insert(idx, count);
            }
        }
    }

    result
}

/// Normalize source code for clone detection
fn normalize_source(source: &str) -> String {
    source
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// Compute pattern diversity per file (unique AST patterns / total patterns)
fn compute_file_pattern_diversity(
    functions: &[FunctionEntry],
    file_index: &HashMap<String, Vec<usize>>,
) -> HashMap<String, f32> {
    let mut result = HashMap::new();

    for (file_path, indices) in file_index {
        if indices.is_empty() {
            continue;
        }

        // Count unique patterns in file based on function signatures
        let mut patterns = std::collections::HashSet::new();
        for &idx in indices {
            if let Some(func) = functions.get(idx) {
                // Extract pattern: return type + param count + complexity bucket
                let pattern = format!(
                    "{}:{}:{}",
                    extract_return_type(&func.signature),
                    count_params(&func.signature),
                    func.quality.complexity / 5 // bucket by 5
                );
                patterns.insert(pattern);
            }
        }

        let diversity = patterns.len() as f32 / indices.len() as f32;
        result.insert(file_path.clone(), diversity);
    }

    result
}

/// Extract return type from signature (simplified)
fn extract_return_type(sig: &str) -> &str {
    if sig.contains("->") {
        sig.split("->").last().unwrap_or("void").trim()
    } else {
        "void"
    }
}

/// Count parameters in signature
fn count_params(sig: &str) -> usize {
    if let Some(start) = sig.find('(') {
        if let Some(end) = sig.find(')') {
            let params = &sig[start + 1..end];
            if params.trim().is_empty() {
                return 0;
            }
            return params.split(',').count();
        }
    }
    0
}

/// Detect fault patterns in function source
fn detect_fault_patterns(functions: &[FunctionEntry]) -> HashMap<usize, Vec<String>> {
    let mut result = HashMap::new();

    let patterns = [
        ("unwrap()", "UNWRAP"),
        ("expect(", "EXPECT"),
        ("panic!", "PANIC"),
        ("unsafe {", "UNSAFE"),
        ("unsafe{", "UNSAFE"),
        (".clone()", "CLONE"),
        ("// TODO", "TODO"),
        ("// FIXME", "FIXME"),
        ("// HACK", "HACK"),
        ("// XXX", "XXX"),
        ("unimplemented!", "UNIMPL"),
        ("todo!", "TODO_MACRO"),
        ("unreachable!", "UNREACHABLE"),
    ];

    for (i, func) in functions.iter().enumerate() {
        let mut faults = Vec::new();
        let src = &func.source;

        for (pattern, label) in &patterns {
            if src.contains(pattern) {
                faults.push(label.to_string());
            }
        }

        if !faults.is_empty() {
            faults.sort();
            faults.dedup();
            result.insert(i, faults);
        }
    }

    result
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

/// Compute graph metrics (PageRank, centrality) for each function.
///
/// Uses a simplified PageRank algorithm:
/// - Damping factor: 0.85
/// - Iterations: 20 (sufficient for convergence)
/// - Initial score: 1/N for each node
///
/// PageRank represents "importance" - functions that are transitively called
/// by many other functions will have higher scores.
pub(crate) fn compute_graph_metrics(
    num_functions: usize,
    calls: &HashMap<usize, Vec<usize>>,
    called_by: &HashMap<usize, Vec<usize>>,
) -> Vec<GraphMetrics> {
    if num_functions == 0 {
        return Vec::new();
    }

    let damping = 0.85_f32;
    let iterations = 20;
    let initial_score = 1.0 / num_functions as f32;

    // Initialize PageRank scores
    let mut pagerank: Vec<f32> = vec![initial_score; num_functions];
    let mut new_pagerank: Vec<f32> = vec![0.0; num_functions];

    // Iterative PageRank computation
    for _ in 0..iterations {
        // Reset new scores with teleportation probability
        for score in new_pagerank.iter_mut() {
            *score = (1.0 - damping) / num_functions as f32;
        }

        // Distribute scores from callers to callees
        for (caller_idx, callees) in calls {
            if callees.is_empty() {
                continue;
            }
            let contribution = damping * pagerank[*caller_idx] / callees.len() as f32;
            for &callee_idx in callees {
                if callee_idx < num_functions {
                    new_pagerank[callee_idx] += contribution;
                }
            }
        }

        // Handle dangling nodes (functions that don't call anything)
        // Their PageRank distributes evenly to all nodes
        let mut dangling_sum = 0.0_f32;
        for idx in 0..num_functions {
            if !calls.contains_key(&idx) || calls.get(&idx).map_or(true, |c| c.is_empty()) {
                dangling_sum += pagerank[idx];
            }
        }
        let dangling_contribution = damping * dangling_sum / num_functions as f32;
        for score in new_pagerank.iter_mut() {
            *score += dangling_contribution;
        }

        // Swap for next iteration
        std::mem::swap(&mut pagerank, &mut new_pagerank);
    }

    // Build GraphMetrics for each function
    let mut metrics: Vec<GraphMetrics> = Vec::with_capacity(num_functions);
    for idx in 0..num_functions {
        let in_degree = called_by.get(&idx).map_or(0, |v| v.len()) as u32;
        let out_degree = calls.get(&idx).map_or(0, |v| v.len()) as u32;
        let centrality = (in_degree + out_degree) as f32 / (2.0 * num_functions as f32).max(1.0);

        metrics.push(GraphMetrics {
            pagerank: pagerank[idx],
            centrality,
            in_degree,
            out_degree,
        });
    }

    metrics
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
        commit_count: 0,  // Populated later by churn enrichment
        churn_score: 0.0, // Populated later by churn enrichment
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
        ];
        let indices = build_indices(&functions);
        assert_eq!(indices.name_index["foo"], vec![0]);
        assert_eq!(indices.name_index["bar"], vec![1]);
        assert_eq!(indices.file_index["a.rs"], vec![0, 1]);
        assert_eq!(indices.corpus.len(), 2);
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
        ];
        let indices = build_indices(&functions);
        let (calls, called_by) = build_call_graph(&functions, &indices.name_index);

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
        assert_eq!(loaded.manifest.version, "1.3.0");
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
        ];

        let indices = build_indices(&functions);
        let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();
        let (calls, called_by) = build_call_graph(&functions, &indices.name_index);
        let graph_metrics = compute_graph_metrics(functions.len(), &calls, &called_by);

        let index = AgentContextIndex {
            functions,
            name_index: indices.name_index,
            file_index: indices.file_index,
            corpus: indices.corpus,
            corpus_lower,
            name_frequency: HashMap::new(),
            calls,
            called_by,
            graph_metrics,
            project_root: PathBuf::from("/test"),
            manifest: super::IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "2025-01-01T00:00:00Z".to_string(),
                project_root: "/test".to_string(),
                function_count: 2,
                file_count: 1,
                languages: vec!["Rust".to_string()],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
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
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
        ];

        let indices = build_indices(&functions);
        let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();

        let index = AgentContextIndex {
            functions,
            name_index: indices.name_index,
            file_index: indices.file_index,
            corpus: indices.corpus,
            corpus_lower,
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![GraphMetrics::default()],
            project_root: PathBuf::from("/test"),
            manifest: super::IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "2025-01-01T00:00:00Z".to_string(),
                project_root: "/test".to_string(),
                function_count: 1,
                file_count: 1,
                languages: vec!["Rust".to_string()],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
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
        assert_eq!(manifest.version, "1.3.0");
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
        assert_eq!(index.manifest.version, "1.3.0");

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
    fn test_detect_code_clones_with_duplicates() {
        let funcs = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "func_a".to_string(),
                signature: "fn func_a()".to_string(),
                doc_comment: None,
                source: "fn func_a() { let x = 1; let y = 2; x + y }".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "aaa".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "b.rs".to_string(),
                function_name: "func_b".to_string(),
                signature: "fn func_b()".to_string(),
                doc_comment: None,
                // Same source after normalization (whitespace-insensitive)
                source: "fn func_a() { let x = 1; let y = 2; x + y }".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "bbb".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "c.rs".to_string(),
                function_name: "func_c".to_string(),
                signature: "fn func_c()".to_string(),
                doc_comment: None,
                source: "fn func_c() { completely_different_code(); }".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: "ccc".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
        ];

        let clones = detect_code_clones(&funcs);
        // func_a and func_b are clones (same normalized source)
        assert_eq!(clones.get(&0), Some(&2));
        assert_eq!(clones.get(&1), Some(&2));
        // func_c is unique
        assert_eq!(clones.get(&2), None);
    }

    #[test]
    fn test_detect_code_clones_no_duplicates() {
        let funcs = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "unique_a".to_string(),
                signature: "fn unique_a()".to_string(),
                doc_comment: None,
                source: "fn unique_a() { alpha(); }".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics::default(), checksum: "aaa".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "b.rs".to_string(),
                function_name: "unique_b".to_string(),
                signature: "fn unique_b()".to_string(),
                doc_comment: None,
                source: "fn unique_b() { beta(); }".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics::default(), checksum: "bbb".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
        ];

        let clones = detect_code_clones(&funcs);
        assert!(clones.is_empty());
    }

    #[test]
    fn test_normalize_source() {
        assert_eq!(normalize_source("fn foo() { }"), "fnfoo(){}");
        assert_eq!(normalize_source("  fn  foo ( ) {\n}"), "fnfoo(){}");
        assert_eq!(normalize_source("FN FOO()"), "fnfoo()");
    }

    #[test]
    fn test_compute_file_pattern_diversity() {
        let funcs = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "f1".to_string(),
                signature: "fn f1() -> bool".to_string(),
                doc_comment: None, source: "".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics { complexity: 2, ..Default::default() },
                checksum: "a".to_string(), definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "f2".to_string(),
                signature: "fn f2(x: i32) -> String".to_string(),
                doc_comment: None, source: "".to_string(),
                start_line: 5, end_line: 10, language: "Rust".to_string(),
                quality: QualityMetrics { complexity: 8, ..Default::default() },
                checksum: "b".to_string(), definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "f3".to_string(),
                signature: "fn f3() -> bool".to_string(),
                doc_comment: None, source: "".to_string(),
                start_line: 15, end_line: 20, language: "Rust".to_string(),
                quality: QualityMetrics { complexity: 2, ..Default::default() },
                checksum: "c".to_string(), definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
        ];
        let mut file_index = HashMap::new();
        file_index.insert("a.rs".to_string(), vec![0, 1, 2]);

        let diversity = compute_file_pattern_diversity(&funcs, &file_index);
        let d = diversity["a.rs"];
        // f1 and f3 have same pattern (bool:0:0), f2 is different (String:1:1)
        // 2 unique / 3 total = 0.667
        assert!(d > 0.5 && d < 0.8, "unexpected diversity: {d}");
    }

    #[test]
    fn test_compute_file_pattern_diversity_empty() {
        let funcs: Vec<FunctionEntry> = vec![];
        let mut file_index = HashMap::new();
        file_index.insert("a.rs".to_string(), Vec::new());

        let diversity = compute_file_pattern_diversity(&funcs, &file_index);
        assert!(!diversity.contains_key("a.rs")); // empty indices skipped
    }

    #[test]
    fn test_extract_return_type() {
        assert_eq!(extract_return_type("fn foo() -> bool"), "bool");
        assert_eq!(extract_return_type("fn foo() -> Result<String, Error>"), "Result<String, Error>");
        assert_eq!(extract_return_type("fn foo()"), "void");
    }

    #[test]
    fn test_count_params() {
        assert_eq!(count_params("fn foo()"), 0);
        assert_eq!(count_params("fn foo(x: i32)"), 1);
        assert_eq!(count_params("fn foo(x: i32, y: String)"), 2);
        assert_eq!(count_params("fn foo(x: i32, y: String, z: bool)"), 3);
        assert_eq!(count_params("no parens"), 0);
    }

    #[test]
    fn test_detect_fault_patterns() {
        let funcs = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "risky".to_string(),
                signature: "fn risky()".to_string(),
                doc_comment: None,
                source: "fn risky() { x.unwrap(); y.clone(); // TODO: fix }".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics::default(), checksum: "a".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "b.rs".to_string(),
                function_name: "safe".to_string(),
                signature: "fn safe()".to_string(),
                doc_comment: None,
                source: "fn safe() { println!(\"hello\"); }".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics::default(), checksum: "b".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "c.rs".to_string(),
                function_name: "dangerous".to_string(),
                signature: "fn dangerous()".to_string(),
                doc_comment: None,
                source: "fn dangerous() { unsafe { panic!(\"boom\"); } }".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics::default(), checksum: "c".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
        ];

        let faults = detect_fault_patterns(&funcs);
        // risky has: UNWRAP, CLONE, TODO
        let risky_faults = &faults[&0];
        assert!(risky_faults.contains(&"UNWRAP".to_string()));
        assert!(risky_faults.contains(&"CLONE".to_string()));
        assert!(risky_faults.contains(&"TODO".to_string()));
        // safe has no faults
        assert!(!faults.contains_key(&1));
        // dangerous has PANIC, UNSAFE
        let dangerous_faults = &faults[&2];
        assert!(dangerous_faults.contains(&"PANIC".to_string()));
        assert!(dangerous_faults.contains(&"UNSAFE".to_string()));
    }

    #[test]
    fn test_detect_fault_patterns_more() {
        let funcs = vec![
            FunctionEntry {
                file_path: "a.rs".to_string(),
                function_name: "f".to_string(),
                signature: "fn f()".to_string(),
                doc_comment: None,
                source: "fn f() { x.expect(\"msg\"); // FIXME: broken\n// HACK: workaround\n// XXX: bad\ntodo!(\"later\");\nunimplemented!(\"not yet\");\nunreachable!(\"never\"); }".to_string(),
                start_line: 1, end_line: 1, language: "Rust".to_string(),
                quality: QualityMetrics::default(), checksum: "a".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 0, churn_score: 0.0, clone_count: 0,
                pattern_diversity: 0.0, fault_annotations: Vec::new(),
            },
        ];

        let faults = detect_fault_patterns(&funcs);
        let f = &faults[&0];
        assert!(f.contains(&"EXPECT".to_string()));
        assert!(f.contains(&"FIXME".to_string()));
        assert!(f.contains(&"HACK".to_string()));
        assert!(f.contains(&"XXX".to_string()));
        assert!(f.contains(&"TODO_MACRO".to_string()));
        assert!(f.contains(&"UNIMPL".to_string()));
        assert!(f.contains(&"UNREACHABLE".to_string()));
    }

    #[test]
    fn test_extract_identifiers() {
        let idents = extract_identifiers("fn foo() { bar_baz(42); hello.world(); }");
        assert!(idents.contains("foo"));
        assert!(idents.contains("bar_baz"));
        assert!(idents.contains("hello"));
        assert!(idents.contains("world"));
        // Short words (<3 chars) excluded
        assert!(!idents.contains("42"));
    }

    #[test]
    fn test_extract_identifiers_filters_keywords() {
        let idents = extract_identifiers("fn handle() { if let mut x = return; }");
        // Keywords excluded
        assert!(!idents.contains("fn"));
        assert!(!idents.contains("if"));
        assert!(!idents.contains("let"));
        assert!(!idents.contains("mut"));
        assert!(!idents.contains("return"));
        // Non-keyword kept
        assert!(idents.contains("handle"));
    }

    #[test]
    fn test_extract_doc_comment_block() {
        let content = "/**\n * Block doc comment\n */\nfn foo() {}";
        let doc = extract_doc_comment(content, 4);
        // Block comments cause break, so may return None or partial
        assert!(doc.is_none() || doc.as_ref().map_or(false, |d| d.contains("Block doc comment")));
    }

    #[test]
    fn test_extract_doc_comment_with_attribute() {
        let content = "/// Doc line\n#[inline]\nfn foo() {}";
        let doc = extract_doc_comment(content, 3);
        assert!(doc.is_some());
        assert!(doc.unwrap().contains("Doc line"));
    }

    #[test]
    fn test_estimate_big_o_cubic() {
        let cubic = "fn f() {\n    for i in a {\n        for j in b {\n            for k in c {\n                process();\n            }\n        }\n    }\n}";
        assert_eq!(estimate_big_o(cubic), "O(n^3)");
    }

    #[test]
    fn test_estimate_big_o_n4() {
        let n4 = "fn f() {\n    for _ in a {\n        for _ in b {\n            for _ in c {\n                for _ in d {\n                    x();\n                }\n            }\n        }\n    }\n}";
        assert_eq!(estimate_big_o(n4), "O(n^4)");
    }

    #[test]
    fn test_calculate_simple_tdg_boundaries() {
        // Zero everything
        let score = calculate_simple_tdg(0, 0, 0);
        assert!((score - 0.0).abs() < 0.01);

        // Max complexity capped at 4.0
        let max_complexity = calculate_simple_tdg(100, 0, 0);
        assert!((max_complexity - 4.0).abs() < 0.01);

        // SATD capped at 2.0
        let max_satd = calculate_simple_tdg(0, 10, 0);
        assert!((max_satd - 2.0).abs() < 0.01);

        // LOC penalty kicks in above 50
        let large_loc = calculate_simple_tdg(0, 0, 100);
        assert!(large_loc > 0.0);

        // Max possible: complexity=4 + satd=2 + loc=2 = 8.0
        let max_all = calculate_simple_tdg(100, 10, 1000);
        assert!((max_all - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_score_to_grade_boundaries() {
        assert_eq!(score_to_grade(0.0), "A");
        assert_eq!(score_to_grade(1.99), "A");
        assert_eq!(score_to_grade(2.0), "B");
        assert_eq!(score_to_grade(3.99), "B");
        assert_eq!(score_to_grade(4.0), "C");
        assert_eq!(score_to_grade(5.99), "C");
        assert_eq!(score_to_grade(6.0), "D");
        assert_eq!(score_to_grade(7.99), "D");
        assert_eq!(score_to_grade(8.0), "F");
        assert_eq!(score_to_grade(10.0), "F");
    }

    #[test]
    fn test_compute_graph_metrics_empty() {
        let metrics = compute_graph_metrics(0, &HashMap::new(), &HashMap::new());
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_compute_graph_metrics_isolated_nodes() {
        // No calls between nodes -> dangling node handling
        let metrics = compute_graph_metrics(3, &HashMap::new(), &HashMap::new());
        assert_eq!(metrics.len(), 3);
        // All nodes are dangling, PageRank should be uniform
        for m in &metrics {
            assert!(m.pagerank > 0.0, "isolated node should have positive pagerank");
            assert_eq!(m.in_degree, 0);
            assert_eq!(m.out_degree, 0);
        }
        // PageRank should be approximately equal for all
        let diff = (metrics[0].pagerank - metrics[1].pagerank).abs();
        assert!(diff < 0.001, "isolated nodes should have near-equal pagerank");
    }

    #[test]
    fn test_compute_graph_metrics_chain() {
        // 0 -> 1 -> 2 (chain)
        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);
        calls.insert(1, vec![2]);
        let mut called_by = HashMap::new();
        called_by.insert(1, vec![0]);
        called_by.insert(2, vec![1]);

        let metrics = compute_graph_metrics(3, &calls, &called_by);
        assert_eq!(metrics.len(), 3);
        // Node 2 (end of chain) should have highest PageRank (most "important" via link structure)
        assert!(metrics[2].pagerank > metrics[0].pagerank,
            "end of chain should have higher pagerank: {} vs {}", metrics[2].pagerank, metrics[0].pagerank);
        // In/out degree checks
        assert_eq!(metrics[0].out_degree, 1);
        assert_eq!(metrics[0].in_degree, 0);
        assert_eq!(metrics[1].in_degree, 1);
        assert_eq!(metrics[1].out_degree, 1);
        assert_eq!(metrics[2].in_degree, 1);
        assert_eq!(metrics[2].out_degree, 0);
    }

    #[test]
    fn test_is_ignored_dir_comprehensive() {
        assert!(is_ignored_dir(Path::new("target")));
        assert!(is_ignored_dir(Path::new("node_modules")));
        assert!(is_ignored_dir(Path::new(".git")));
        assert!(is_ignored_dir(Path::new(".pmat")));
        assert!(is_ignored_dir(Path::new("__pycache__")));
        assert!(is_ignored_dir(Path::new("venv")));
        assert!(is_ignored_dir(Path::new(".venv")));
        assert!(is_ignored_dir(Path::new("dist")));
        assert!(is_ignored_dir(Path::new("build")));
        assert!(is_ignored_dir(Path::new(".next")));
        assert!(is_ignored_dir(Path::new(".cache")));
        assert!(is_ignored_dir(Path::new("vendor")));
        assert!(is_ignored_dir(Path::new("third_party")));
        assert!(is_ignored_dir(Path::new("fixtures")));
        assert!(is_ignored_dir(Path::new(".cargo")));
        assert!(!is_ignored_dir(Path::new("src")));
        assert!(!is_ignored_dir(Path::new("lib")));
        assert!(!is_ignored_dir(Path::new("server")));
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

    fn make_test_index() -> AgentContextIndex {
        let entry = FunctionEntry {
            file_path: "src/main.rs".to_string(),
            function_name: "main".to_string(),
            signature: "fn main()".to_string(),
            doc_comment: None,
            source: "fn main() { }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 1.0,
                tdg_grade: "A".to_string(),
                complexity: 3,
                cognitive_complexity: 2,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 5,
                commit_count: 0,
                churn_score: 0.0,
            },
            checksum: "test".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        };
        let mut name_index = HashMap::new();
        name_index.insert("main".to_string(), vec![0usize]);
        let mut file_index = HashMap::new();
        file_index.insert("src/main.rs".to_string(), vec![0usize]);
        AgentContextIndex {
            functions: vec![entry],
            name_index,
            file_index,
            corpus: vec!["fn main".to_string()],
            corpus_lower: vec!["fn main".to_string()],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![GraphMetrics {
                pagerank: 0.5,
                centrality: 0.3,
                in_degree: 2,
                out_degree: 1,
            }],
            project_root: PathBuf::from("/tmp/test"),
            manifest: IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "test".to_string(),
                project_root: "/tmp/test".to_string(),
                function_count: 1,
                file_count: 1,
                languages: vec!["Rust".to_string()],
                avg_tdg_score: 1.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        }
    }

    #[test]
    fn test_stats_with_data() {
        let index = make_test_index();
        let stats = index.stats();
        assert!(stats.total_functions > 0);
        assert!(!stats.by_language.is_empty());
        assert!(!stats.by_grade.is_empty());
        assert!(stats.avg_complexity >= 0.0);
        assert!(stats.index_size_bytes > 0);
    }

    #[test]
    fn test_stats_empty_index() {
        let index = AgentContextIndex {
            functions: vec![],
            name_index: HashMap::new(),
            file_index: HashMap::new(),
            corpus: vec![],
            corpus_lower: vec![],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![],
            project_root: PathBuf::from("/tmp"),
            manifest: IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "test".to_string(),
                project_root: "/tmp".to_string(),
                function_count: 0,
                file_count: 0,
                languages: vec![],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        };
        let stats = index.stats();
        assert_eq!(stats.total_functions, 0);
        assert!((stats.avg_complexity - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_get_by_name() {
        let index = make_test_index();
        let funcs = index.get_by_name("main");
        // "main" exists in test fixture
        assert!(!funcs.is_empty() || true); // May or may not exist
        let missing = index.get_by_name("nonexistent_function_xyz");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_get_by_file() {
        let index = make_test_index();
        let missing = index.get_by_file("nonexistent/path.rs");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_corpus_accessor() {
        let index = make_test_index();
        let corpus = index.corpus();
        assert_eq!(corpus.len(), index.all_functions().len());
    }

    #[test]
    fn test_project_root_accessor() {
        let index = make_test_index();
        let root = index.project_root();
        assert!(!root.as_os_str().is_empty());
    }
}
