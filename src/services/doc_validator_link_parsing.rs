/// Extracts all markdown links from a file
///
/// # Examples
///
/// ```ignore
/// use pmat::services::doc_validator::extract_links;
/// use std::path::PathBuf;
///
/// let content = "[example](https://example.com) and [local](./file.md)";
/// let links = extract_links(content, &PathBuf::from("test.md"));
/// assert_eq!(links.len(), 2);
/// ```ignore
pub fn extract_links(content: &str, source_file: &Path) -> Vec<Link> {
    debug_assert!(source_file.exists(), "source_file must exist: {}", source_file.display());
    let mut links = Vec::new();
    let regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("Invalid regex");

    for (line_num, line) in content.lines().enumerate() {
        // Skip code blocks (lines starting with backticks)
        if line.trim_start().starts_with("```") {
            continue;
        }

        for cap in regex.captures_iter(line) {
            let text = cap[1].to_string();
            let target = cap[2].to_string();
            let link_type = classify_link(&target);

            links.push(Link {
                text,
                target,
                source_file: source_file.to_path_buf(),
                line_number: line_num + 1,
                link_type,
            });
        }
    }

    links
}

/// Classifies a link target into its type
///
/// # Examples
///
/// ```ignore
/// use pmat::services::doc_validator::{classify_link, LinkType};
///
/// assert_eq!(classify_link("https://example.com"), LinkType::ExternalHttp);
/// assert_eq!(classify_link("./local.md"), LinkType::Internal);
/// assert_eq!(classify_link("#anchor"), LinkType::Anchor);
/// assert_eq!(classify_link("mailto:user@example.com"), LinkType::Email);
/// ```ignore
pub fn classify_link(target: &str) -> LinkType {
    if target.starts_with("http://") || target.starts_with("https://") {
        LinkType::ExternalHttp
    } else if target.starts_with('#') {
        LinkType::Anchor
    } else if target.starts_with("mailto:") {
        LinkType::Email
    } else if target.contains("://") {
        LinkType::Other(
            target
                .split("://")
                .next()
                .expect("split should have at least one element")
                .to_string(),
        )
    } else {
        LinkType::Internal
    }
}

/// Normalizes a path by resolving `.` and `..` components
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory
            }
            _ => {
                components.push(component);
            }
        }
    }

    components.iter().collect()
}
