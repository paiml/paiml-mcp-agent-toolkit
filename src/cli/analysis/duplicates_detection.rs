/// The clone class a duplicate group was actually MEASURED to be.
///
/// Every block used to be reported as an exact duplicate: `exact_duplicates`
/// was `blocks with similarity >= 1.0` and the only hash-bucketing constructor
/// wrote `similarity: 1.0` as a literal, so `--detection-type renamed` reported
/// `exact_duplicates == total_duplicates` and the CSV `Type` column was the
/// string `"exact"` on every row. On this repo that told a consumer that
/// `rules.rs:40-78` and `rules.rs:98-136` — which differ on 8 of 39 lines
/// (`CyclomaticComplexityRule` vs `CognitiveComplexityRule`) — are byte
/// identical.
///
/// The class is derived from the group's CONTENTS, not from which extractor
/// produced it: a fuzzy-hashed group whose members happen to be byte identical
/// is still a Type-1 clone, and saying so costs one string comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloneType {
    /// Type-1: the members are identical once comments and blank lines are
    /// removed. `similarity` is exactly 1.0 and that is a measurement.
    Exact,
    /// Type-2: the members are identical up to identifier and literal NAMES —
    /// they hash alike only after identifier normalisation, and their text
    /// differs. `similarity` is the measured text similarity, which is < 1.0.
    Renamed,
    /// Type-3: near-miss — statements added, removed or reordered.
    /// `similarity` is the clone engine's measured mean similarity.
    NearMiss,
}

impl CloneType {
    /// The one spelling of this class used by every renderer (CSV `Type`
    /// column, JSON `clone_type`), so two surfaces of one report cannot
    /// disagree about what a block is.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            CloneType::Exact => "exact",
            CloneType::Renamed => "renamed",
            CloneType::NearMiss => "near-miss",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Duplicate block.
pub struct DuplicateBlock {
    pub hash: String,
    pub locations: Vec<DuplicateLocation>,
    pub lines: usize,
    pub tokens: usize,
    pub similarity: f32,
    /// Which clone class this group was measured to be. Never a constant: see
    /// [`CloneType`].
    pub clone_type: CloneType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Duplicate location.
pub struct DuplicateLocation {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content_preview: String,
}

#[derive(Debug, Serialize)]
/// Report containing duplicate data.
pub struct DuplicateReport {
    pub total_duplicates: usize,
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_percentage: f32,
    pub duplicate_blocks: Vec<DuplicateBlock>,
    /// DETERMINISM (round-3 sweep): a `BTreeMap`, not a `HashMap`. serde emits
    /// a map in iteration order, so `analyze duplicates --format json` listed
    /// the same per-file statistics under a different key order on every run.
    pub file_statistics: BTreeMap<String, FileStats>,
}

#[derive(Debug, Serialize)]
/// Statistics for file.
pub struct FileStats {
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_percentage: f32,
}

/// Main entry point for duplicate analysis
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_duplicates(
    project_path: PathBuf,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    format: crate::cli::DuplicateOutputFormat,
    perf: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    // A nonexistent path previously produced an empty report and exit 0.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    {
        use crate::cli::colors as c;
        crate::status_eprintln!("{}", c::dim("Analyzing code similarity..."));
    }

    let start_time = std::time::Instant::now();

    // `--top-files` used to be applied HERE, dropping duplicate blocks and then
    // recomputing the report from what survived: the same tree reported 12.2%
    // duplication at `--top-files 1` and 19.4% at `--top-files 0`, and one
    // 301-line file read as "17.6% (53 / 301 lines)" or "33.6% (101 / 301
    // lines)" depending only on how many rows the user asked to see. A display
    // limit cannot be allowed to change the measurement, so the report is now
    // whole-project and `top_files` travels to the renderer instead.
    let report = run_duplicate_detection(
        &project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        &include,
        &exclude,
    )
    .await?;

