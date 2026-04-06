pub fn format_context_as_markdown(context: &ProjectContext) -> String {
    let mut output = String::new();

    format_header(&mut output, context);
    format_summary(&mut output, &context.summary);
    format_dependencies(&mut output, &context.summary.dependencies);
    format_files(&mut output, &context.files);
    format_footer(&mut output);

    output
}

fn format_header(output: &mut String, context: &ProjectContext) {
    output.push_str(&format!(
        "# Project Context: {} Project\n\n",
        context.project_type
    ));
    output.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
}

fn format_summary(output: &mut String, summary: &ProjectSummary) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Files analyzed: {}\n", summary.total_files));
    output.push_str(&format!("- Functions: {}\n", summary.total_functions));
    output.push_str(&format!("- Structs: {}\n", summary.total_structs));
    output.push_str(&format!("- Enums: {}\n", summary.total_enums));
    output.push_str(&format!("- Traits: {}\n", summary.total_traits));
    output.push_str(&format!("- Implementations: {}\n", summary.total_impls));
}

fn format_dependencies(output: &mut String, dependencies: &[String]) {
    if !dependencies.is_empty() {
        output.push_str("\n## Dependencies\n\n");
        for dep in dependencies {
            output.push_str(&format!("- {dep}\n"));
        }
    }
}

fn format_files(output: &mut String, files: &[FileContext]) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    output.push_str("\n## Files\n\n");

    for file in files {
        output.push_str(&format!("### {}\n\n", file.path));

        let grouped_items = group_items_by_type(&file.items);
        format_item_groups(output, &grouped_items);
    }
}

fn group_items_by_type(items: &[AstItem]) -> GroupedItems<'_> {
    debug_assert!(!items.is_empty(), "items must not be empty");
    let mut grouped = GroupedItems {
        functions: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        modules: Vec::new(),
    };

    for item in items {
        match item {
            AstItem::Function { .. } => grouped.functions.push(item),
            AstItem::Struct { .. } => grouped.structs.push(item),
            AstItem::Enum { .. } => grouped.enums.push(item),
            AstItem::Trait { .. } => grouped.traits.push(item),
            AstItem::Impl { .. } => grouped.impls.push(item),
            AstItem::Module { .. } => grouped.modules.push(item),
            _ => {}
        }
    }

    grouped
}

fn format_item_groups(output: &mut String, groups: &GroupedItems) {
    format_item_group(output, "Modules", &groups.modules, format_module_item);
    format_item_group(output, "Structs", &groups.structs, format_struct_item);
    format_item_group(output, "Enums", &groups.enums, format_enum_item);
    format_item_group(output, "Traits", &groups.traits, format_trait_item);
    format_item_group(output, "Functions", &groups.functions, format_function_item);
    format_item_group(output, "Implementations", &groups.impls, format_impl_item);
}

fn format_item_group<F>(output: &mut String, title: &str, items: &[&AstItem], formatter: F)
where
    F: Fn(&AstItem) -> String,
{
    debug_assert!(!items.is_empty(), "items must not be empty");
    if !items.is_empty() {
        output.push_str(&format!("**{title}:**\n"));
        for item in items {
            output.push_str(&format!("{}\n", formatter(item)));
        }
        output.push('\n');
    }
}

fn format_module_item(item: &AstItem) -> String {
    if let AstItem::Module {
        name,
        visibility,
        line,
    } = item
    {
        format!("- `{visibility} mod {name}` (line {line})")
    } else {
        String::new()
    }
}

fn format_struct_item(item: &AstItem) -> String {
    if let AstItem::Struct {
        name,
        visibility,
        fields_count,
        derives,
        line,
    } = item
    {
        let mut result = format!("- `{visibility} struct {name}` ({fields_count} fields)");
        if !derives.is_empty() {
            result.push_str(&format!(" [derives: {}]", derives.join(", ")));
        }
        result.push_str(&format!(" (line {line})"));
        result
    } else {
        String::new()
    }
}

fn format_enum_item(item: &AstItem) -> String {
    if let AstItem::Enum {
        name,
        visibility,
        variants_count,
        line,
    } = item
    {
        format!("- `{visibility} enum {name}` ({variants_count} variants) (line {line})")
    } else {
        String::new()
    }
}

fn format_trait_item(item: &AstItem) -> String {
    if let AstItem::Trait {
        name,
        visibility,
        line,
    } = item
    {
        format!("- `{visibility} trait {name}` (line {line})")
    } else {
        String::new()
    }
}

fn format_function_item(item: &AstItem) -> String {
    if let AstItem::Function {
        name,
        visibility,
        is_async,
        line,
    } = item
    {
        format!(
            "- `{} {}fn {}` (line {})",
            visibility,
            if *is_async { "async " } else { "" },
            name,
            line
        )
    } else {
        String::new()
    }
}

fn format_impl_item(item: &AstItem) -> String {
    if let AstItem::Impl {
        type_name,
        trait_name,
        line,
    } = item
    {
        match trait_name {
            Some(trait_name) => {
                format!("- `impl {trait_name} for {type_name}` (line {line})")
            }
            None => format!("- `impl {type_name}` (line {line})"),
        }
    } else {
        String::new()
    }
}

fn format_footer(output: &mut String) {
    output.push_str("---\n");
    output.push_str("Generated by paiml-mcp-agent-toolkit\n");
}
