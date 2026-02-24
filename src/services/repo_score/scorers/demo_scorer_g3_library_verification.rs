// DemoScorer G3: Library usage verification (Genchi Genbutsu - Posnett et al. 2011)
// Verifies that detected libraries are actually used in source code, not just in manifest

fn build_usage_patterns() -> std::collections::HashMap<&'static str, &'static [&'static str]> {
    [
        ("indicatif", &["ProgressBar", "MultiProgress", "ProgressStyle"][..]),
        ("console", &["Term", "Style", "Color"][..]),
        ("colored", &[".red()", ".green()", ".blue()", "Colorize"][..]),
        ("termcolor", &["StandardStream", "ColorChoice", "WriteColor"][..]),
        ("ratatui", &["Terminal", "Frame", "Widget"][..]),
        ("crossterm", &["execute!", "queue!", "cursor::"][..]),
        ("comfy-table", &["Table", "Row", "Cell"][..]),
        ("prettytable", &["Table", "row!"][..]),
        ("dialoguer", &["Select", "Input", "Confirm"][..]),
        ("owo-colors", &["OwoColorize", ".style("][..]),
        ("chalk", &["chalk.red", "chalk.green", "chalk.blue"][..]),
        ("ora", &["Ora", "ora("][..]),
        ("ink", &["render", "<Box", "<Text"][..]),
        ("rich", &["Console", "Table", "Progress"][..]),
        ("tqdm", &["tqdm(", "trange"][..]),
        ("colorama", &["Fore.", "Back.", "Style."][..]),
        ("click", &["@click.command", "click.echo"][..]),
        ("typer", &["typer.Typer", "@app.command"][..]),
    ]
    .into_iter()
    .collect()
}

fn is_source_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    ["rs", "py", "js", "ts"].contains(&ext)
}

fn file_contains_usage(
    content: &str,
    libs: &[&str],
    usage_patterns: &std::collections::HashMap<&str, &[&str]>,
) -> bool {
    for lib in libs {
        if let Some(patterns) = usage_patterns.get(lib) {
            for pattern in *patterns {
                if content.contains(pattern) {
                    return true;
                }
            }
        }
    }
    false
}

impl DemoScorer {
    /// Verify that detected libraries are actually used in source code
    /// Implements Genchi Genbutsu principle - go and see the actual usage
    async fn verify_library_usage(&self, src_path: &Path, libs: &[&str]) -> bool {
        let usage_patterns = build_usage_patterns();

        if let Ok(entries) = std::fs::read_dir(src_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && is_source_file(&path) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if file_contains_usage(&content, libs, &usage_patterns) {
                            return true;
                        }
                    }
                } else if path.is_dir() {
                    if Box::pin(self.verify_library_usage(&path, libs)).await {
                        return true;
                    }
                }
            }
        }
        false
    }
}
