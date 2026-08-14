// Language dispatch for the Known-Defects database.
//
// #926: which detector grades a file used to be decided at every call site.
// `analyze defects` decided it twice, in two different ways — it walked the
// tree keeping only `ext == "rs"` (handler.rs `collect_rust_files`) and then
// constructed a bare `RustDefectDetector` — while `tdg::critical_defect_gate`
// decided it a third time by matching on `Language`. The Lua rule set, the only
// one that emitted anything other than `Critical`, was therefore unreachable
// from the command: `--severity high|medium|low` retained `total_defects: 0`
// and exit 0 for every project on earth, and a directory of Lua files reported
// `total_files_scanned: 0`.
//
// The mapping from a source file to the rule set that grades it is one rule, so
// it is written once, here. A caller collects files with
// [`SUPPORTED_EXTENSIONS`] and grades them with [`detect_defects`]; there is no
// second place for the two halves to disagree about which languages exist.

/// The file extensions pmat has a Known-Defects rule set for.
///
/// This is the collection filter AND the dispatch key. A file whose extension
/// is absent from this list has no rule set — which is NOT the same as having
/// no defects, and [`detector_for`] returns `None` rather than an empty
/// `Vec<DefectPattern>` so a caller cannot silently render the first as the
/// second.
pub const SUPPORTED_EXTENSIONS: [&str; 8] = ["rs", "lua", "py", "ts", "tsx", "js", "jsx", "mjs"];

/// Which rule set grades a file. `None` means "pmat has no rules for this
/// language", an answer distinct from "this file is clean".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectorLanguage {
    Rust,
    Lua,
    Python,
    TypeScript,
}

impl DetectorLanguage {
    /// The language name, for reporting which rule set produced a finding.
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectorLanguage::Rust => "rust",
            DetectorLanguage::Lua => "lua",
            DetectorLanguage::Python => "python",
            DetectorLanguage::TypeScript => "typescript",
        }
    }
}

/// The rule set for `file_path`, by extension, or `None` when pmat has none.
pub fn detector_for(file_path: &std::path::Path) -> Option<DetectorLanguage> {
    let ext = file_path.extension().and_then(|e| e.to_str())?;
    match ext {
        "rs" => Some(DetectorLanguage::Rust),
        "lua" => Some(DetectorLanguage::Lua),
        "py" => Some(DetectorLanguage::Python),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => Some(DetectorLanguage::TypeScript),
        _ => None,
    }
}

/// True when `file_path` has a rule set — the collection filter that pairs with
/// [`detect_defects`], so a walker can never gather files nothing can grade nor
/// skip files something could.
pub fn is_supported(file_path: &std::path::Path) -> bool {
    detector_for(file_path).is_some()
}

/// Grade `content` with the rule set for `file_path`.
///
/// Returns an empty vector both for a clean file and for a language with no
/// rules; callers that must tell those apart ask [`detector_for`] first.
pub fn detect_defects(content: &str, file_path: &std::path::Path) -> Vec<DefectPattern> {
    match detector_for(file_path) {
        Some(DetectorLanguage::Rust) => RustDefectDetector::new().detect(content, file_path),
        Some(DetectorLanguage::Lua) => LuaDefectDetector::new().detect(content, file_path),
        Some(DetectorLanguage::Python) => PythonDefectDetector::new().detect(content, file_path),
        Some(DetectorLanguage::TypeScript) => {
            TypeScriptDefectDetector::new().detect(content, file_path)
        }
        None => Vec::new(),
    }
}

/// Why `file_path` produced no measurement, or `None` when it is graded.
///
/// The ONE exclusion question `analyze defects` asks per file, so the answer
/// cannot depend on which detector the caller happened to construct. It speaks
/// the vocabulary [`unmeasured::Reason`] already defines rather than inventing
/// a second one: a Rust file defers to
/// [`RustDefectDetector::exclusion_reason`], every other language to
/// [`support_scope::support_reason`], and a language with no rule set at all is
/// [`unmeasured::Reason::NoRuleSet`] — NOT `None`.
///
/// That last arm is the #926 hole: `analyze defects --file main.go` read the
/// file, graded it with nothing, and printed `total_files_scanned: 1,
/// total_defects: 0, exit 0 (no critical defects)`. An analyzer with no rules
/// for Go has not found Go code clean; it has not looked.
pub(crate) fn exclusion_reason(file_path: &std::path::Path) -> Option<unmeasured::Reason> {
    match detector_for(file_path) {
        Some(DetectorLanguage::Rust) => RustDefectDetector::new().exclusion_reason(file_path),
        Some(_) => support_scope::support_reason(file_path),
        None => Some(unmeasured::Reason::NoRuleSet),
    }
}

