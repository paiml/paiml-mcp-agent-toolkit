// Badge generation and README update functions for repo-score
// Included from repo_score_handlers.rs — no `use` imports or inner attributes

/// Update README.md with repository health badge
fn update_readme_badge(repo_path: &Path, score: &RepoScore) -> Result<()> {
    debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
    let readme_path = repo_path.join("README.md");

    if !readme_path.exists() {
        println!("⚠️  README.md not found - skipping badge update");
        return Ok(());
    }

    let content = fs::read_to_string(&readme_path).context("Failed to read README.md")?;

    let badge_url = generate_badge_url(score);
    let badge_markdown = format!(
        "<!-- PMAT-REPO-SCORE:START -->\n![Repository Health]({})\n<!-- PMAT-REPO-SCORE:END -->",
        badge_url
    );

    let updated = if content.contains("<!-- PMAT-REPO-SCORE:START -->") {
        // Replace existing badge
        replace_badge_section(&content, &badge_markdown)
    } else {
        // Insert badge after main heading
        insert_badge_after_title(&content, &badge_markdown)
    };

    fs::write(&readme_path, updated).context("Failed to write updated README.md")?;

    println!("✅ Updated README.md with repository health badge");

    Ok(())
}

/// Generate shields.io badge URL from repository score
fn generate_badge_url(score: &RepoScore) -> String {
    let final_score = score.total_score.round() as u8;
    let max_score = 100;

    let color = match score.grade {
        Grade::APlus | Grade::A => "brightgreen",
        Grade::AMinus | Grade::BPlus => "green",
        Grade::B => "yellow",
        Grade::C => "orange",
        Grade::D | Grade::F => "red",
    };

    // URL encode the grade (e.g., "A+" -> "A%2B")
    let grade_str = score.grade.as_str();
    let encoded_grade = grade_str.replace('+', "%2B");

    format!(
        "https://img.shields.io/badge/repo%20health-{}%2F{}%20({})-{}?style=flat-square",
        final_score, max_score, encoded_grade, color
    )
}

/// Replace existing badge section in README
fn replace_badge_section(content: &str, new_badge: &str) -> String {
    let start_marker = "<!-- PMAT-REPO-SCORE:START -->";
    let end_marker = "<!-- PMAT-REPO-SCORE:END -->";

    if let Some(start) = content.find(start_marker) {
        if let Some(end) = content.get(start..).unwrap_or_default().find(end_marker) {
            let end_pos = start + end + end_marker.len();
            let mut result = String::with_capacity(content.len());
            result.push_str(content.get(..start).unwrap_or_default());
            result.push_str(new_badge);
            result.push_str(content.get(end_pos..).unwrap_or_default());
            return result;
        }
    }

    // Fallback: append at end if markers found but parsing failed
    format!("{}\n\n{}", content, new_badge)
}

/// Insert badge after main title (first # heading)
fn insert_badge_after_title(content: &str, badge: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Find first heading line
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("# ") {
            // Insert badge after heading and any immediate blank lines
            let mut insert_pos = i + 1;
            while insert_pos < lines.len() && lines[insert_pos].trim().is_empty() {
                insert_pos += 1;
            }

            let mut result = Vec::with_capacity(lines.len() + 3);
            result.extend_from_slice(&lines[..insert_pos]);
            result.push("");
            result.push(badge);
            result.push("");
            result.extend_from_slice(&lines[insert_pos..]);

            return result.join("\n");
        }
    }

    // No heading found - prepend badge
    format!("{}\n\n{}", badge, content)
}
