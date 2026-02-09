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
    pub fn build(project_path: &Path) -> Result<Self, String> {
        let project_root = project_path
            .canonicalize()
            .map_err(|e| format!("Invalid project path: {e}"))?;

        let mut functions = Vec::new();
        let mut file_count = 0;
        let mut languages_seen = HashMap::new();
        let mut file_checksums: HashMap<String, String> = HashMap::new();
        let mut coverage_off_files = HashSet::new();

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

            // Detect module-level coverage(off) — cached for O(1) query-time lookup
            if has_coverage_off(&content) {
                coverage_off_files.insert(relative_path.clone());
            }

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

                // Skip test functions and test files (#159: reduce index bloat)
                if is_test_chunk(&chunk.chunk_name, &relative_path) {
                    continue;
                }

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
    /// a `.pmat/context.idx` or `.pmat/context.db`. Silently skips siblings
    /// without an index.
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

            // Check for SQLite index first (v2.0), then blob directory (v1.x)
            let db_path = resolved.join(".pmat/context.db");
            let idx_path = resolved.join(".pmat/context.idx");
            if db_path.exists() || idx_path.exists() {
                // Pass the blob directory path; load() will detect .db alongside it
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
    /// After all siblings are merged, rebuilds the unified call graph so that
    /// cross-project call edges are resolved and PageRank reflects cross-project
    /// importance.
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
        // Rebuild unified call graph + PageRank across all projects
        if !siblings.is_empty() {
            self.rebuild_cross_project_graph();
        }
    }

    /// Rebuild call graph and graph metrics on the merged index.
    ///
    /// After `merge_fast()` appends per-project call graphs with offset-adjusted
    /// indices, cross-project calls are not yet resolved (e.g., a function in
    /// aprender calling a trueno function). This method rebuilds the entire call
    /// graph from scratch using the unified `name_index`, then recomputes
    /// PageRank and centrality so cross-project importance is reflected.
    pub fn rebuild_cross_project_graph(&mut self) {
        let (calls, called_by) = build_call_graph(&self.functions, &self.name_index);
        let graph_metrics = compute_graph_metrics(self.functions.len(), &calls, &called_by);
        self.calls = calls;
        self.called_by = called_by;
        self.graph_metrics = graph_metrics;
    }

    /// Fast merge: concatenate pre-built data with index offset adjustment.
    ///
    /// Unlike `merge()` which rebuilds all indices from scratch (O(n*m) for
    /// call graph), this just offsets positional indices and appends. O(n).
    pub(super) fn merge_fast(&mut self, other: Self) {
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

        // Append graph_metrics (per-function values, no offset needed)
        self.graph_metrics.extend(other.graph_metrics);

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

        // Phase 3 (#159): Write only SQLite + FTS5 index (no more LZ4 blob)
        // context.db lives alongside context.idx directory
        let db_path = index_path.with_extension("db");
        super::sqlite_backend::save_to_sqlite(
            &db_path,
            &self.functions,
            &self.calls,
            &self.graph_metrics,
            &self.manifest,
            &self.coverage_off_files,
        )?;

        Ok(())
    }

    /// Load index from directory.
    ///
    /// Prefers SQLite `context.db` when available (v2.0), falls back to
    /// LZ4+bincode blob `context.idx/functions.lz4` (v1.x).
    pub fn load(index_path: &Path) -> Result<Self, String> {
        // Try SQLite path first (v2.0)
        let db_candidate = index_path.with_extension("db");
        if db_candidate.exists() {
            // Validate schema before attempting full load — stale DBs from older
            // versions may lack required tables, producing confusing warnings.
            let schema_ok = super::sqlite_backend::open_db(&db_candidate)
                .map(|conn| super::sqlite_backend::has_valid_schema(&conn))
                .unwrap_or(false);

            if schema_ok {
                match Self::load_from_sqlite(&db_candidate) {
                    Ok(index) => return Ok(index),
                    Err(e) => {
                        eprintln!("  Warning: SQLite load failed, falling back to blob: {e}");
                    }
                }
            } else {
                // Delete broken/stale DB so next save() regenerates it
                let _ = std::fs::remove_file(&db_candidate);
            }
        }

        Self::load_from_blob(index_path)
    }

    /// Load index from SQLite database (v2.0 fast path).
    ///
    /// Reads functions and graph metrics from `context.db`.
    /// Skips corpus (FTS5 handles search) and call graph (queried on-demand).
    /// Saves ~86MB of memory and ~600ms of load time for 90K functions.
    fn load_from_sqlite(db_path: &Path) -> Result<Self, String> {
        use super::sqlite_backend::{load_functions, load_graph_metrics, load_metadata, open_db};

        let conn = open_db(db_path)?;
        let manifest = load_metadata(&conn)?;
        let functions = load_functions(&conn)?;
        let graph_metrics = load_graph_metrics(&conn)?;
        // Load call graph eagerly — avoids 71K individual SQLite queries at query time
        let (calls, called_by) = super::sqlite_backend::load_call_graph(&conn)?;

        // Build name_index + file_index only (no corpus — FTS5 handles search)
        let indices = build_indices_without_corpus(&functions);
        let name_frequency = compute_name_frequency(&indices.name_index, functions.len());

        let project_root = PathBuf::from(&manifest.project_root);

        // Load cached coverage_off_files from SQLite metadata
        let coverage_off_files = load_coverage_off_files(&conn);

        Ok(Self {
            functions,
            name_index: indices.name_index,
            file_index: indices.file_index,
            corpus: Vec::new(),
            corpus_lower: Vec::new(),
            name_frequency,
            calls,
            called_by,
            graph_metrics,
            project_root,
            manifest,
            db_path: Some(db_path.to_path_buf()),
            coverage_off_files,
        })
    }

    /// Load index from LZ4+bincode blob (v1.x legacy path).
    fn load_from_blob(index_path: &Path) -> Result<Self, String> {
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
            let corpus_lower = if payload.corpus_lower.is_empty() {
                corpus.iter().map(|d| d.to_lowercase()).collect()
            } else {
                payload.corpus_lower
            };
            (
                payload.name_index,
                payload.file_index,
                payload.graph_metrics,
                corpus_lower,
                payload.name_frequency,
            )
        } else {
            // Slow path: rebuild indices for legacy formats (v1.0-v1.2)
            let is_legacy = manifest.version.starts_with("1.0") || manifest.version.starts_with("1.1");
            let indices = build_indices(&functions);

            let (calls_rebuilt, called_by_rebuilt) = if is_legacy && calls.is_empty() {
                build_call_graph(&functions, &indices.name_index)
            } else {
                (calls.clone(), called_by.clone())
            };

            let graph_metrics = compute_graph_metrics(functions.len(), &calls_rebuilt, &called_by_rebuilt);
            let name_frequency = compute_name_frequency(&indices.name_index, functions.len());
            let corpus_lower: Vec<String> = corpus.iter().map(|d| d.to_lowercase()).collect();

            (indices.name_index, indices.file_index, graph_metrics, corpus_lower, name_frequency)
        };

        let project_root = PathBuf::from(&manifest.project_root);

        // Detect SQLite FTS5 database alongside blob
        let db_candidate = index_path.with_extension("db");
        let db_path = if db_candidate.exists() {
            Some(db_candidate)
        } else {
            None
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
            graph_metrics,
            project_root,
            manifest,
            db_path,
            coverage_off_files: HashSet::new(), // Legacy blob path — no cached data
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
        let mut coverage_off_files = HashSet::new();

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

            // Detect module-level coverage(off) — cached for O(1) query-time lookup
            if has_coverage_off(&content) {
                coverage_off_files.insert(relative_path.clone());
            }

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

                    // Skip test functions and test files (#159: reduce index bloat)
                    if is_test_chunk(&chunk.chunk_name, &relative_path) {
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
            version: "1.4.0".to_string(),
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
            db_path: None,
            coverage_off_files,
        })
    }

    /// Get callees for a given function index.
    ///
    /// Uses in-memory HashMap when available (blob load), falls back to
    /// on-demand SQLite query (SQLite load — avoids loading 3.8M edges upfront).
    pub fn get_calls(&self, func_idx: usize) -> Vec<&str> {
        if let Some(indices) = self.calls.get(&func_idx) {
            return indices
                .iter()
                .map(|&i| self.functions[i].function_name.as_str())
                .collect();
        }
        // On-demand SQLite query when call graph not loaded in memory
        if let Some(ref db_path) = self.db_path {
            if let Ok(conn) = super::sqlite_backend::open_db(db_path) {
                if let Ok(indices) = super::sqlite_backend::query_callees(&conn, func_idx) {
                    return indices
                        .iter()
                        .filter(|&&i| i < self.functions.len())
                        .map(|&i| self.functions[i].function_name.as_str())
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Get callers for a given function index.
    ///
    /// Uses in-memory HashMap when available (blob load), falls back to
    /// on-demand SQLite query (SQLite load — avoids loading 3.8M edges upfront).
    pub fn get_called_by(&self, func_idx: usize) -> Vec<&str> {
        if let Some(indices) = self.called_by.get(&func_idx) {
            return indices
                .iter()
                .map(|&i| self.functions[i].function_name.as_str())
                .collect();
        }
        // On-demand SQLite query when call graph not loaded in memory
        if let Some(ref db_path) = self.db_path {
            if let Ok(conn) = super::sqlite_backend::open_db(db_path) {
                if let Ok(indices) = super::sqlite_backend::query_callers(&conn, func_idx) {
                    return indices
                        .iter()
                        .filter(|&&i| i < self.functions.len())
                        .map(|&i| self.functions[i].function_name.as_str())
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Get raw caller indices for a function (O(1) from in-memory map).
    /// Returns None if call graph is not loaded in memory (SQLite-only mode).
    pub fn called_by_indices(&self, func_idx: usize) -> Option<&[usize]> {
        self.called_by.get(&func_idx).map(|v| v.as_slice())
    }

    /// Get raw callee indices for a function (O(1) from in-memory map).
    /// Returns None if call graph is not loaded in memory (SQLite-only mode).
    pub fn calls_indices(&self, func_idx: usize) -> Option<&[usize]> {
        self.calls.get(&func_idx).map(|v| v.as_slice())
    }

    /// Find function index by file path and name
    pub fn find_function_index(&self, file_path: &str, function_name: &str) -> Option<usize> {
        self.functions
            .iter()
            .position(|f| f.file_path == file_path && f.function_name == function_name)
    }

    /// Count callers from different projects for a given function.
    ///
    /// In a workspace index, file paths are prefixed with the project name
    /// (e.g., `aprender/src/lib.rs`). A cross-project caller is one whose
    /// project prefix differs from the callee's prefix.
    pub fn count_cross_project_callers(&self, func_idx: usize) -> u32 {
        if func_idx >= self.functions.len() {
            return 0;
        }
        let callee_project = project_prefix(&self.functions[func_idx].file_path);
        let caller_indices = match self.called_by.get(&func_idx) {
            Some(indices) => indices,
            None => return 0,
        };
        caller_indices
            .iter()
            .filter(|&&i| {
                i < self.functions.len()
                    && project_prefix(&self.functions[i].file_path) != callee_project
            })
            .count() as u32
    }
}

/// Extract the project prefix from a file path (everything before the first `/`).
///
/// For workspace-merged paths like `aprender/src/lib.rs`, returns `aprender`.
/// For local paths like `src/lib.rs`, returns `src`.
fn project_prefix(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}
