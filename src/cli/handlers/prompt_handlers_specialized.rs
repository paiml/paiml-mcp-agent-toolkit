/// Handle PMAT compliance enforcement prompt
async fn handle_comply_prompt(
    min_grade: &str,
    baseline: Option<&PathBuf>,
    roadmap: Option<&PathBuf>,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();
    vars.insert(
        "MIN_GRADE".to_string(),
        Value::String(min_grade.to_string()),
    );

    if let Some(baseline_path) = baseline {
        vars.insert(
            "BASELINE_PATH".to_string(),
            Value::String(baseline_path.display().to_string()),
        );
    }

    if let Some(roadmap_path) = roadmap {
        vars.insert(
            "ROADMAP_PATH".to_string(),
            Value::String(roadmap_path.display().to_string()),
        );
    }

    show_prompt(
        "comply-pmat",
        false,
        vars.into_iter().collect(),
        PromptOutputFormat::Yaml,
        output.clone(),
    )?;
    Ok(())
}

/// Handle book documentation prompt
async fn handle_book_prompt(
    title: Option<&str>,
    book_type: &str,
    target_pages: u32,
    min_pass_rate: u8,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();

    if let Some(t) = title {
        vars.insert("BOOK_TITLE".to_string(), Value::String(t.to_string()));
    }

    vars.insert(
        "BOOK_TYPE".to_string(),
        Value::String(book_type.to_string()),
    );
    vars.insert(
        "TARGET_PAGES".to_string(),
        Value::String(target_pages.to_string()),
    );
    vars.insert(
        "MIN_PASS_RATE".to_string(),
        Value::String(min_pass_rate.to_string()),
    );

    show_prompt(
        "book-documentation",
        false,
        vars.into_iter().collect(),
        PromptOutputFormat::Yaml,
        output.clone(),
    )?;
    Ok(())
}

/// Handle repository image/documentation prompt
async fn handle_repo_image_prompt(
    repo_name: Option<&str>,
    description: Option<&str>,
    github_org: &str,
    language: Option<&str>,
    course_series: bool,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();

    if let Some(name) = repo_name {
        vars.insert("REPO_NAME".to_string(), Value::String(name.to_string()));
    }

    if let Some(desc) = description {
        vars.insert(
            "REPO_DESCRIPTION".to_string(),
            Value::String(desc.to_string()),
        );
    }

    vars.insert(
        "GITHUB_ORG".to_string(),
        Value::String(github_org.to_string()),
    );

    if let Some(lang) = language {
        vars.insert(
            "PRIMARY_LANGUAGE".to_string(),
            Value::String(lang.to_string()),
        );
    }

    vars.insert(
        "COURSE_SERIES".to_string(),
        Value::String(course_series.to_string()),
    );

    show_prompt(
        "repo-image",
        false,
        vars.into_iter().collect(),
        PromptOutputFormat::Yaml,
        output.clone(),
    )?;
    Ok(())
}

/// Handle GitHub issue-driven development prompt
async fn handle_github_issue_prompt(
    issue: &str,
    org: Option<&str>,
    repo: Option<&str>,
    test_cmd: &str,
    build_cmd: &str,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();

    // Determine if issue is a URL or number
    if issue.starts_with("http") {
        vars.insert("ISSUE_URL".to_string(), Value::String(issue.to_string()));
    } else {
        vars.insert("ISSUE_NUMBER".to_string(), Value::String(issue.to_string()));

        if let Some(organization) = org {
            vars.insert(
                "GITHUB_ORG".to_string(),
                Value::String(organization.to_string()),
            );
        }

        if let Some(repository) = repo {
            vars.insert(
                "GITHUB_REPO".to_string(),
                Value::String(repository.to_string()),
            );
        }
    }

    vars.insert("TEST_CMD".to_string(), Value::String(test_cmd.to_string()));
    vars.insert(
        "BUILD_CMD".to_string(),
        Value::String(build_cmd.to_string()),
    );

    show_prompt(
        "github-ticket",
        false,
        vars.into_iter().collect(),
        PromptOutputFormat::Yaml,
        output.clone(),
    )?;
    Ok(())
}
