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
