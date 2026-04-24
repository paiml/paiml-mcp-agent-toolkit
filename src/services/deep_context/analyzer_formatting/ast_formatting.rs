// AST section formatting, categorization, and helper structs for DeepContextAnalyzer

use super::{DeepContextAnalyzer, EnhancedFileContext};

// Helper structs for organizing AST items
#[derive(Debug, Clone)]
pub(crate) struct CategorizedAstItems {
    functions: Vec<AstFunction>,
    structs: Vec<AstStruct>,
    enums: Vec<AstEnum>,
    traits: Vec<AstTrait>,
    impls: Vec<AstImpl>,
    modules: Vec<AstModule>,
    uses: Vec<AstUse>,
}

impl CategorizedAstItems {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            modules: Vec::new(),
            uses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct AstFunction {
    name: String,
    visibility: String,
    is_async: bool,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstStruct {
    name: String,
    visibility: String,
    fields_count: usize,
    derives: Vec<String>,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstEnum {
    name: String,
    visibility: String,
    variants_count: usize,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstTrait {
    name: String,
    visibility: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstImpl {
    type_name: String,
    trait_name: Option<String>,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstModule {
    name: String,
    visibility: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstUse {
    path: String,
    line: usize,
}

impl DeepContextAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Format enhanced ast section.
    pub fn format_enhanced_ast_section(
        &self,
        output: &mut String,
        ast_contexts: &[EnhancedFileContext],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Enhanced AST Analysis\n")?;

        for context in ast_contexts {
            self.format_single_file_ast(output, context)?;
        }

        Ok(())
    }

    fn format_single_file_ast(
        &self,
        output: &mut String,
        context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        writeln!(output, "### {}\n", context.base.path)?;
        writeln!(output, "**Language:** {}", context.base.language)?;
        writeln!(output, "**Total Symbols:** {}", context.base.items.len())?;

        // Categorize AST items
        let categorized_items = self.categorize_ast_items(&context.base.items);

        // Write summary counts
        self.write_ast_summary(output, &categorized_items)?;

        // Write detailed breakdowns
        self.write_ast_details(output, &categorized_items)?;

        // Write metrics
        self.write_file_metrics(output, context)?;

        Ok(())
    }

    fn categorize_ast_items(
        &self,
        items: &[crate::services::context::AstItem],
    ) -> CategorizedAstItems {
        let mut categorized = CategorizedAstItems::new();

        for item in items {
            self.categorize_single_ast_item(item, &mut categorized);
        }

        categorized
    }

    fn categorize_single_ast_item(
        &self,
        item: &crate::services::context::AstItem,
        categorized: &mut CategorizedAstItems,
    ) {
        match item {
            crate::services::context::AstItem::Function {
                name,
                visibility,
                is_async,
                line,
            } => {
                categorized.functions.push(AstFunction {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    is_async: *is_async,
                    line: *line,
                });
            }
            crate::services::context::AstItem::Struct {
                name,
                visibility,
                fields_count,
                derives,
                line,
            } => {
                categorized.structs.push(AstStruct {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    fields_count: *fields_count,
                    derives: derives.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Enum {
                name,
                visibility,
                variants_count,
                line,
            } => {
                categorized.enums.push(AstEnum {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    variants_count: *variants_count,
                    line: *line,
                });
            }
            crate::services::context::AstItem::Trait {
                name,
                visibility,
                line,
            } => {
                categorized.traits.push(AstTrait {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Impl {
                type_name,
                trait_name,
                line,
            } => {
                categorized.impls.push(AstImpl {
                    type_name: type_name.clone(),
                    trait_name: trait_name.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Module {
                name,
                visibility,
                line,
            } => {
                categorized.modules.push(AstModule {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Use { path, line } => {
                categorized.uses.push(AstUse {
                    path: path.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Import {
                module,
                items,
                alias,
                line,
            } => {
                let path = self.format_import_path(module, items, alias);
                categorized.uses.push(AstUse { path, line: *line });
            }
        }
    }

    fn format_import_path(&self, module: &str, items: &[String], alias: &Option<String>) -> String {
        if let Some(alias) = alias {
            format!("{module} as {alias}")
        } else if !items.is_empty() {
            format!("{} ({})", module, items.join(", "))
        } else {
            module.to_string()
        }
    }

    fn write_ast_summary(
        &self,
        output: &mut String,
        items: &CategorizedAstItems,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Functions:** {} | **Structs:** {} | **Enums:** {} | **Traits:** {} | **Impls:** {} | **Modules:** {} | **Imports:** {}",
            items.functions.len(), items.structs.len(), items.enums.len(),
            items.traits.len(), items.impls.len(), items.modules.len(), items.uses.len())?;
        Ok(())
    }

    fn write_ast_details(
        &self,
        output: &mut String,
        items: &CategorizedAstItems,
    ) -> anyhow::Result<()> {
        self.write_functions_section(output, &items.functions)?;
        self.write_structs_section(output, &items.structs)?;
        self.write_enums_section(output, &items.enums)?;
        self.write_traits_section(output, &items.traits)?;
        self.write_impls_section(output, &items.impls)?;
        self.write_modules_section(output, &items.modules)?;
        self.write_imports_section(output, &items.uses)?;
        Ok(())
    }

    fn write_functions_section(
        &self,
        output: &mut String,
        functions: &[AstFunction],
    ) -> anyhow::Result<()> {
        if functions.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Functions:**")?;

        for func in functions.iter().take(10) {
            let async_marker = if func.is_async { " (async)" } else { "" };
            writeln!(
                output,
                "  - `{}{}` ({}) at line {}",
                func.name, async_marker, func.visibility, func.line
            )?;
        }

        if functions.len() > 10 {
            writeln!(
                output,
                "  - ... and {} more functions",
                functions.len() - 10
            )?;
        }

        Ok(())
    }

    fn write_structs_section(
        &self,
        output: &mut String,
        structs: &[AstStruct],
    ) -> anyhow::Result<()> {
        if structs.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Structs:**")?;

        for struct_item in structs.iter().take(5) {
            let derives_str = if struct_item.derives.is_empty() {
                String::with_capacity(1024)
            } else {
                format!(" (derives: {})", struct_item.derives.join(", "))
            };
            let field_plural = if struct_item.fields_count == 1 {
                ""
            } else {
                "s"
            };
            writeln!(
                output,
                "  - `{}` ({}) with {} field{}{} at line {}",
                struct_item.name,
                struct_item.visibility,
                struct_item.fields_count,
                field_plural,
                derives_str,
                struct_item.line
            )?;
        }

        if structs.len() > 5 {
            writeln!(output, "  - ... and {} more structs", structs.len() - 5)?;
        }

        Ok(())
    }

    fn write_enums_section(&self, output: &mut String, enums: &[AstEnum]) -> anyhow::Result<()> {
        if enums.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Enums:**")?;

        for enum_item in enums.iter().take(5) {
            let variant_plural = if enum_item.variants_count == 1 {
                ""
            } else {
                "s"
            };
            writeln!(
                output,
                "  - `{}` ({}) with {} variant{} at line {}",
                enum_item.name,
                enum_item.visibility,
                enum_item.variants_count,
                variant_plural,
                enum_item.line
            )?;
        }

        if enums.len() > 5 {
            writeln!(output, "  - ... and {} more enums", enums.len() - 5)?;
        }

        Ok(())
    }

    fn write_traits_section(&self, output: &mut String, traits: &[AstTrait]) -> anyhow::Result<()> {
        if traits.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Traits:**")?;

        for trait_item in traits.iter().take(5) {
            writeln!(
                output,
                "  - `{}` ({}) at line {}",
                trait_item.name, trait_item.visibility, trait_item.line
            )?;
        }

        if traits.len() > 5 {
            writeln!(output, "  - ... and {} more traits", traits.len() - 5)?;
        }

        Ok(())
    }

    fn write_impls_section(&self, output: &mut String, impls: &[AstImpl]) -> anyhow::Result<()> {
        if impls.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Implementations:**")?;

        for impl_item in impls.iter().take(5) {
            if let Some(trait_name) = &impl_item.trait_name {
                writeln!(
                    output,
                    "  - `{} for {}` at line {}",
                    trait_name, impl_item.type_name, impl_item.line
                )?;
            } else {
                writeln!(
                    output,
                    "  - `impl {}` at line {}",
                    impl_item.type_name, impl_item.line
                )?;
            }
        }

        if impls.len() > 5 {
            writeln!(
                output,
                "  - ... and {} more implementations",
                impls.len() - 5
            )?;
        }

        Ok(())
    }

    fn write_modules_section(
        &self,
        output: &mut String,
        modules: &[AstModule],
    ) -> anyhow::Result<()> {
        if modules.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Modules:**")?;

        for module_item in modules.iter().take(5) {
            writeln!(
                output,
                "  - `{}` ({}) at line {}",
                module_item.name, module_item.visibility, module_item.line
            )?;
        }

        if modules.len() > 5 {
            writeln!(output, "  - ... and {} more modules", modules.len() - 5)?;
        }

        Ok(())
    }

    fn write_imports_section(&self, output: &mut String, uses: &[AstUse]) -> anyhow::Result<()> {
        if uses.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;

        if uses.len() <= 8 {
            writeln!(output, "\n**Key Imports:**")?;
            for use_item in uses.iter().take(8) {
                writeln!(output, "  - `{}` at line {}", use_item.path, use_item.line)?;
            }
        } else {
            writeln!(output, "\n**Imports:** {} import statements", uses.len())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! PMAT-644: cover ast_formatting.rs surface.
    use super::super::super::{
        DeepContextAnalyzer, DeepContextConfig, DefectAnnotations, EnhancedFileContext,
    };
    use super::*;
    use crate::services::context::{AstItem, FileContext};

    fn analyzer() -> DeepContextAnalyzer {
        DeepContextAnalyzer::new(DeepContextConfig::default())
    }

    fn ctx_with_items(items: Vec<AstItem>) -> EnhancedFileContext {
        EnhancedFileContext {
            base: FileContext {
                path: "test.rs".to_string(),
                language: "rust".to_string(),
                items,
                complexity_metrics: None,
            },
            complexity_metrics: None,
            churn_metrics: None,
            defects: DefectAnnotations {
                dead_code: None,
                technical_debt: Vec::new(),
                complexity_violations: Vec::new(),
                tdg_score: None,
            },
            symbol_id: String::new(),
        }
    }

    fn func(name: &str, is_async: bool) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "pub".to_string(),
            is_async,
            line: 10,
        }
    }

    fn struct_item(name: &str, fields: usize, derives: Vec<&str>) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: "pub".to_string(),
            fields_count: fields,
            derives: derives.into_iter().map(String::from).collect(),
            line: 20,
        }
    }

    // --- format_import_path (3 branches) ---

    #[test]
    fn test_format_import_path_with_alias() {
        let a = analyzer();
        let s = a.format_import_path("numpy", &[], &Some("np".to_string()));
        assert_eq!(s, "numpy as np");
    }

    #[test]
    fn test_format_import_path_with_items_no_alias() {
        let a = analyzer();
        let s = a.format_import_path("os", &["path".to_string(), "sep".to_string()], &None);
        assert_eq!(s, "os (path, sep)");
    }

    #[test]
    fn test_format_import_path_bare_module() {
        let a = analyzer();
        let s = a.format_import_path("json", &[], &None);
        assert_eq!(s, "json");
    }

    // --- categorize_ast_items: cover every AstItem variant ---

    #[test]
    fn test_categorize_all_variants_distributes_counts() {
        let a = analyzer();
        let items = vec![
            func("f1", false),
            func("f2", true),
            struct_item("S", 2, vec!["Debug", "Clone"]),
            AstItem::Enum {
                name: "E".into(),
                visibility: "pub".into(),
                variants_count: 3,
                line: 1,
            },
            AstItem::Trait {
                name: "T".into(),
                visibility: "pub".into(),
                line: 1,
            },
            AstItem::Impl {
                type_name: "S".into(),
                trait_name: Some("Debug".into()),
                line: 1,
            },
            AstItem::Module {
                name: "m".into(),
                visibility: "pub".into(),
                line: 1,
            },
            AstItem::Use {
                path: "std::fs".into(),
                line: 1,
            },
            AstItem::Import {
                module: "numpy".into(),
                items: vec![],
                alias: Some("np".into()),
                line: 1,
            },
        ];
        let cat = a.categorize_ast_items(&items);
        assert_eq!(cat.functions.len(), 2);
        assert_eq!(cat.structs.len(), 1);
        assert_eq!(cat.enums.len(), 1);
        assert_eq!(cat.traits.len(), 1);
        assert_eq!(cat.impls.len(), 1);
        assert_eq!(cat.modules.len(), 1);
        // Use + Import both land in `uses`.
        assert_eq!(cat.uses.len(), 2);
    }

    // --- write_*_section early-return (empty collection) ---

    #[test]
    fn test_write_sections_empty_are_no_ops() {
        let a = analyzer();
        let mut out = String::new();
        a.write_functions_section(&mut out, &[]).unwrap();
        a.write_structs_section(&mut out, &[]).unwrap();
        a.write_enums_section(&mut out, &[]).unwrap();
        a.write_traits_section(&mut out, &[]).unwrap();
        a.write_impls_section(&mut out, &[]).unwrap();
        a.write_modules_section(&mut out, &[]).unwrap();
        a.write_imports_section(&mut out, &[]).unwrap();
        assert!(out.is_empty(), "empty sections must produce no output");
    }

    // --- write_functions_section: async marker + truncation ---

    #[test]
    fn test_write_functions_marks_async_and_lists_up_to_10() {
        let a = analyzer();
        let funcs = vec![
            AstFunction {
                name: "plain".into(),
                visibility: "pub".into(),
                is_async: false,
                line: 1,
            },
            AstFunction {
                name: "awaits".into(),
                visibility: "pub".into(),
                is_async: true,
                line: 2,
            },
        ];
        let mut out = String::new();
        a.write_functions_section(&mut out, &funcs).unwrap();
        assert!(out.contains("`plain` (pub)"));
        assert!(out.contains("`awaits (async)` (pub)"));
        assert!(!out.contains("and"));
    }

    #[test]
    fn test_write_functions_truncates_over_10_and_emits_more_line() {
        let a = analyzer();
        let funcs: Vec<AstFunction> = (0..13)
            .map(|i| AstFunction {
                name: format!("f{i}"),
                visibility: "pub".into(),
                is_async: false,
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_functions_section(&mut out, &funcs).unwrap();
        assert!(out.contains("and 3 more functions"));
    }

    // --- write_structs_section ---

    #[test]
    fn test_write_structs_derives_and_field_singular_plural() {
        let a = analyzer();
        let structs = vec![
            AstStruct {
                name: "One".into(),
                visibility: "pub".into(),
                fields_count: 1,
                derives: vec![],
                line: 1,
            },
            AstStruct {
                name: "Many".into(),
                visibility: "pub".into(),
                fields_count: 3,
                derives: vec!["Debug".into(), "Clone".into()],
                line: 2,
            },
        ];
        let mut out = String::new();
        a.write_structs_section(&mut out, &structs).unwrap();
        // Singular field for fields_count=1, no derives list.
        assert!(out.contains("with 1 field "));
        // Plural form with derives string.
        assert!(out.contains("with 3 fields"));
        assert!(out.contains("derives: Debug, Clone"));
    }

    #[test]
    fn test_write_structs_truncates_over_5() {
        let a = analyzer();
        let structs: Vec<AstStruct> = (0..7)
            .map(|i| AstStruct {
                name: format!("S{i}"),
                visibility: "pub".into(),
                fields_count: 1,
                derives: vec![],
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_structs_section(&mut out, &structs).unwrap();
        assert!(out.contains("and 2 more structs"));
    }

    // --- write_enums_section ---

    #[test]
    fn test_write_enums_singular_and_plural_variant_count() {
        let a = analyzer();
        let enums = vec![
            AstEnum {
                name: "One".into(),
                visibility: "pub".into(),
                variants_count: 1,
                line: 1,
            },
            AstEnum {
                name: "Many".into(),
                visibility: "pub".into(),
                variants_count: 4,
                line: 2,
            },
        ];
        let mut out = String::new();
        a.write_enums_section(&mut out, &enums).unwrap();
        assert!(out.contains("with 1 variant "));
        assert!(out.contains("with 4 variants"));
    }

    #[test]
    fn test_write_enums_truncates_over_5() {
        let a = analyzer();
        let enums: Vec<AstEnum> = (0..9)
            .map(|i| AstEnum {
                name: format!("E{i}"),
                visibility: "pub".into(),
                variants_count: 1,
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_enums_section(&mut out, &enums).unwrap();
        assert!(out.contains("and 4 more enums"));
    }

    // --- write_traits_section ---

    #[test]
    fn test_write_traits_truncates_over_5() {
        let a = analyzer();
        let traits: Vec<AstTrait> = (0..6)
            .map(|i| AstTrait {
                name: format!("T{i}"),
                visibility: "pub".into(),
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_traits_section(&mut out, &traits).unwrap();
        assert!(out.contains("and 1 more traits"));
    }

    // --- write_impls_section: inherent vs trait impl ---

    #[test]
    fn test_write_impls_both_trait_and_inherent_forms() {
        let a = analyzer();
        let impls = vec![
            AstImpl {
                type_name: "S".into(),
                trait_name: None,
                line: 1,
            },
            AstImpl {
                type_name: "S".into(),
                trait_name: Some("Debug".into()),
                line: 2,
            },
        ];
        let mut out = String::new();
        a.write_impls_section(&mut out, &impls).unwrap();
        assert!(out.contains("`impl S` at line 1"));
        assert!(out.contains("`Debug for S` at line 2"));
    }

    #[test]
    fn test_write_impls_truncates_over_5() {
        let a = analyzer();
        let impls: Vec<AstImpl> = (0..7)
            .map(|i| AstImpl {
                type_name: format!("T{i}"),
                trait_name: None,
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_impls_section(&mut out, &impls).unwrap();
        assert!(out.contains("and 2 more implementations"));
    }

    // --- write_modules_section ---

    #[test]
    fn test_write_modules_truncates_over_5() {
        let a = analyzer();
        let mods: Vec<AstModule> = (0..10)
            .map(|i| AstModule {
                name: format!("m{i}"),
                visibility: "pub".into(),
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_modules_section(&mut out, &mods).unwrap();
        assert!(out.contains("and 5 more modules"));
    }

    // --- write_imports_section: ≤8 shows paths, >8 shows count ---

    #[test]
    fn test_write_imports_lists_paths_when_small() {
        let a = analyzer();
        let uses: Vec<AstUse> = (0..5)
            .map(|i| AstUse {
                path: format!("mod::item{i}"),
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_imports_section(&mut out, &uses).unwrap();
        assert!(out.contains("Key Imports"));
        assert!(out.contains("mod::item0"));
        assert!(out.contains("mod::item4"));
    }

    #[test]
    fn test_write_imports_shows_count_only_when_over_8() {
        let a = analyzer();
        let uses: Vec<AstUse> = (0..12)
            .map(|i| AstUse {
                path: format!("x{i}"),
                line: i,
            })
            .collect();
        let mut out = String::new();
        a.write_imports_section(&mut out, &uses).unwrap();
        assert!(out.contains("12 import statements"));
        assert!(!out.contains("Key Imports"));
    }

    // --- Integration: format_enhanced_ast_section end-to-end ---

    #[test]
    fn test_format_enhanced_ast_section_integration() {
        let a = analyzer();
        let items = vec![
            func("parse", false),
            func("load", true),
            struct_item("Config", 3, vec!["Debug"]),
            AstItem::Enum {
                name: "Status".into(),
                visibility: "pub".into(),
                variants_count: 2,
                line: 30,
            },
            AstItem::Trait {
                name: "Validate".into(),
                visibility: "pub".into(),
                line: 40,
            },
            AstItem::Impl {
                type_name: "Config".into(),
                trait_name: Some("Default".into()),
                line: 50,
            },
            AstItem::Module {
                name: "helpers".into(),
                visibility: "pub".into(),
                line: 60,
            },
            AstItem::Use {
                path: "std::path::Path".into(),
                line: 1,
            },
            AstItem::Import {
                module: "os".into(),
                items: vec!["path".into()],
                alias: None,
                line: 2,
            },
        ];
        let ctxs = vec![ctx_with_items(items)];
        let mut out = String::new();
        a.format_enhanced_ast_section(&mut out, &ctxs).unwrap();

        // Header
        assert!(out.contains("## Enhanced AST Analysis"));
        assert!(out.contains("### test.rs"));
        assert!(out.contains("**Language:** rust"));
        // Summary line
        assert!(out.contains("Functions:** 2"));
        assert!(out.contains("Structs:** 1"));
        assert!(out.contains("Enums:** 1"));
        assert!(out.contains("Traits:** 1"));
        assert!(out.contains("Impls:** 1"));
        assert!(out.contains("Modules:** 1"));
        // Both Use + Import items went into `uses`; expect count 2.
        assert!(out.contains("Imports:** 2"));
        // Detail sections
        assert!(out.contains("parse"));
        assert!(out.contains("load (async)"));
        assert!(out.contains("Config"));
        assert!(out.contains("Default for Config"));
    }

    #[test]
    fn test_format_enhanced_ast_section_empty_items_list() {
        let a = analyzer();
        let ctxs = vec![ctx_with_items(vec![])];
        let mut out = String::new();
        a.format_enhanced_ast_section(&mut out, &ctxs).unwrap();
        // Summary line with all zeros + no detail sections.
        assert!(out.contains("Functions:** 0"));
        assert!(!out.contains("Implementations:"));
        assert!(!out.contains("Key Imports"));
    }
}
