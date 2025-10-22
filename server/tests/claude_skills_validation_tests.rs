//! RED Phase Tests for Claude Code Skills Validation
//!
//! These tests define the expected behavior of Claude Skills integration.
//! Following EXTREME TDD: Write tests first, watch them fail, then implement.
//!
//! Test Coverage Requirements (from specification):
//! - Skill parsing: 100%
//! - Skill validation: 95%
//! - Skill execution: 85%
//! - Error handling: 90%

use std::fs;
use std::path::{Path, PathBuf};

/// Represents a Claude Code Skill with YAML frontmatter
#[derive(Debug, Clone)]
struct ClaudeSkill {
    name: String,
    description: String,
    allowed_tools: Vec<String>,
    prompt_content: String,
    file_path: PathBuf,
}

/// Parse a skill.md file into a ClaudeSkill struct
fn parse_skill_file(path: &Path) -> Result<ClaudeSkill, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read skill file: {}", e))?;

    // Split YAML frontmatter from prompt content
    let parts: Vec<&str> = content.split("---").collect();
    if parts.len() < 3 {
        return Err("Invalid skill format: missing YAML frontmatter".to_string());
    }

    let yaml_content = parts[1];
    let prompt_content = parts[2..].join("---").trim().to_string();

    // Parse YAML fields (simple parsing for now, can use serde_yaml later)
    let mut name = String::new();
    let mut description = String::new();
    let mut allowed_tools = Vec::new();

    for line in yaml_content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            name = line.strip_prefix("name:").unwrap().trim().to_string();
        } else if line.starts_with("allowed-tools:") {
            let tools_str = line.strip_prefix("allowed-tools:").unwrap().trim();
            allowed_tools = tools_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        // Note: description parsing happens in the multiline fallback code below
    }

    // Handle multiline description (YAML pipe `|` syntax)
    if description.is_empty() {
        let mut in_description = false;
        let mut desc_lines = Vec::new();
        for line in yaml_content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("description:") {
                // Check if description is inline (not pipe syntax)
                if trimmed.len() > "description:".len() {
                    let inline = trimmed.strip_prefix("description:").unwrap().trim();
                    if !inline.is_empty() && inline != "|" {
                        description = inline.to_string();
                        in_description = false;
                        continue;
                    } else if inline == "|" {
                        // Pipe syntax - enable multiline mode
                        in_description = true;
                        continue;
                    }
                }
                // No inline content, enable multiline mode
                in_description = true;
                continue;
            }

            if in_description {
                if trimmed.starts_with("allowed-tools:") || trimmed.starts_with("name:") {
                    break;
                }
                // Skip the pipe character if it's on its own line
                if trimmed == "|" {
                    continue;
                }
                // Add non-empty lines
                if !trimmed.is_empty() {
                    // Remove leading dashes for lists
                    let cleaned = if trimmed.starts_with("- ") {
                        trimmed.strip_prefix("- ").unwrap()
                    } else {
                        trimmed
                    };
                    desc_lines.push(cleaned.to_string());
                }
            }
        }
        description = desc_lines.join(" ");
    }

    if name.is_empty() {
        return Err("Missing required field: name".to_string());
    }
    if description.is_empty() {
        return Err("Missing required field: description".to_string());
    }
    if allowed_tools.is_empty() || (allowed_tools.len() == 1 && allowed_tools[0].is_empty()) {
        return Err("Missing required field: allowed-tools".to_string());
    }

    Ok(ClaudeSkill {
        name,
        description,
        allowed_tools,
        prompt_content,
        file_path: path.to_path_buf(),
    })
}

/// Get the workspace root directory (parent of server/)
fn get_workspace_root() -> PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    // If we're in server/ directory, go up one level
    if path.ends_with("server") {
        path.pop();
    }
    path
}

#[cfg(test)]
mod red_phase_skill_validation_tests {
    use super::*;

