//! Run every wave-2 detector over a directory of workflow files.

use super::detectors::{detect_gs03, detect_gs04, detect_gs05, detect_gs12, detect_gs14};
use super::model::{Report, Workflow};
use std::path::Path;

/// Scan `dir` (normally `.github/workflows`). Unparseable files are `not_measured`.
pub fn scan_workflows(
    dir: &Path,
    reachable: &dyn Fn(&str, &str) -> Option<bool>,
) -> std::io::Result<Report> {
    let mut report = Report::default();
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("yml" | "yaml")))
        .collect();
    files.sort();
    for path in files {
        let name = path.display().to_string();
        let text = std::fs::read_to_string(&path)?;
        match Workflow::parse(&name, &text) {
            Ok(wf) => {
                report.extend(detect_gs03(&wf));
                report.extend(detect_gs04(&wf));
                report.extend(detect_gs05(&wf));
                report.extend(detect_gs12(&wf, reachable));
                report.extend(detect_gs14(&wf));
            }
            Err(e) => report.not_measured.push(e),
        }
    }
    Ok(report)
}
