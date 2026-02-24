// ── Core analysis ──────────────────────────────────────────────────────────

/// Result from a signal analyzer: (suggested_name, confidence, reasoning).
type SignalResult = Option<(String, f32, String)>;

fn analyze_file_for_rename(
    file_path: &str,
    entries: &[&FunctionEntry],
    index: &AgentContextIndex,
) -> RenameSuggestion {
    let parent_file = detect_parent_file(file_path, index);

    // Cascading signal priority: try each analyzer in order
    let signals: [(RenameSignal, SignalResult); 6] = [
        (RenameSignal::DominantType, try_dominant_type(entries)),
        (
            RenameSignal::ExistingSuffix,
            try_existing_suffix(file_path, entries),
        ),
        (RenameSignal::OriginalBase, try_original_base(file_path)),
        (RenameSignal::FunctionTheme, try_function_theme(entries)),
        (RenameSignal::CommonPrefix, try_common_prefix(entries)),
        (
            RenameSignal::DocCommentConsensus,
            try_doc_comment_consensus(entries),
        ),
    ];

    let ctx = SuggestionCtx {
        file_path,
        parent_file,
        definition_count: entries.len(),
        index,
    };

    for (signal, result) in signals {
        if let Some((name, confidence, reasoning)) = result {
            return build_suggestion(&ctx, &name, confidence, reasoning, signal);
        }
    }

    // Fallback: no strong signal
    RenameSuggestion {
        current_path: file_path.to_string(),
        suggested_name: String::new(),
        suggested_path: String::new(),
        confidence: 0.30,
        reasoning: "No dominant signal found".to_string(),
        signal: RenameSignal::NoSignal,
        parent_file: ctx.parent_file,
        inclusion_pattern: Some("include!".to_string()),
        definition_count: ctx.definition_count,
    }
}

/// Context passed to build_suggestion to reduce argument count.
struct SuggestionCtx<'a> {
    file_path: &'a str,
    parent_file: Option<String>,
    definition_count: usize,
    index: &'a AgentContextIndex,
}

/// Build a RenameSuggestion from a successful signal analysis.
fn build_suggestion(
    ctx: &SuggestionCtx<'_>,
    name: &str,
    confidence: f32,
    reasoning: String,
    signal: RenameSignal,
) -> RenameSuggestion {
    let suggested_name = format!("{name}.rs");
    let suggested_path = replace_filename(ctx.file_path, &suggested_name);

    // Reject if suggestion collides with parent file
    if collides_with_parent(&suggested_path, &ctx.parent_file) {
        return RenameSuggestion {
            current_path: ctx.file_path.to_string(),
            suggested_name: String::new(),
            suggested_path: String::new(),
            confidence: 0.10,
            reasoning: format!("{reasoning} [same as parent]"),
            signal: RenameSignal::NoSignal,
            parent_file: ctx.parent_file.clone(),
            inclusion_pattern: Some("include!".to_string()),
            definition_count: ctx.definition_count,
        };
    }

    // Penalize if name matches parent directory (redundant: graph/graph.rs)
    let dir_penalty = matches_parent_dir(ctx.file_path, name);

    let collision = check_collision(&suggested_path, ctx.index);
    let mut final_confidence = confidence;
    let mut final_reasoning = reasoning;
    if collision {
        final_confidence *= 0.5;
    }
    if dir_penalty {
        final_confidence *= 0.70;
        final_reasoning = format!("{final_reasoning} [redundant with parent dir]");
    }
    RenameSuggestion {
        current_path: ctx.file_path.to_string(),
        suggested_name,
        suggested_path,
        confidence: final_confidence,
        reasoning: final_reasoning,
        signal,
        parent_file: ctx.parent_file.clone(),
        inclusion_pattern: Some("include!".to_string()),
        definition_count: ctx.definition_count,
    }
}

/// Check if the suggested path matches the parent file path.
fn collides_with_parent(suggested_path: &str, parent_file: &Option<String>) -> bool {
    parent_file
        .as_ref()
        .is_some_and(|parent| suggested_path == parent)
}

/// Check if the suggested name matches the immediate parent directory.
/// E.g., `graph/mod_part_02.rs → graph.rs` is redundant inside a `graph/` dir.
fn matches_parent_dir(file_path: &str, suggested_name: &str) -> bool {
    let parts: Vec<&str> = file_path.rsplitn(2, '/').collect();
    if parts.len() < 2 {
        return false;
    }
    // parts[0] = filename, parts[1] = everything before last /
    let parent_dir = parts[1].rsplit('/').next().unwrap_or("");
    parent_dir == suggested_name
}