    /// RED Test: pmat-quality skill file must exist
    #[test]
    fn test_pmat_quality_skill_file_exists() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        assert!(
            skill_path.exists(),
            "pmat-quality skill file must exist at {:?}. Current dir: {:?}",
            skill_path,
            std::env::current_dir()
        );
    }

    /// RED Test: pmat-quality skill must have valid YAML frontmatter
    #[test]
    fn test_pmat_quality_yaml_valid() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        if !skill_path.exists() {
            // Skip if file doesn't exist (other test will catch this)
            return;
        }

        let skill = parse_skill_file(&skill_path).expect("Failed to parse skill file");

        // Debug: print what we parsed
        println!("Parsed name: '{}'", skill.name);
        println!("Parsed description length: {}", skill.description.len());
        println!("Parsed description: '{}'", skill.description);
        println!("Parsed allowed_tools: {:?}", skill.allowed_tools);

        // Validate required fields
        assert!(!skill.name.is_empty(), "Skill name must not be empty");
        assert!(
            skill.description.len() > 50,
            "Skill description must be meaningful (>50 chars), got: '{}'",
            skill.description
        );
        assert!(
            !skill.allowed_tools.is_empty(),
            "Skill must specify allowed tools"
        );
    }

    /// RED Test: pmat-quality skill name must match convention
    #[test]
    fn test_pmat_quality_naming_convention() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        if !skill_path.exists() {
            return;
        }

        let skill = parse_skill_file(&skill_path).expect("Failed to parse skill file");

        assert!(
            skill.name.contains("PMAT") || skill.name.contains("Quality"),
            "Skill name should reference PMAT or Quality"
        );
    }

    /// RED Test: pmat-quality skill must specify required tools
    #[test]
    fn test_pmat_quality_required_tools() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        if !skill_path.exists() {
            return;
        }

        let skill = parse_skill_file(&skill_path).expect("Failed to parse skill file");

        // pmat-quality should at minimum have Bash tool (to run pmat commands)
        let tools_lower: Vec<String> = skill.allowed_tools
            .iter()
            .map(|t| t.to_lowercase())
            .collect();

        assert!(
            tools_lower.contains(&"bash".to_string()),
            "pmat-quality skill must include Bash tool to run pmat commands"
        );
    }

    /// RED Test: pmat-quality skill prompt must contain usage instructions
    #[test]
    fn test_pmat_quality_prompt_completeness() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        if !skill_path.exists() {
            return;
        }

        let skill = parse_skill_file(&skill_path).expect("Failed to parse skill file");

        // Prompt should be substantial and contain key terms
        assert!(
            skill.prompt_content.len() > 200,
            "Skill prompt must be detailed (>200 chars)"
        );

        // Check for essential content
        let content_lower = skill.prompt_content.to_lowercase();
        assert!(
            content_lower.contains("pmat"),
            "Prompt must reference pmat tool"
        );
        assert!(
            content_lower.contains("quality") || content_lower.contains("complexity"),
            "Prompt must reference quality or complexity analysis"
        );
    }

    /// RED Test: pmat-quality skill must document activation triggers
    #[test]
    fn test_pmat_quality_activation_triggers() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        if !skill_path.exists() {
            return;
        }

        let skill = parse_skill_file(&skill_path).expect("Failed to parse skill file");

        // Description or prompt should mention when to activate
        let full_text = format!("{} {}", skill.description, skill.prompt_content).to_lowercase();

        assert!(
            full_text.contains("when") || full_text.contains("use this"),
            "Skill must document activation triggers"
        );
    }

    /// RED Test: Skill prompt must include example usage
    #[test]
    fn test_pmat_quality_includes_examples() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        if !skill_path.exists() {
            return;
        }

        let skill = parse_skill_file(&skill_path).expect("Failed to parse skill file");

        let content_lower = skill.prompt_content.to_lowercase();
        assert!(
            content_lower.contains("example") || content_lower.contains("usage"),
            "Skill prompt should include usage examples"
        );
    }
}

#[cfg(test)]
mod red_phase_skill_structure_tests {
    use super::*;

    /// RED Test: Skills directory must exist
    #[test]
    fn test_skills_directory_exists() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");
        assert!(
            skills_dir.exists() && skills_dir.is_dir(),
            "Skills directory must exist at .claude/skills"
        );
    }

    /// RED Test: pmat-quality directory must exist
    #[test]
    fn test_pmat_quality_directory_exists() {
        let workspace_root = get_workspace_root();
        let quality_dir = workspace_root.join(".claude/skills/pmat-quality");
        assert!(
            quality_dir.exists() && quality_dir.is_dir(),
            "pmat-quality directory must exist"
        );
    }

    /// RED Test: Skill file must be named skill.md
    #[test]
    fn test_skill_file_naming_convention() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-quality/skill.md");
        assert!(
            skill_path.exists(),
            "Skill file must be named skill.md (convention)"
        );
    }
}

#[cfg(test)]
mod red_phase_error_handling_tests {
    use super::*;

    /// RED Test: Parser must handle missing YAML frontmatter
    #[test]
    fn test_parse_skill_missing_frontmatter() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("invalid_skill.md");

        fs::write(&test_file, "Just some content without frontmatter").unwrap();

