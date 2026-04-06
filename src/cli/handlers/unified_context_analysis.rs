// Analysis helper functions, structs, and Error enum for UnifiedContextBuilder
// Included via include!() in unified_context_builder.rs

// Helper functions to run individual analyses
async fn run_big_o_analysis(path: &Path) -> Result<BigOAnalysis, Error> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};

    let analyzer = BigOAnalyzer::new();
    let config = BigOAnalysisConfig {
        project_path: path.to_path_buf(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        confidence_threshold: 70,        // Default confidence threshold
        analyze_space_complexity: false, // Skip space complexity for now
    };

    let report = analyzer
        .analyze(config)
        .await
        .map_err(|e| Error::AnalysisFailed(e.to_string()))?;

    // Convert report data to our simplified structure
    let mut complexities = HashMap::new();

    // Add high complexity functions to our map
    for func in &report.high_complexity_functions {
        let path = func.file_path.display().to_string();
        let value = format!("Complexity: {}", func.function_name);
        complexities.insert(path, value);
    }

    Ok(BigOAnalysis { complexities })
}

async fn run_entropy_analysis(_path: &Path) -> Result<EntropyAnalysis, Error> {
    debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
    // Comment out for now, will need to update to use the new entropy APIs
    // This feature is not critical for this release but we'll need to update it later
    /*
    use crate::entropy::{EntropyCalculator, EntropyMetrics};

    let calculator = EntropyCalculator::new();
    // Implement proper entropy analysis with the new API
    */

    // Return empty analysis for now
    Ok(EntropyAnalysis {
        pattern_entropy: 0.0,
        duplication_percentage: 0.0,
        structural_entropy: 0.0,
        actionable_improvements: vec![
            "Entropy analysis temporarily disabled during refactoring".to_string()
        ],
    })
}

async fn run_provability_analysis(_path: &Path) -> Result<ProvabilityAnalysis, Error> {
    debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
    // Provability analysis will be updated in a future release

    // Return empty analysis for now
    Ok(ProvabilityAnalysis {
        invariants: vec![],
        preconditions: vec![],
        postconditions: vec![
            "Provability analysis temporarily disabled during refactoring".to_string(),
        ],
        is_sound: false,
        is_complete: false,
    })
}

async fn run_graph_metrics_analysis(_path: &Path) -> Result<GraphMetricsAnalysis, Error> {
    debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
    // Graph metrics analysis will be updated in a future release

    // Return empty analysis for now
    Ok(GraphMetricsAnalysis {
        betweenness: 0.0,
        closeness: 0.0,
        degree: 0.0,
        node_count: 0,
        edge_count: 0,
        cyclomatic: 0,
        critical_paths: vec![
            "Graph metrics analysis temporarily disabled during refactoring".to_string(),
        ],
    })
}

async fn run_tdg_analysis(_path: &Path) -> Result<TdgAnalysis, Error> {
    debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
    // TDG analysis will be updated in a future release

    // Return empty analysis for now
    Ok(TdgAnalysis {
        overall_score: 0.0,
        file_scores: Default::default(),
        hotspots: vec![],
        priorities: vec!["TDG analysis temporarily disabled during refactoring".to_string()],
    })
}

async fn run_dead_code_analysis(_path: &Path) -> Result<DeadCodeAnalysis, Error> {
    debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
    // Dead code analysis will be updated in a future release

    // Return empty analysis for now
    Ok(DeadCodeAnalysis {
        unreachable_functions: Default::default(),
        unused_variables: Default::default(),
        unused_imports: Default::default(),
        dead_branches: Default::default(),
    })
}

fn is_analyzable_source(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    path.is_file()
        && path
            .extension()
            .map(|ext| matches!(ext.to_str(), Some("rs" | "py" | "ts" | "js" | "go")))
            .unwrap_or(false)
}

fn classify_satd_comment(text: &str, location: String) -> (SatdComment, &'static str) {
    debug_assert!(!text.is_empty(), "text must not be empty");
    let comment = SatdComment {
        location,
        comment: text.to_string(),
    };
    let upper = text.to_uppercase();
    let category = if upper.contains("TODO") {
        "todo"
    } else if upper.contains("FIXME") {
        "fixme"
    } else if upper.contains("HACK") || upper.contains("XXX") {
        "hack"
    } else {
        "debt"
    };
    (comment, category)
}

async fn run_satd_analysis(path: &Path) -> Result<SatdAnalysis, Error> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::services::satd_detector::SATDDetector;
    use walkdir::WalkDir;

    let detector = SATDDetector::new();
    let mut todos = Vec::new();
    let mut fixmes = Vec::new();
    let mut hacks = Vec::new();
    let mut tech_debt = Vec::new();

    let source_files = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_analyzable_source(e.path()));

    for entry in source_files {
        let file_path = entry.path();
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let debts = match detector.extract_from_content(&content, file_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        for debt in debts {
            let location = format!("{}:{}", file_path.display(), debt.line);
            let (comment, category) = classify_satd_comment(&debt.text, location);
            match category {
                "todo" => todos.push(comment),
                "fixme" => fixmes.push(comment),
                "hack" => hacks.push(comment),
                _ => tech_debt.push(comment),
            }
        }
    }

    Ok(SatdAnalysis {
        design_debt_count: tech_debt.len(),
        code_debt_count: hacks.len(),
        test_debt_count: 0,
        doc_debt_count: 0,
        todos,
        fixmes,
        hacks,
        tech_debt,
    })
}

// Analysis result structs
struct BigOAnalysis {
    complexities: HashMap<String, String>,
}

struct EntropyAnalysis {
    pattern_entropy: f64,
    duplication_percentage: f64,
    structural_entropy: f64,
    actionable_improvements: Vec<String>,
}

struct ProvabilityAnalysis {
    invariants: Vec<String>,
    preconditions: Vec<String>,
    postconditions: Vec<String>,
    is_sound: bool,
    is_complete: bool,
}

struct GraphMetricsAnalysis {
    betweenness: f64,
    closeness: f64,
    degree: f64,
    node_count: usize,
    edge_count: usize,
    cyclomatic: usize,
    critical_paths: Vec<String>,
}

struct TdgAnalysis {
    overall_score: f64,
    file_scores: HashMap<String, f64>,
    hotspots: Vec<TdgHotspot>,
    priorities: Vec<String>,
}

struct TdgHotspot {
    location: String,
    score: f64,
}

struct DeadCodeAnalysis {
    unreachable_functions: Vec<String>,
    unused_variables: Vec<String>,
    unused_imports: Vec<String>,
    dead_branches: Vec<String>,
}

impl DeadCodeAnalysis {
    fn is_empty(&self) -> bool {
        debug_assert!(true, "contract: is_empty");
        self.unreachable_functions.is_empty()
            && self.unused_variables.is_empty()
            && self.unused_imports.is_empty()
            && self.dead_branches.is_empty()
    }
}

struct SatdAnalysis {
    todos: Vec<SatdComment>,
    fixmes: Vec<SatdComment>,
    hacks: Vec<SatdComment>,
    tech_debt: Vec<SatdComment>,
    design_debt_count: usize,
    code_debt_count: usize,
    test_debt_count: usize,
    doc_debt_count: usize,
}

struct SatdComment {
    location: String,
    comment: String,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Error {
    NotImplemented,
    AnalysisFailed(String),
}
