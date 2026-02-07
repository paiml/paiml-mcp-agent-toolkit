#![cfg_attr(coverage_nightly, coverage(off))]

use super::helpers::*;
use super::types::*;
use crate::services::semantic::chunk_code;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

        // Save payload (functions + corpus + call graph + cached indices) v1.4.0 format
        let payload = IndexPayload {
            functions: self.functions.clone(),
            corpus: self.corpus.clone(),
            calls: self.calls.clone(),
            called_by: self.called_by.clone(),
            // v1.3.0+: Cache computed indices to avoid rebuild on load
            name_index: self.name_index.clone(),
            file_index: self.file_index.clone(),
            graph_metrics: self.graph_metrics.clone(),
            // v1.4.0: corpus_lower computed lazily on load (saves ~50MB)
            corpus_lower: Vec::new(),
            name_frequency: self.name_frequency.clone(),
        };
        let payload_bin = bincode::serialize(&payload)
            .map_err(|e| format!("Failed to serialize payload: {e}"))?;

        // Compress with LZ4
        let compressed = lz4_flex::compress_prepend_size(&payload_bin);
        fs::write(index_path.join("functions.lz4"), compressed)
            .map_err(|e| format!("Failed to write functions: {e}"))?;

        // Dual-write: also save SQLite + FTS5 index (Phase 1, #159)
        // context.db lives alongside context.idx directory
        let db_path = index_path.with_extension("db");
        if let Err(e) = super::sqlite_backend::save_to_sqlite(
            &db_path,
            &self.functions,
            &self.calls,
            &self.graph_metrics,
            &self.manifest,
        ) {
            eprintln!("  Warning: SQLite index save failed: {e}");
            // Non-fatal: blob is the primary format in Phase 1
        }

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
            // v1.4.0: corpus_lower no longer persisted, compute lazily (~50ms for 50K functions)
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
