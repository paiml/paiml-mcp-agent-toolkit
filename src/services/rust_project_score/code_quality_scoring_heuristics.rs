// Code quality scoring: heuristic checks (complexity, unsafe, dead code)
// Cache-aware: uses FileCache if available, falls back to filesystem

fn count_deep_nesting(content: &str) -> usize {
    // 10 levels of 4-space indent = 40 chars. Rust match/if-let chains
    // commonly reach 8 levels (32 chars), so threshold at 10 levels.
    content
        .lines()
        .filter(|line| {
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            indent > 40
        })
        .count()
}

fn analyze_unsafe_in_content(content: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut unsafe_blocks = 0;
    let mut documented = 0;

    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        // Skip lines that mention "unsafe" in non-code contexts
        if t.starts_with("//") || t.starts_with("*") || t.starts_with("\"")
            || t.starts_with("r#") || t.starts_with("r\"") || t.starts_with("let ")
            || t.contains(".contains(\"unsafe") || t.contains(".starts_with(\"unsafe")
            || t.contains("\"unsafe {\"") || t.contains("\"unsafe{\"")
        {
            continue;
        }
        if t.starts_with("unsafe {") || t.starts_with("unsafe{")
            || t.contains("= unsafe {") || t.contains("} unsafe {")
        {
            unsafe_blocks += 1;
            let start = i.saturating_sub(10);
            if lines[start..=i]
                .iter()
                .any(|l| l.contains("SAFETY:") || l.contains("Safety:"))
            {
                documented += 1;
            }
        }
    }

    (unsafe_blocks, documented)
}

fn count_dead_code_attrs(content: &str) -> usize {
    content.matches("#[allow(dead_code)]").count()
        + content.matches("#![allow(dead_code)]").count()
}

fn score_from_nesting(count: usize) -> f64 {
    if count == 0 { 3.0 }
    else if count <= 5 { 2.0 }
    else if count <= 20 { 1.0 }
    else { 0.0 }
}

fn score_from_unsafe(unsafe_blocks: usize, documented: usize) -> f64 {
    if unsafe_blocks == 0 {
        return 9.0;
    }
    let doc_ratio = documented as f64 / unsafe_blocks as f64;
    if doc_ratio >= 0.9 { 9.0 }
    else if doc_ratio >= 0.7 { 7.0 }
    else if doc_ratio >= 0.5 { 5.0 }
    else if doc_ratio >= 0.3 { 3.0 }
    else { 1.0 }
}

fn for_each_rs_content_cached(cache: &FileCache, src_path: &Path, mut f: impl FnMut(&str)) {
    for (_path, content) in cache.get_rust_files_in_dir(src_path) {
        f(content);
    }
}

fn for_each_rs_content_fs(src_path: &Path, f: &mut impl FnMut(&str)) {
    // #237 bug 3: Must be recursive — read_dir is non-recursive and missed 98%+ of code
    if let Ok(entries) = std::fs::read_dir(src_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                for_each_rs_content_fs(&path, f);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    f(&content);
                }
            }
        }
    }
}

impl CodeQualityScorer {
    fn score_complexity_simple(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(3.0);
        }

        let mut deep_nesting_count = 0;
        let mut accumulate = |content: &str| {
            deep_nesting_count += count_deep_nesting(content);
        };

        if let Some(cache) = cache {
            for_each_rs_content_cached(cache, &src_path, accumulate);
        } else {
            for_each_rs_content_fs(&src_path, &mut accumulate);
        }

        Ok(score_from_nesting(deep_nesting_count))
    }

    fn score_unsafe(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(9.0);
        }

        let mut total_unsafe = 0;
        let mut total_documented = 0;
        let mut accumulate = |content: &str| {
            let (ub, doc) = analyze_unsafe_in_content(content);
            total_unsafe += ub;
            total_documented += doc;
        };

        if let Some(cache) = cache {
            for_each_rs_content_cached(cache, &src_path, accumulate);
        } else {
            for_each_rs_content_fs(&src_path, &mut accumulate);
        }

        Ok(score_from_unsafe(total_unsafe, total_documented))
    }

    fn score_dead_code(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(2.0);
        }

        let mut dead_code_count = 0;
        let mut accumulate = |content: &str| {
            dead_code_count += count_dead_code_attrs(content);
        };

        if let Some(cache) = cache {
            for_each_rs_content_cached(cache, &src_path, accumulate);
        } else {
            for_each_rs_content_fs(&src_path, &mut accumulate);
        }

        if dead_code_count == 0 { Ok(2.0) }
        else if dead_code_count <= 3 { Ok(1.0) }
        else { Ok(0.0) }
    }
}
