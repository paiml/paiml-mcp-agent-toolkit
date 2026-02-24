#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_prompts_parse() {
        for (name, yaml) in PROMPTS {
            let result = WorkflowPrompt::from_yaml(yaml);
            assert!(
                result.is_ok(),
                "Failed to parse prompt {}: {:?}",
                name,
                result.err()
            );
        }
    }

    #[test]
    fn test_all_prompts_have_required_fields() {
        for (name, yaml) in PROMPTS {
            let prompt = WorkflowPrompt::from_yaml(yaml).unwrap();
            assert!(!prompt.name.is_empty(), "Prompt {name} missing name");
            assert!(
                !prompt.description.is_empty(),
                "Prompt {name} missing description"
            );
            assert!(
                !prompt.category.is_empty(),
                "Prompt {name} missing category"
            );
            assert!(
                !prompt.priority.is_empty(),
                "Prompt {name} missing priority"
            );
            assert!(
                !prompt.prompt.is_empty(),
                "Prompt {name} missing prompt text"
            );
        }
    }

    #[test]
    fn test_prompt_names_match_keys() {
        for (key, yaml) in PROMPTS {
            let prompt = WorkflowPrompt::from_yaml(yaml).unwrap();
            assert_eq!(
                &prompt.name, key,
                "Prompt name mismatch: key={key}, name={}",
                prompt.name
            );
        }
    }

    #[tokio::test]
    async fn test_handle_prompt_list() {
        let result = handle_prompt(None, true, false, vec![], PromptOutputFormat::Yaml, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_show_code_coverage() {
        let result = handle_prompt(
            Some("code-coverage".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_show_variables() {
        let result = handle_prompt(
            Some("code-coverage".to_string()),
            false,
            true,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_not_found() {
        let result = handle_prompt(
            Some("nonexistent".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_prompt_json_format() {
        let result = handle_prompt(
            Some("continue".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Json,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_text_format() {
        let result = handle_prompt(
            Some("debug".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Text,
            None,
        )
        .await;
        assert!(result.is_ok());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_handle_prompt_with_valid_names(name in prop::sample::select(vec![
            "code-coverage",
            "clean-repo-cruft",
            "continue",
            "assert-cmd-testing",
            "documentation",
            "debug",
            "mutation-testing",
            "performance-optimization",
            "quality-enforcement",
            "refactor-hotspots",
            "security-audit",
        ])) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(handle_prompt(
                Some(name.to_string()),
                false,
                false,
                vec![],
                PromptOutputFormat::Yaml,
                None,
            ));
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_invalid_prompt_name_fails(invalid_name in "[a-z]{1,20}") {
            // Only test names that don't match our valid prompts
            let valid_names = ["code-coverage", "clean-repo-cruft", "continue",
                "assert-cmd-testing", "documentation", "debug",
                "mutation-testing", "performance-optimization",
                "quality-enforcement", "refactor-hotspots", "security-audit"];

            if !valid_names.contains(&invalid_name.as_str()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(handle_prompt(
                    Some(invalid_name),
                    false,
                    false,
                    vec![],
                    PromptOutputFormat::Yaml,
                    None,
                ));
                prop_assert!(result.is_err());
            }
        }
    }
}