    print_duplicate_summary(&report);

    if perf {
        use crate::cli::colors as c;
        let duration = start_time.elapsed();
        eprintln!("\n{}Performance Metrics:{}", c::BOLD, c::RESET);
        eprintln!(
            "   {}Analysis time:{} {}{:.2}ms{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            duration.as_millis(),
            c::RESET
        );
        eprintln!(
            "   {}Files processed:{} {}{}{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            report.file_statistics.len(),
            c::RESET
        );
        eprintln!(
            "   {}Blocks analyzed:{} {}{}{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            report.duplicate_blocks.len(),
            c::RESET
        );
    }

    {
        use crate::cli::colors as c;
        crate::status_eprintln!("\n{}", c::pass("Analysis Complete"));
    }

    write_duplicate_output(&report, format, output, top_files).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod top_files_does_not_change_the_measurement_tests {
    use super::*;

    /// `--top-files N` dropped every duplicate block outside the top N files and
    /// then restated the report from the survivors, so the SAME tree reported
    /// "8 blocks / 12.2%" at `--top-files 1`, "17 / 14.8%" at 3 and "32 / 19.4%"
    /// at 0 — the answer moved with the size of the list the user asked to see.
    #[tokio::test]
    async fn duplication_metrics_are_independent_of_top_files() {
        use tempfile::TempDir;

        let project = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        // Two independent clone families plus one unique file: a top-N cut over
        // files drops the whole second family, which is what used to move the
        // headline numbers.
        let family_a: String = (0..14)
            .map(|line| format!("    let a{} = {line} * 3;\n", line % 3))
            .collect();
        let family_b: String = (0..14)
            .map(|line| format!("    let b{} = {line} - 7;\n", line % 4))
            .collect();
        for name in ["a0.rs", "a1.rs", "a2.rs"] {
            std::fs::write(project.path().join(name), &family_a).unwrap();
        }
        for name in ["b0.rs", "b1.rs"] {
            std::fs::write(project.path().join(name), &family_b).unwrap();
        }
        std::fs::write(
            project.path().join("c0.rs"),
            "fn unique() {\n    let only = 1;\n}\n",
        )
        .unwrap();

        let mut baseline: Option<(u64, u64, String)> = None;
        for top_files in [1usize, 2, 3, 10, 0] {
            let report_path = out.path().join(format!("dup-{top_files}.json"));
            handle_analyze_duplicates(
                project.path().to_path_buf(),
                crate::cli::DuplicateType::Exact,
                0.8,
                5,
                100,
                crate::cli::DuplicateOutputFormat::Json,
                false,
                None,
                None,
                Some(report_path.clone()),
                top_files,
            )
            .await
            .expect("analysis must succeed");

            let json: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
            let measured = (
                json["total_duplicates"].as_u64().unwrap(),
                json["duplicate_lines"].as_u64().unwrap(),
                format!("{:.4}", json["duplication_percentage"].as_f64().unwrap()),
            );

            match &baseline {
                None => baseline = Some(measured),
                Some(expected) => assert_eq!(
                    *expected, measured,
                    "--top-files {top_files} changed the measured duplication"
                ),
            }
        }
    }
}

/// Run duplicate detection analysis
async fn run_duplicate_detection(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<DuplicateReport> {
    detect_duplicates(
        project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        include,
        exclude,
    )
    .await
}

/// Print duplicate analysis summary
fn print_duplicate_summary(report: &DuplicateReport) {
    use crate::cli::colors as c;
    crate::status_eprintln!(
        "{} Found {} duplicate blocks",
        c::pass(""),
        c::number(&report.total_duplicates.to_string())
    );
    crate::status_eprintln!(
        "  {}Duplication:{} {} ({} / {} lines)",
        c::BOLD,
        c::RESET,
        c::pct(report.duplication_percentage as f64, 5.0, 15.0),
        c::number(&report.duplicate_lines.to_string()),
        c::number(&report.total_lines.to_string()),
    );
}

/// Write duplicate output to file or stdout
///
/// `top_files` limits the rendered "Top Files by Duplication" list and nothing
/// else — the machine-readable formats carry the whole measurement.
async fn write_duplicate_output(
    report: &DuplicateReport,
    format: crate::cli::DuplicateOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    let content = match format {
        // The three text formats used to collapse onto one renderer here, so
        // `--format summary` printed the full per-block listing and
        // `--format detailed` printed no more than `--format human`.
        crate::cli::DuplicateOutputFormat::Human => {
            format_text_output(report, top_files, TextDetail::Human)?
        }
        crate::cli::DuplicateOutputFormat::Summary => {
            format_text_output(report, top_files, TextDetail::Summary)?
        }
        crate::cli::DuplicateOutputFormat::Detailed => {
            format_text_output(report, top_files, TextDetail::Detailed)?
        }
        other => format_output(report, other)?,
    };

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Detect duplicate code blocks
async fn detect_duplicates(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<DuplicateReport> {
    let (all_blocks, sources, total_lines, mut file_stats) = collect_code_blocks(
        project_path,
        detection_type.clone(),
        min_lines,
        max_tokens,
        include,
        exclude,
    )
    .await?;

    warn_threshold_has_no_effect(threshold, &detection_type);

    let mut duplicate_blocks = find_duplicate_blocks(all_blocks);

    // Type-3 (near-miss) clones. Hash bucketing above can only ever return
    // groups whose members hash identically, i.e. similarity exactly 1.0, so
    // `structural_similarities` — the count of blocks in [threshold, 1.0) — was
    // 0 for every possible input while `exact_duplicates` beside it reported
    // 176. This pass measures the similarity of blocks that do NOT hash alike.
    duplicate_blocks.extend(find_structural_similarities(
        &sources,
        detection_type,
        threshold,
        min_lines,
    ));
    sort_duplicate_blocks(&mut duplicate_blocks);
    let duplicate_lines = calculate_duplicate_statistics(&duplicate_blocks, &mut file_stats);
    let duplication_percentage = calculate_duplication_percentage(duplicate_lines, total_lines);

    Ok(build_duplicate_report(
        duplicate_blocks,
        duplicate_lines,
        total_lines,
        duplication_percentage,
        file_stats,
    ))
}

/// Collect code blocks from all source files
async fn collect_code_blocks(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<(
    Vec<(String, String, usize, usize, String)>,
    Vec<(PathBuf, String)>,
    usize,
    BTreeMap<String, FileStats>,
)> {
    use crate::services::file_discovery::ProjectFileDiscovery;

    let mut all_blocks = Vec::new();
    // The same file contents the block extractor saw, kept so the near-miss
    // pass measures exactly the file set this report describes rather than
    // walking the tree a second time and possibly disagreeing about it.
    let mut sources = Vec::new();
    let mut total_lines = 0usize;
    let mut file_stats = BTreeMap::new();

    let discovered_files = ProjectFileDiscovery::new(project_path.to_path_buf())
        .discover_files()
        .unwrap_or_default();

    for path in discovered_files {
        let path = path.as_path();

        if should_analyze_file(path, include, exclude) {
            if let Some((blocks, lines_count, content)) =
                process_source_file(path, detection_type.clone(), min_lines, max_tokens).await
            {
                all_blocks.extend(blocks);
                total_lines += lines_count;
                sources.push((path.to_path_buf(), content));

                file_stats.insert(
                    path.to_string_lossy().to_string(),
                    FileStats {
                        duplicate_lines: 0,
                        total_lines: lines_count,
                        duplication_percentage: 0.0,
                    },
                );
            }
        }
    }

    Ok((all_blocks, sources, total_lines, file_stats))
}

/// Check if file should be analyzed
fn should_analyze_file(path: &Path, include: &Option<String>, exclude: &Option<String>) -> bool {
    path.is_file() && is_source_file(path) && should_process_file(path, include, exclude)
}

/// Process a single source file for duplicate detection
async fn process_source_file(
    path: &Path,
    detection_type: crate::cli::DuplicateType,
    min_lines: usize,
    max_tokens: usize,
) -> Option<(Vec<(String, String, usize, usize, String)>, usize, String)> {
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        let lines: Vec<&str> = content.lines().collect();
        let blocks = extract_blocks(&lines, path, min_lines, max_tokens, detection_type);
        let line_count = lines.len();
        Some((blocks, line_count, content))
    } else {
        None
    }
}

/// Whether a detection type asks for near-miss (Type-3) matching.
///
/// `exact` is Type-1 by definition and `renamed` is Type-2: for those two a
/// structural-similarity count of 0 is a MEASUREMENT ("no near-misses were
/// looked for"), not a missing feature. `gapped` (Type-3), `fuzzy` and `all`
/// all promise tolerance for added or removed statements, which is exactly what
/// this pass measures. `semantic` (Type-4) still has no implementation and says
/// so on stderr.
fn near_miss_enabled(detection_type: &crate::cli::DuplicateType) -> bool {
    matches!(
        detection_type,
        crate::cli::DuplicateType::Gapped
            | crate::cli::DuplicateType::Fuzzy
            | crate::cli::DuplicateType::All
    )
}

/// Map a source file to the clone engine's language, or `None` if it does not
/// tokenize that language (Java is discovered by `is_source_file` but the
/// engine has no Java tokenizer, so it is left to the exact pass).
fn engine_language(path: &Path) -> Option<crate::services::duplicate_detector::Language> {
    use crate::services::duplicate_detector::Language;
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Some(Language::Rust),
        Some("ts") => Some(Language::TypeScript),
        Some("js") => Some(Language::JavaScript),
        Some("py") => Some(Language::Python),
        Some("c") => Some(Language::C),
        Some("cpp" | "cc" | "cxx") => Some(Language::Cpp),
        Some("kt" | "kts") => Some(Language::Kotlin),
        _ => None,
    }
}

/// Find Type-3 (near-miss) clone groups and report them with their MEASURED
/// similarity.
///
/// Hash bucketing (`find_duplicate_blocks`) can only return groups whose
/// members hash identically under the detection type's normalisation, so every
/// block it produces has similarity exactly 1.0 — which made
/// `structural_similarities` (blocks in `[threshold, 1.0)`) an unsatisfiable
/// predicate, printing a constant 0 next to a real `exact_duplicates` count.
///
/// This pass runs the project's existing clone engine
/// (`DuplicateDetectionEngine`, the MinHash + LSH detector already used by
/// `DuplicationDefectAnalyzer` and `analyze dag --include-duplicates`) over the
/// same files, at the user's `--threshold`, and keeps only the groups that are
/// NOT exact — the ones hash bucketing cannot see. `--threshold` therefore
/// stops being inert: it is the similarity cut-off this search uses.
fn find_structural_similarities(
    sources: &[(PathBuf, String)],
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
) -> Vec<DuplicateBlock> {
    use crate::services::duplicate_detector::{DuplicateDetectionConfig, DuplicateDetectionEngine};

    if !near_miss_enabled(&detection_type) {
        return Vec::new();
    }

    let files: Vec<_> = sources
        .iter()
        .filter_map(|(path, content)| {
            engine_language(path).map(|lang| (path.clone(), content.clone(), lang))
        })
        .collect();
    if files.is_empty() {
        return Vec::new();
    }

    let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig {
        // The user's cut-off, not the engine's default: a block is a near-miss
        // clone of another when their measured similarity reaches --threshold.
        similarity_threshold: f64::from(threshold),
        min_group_size: 2,
        ..DuplicateDetectionConfig::default()
    });

    let Ok(report) = engine.detect_duplicates(&files) else {
        return Vec::new();
    };

    let contents: BTreeMap<&Path, &str> = sources
        .iter()
        .map(|(path, content)| (path.as_path(), content.as_str()))
        .collect();

    let mut blocks: Vec<DuplicateBlock> = report
        .groups
        .iter()
        // A group whose members are all identical to the representative is an
        // exact clone family; hash bucketing already reported it, and adding it
        // here would double-count it as a near-miss.
        .filter(|group| group.average_similarity < 1.0)
        .filter_map(|group| near_miss_block(group, &contents, min_lines))
        .collect();

    sort_duplicate_blocks(&mut blocks);
    blocks
}

/// Turn one near-miss clone group into a `DuplicateBlock`, or drop it when
/// fewer than two of its members are at least `--min-lines` long.
///
/// `--max-tokens` is NOT applied here, and that is deliberate: it bounds the
/// size of the sliding WINDOW the exact pass hashes (default 128 whitespace
/// tokens, about six lines of Rust). A near-miss fragment is a whole function,
/// so applying the same cap would discard every function in the project and the
/// pass would report nothing at all — which is the failure being fixed.
fn near_miss_block(
    group: &crate::services::duplicate_detector::CloneGroup,
    contents: &BTreeMap<&Path, &str>,
    min_lines: usize,
) -> Option<DuplicateBlock> {
    let mut sites: Vec<(String, usize, usize, String)> = group
        .fragments
        .iter()
        // The length test is on SUBSTANTIVE lines, inside `fragment_text`.
        // Testing the raw span (`end_line - start_line + 1 >= min_lines`) let a
        // fragment that is `min_lines` lines of comment through, and comments
        // normalise away to the empty string — the same nothing-is-a-duplicate
        // hole the exact pass had.
        .filter_map(|f| {
            let content = fragment_text(contents, &f.file, f.start_line, f.end_line, min_lines)?;
            Some((
                f.file.to_string_lossy().to_string(),
                f.start_line,
                f.end_line,
                content,
            ))
        })
        .collect();

    if sites.len() < 2 {
        return None;
    }
    sites.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let lines = sites[0].2 - sites[0].1 + 1;
    let tokens = count_tokens(&sites[0].3);
    let hash = near_miss_hash(&sites);

    let locations = sites
        .into_iter()
        .map(|(file, start_line, end_line, content)| {
            let preview = content.lines().take(3).collect::<Vec<_>>().join("\n");
            DuplicateLocation {
                file,
                start_line,
                end_line,
                content_preview: if content.lines().count() > 3 {
                    format!("{preview}...")
                } else {
                    preview
                },
            }
        })
        .collect();

    Some(DuplicateBlock {
        hash,
        locations,
        lines,
        tokens,
        // MEASURED: the mean MinHash similarity of the group's members to its
        // representative, computed by the same comparison that admitted them.
        #[allow(clippy::cast_possible_truncation)]
        similarity: group.average_similarity as f32,
        // Type-3 by construction: `find_structural_similarities` keeps only the
        // groups whose average similarity is BELOW 1.0, i.e. exactly the ones
        // hash bucketing cannot see.
        clone_type: CloneType::NearMiss,
    })
}

/// The source text of one fragment, normalised the same way the exact pass
/// normalises a block so that `tokens` means the same thing in both — or `None`
/// when the fragment does not carry `min_lines` lines of actual code.
///
/// Both passes apply the SAME floor, from the same predicate
/// (`is_substantive_line`): a fragment made only of comments and blank lines
/// normalises to the empty string, and empty strings are all equal to each
/// other.
fn fragment_text(
    contents: &BTreeMap<&Path, &str>,
    file: &Path,
    start_line: usize,
    end_line: usize,
    min_lines: usize,
) -> Option<String> {
    let content = contents.get(file)?;
    let lines: Vec<&str> = content.lines().collect();
    let start = start_line.checked_sub(1)?;
    let end = end_line.min(lines.len());
    if start >= end {
        return None;
    }
    if substantive_lines(&lines[start..end]).len() < min_lines.max(1) {
        return None;
    }
    Some(normalize_block(&lines[start..end]))
}

/// A stable identity for a near-miss group, prefixed so it can never collide
/// with an exact (`<hex>`) or fuzzy (`f<hex>`) block hash.
fn near_miss_hash(sites: &[(String, usize, usize, String)]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for (file, start, end, _) in sites {
        file.hash(&mut hasher);
        start.hash(&mut hasher);
        end.hash(&mut hasher);
    }
    format!("n{:x}", hasher.finish())
}

/// Count the DISTINCT physical lines that participate in duplication, per file.
///
/// Detection uses an overlapping sliding window, so one physical line is
/// covered by several blocks that hash differently (a.rs 1-11, 1-5, 2-6, 3-7
/// and 4-8 all appear in one run). Summing `block.lines` per location counted
/// each of those lines once per window: two byte-identical 17-line files were
/// reported as `duplicate_lines: 76, total_lines: 17,
/// duplication_percentage: 447.06` — a part 4.5x its own whole, printed to the
/// user as "1. a.rs - 447.1% duplication (76 / 17 lines)".
///
/// A line is duplicated or it is not, however many windows cover it. See
/// `contracts/pmat-no-fabrication-v1.yaml`, equation `measured_or_absent`.
fn calculate_duplicate_statistics(
    duplicate_blocks: &[DuplicateBlock],
    file_stats: &mut BTreeMap<String, FileStats>,
) -> usize {
    let mut duplicated: HashMap<&str, std::collections::HashSet<usize>> = HashMap::new();
    for block in duplicate_blocks {
        for loc in &block.locations {
            let lines = duplicated.entry(loc.file.as_str()).or_default();
            for line in loc.start_line..=loc.end_line {
                lines.insert(line);
            }
        }
    }

    for (path, stats) in file_stats.iter_mut() {
        let counted = duplicated.get(path.as_str()).map_or(0, |set| {
            // A block may name a line past the end of the file if extraction
            // over-ran; clamp so no part can exceed its whole.
            set.iter()
                .filter(|line| **line <= stats.total_lines)
                .count()
        });
        stats.duplicate_lines = counted;
        stats.duplication_percentage = if stats.total_lines > 0 {
            #[allow(clippy::cast_precision_loss)]
            let pct = (counted as f32 / stats.total_lines as f32) * 100.0;
            pct.min(100.0)
        } else {
            0.0
        };
    }

    // The project total is the sum of the per-file totals, so the headline and
    // the rows can never disagree.
    file_stats.values().map(|s| s.duplicate_lines).sum()
}

/// Calculate overall duplication percentage.
///
/// Clamped at 100: distinct duplicated lines can never exceed the lines
/// counted, but the two come from separate passes, so a disagreement must
/// surface as 100%, never as an impossible number.
fn calculate_duplication_percentage(duplicate_lines: usize, total_lines: usize) -> f32 {
    if total_lines > 0 {
        #[allow(clippy::cast_precision_loss)]
        let pct = (duplicate_lines as f32 / total_lines as f32) * 100.0;
        pct.min(100.0)
    } else {
        0.0
    }
}

/// Build the final duplicate report
fn build_duplicate_report(
    duplicate_blocks: Vec<DuplicateBlock>,
    duplicate_lines: usize,
    total_lines: usize,
    duplication_percentage: f32,
    file_stats: BTreeMap<String, FileStats>,
) -> DuplicateReport {
    DuplicateReport {
        total_duplicates: duplicate_blocks.len(),
        duplicate_lines,
        total_lines,
        duplication_percentage,
        duplicate_blocks,
        file_statistics: file_stats,
    }
}