        let result = parse_skill_file(&test_file);
        assert!(result.is_err(), "Parser should reject file without frontmatter");

        fs::remove_file(test_file).ok();
    }

    /// RED Test: Parser must handle missing required fields
    #[test]
    fn test_parse_skill_missing_name() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("missing_name.md");

        let content = r#"---
description: Some description
allowed-tools: Bash
---
Prompt content
"#;
        fs::write(&test_file, content).unwrap();

        let result = parse_skill_file(&test_file);
        assert!(result.is_err(), "Parser should reject skill missing name field");

        fs::remove_file(test_file).ok();
    }

    /// RED Test: Parser must handle empty allowed-tools
    #[test]
    fn test_parse_skill_empty_tools() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("empty_tools.md");

        let content = r#"---
name: Test Skill
description: Test description
allowed-tools:
---
Prompt content
"#;
        fs::write(&test_file, content).unwrap();

        let result = parse_skill_file(&test_file);
        assert!(
            result.is_err(),
            "Parser should reject skill with empty allowed-tools"
        );

        fs::remove_file(test_file).ok();
    }
}

#[cfg(test)]
mod phase_2_skill_discovery_tests {
    use super::*;

    /// Phase 2 Test: Discover all skill directories
    #[test]
    fn test_discover_all_skills() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");

        assert!(skills_dir.exists(), "Skills directory must exist");

        let entries = fs::read_dir(&skills_dir)
            .expect("Should be able to read skills directory");

        let skill_dirs: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        // Should have exactly 5 skills
        assert_eq!(
            skill_dirs.len(), 5,
            "Should have 5 skill directories, found: {:?}",
            skill_dirs
        );

        // Verify expected skills are present
        let expected_skills = vec![
            "pmat-quality",
            "pmat-context",
            "pmat-refactor",
            "pmat-tech-debt",
            "pmat-multi-lang",
        ];

        for expected in &expected_skills {
            assert!(
                skill_dirs.contains(&expected.to_string()),
                "Missing expected skill: {}",
                expected
            );
        }
    }

    /// Phase 2 Test: All skills have skill.md files
    #[test]
    fn test_all_skills_have_skill_files() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");

        let expected_skills = vec![
            "pmat-quality",
            "pmat-context",
            "pmat-refactor",
            "pmat-tech-debt",
            "pmat-multi-lang",
        ];

        for skill_name in &expected_skills {
            let skill_path = skills_dir.join(skill_name).join("skill.md");
            assert!(
                skill_path.exists(),
                "Skill {} must have skill.md file at {:?}",
                skill_name,
                skill_path
            );
        }
    }

    /// Phase 2 Test: All skills parse successfully
    #[test]
    fn test_all_skills_parse_successfully() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");

        let expected_skills = vec![
            "pmat-quality",
            "pmat-context",
            "pmat-refactor",
            "pmat-tech-debt",
            "pmat-multi-lang",
        ];

        for skill_name in &expected_skills {
            let skill_path = skills_dir.join(skill_name).join("skill.md");
            let result = parse_skill_file(&skill_path);
            assert!(
                result.is_ok(),
                "Skill {} failed to parse: {:?}",
                skill_name,
                result.err()
            );
        }
    }
}

#[cfg(test)]
mod phase_2_all_skills_validation_tests {
    use super::*;

    /// Phase 2 Test: pmat-context skill validation
    #[test]
    fn test_pmat_context_skill_valid() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-context/skill.md");
        let skill = parse_skill_file(&skill_path).expect("Failed to parse pmat-context skill");

        // Validate required fields
        assert!(!skill.name.is_empty());
        assert!(skill.description.len() > 50);
        assert!(!skill.allowed_tools.is_empty());
        assert!(skill.prompt_content.len() > 200);

        // Validate name contains expected keywords
        let name_lower = skill.name.to_lowercase();
        assert!(
            name_lower.contains("context") || name_lower.contains("pmat"),
            "Skill name should reference context or PMAT"
        );

