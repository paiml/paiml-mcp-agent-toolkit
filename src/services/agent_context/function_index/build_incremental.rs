fn process_changed_file(
    content: &str,
    relative_path: &str,
    language: crate::services::semantic::Language,
    functions: &mut Vec<FunctionEntry>,
    languages_seen: &mut HashMap<String, usize>,
) {
    let chunks = match chunk_code(content, language) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lang_str = format!("{language:?}");
    *languages_seen.entry(lang_str.clone()).or_insert(0) += 1;

    for chunk in chunks {
        use crate::services::semantic::ChunkType;
        let definition_type = match &chunk.chunk_type {
            ChunkType::Function => DefinitionType::Function,
            ChunkType::Struct => DefinitionType::Struct,
            ChunkType::Enum => DefinitionType::Enum,
            ChunkType::Trait => DefinitionType::Trait,
            ChunkType::TypeAlias => DefinitionType::TypeAlias,
            _ => continue,
        };

        if is_test_chunk(&chunk.chunk_name, relative_path) {
            continue;
        }

        let quality = extract_quality_metrics(&chunk, content);
        let signature = chunk.content.lines().next().unwrap_or(&chunk.chunk_name).to_string();
        let doc_comment = extract_doc_comment(content, chunk.start_line);

        functions.push(FunctionEntry {
            file_path: relative_path.to_string(),
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
            linked_definition: None,
        });
    }
}

fn reuse_existing_functions(existing: &AgentContextIndex, relative_path: &str, functions: &mut Vec<FunctionEntry>) {
    if let Some(indices) = existing.file_index.get(relative_path) {
        for &idx in indices {
            functions.push(existing.functions[idx].clone());
        }
    }
}

fn finalize_incremental_index(
    functions: Vec<FunctionEntry>,
    project_root: PathBuf,
    file_count: usize,
    files_reparsed: usize,
    mut languages_seen: HashMap<String, usize>,
    file_checksums: HashMap<String, String>,
    coverage_off_files: HashSet<String>,
) -> AgentContextIndex {
    let indices = build_indices(&functions);
    let (calls, called_by) = build_call_graph(&functions, &indices.name_index);
    let graph_metrics = compute_graph_metrics(functions.len(), &calls, &called_by);
    let name_frequency = compute_name_frequency(&indices.name_index, functions.len());
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|d| d.to_lowercase()).collect();

    let avg_tdg = if functions.is_empty() {
        0.0
    } else {
        functions.iter().map(|f| f.quality.tdg_score).sum::<f32>() / functions.len() as f32
    };

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

    AgentContextIndex {
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
    }
}

impl AgentContextIndex {
    /// Build an incremental index update, re-parsing only changed files.
    ///
    /// Uses a two-tier change detection strategy:
    /// 1. **Mtime fast path**: If file mtime < index built_at, skip read+SHA256 entirely
    /// 2. **SHA256 fallback**: For files with newer mtime, read and compare checksums
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
        let mut files_mtime_skipped = 0usize;
        let mut coverage_off_files = HashSet::new();

        let index_built_at = parse_built_at(&existing.manifest.built_at);

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
            let mut language = match detect_language(path) {
                Some(lang) => lang,
                None => continue,
            };
            let relative_path = path.strip_prefix(&project_root).unwrap_or(path).to_string_lossy().to_string();

            if let Some(reuse) = check_mtime_reuse(path, &relative_path, &index_built_at, existing) {
                functions.extend(reuse.functions);
                file_checksums.insert(relative_path.clone(), reuse.checksum);
                if reuse.coverage_off {
                    coverage_off_files.insert(relative_path);
                }
                files_mtime_skipped += 1;
                file_count += 1;
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Upgrade .h files from C to C++ based on content heuristics
            if language == crate::services::semantic::Language::C
                && path.extension().and_then(|e| e.to_str()) == Some("h")
            {
                language = classify_header_language(&content);
            }

            let checksum = compute_file_sha256(&content);
            file_checksums.insert(relative_path.clone(), checksum.clone());

            if has_coverage_off(&content) {
                coverage_off_files.insert(relative_path.clone());
            }

            let unchanged = existing.manifest.file_checksums.get(&relative_path).map(|old| old == &checksum).unwrap_or(false);

            if unchanged {
                reuse_existing_functions(existing, &relative_path, &mut functions);
                files_reused += 1;
            } else {
                process_changed_file(&content, &relative_path, language, &mut functions, &mut languages_seen);
                files_reparsed += 1;
            }

            file_count += 1;
        }

        eprintln!(
            "Incremental update: {} mtime-skipped, {} checksum-reused, {} re-parsed",
            files_mtime_skipped, files_reused, files_reparsed
        );

        Ok(finalize_incremental_index(
            functions, project_root, file_count, files_reparsed,
            languages_seen, file_checksums, coverage_off_files,
        ))
    }
}
