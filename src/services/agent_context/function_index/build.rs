#![cfg_attr(coverage_nightly, coverage(off))]

use super::helpers::*;
use super::types::*;
use crate::services::semantic::{chunk_code, CodeChunk};
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

/// One warning line per language whose files the chunker refused.
///
/// Files that fail to chunk used to vanish silently (`Err(_) => continue`): a
/// tree holding `svc.go` and `util.py` indexed as `file_count: 1,
/// languages: ["Python"]`, and `pmat_query_code language:"go"` answered
/// `{"results":[],"total":0}` — indistinguishable from "your query matched
/// nothing", when the real answer is that this build has no Go parser compiled
/// in (`go-ast` is not in the default feature set). Naming the language, the
/// count and the parser's own reason is what tells those two apart.
fn format_skipped_summary(skipped: &HashMap<String, (usize, String)>) -> Vec<String> {
    let mut by_language: Vec<(&String, &(usize, String))> = skipped.iter().collect();
    by_language.sort_by(|a, b| a.0.cmp(b.0));
    by_language
        .into_iter()
        .map(|(language, (count, reason))| {
            format!("⚠️  Not indexed: {count} {language} file(s) could not be parsed ({reason})")
        })
        .collect()
}

/// Mutable accumulators threaded through the directory walk in [`AgentContextIndex::build`].
///
/// Grouping them lets the per-file work live in [`index_one_file`] instead of
/// being inlined in the walk's closure body.
struct BuildState {
    functions: Vec<FunctionEntry>,
    file_count: usize,
    languages_seen: HashMap<String, usize>,
    file_checksums: HashMap<String, FileRecord>,
    coverage_off_files: HashSet<String>,
    /// Language -> (how many files the chunker refused, the first reason it
    /// gave). Reported after the walk; see `format_skipped_summary`.
    skipped_by_language: HashMap<String, (usize, String)>,
    /// Reusable read buffer — avoids allocating a new String per file (~33 MB saved)
    read_buf: String,
}

impl BuildState {
    fn new() -> Self {
        Self {
            functions: Vec::with_capacity(20_000),
            file_count: 0,
            languages_seen: HashMap::new(),
            file_checksums: HashMap::with_capacity(4_000),
            coverage_off_files: HashSet::new(),
            skipped_by_language: HashMap::new(),
            read_buf: String::with_capacity(32 * 1024),
        }
    }
}