        // Validate required tools
        let tools_lower: Vec<String> = skill.allowed_tools
            .iter()
            .map(|t| t.to_lowercase())
            .collect();
        assert!(
            tools_lower.contains(&"bash".to_string()),
            "pmat-context must include Bash tool"
        );
    }

    /// Phase 2 Test: pmat-refactor skill validation
    #[test]
    fn test_pmat_refactor_skill_valid() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-refactor/skill.md");
        let skill = parse_skill_file(&skill_path).expect("Failed to parse pmat-refactor skill");

        assert!(!skill.name.is_empty());
        assert!(skill.description.len() > 50);
        assert!(!skill.allowed_tools.is_empty());
        assert!(skill.prompt_content.len() > 200);

        let name_lower = skill.name.to_lowercase();
        assert!(
            name_lower.contains("refactor") || name_lower.contains("pmat"),
            "Skill name should reference refactoring or PMAT"
        );

        // Validate refactor-specific tools
        let tools_lower: Vec<String> = skill.allowed_tools
            .iter()
            .map(|t| t.to_lowercase())
            .collect();
        assert!(
            tools_lower.contains(&"bash".to_string()) && tools_lower.contains(&"edit".to_string()),
            "pmat-refactor should include Bash and Edit tools"
        );
    }

    /// Phase 2 Test: pmat-tech-debt skill validation
    #[test]
    fn test_pmat_tech_debt_skill_valid() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-tech-debt/skill.md");
        let skill = parse_skill_file(&skill_path).expect("Failed to parse pmat-tech-debt skill");

        assert!(!skill.name.is_empty());
        assert!(skill.description.len() > 50);
        assert!(!skill.allowed_tools.is_empty());
        assert!(skill.prompt_content.len() > 200);

        let content_lower = skill.prompt_content.to_lowercase();
        assert!(
            content_lower.contains("satd") || content_lower.contains("technical debt"),
            "Prompt must reference SATD or technical debt"
        );
    }

    /// Phase 2 Test: pmat-multi-lang skill validation
    #[test]
    fn test_pmat_multi_lang_skill_valid() {
        let workspace_root = get_workspace_root();
        let skill_path = workspace_root.join(".claude/skills/pmat-multi-lang/skill.md");
        let skill = parse_skill_file(&skill_path).expect("Failed to parse pmat-multi-lang skill");

        assert!(!skill.name.is_empty());
        assert!(skill.description.len() > 50);
        assert!(!skill.allowed_tools.is_empty());
        assert!(skill.prompt_content.len() > 200);

        let content_lower = skill.prompt_content.to_lowercase();
        assert!(
            content_lower.contains("multi-language") || content_lower.contains("polyglot"),
            "Prompt must reference multi-language or polyglot"
        );
    }

    /// Phase 2 Test: All skills have activation triggers documented
    #[test]
    fn test_all_skills_have_activation_triggers() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");

        let skills = vec![
            "pmat-quality",
            "pmat-context",
            "pmat-refactor",
            "pmat-tech-debt",
            "pmat-multi-lang",
        ];

        for skill_name in &skills {
            let skill_path = skills_dir.join(skill_name).join("skill.md");
            let skill = parse_skill_file(&skill_path)
                .expect(&format!("Failed to parse {} skill", skill_name));

            let full_text = format!("{} {}", skill.description, skill.prompt_content).to_lowercase();
            assert!(
                full_text.contains("when") || full_text.contains("use this"),
                "Skill {} must document activation triggers",
                skill_name
            );
        }
    }

    /// Phase 2 Test: All skills include examples
    #[test]
    fn test_all_skills_include_examples() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");

        let skills = vec![
            "pmat-quality",
            "pmat-context",
            "pmat-refactor",
            "pmat-tech-debt",
            "pmat-multi-lang",
        ];

        for skill_name in &skills {
            let skill_path = skills_dir.join(skill_name).join("skill.md");
            let skill = parse_skill_file(&skill_path)
                .expect(&format!("Failed to parse {} skill", skill_name));

            let content_lower = skill.prompt_content.to_lowercase();
            assert!(
                content_lower.contains("example") || content_lower.contains("workflow"),
                "Skill {} should include examples or workflow",
                skill_name
            );
        }
    }

    /// Phase 2 Test: All skills reference PMAT tool
    #[test]
    fn test_all_skills_reference_pmat() {
        let workspace_root = get_workspace_root();
        let skills_dir = workspace_root.join(".claude/skills");

        let skills = vec![
            "pmat-quality",
            "pmat-context",
            "pmat-refactor",
            "pmat-tech-debt",
            "pmat-multi-lang",
        ];

        for skill_name in &skills {
            let skill_path = skills_dir.join(skill_name).join("skill.md");
            let skill = parse_skill_file(&skill_path)
                .expect(&format!("Failed to parse {} skill", skill_name));

            let full_text = format!("{} {}", skill.description, skill.prompt_content).to_lowercase();
            assert!(
                full_text.contains("pmat"),
                "Skill {} must reference PMAT tool",
                skill_name
            );
        }
    }
}