/// The "is this support code rather than shipped code?" rule for every
/// non-Rust rule set, in one place.
///
/// The Lua detector used to carry its own copy that tested `"/tests/"`,
/// `"/test/"` and `"/spec/"` as substrings of the **absolute** path — the
/// #923 defect, re-committed. It was dormant only because the sole caller
/// (`tdg::critical_defect_gate`) hands the detector the label `<source>`
/// instead of a real path; wiring the detectors into `analyze defects` makes
/// real paths flow in, so any checkout that happens to live under a directory
/// called `tests/`, `spec/` or `test/` would have reported zero defects for its
/// whole Lua/Python/TypeScript tree.
///
/// The directory test therefore runs against the path RELATIVE TO ITS OWN
/// PROJECT ROOT, exactly as [`RustDefectDetector::should_exclude_file`] does,
/// reusing that module's [`source_scope`] rather than restating it.
pub(crate) mod support_scope {
    use std::path::Path;

    /// Directories that hold test or support code across script-language
    /// ecosystems. Superset of Rust's [`super::source_scope::NON_PRODUCTION_DIRS`]
    /// because `spec/`, `specs/` and `__tests__/` are the conventions in the
    /// Lua, Python and JavaScript worlds respectively.
    const SUPPORT_DIRS: [&str; 9] = [
        "tests",
        "test",
        "spec",
        "specs",
        "__tests__",
        "benches",
        "examples",
        "fuzz",
        "node_modules",
    ];

    /// Infixes that mark a test file across all three naming conventions:
    /// `foo_test.lua`, `foo_spec.lua`, `test_foo.py`, `foo_test.py`,
    /// `foo.test.ts`, `foo.spec.ts`.
    const TEST_NAME_MARKERS: [&str; 4] = ["_test.", "_spec.", ".test.", ".spec."];

    /// Why `file_path` is support code rather than shipped code, in the
    /// vocabulary the refusal message is written in, or `None` when it is
    /// shipped code.
    pub(crate) fn support_reason(file_path: &Path) -> Option<super::unmeasured::Reason> {
        let relative = super::source_scope::project_relative_str(file_path);
        if super::source_scope::has_dir_component(&relative, &SUPPORT_DIRS) {
            return Some(super::unmeasured::Reason::NonProductionDir);
        }
        // A checked-in bundle is where a walk that has learned JavaScript does
        // most of its damage: `assets/vendor/mermaid.min.js` alone produced
        // 124 of the 146 findings this command reported on pmat the first time
        // it could see `.js` at all.
        if super::source_scope::is_vendored_or_minified(file_path) {
            return Some(super::unmeasured::Reason::VendoredOrMinified);
        }
        let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let named_as_test = name.starts_with("test_")
            || TEST_NAME_MARKERS.iter().any(|marker| name.contains(marker));
        named_as_test.then_some(super::unmeasured::Reason::TestFileName)
    }

    /// Whether `file_path` is support code rather than shipped code.
    pub(crate) fn is_support_file(file_path: &Path) -> bool {
        support_reason(file_path).is_some()
    }

    /// Whether a trimmed line is a comment in a `#`- or `//`-commented
    /// language. Shared so no rule set grows its own copy.
    pub(crate) fn is_comment(trimmed: &str) -> bool {
        trimmed.starts_with('#')
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("--")
    }
}

/// Build a [`DefectPattern`] from a rule's fixed metadata plus the instances
/// found, or nothing when there are none.
///
/// Every rule set was repeating this `if !instances.is_empty() { push(...) }`
/// block verbatim; sharing it is what keeps a new rule from quietly forgetting
/// the emptiness check and emitting a pattern with zero instances (which the
/// summary would then count as a file with defects).
#[allow(clippy::too_many_arguments)]
pub(crate) fn pattern_from(
    id: &str,
    name: &str,
    severity: Severity,
    fix_recommendation: &str,
    bad_example: &str,
    good_example: &str,
    evidence_description: &str,
    instances: Vec<DefectInstance>,
) -> Option<DefectPattern> {
    if instances.is_empty() {
        return None;
    }
    Some(DefectPattern {
        id: id.to_string(),
        name: name.to_string(),
        severity,
        fix_recommendation: fix_recommendation.to_string(),
        bad_example: bad_example.to_string(),
        good_example: good_example.to_string(),
        evidence_description: evidence_description.to_string(),
        evidence_url: None,
        instances,
    })
}

/// Record one finding at `line_num` (0-based) of `file_path`.
pub(crate) fn instance_at(
    file_path: &std::path::Path,
    line_num: usize,
    text: &str,
) -> DefectInstance {
    DefectInstance {
        file: file_path.to_string_lossy().to_string(),
        line: line_num + 1,
        column: 1,
        code_snippet: text.to_string(),
    }
}