/// Turn one chunker result into `FunctionEntry` rows and push them onto `functions`.
///
/// Extracted from the body of [`index_one_file`]'s chunk loop; behaviour (which
/// chunk types are indexed, which are skipped as tests, and every field of the
/// resulting `FunctionEntry`) is unchanged.
fn push_chunk_functions(
    chunks: Vec<CodeChunk>,
    content: &str,
    relative_path: &str,
    lang_str: &str,
    functions: &mut Vec<FunctionEntry>,
) {
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
        if is_test_chunk(&chunk.chunk_name, relative_path) {
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
            file_path: relative_path.to_string(),
            function_name: std::mem::take(&mut chunk.chunk_name),
            signature,
            definition_type,
            doc_comment,
            source: std::mem::take(&mut chunk.content),
            start_line: chunk.start_line,
            end_line: chunk.end_line,
            language: lang_str.to_string(),
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
}

/// Read, checksum and chunk one directory-walk entry into `state`.
///
/// Extracted from the body of the sorted walk in [`AgentContextIndex::build`];
/// every early return here corresponds exactly to a `continue` in the original
/// loop, so the walk's side effects (which files count, which are skipped and
/// why) are unchanged.
fn index_one_file(state: &mut BuildState, path: &Path, project_root: &Path) {
    if !path.is_file() {
        return;
    }

    // Detect language from extension
    let language = match detect_language(path) {
        Some(lang) => lang,
        None => return,
    };

    // Read file content into reusable buffer
    state.read_buf.clear();
    let content = match std::fs::File::open(path).and_then(|mut f| {
        use std::io::Read;
        f.read_to_string(&mut state.read_buf)
    }) {
        Ok(_) => state.read_buf.as_str(),
        Err(_) => return, // Skip binary/unreadable files
    };

    let relative_path = path
        .strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    // Compute SHA256 checksum, and record the stat evidence the
    // incremental fast path needs before it may skip a read.
    let checksum = compute_file_sha256(content);
    state
        .file_checksums
        .insert(relative_path.clone(), file_record(path, checksum));

    // Detect module-level coverage(off) — cached for O(1) query-time lookup
    if has_coverage_off(content) {
        state.coverage_off_files.insert(relative_path.clone());
    }

    // Extract functions using AST chunker. A refusal here is recorded,
    // not swallowed — a language this build cannot parse must not look
    // like a language that simply has no matches.
    let chunks = match chunk_code(content, language) {
        Ok(c) => c,
        Err(reason) => {
            let entry = state
                .skipped_by_language
                .entry(format!("{language:?}"))
                .or_insert((0, reason));
            entry.0 += 1;
            return;
        }
    };

    let lang_str = format!("{language:?}");
    *state.languages_seen.entry(lang_str.clone()).or_insert(0) += 1;

    push_chunk_functions(
        chunks,
        content,
        &relative_path,
        &lang_str,
        &mut state.functions,
    );

    state.file_count += 1;
    if state.file_count.is_multiple_of(500) {
        eprint!("\r  Indexing... {} files", state.file_count);
    }
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn build(project_path: &Path) -> Result<Self, String> {
        let project_root = project_path
            .canonicalize()
            .map_err(|e| format!("Invalid project path: {e}"))?;

        let mut state = BuildState::new();

        // Load compile_commands.json for C/C++ include path discovery
        let _compile_commands = load_compile_commands(&project_root);

        // Walk the project directory respecting .gitignore (fixes issue #146),
        // in sorted path order: this walk defines the order functions are
        // persisted in, and therefore the order tied results are returned in.
        // Unsorted, two indexes of the same commit differ by directory order.
        for entry in WalkBuilder::new(&project_root)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .filter_entry(|e| !is_ignored_dir(e.path()))
            .sort_by_file_path(std::path::Path::cmp)
            .build()
            .filter_map(|e| e.ok())
        {
            index_one_file(&mut state, entry.path(), &project_root);
        }
        if state.file_count >= 500 {
            eprintln!("\r  Indexed {} files", state.file_count);
        }
        for line in format_skipped_summary(&state.skipped_by_language) {
            eprintln!("{line}");
        }

        let BuildState {
            mut functions,
            file_count,
            languages_seen,
            file_checksums,
            coverage_off_files,
            ..
        } = state;

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
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
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
        self.name_index
            .get(name)
            .map(|indices| indices.iter().map(|&i| &self.functions[i]).collect())
            .unwrap_or_default()
    }

    /// Get functions in a file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_by_file(&self, file_path: &str) -> Vec<&FunctionEntry> {
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod skipped_file_reporting_tests {
    use super::*;

    /// The indexer used to drop every file its chunker refused with
    /// `Err(_) => continue`, so a build without `go-ast` reported a Go+Python
    /// tree as `file_count: 1, languages: ["Python"]` and said nothing at all
    /// about the Go file it could not read.
    #[test]
    fn refused_files_are_counted_and_named_per_language() {
        let mut skipped: HashMap<String, (usize, String)> = HashMap::new();
        skipped.insert(
            "Go".to_string(),
            (2, "go-ast feature is disabled".to_string()),
        );
        skipped.insert("Lua".to_string(), (1, "parse error".to_string()));

        let lines = format_skipped_summary(&skipped);

        assert_eq!(lines.len(), 2, "{lines:?}");
        // Deterministic order — the map's iteration order is not.
        assert!(lines[0].contains("Go"), "{lines:?}");
        assert!(lines[0].contains('2'), "{lines:?}");
        assert!(
            lines[0].contains("go-ast feature is disabled"),
            "the parser's own reason must survive: {lines:?}"
        );
        assert!(lines[1].contains("Lua"), "{lines:?}");
    }

    #[test]
    fn nothing_is_reported_when_every_file_parsed() {
        assert!(format_skipped_summary(&HashMap::new()).is_empty());
    }

    /// The condition the report exists for: in a build without `go-ast`, Go
    /// source really is refused, so `.go` files really are missing from the
    /// index — which is exactly what must not happen silently.
    #[cfg(not(feature = "go-ast"))]
    #[test]
    fn go_source_is_refused_by_this_build_and_therefore_reportable() {
        let reason = crate::services::semantic::chunk_code(
            "package main\n\nfunc main() {}\n",
            crate::services::semantic::Language::Go,
        )
        .expect_err("a build without go-ast cannot chunk Go");

        let mut skipped: HashMap<String, (usize, String)> = HashMap::new();
        skipped.insert("Go".to_string(), (1, reason));
        let lines = format_skipped_summary(&skipped);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("Go"), "{lines:?}");
    }
}
