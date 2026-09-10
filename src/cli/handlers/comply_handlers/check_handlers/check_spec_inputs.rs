// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// PMAT-728 (goal-mode.md §11 step 6): the inputs CB-2110 judges from — every
// specification under docs/specifications/, read here so the service stays
// pure.

/// Every `*.md` under `docs/specifications/` (recursively, `components/`
/// included), in path order, each as its project-relative path and its text.
/// `Ok(empty)` means the directory exists and holds no spec; `Err` names the
/// file that could not be read — an input the rule expected and could not
/// read is a failure, not a pass (goal-mode.md doctrine 2).
pub(crate) fn list_specs(
    project_path: &Path,
) -> std::io::Result<Vec<crate::services::spec_epic::SpecInput>> {
    use crate::services::spec_epic::{SpecInput, SPECS_DIR};
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
        Ok(())
    }
    let root = project_path.join(SPECS_DIR);
    let mut paths = Vec::new();
    walk(&root, &mut paths)?;
    let mut specs = Vec::with_capacity(paths.len());
    for path in paths {
        let rel = path
            .strip_prefix(project_path)
            .unwrap_or(&path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        let text = std::fs::read_to_string(&path)?;
        specs.push(SpecInput { path: rel, text });
    }
    specs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(specs)
}
