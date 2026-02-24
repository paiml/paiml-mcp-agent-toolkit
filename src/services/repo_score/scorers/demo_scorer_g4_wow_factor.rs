// DemoScorer G4: "Wow" Factor (2 points)
// Checks for interactive components, web demos, ASCII art, badges
// Based on Treude et al. (2011) - badges have diminishing returns

fn check_patterns(content: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(content))
            .unwrap_or(false)
    })
}

fn score_badges(content: &str) -> (f64, Finding) {
    let badge_count = content.matches("![").count();
    let badge_score = (badge_count.min(2) as f64) * 0.25;

    if badge_count == 0 {
        return (0.0, Finding {
            severity: Severity::Info,
            category: "Demo Quality".to_string(),
            message: "No badges detected".to_string(),
            location: None,
            impact_points: 0.0,
        });
    }

    if badge_count <= 2 {
        (badge_score, Finding {
            severity: Severity::Success,
            category: "Demo Quality".to_string(),
            message: format!("{} badges detected (professional appearance)", badge_count),
            location: Some("README.md".to_string()),
            impact_points: badge_score,
        })
    } else {
        (0.5, Finding {
            severity: Severity::Info,
            category: "Demo Quality".to_string(),
            message: format!(
                "{} badges detected (excessive - consider reducing to 2-4 essential badges)",
                badge_count
            ),
            location: Some("README.md".to_string()),
            impact_points: 0.5,
        })
    }
}

const DEMO_MEDIA_PATTERNS: &[&str] = &[
    r#"(?i)!\[.*demo.*\]\([^)]+\.gif\)"#,
    r#"(?i)!\[.*demo.*\]\([^)]+\.mp4\)"#,
    r#"(?i)!\[.*demo.*\]\([^)]+\.webm\)"#,
    r#"(?i)<video[^>]+>"#,
    r#"(?i)asciinema\.org"#,
    r#"(?i)!\[.*\]\([^)]+asciicast[^)]+\)"#,
];

const PLAYGROUND_PATTERNS: &[&str] = &[
    r#"(?i)replit\.com"#,
    r#"(?i)codesandbox\.io"#,
    r#"(?i)stackblitz\.com"#,
    r#"(?i)play\.rust-lang\.org"#,
    r#"(?i)playground"#,
    r#"(?i)try\s+it\s+(online|now|live)"#,
];

const ASCII_ART_PATTERNS: &[&str] = &[
    r#"```\n[^\n]*[|/\\─━═╔╗╚╝][^\n]*\n"#,
    r#"<pre>[^<]*[|/\\─━═][^<]*</pre>"#,
    r#"<img[^>]+logo[^>]+>"#,
    r#"<img[^>]+hero[^>]+>"#,
];

const WEB_DEMO_PATHS: &[&str] = &[
    "docs/index.html",
    "demo/index.html",
    "public/index.html",
    "www/index.html",
];

impl DemoScorer {
    /// Score "Wow" Factor (G4: 2 points)
    async fn score_wow_factor(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let mut score: f64 = 0.0;
        let mut findings = vec![];

        let readme_path = repo_path.join("README.md");
        if let Ok(content) = tokio::fs::read_to_string(&readme_path).await {
            if check_patterns(&content, DEMO_MEDIA_PATTERNS) {
                score += 1.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: "Demo GIF/video/screencast found in README".to_string(),
                    location: Some("README.md".to_string()),
                    impact_points: 1.0,
                });
            }

            if check_patterns(&content, PLAYGROUND_PATTERNS) {
                score += 0.75;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: "Interactive playground/demo link detected".to_string(),
                    location: Some("README.md".to_string()),
                    impact_points: 0.75,
                });
            }

            let (badge_pts, badge_finding) = score_badges(&content);
            if badge_pts > 0.0 {
                score += badge_pts;
                findings.push(badge_finding);
            }

            if check_patterns(&content, ASCII_ART_PATTERNS) {
                score += 0.25;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: "Logo/ASCII art detected".to_string(),
                    location: Some("README.md".to_string()),
                    impact_points: 0.25,
                });
            }
        }

        for path in WEB_DEMO_PATHS {
            if repo_path.join(path).exists() {
                score += 0.75;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: format!("Web demo found at {}", path),
                    location: Some(path.to_string()),
                    impact_points: 0.75,
                });
                break;
            }
        }

        score = score.min(2.0_f64);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: "Consider adding demo GIF/video or interactive web demo".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G4".to_string(),
            name: "Wow Factor".to_string(),
            score,
            max_score: 2.0,
            findings,
        })
    }
}
