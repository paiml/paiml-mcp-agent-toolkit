// Property-based tests for project_meta types
// Included from project_meta.rs - shares parent module scope
// NO `use` imports or `#!` inner attributes allowed

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_make_target_roundtrip(
            name in "[a-z_]+",
            recipe in "[a-z -]+",
        ) {
            let target = MakeTarget {
                name: name.clone(),
                deps: vec![],
                recipe_summary: recipe.clone(),
            };

            let json = serde_json::to_string(&target).unwrap();
            let deserialized: MakeTarget = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.recipe_summary, recipe);
        }

        #[test]
        fn test_compressed_section_roundtrip(
            title in "[A-Za-z ]+",
            content in "[A-Za-z ]+",
        ) {
            let section = CompressedSection {
                title: title.clone(),
                content: content.clone(),
            };

            let json = serde_json::to_string(&section).unwrap();
            let deserialized: CompressedSection = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.title, title);
            prop_assert_eq!(deserialized.content, content);
        }

        #[test]
        fn test_build_info_toolchain_fallback(
            toolchain_opt in prop::option::of("[a-z]+"),
        ) {
            let compressed = CompressedMakefile {
                variables: vec![],
                targets: vec![],
                detected_toolchain: toolchain_opt.clone(),
                key_dependencies: vec![],
            };

            let build_info = BuildInfo::from_makefile(compressed);

            match toolchain_opt {
                Some(t) => prop_assert_eq!(build_info.toolchain, t),
                None => prop_assert_eq!(build_info.toolchain, "unknown"),
            }
        }
    }
}
