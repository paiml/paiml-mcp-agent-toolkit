
struct RustVisitor {
    items: Vec<AstItem>,

    source: String,
    /// Byte offsets of the start of every line in `source`.
    line_starts: Vec<usize>,
    /// How far into `source` declaration lookup has already consumed. Items are
    /// visited in source order, so the scan never has to restart.
    scan_offset: usize,
}

impl RustVisitor {
    fn new(source: String) -> Self {
        let mut line_starts = vec![0usize];
        line_starts.extend(
            source
                .char_indices()
                .filter(|(_, c)| *c == '\n')
                .map(|(i, _)| i + 1),
        );
        Self {
            items: Vec::new(),
            source,
            line_starts,
            scan_offset: 0,
        }
    }

    /// Line on which `keyword name` is declared, e.g. `("fn", "collect_nodes")`.
    ///
    /// This used to be `get_line(_span) -> 1` ("for simplicity, return 1"), so
    /// every item in a project-AST `FileContext` — and therefore every node of
    /// the DAG built from it — reported line 1 regardless of the file. syn's
    /// spans carry no location unless proc-macro2's `span-locations` feature is
    /// on (it is not, and turning it on taxes every build), so resolve the line
    /// from the source text the visitor already holds. Items arrive in source
    /// order, so a forward scan is both cheap and unambiguous. A declaration we
    /// cannot locate reports 0 ("unknown") rather than a plausible 1.
    fn line_of_decl(&mut self, keyword: &str, name: &str) -> usize {
        match self.find_decl_offset(keyword, Some(name)) {
            Some(offset) => {
                self.scan_offset = offset + keyword.len();
                self.line_at(offset)
            }
            None => 0,
        }
    }

    /// Line of the next bare `keyword` token (used for `impl` / `use`, which
    /// have no single declared identifier).
    fn line_of_keyword(&mut self, keyword: &str) -> usize {
        match self.find_decl_offset(keyword, None) {
            Some(offset) => {
                self.scan_offset = offset + keyword.len();
                self.line_at(offset)
            }
            None => 0,
        }
    }

    fn line_at(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(index) => index + 1,
            Err(index) => index, // index of the following line start == 1-based line
        }
    }

    /// Byte offset of the next `keyword` token (optionally followed by `name`)
    /// at or after the scan cursor, skipping matches inside `//` comments.
    fn find_decl_offset(&self, keyword: &str, name: Option<&str>) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let is_ident = |b: u8| b == b'_' || b.is_ascii_alphanumeric();
        let mut from = self.scan_offset.min(self.source.len());

        while let Some(rel) = self.source.get(from..)?.find(keyword) {
            let start = from + rel;
            from = start + keyword.len();

            // `keyword` must stand alone, not end a longer identifier.
            if start > 0 && is_ident(bytes[start - 1]) {
                continue;
            }
            let after_keyword = start + keyword.len();
            if after_keyword < bytes.len() && is_ident(bytes[after_keyword]) {
                continue;
            }

            if let Some(name) = name {
                let mut cursor = after_keyword;
                while cursor < bytes.len() && (bytes[cursor] as char).is_whitespace() {
                    cursor += 1;
                }
                if !self.source[cursor..].starts_with(name) {
                    continue;
                }
                let after_name = cursor + name.len();
                if after_name < bytes.len() && is_ident(bytes[after_name]) {
                    continue;
                }
            }

            if self.is_line_commented(start) {
                continue;
            }

            return Some(start);
        }

        None
    }

    /// Is `offset` preceded by `//` on its own line? Keeps doc comments and
    /// commented-out code from being read as declarations.
    fn is_line_commented(&self, offset: usize) -> bool {
        let line_start = self.source[..offset]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        self.source[line_start..offset].contains("//")
    }

    fn get_visibility(&self, vis: &syn::Visibility) -> String {
        match vis {
            syn::Visibility::Public(_) => "pub".to_string(),
            syn::Visibility::Restricted(r) => format!(
                "pub({})",
                r.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            ),
            syn::Visibility::Inherited => "private".to_string(),
        }
    }

    fn get_derives(_attrs: &[syn::Attribute]) -> Vec<String> {
        // Simplified version - in production, parse derive attributes properly
        Vec::new()
    }
}

impl<'ast> Visit<'ast> for RustVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();
        let line = self.line_of_decl("fn", &name);
        self.items.push(AstItem::Function {
            name,
            visibility: self.get_visibility(&node.vis),
            is_async: node.sig.asyncness.is_some(),
            line,
        });
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let fields_count = match &node.fields {
            syn::Fields::Named(fields) => fields.named.len(),
            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
            syn::Fields::Unit => 0,
        };

        let name = node.ident.to_string();
        let line = self.line_of_decl("struct", &name);
        self.items.push(AstItem::Struct {
            name,
            visibility: self.get_visibility(&node.vis),
            fields_count,
            derives: Self::get_derives(&node.attrs),
            line,
        });
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let name = node.ident.to_string();
        let line = self.line_of_decl("enum", &name);
        self.items.push(AstItem::Enum {
            name,
            visibility: self.get_visibility(&node.vis),
            variants_count: node.variants.len(),
            line,
        });
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        let name = node.ident.to_string();
        let line = self.line_of_decl("trait", &name);
        self.items.push(AstItem::Trait {
            name,
            visibility: self.get_visibility(&node.vis),
            line,
        });
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let type_name = if let syn::Type::Path(type_path) = &*node.self_ty {
            type_path
                .path
                .segments
                .last()
                .map_or_else(|| "Unknown".to_string(), |s| s.ident.to_string())
        } else {
            "Unknown".to_string()
        };

        let trait_name = node.trait_.as_ref().map(|(_, path, _)| {
            path.segments
                .last()
                .map_or_else(|| "Unknown".to_string(), |s| s.ident.to_string())
        });

        let line = self.line_of_keyword("impl");
        self.items.push(AstItem::Impl {
            type_name,
            trait_name,
            line,
        });
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        let name = node.ident.to_string();
        let line = self.line_of_decl("mod", &name);
        self.items.push(AstItem::Module {
            name,
            visibility: self.get_visibility(&node.vis),
            line,
        });
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        // Defect #653: this recorded only the FIRST segment of the use tree, so
        // `use crate::services::dag_builder::DagBuilder;` was reported as the path
        // "crate". Every intra-crate import collapsed onto "crate"/"super"/"std",
        // which made import resolution (and therefore every DAG import edge)
        // impossible. Record the full path down to the first group/glob instead.
        let mut path = String::new();
        collect_use_path(&node.tree, &mut path);

        let line = self.line_of_keyword("use");
        self.items.push(AstItem::Use { path, line });
    }
}

/// Flatten a `use` tree into its `::`-joined path, stopping at the first group.
///
/// `use std::io;` -> "std::io", `use std::io as x;` -> "std::io",
/// `use std::prelude::*;` -> "std::prelude::*", `use std::{io, fs};` -> "std"
/// (the shared prefix, which is what import resolution keys on).
fn collect_use_path(tree: &syn::UseTree, out: &mut String) {
    let push = |out: &mut String, segment: &str| {
        if !out.is_empty() {
            out.push_str("::");
        }
        out.push_str(segment);
    };

    match tree {
        syn::UseTree::Path(p) => {
            push(out, &p.ident.to_string());
            collect_use_path(&p.tree, out);
        }
        syn::UseTree::Name(n) => push(out, &n.ident.to_string()),
        syn::UseTree::Rename(r) => push(out, &r.ident.to_string()),
        syn::UseTree::Glob(_) => push(out, "*"),
        syn::UseTree::Group(_) => {}
    }
}

#[cfg(test)]
mod visitor_line_number_tests {
    //! Every item used to report line 1, whatever the file.
    use super::*;

    const SOURCE: &str = r"// a comment that mentions fn beta and struct Alpha
use std::io;

pub struct Alpha {
    x: u32,
}

fn beta() -> u32 {
    1
}

pub mod inner {}
";

    fn items_of(source: &str) -> Vec<AstItem> {
        let parsed = syn::parse_file(source).expect("fixture must parse");
        let mut visitor = RustVisitor::new(source.to_string());
        visitor.visit_file(&parsed);
        visitor.items
    }

    #[test]
    fn test_items_report_their_real_declaration_line() {
        let items = items_of(SOURCE);

        let mut seen = Vec::new();
        for item in &items {
            match item {
                AstItem::Use { path, line } => seen.push((path.clone(), *line)),
                AstItem::Struct { name, line, .. }
                | AstItem::Function { name, line, .. }
                | AstItem::Module { name, line, .. } => seen.push((name.clone(), *line)),
                _ => {}
            }
        }

        assert_eq!(
            seen,
            vec![
                ("std::io".to_string(), 2),
                ("Alpha".to_string(), 4),
                ("beta".to_string(), 8),
                ("inner".to_string(), 12),
            ],
            "declaration lines were {seen:?}"
        );
    }

    #[test]
    fn test_line_numbers_are_not_all_one() {
        // The stub returned a literal 1 for every item; any file with more than
        // one declaration must produce more than one distinct line.
        let lines: std::collections::BTreeSet<usize> = items_of(SOURCE)
            .iter()
            .map(|item| match item {
                AstItem::Use { line, .. }
                | AstItem::Struct { line, .. }
                | AstItem::Function { line, .. }
                | AstItem::Module { line, .. } => *line,
                _ => 0,
            })
            .collect();
        assert!(lines.len() > 1, "all items landed on the same line: {lines:?}");
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_rust_file_with_cache(path, None).await
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file_with_cache(
    path: &Path,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Result<FileContext, TemplateError> {
    if let Some(cache) = cache_manager {
        cache
            .get_or_compute_ast(path, || async {
                // Parse the file
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;

                let syntax = syn::parse_file(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Rust file: {e}"))?;

                let mut visitor = RustVisitor::new(content);
                visitor.visit_file(&syntax);

                Ok(FileContext {
                    path: path.display().to_string(),
                    language: "rust".to_string(),
                    items: visitor.items,
                    complexity_metrics: None,
                })
            })
            .await
            .map(|arc| (*arc).clone())
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
    } else {
        // No cache, compute directly
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;

        let syntax =
            syn::parse_file(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        let mut visitor = RustVisitor::new(content);
        visitor.visit_file(&syntax);

        Ok(FileContext {
            path: path.display().to_string(),
            language: "rust".to_string(),
            items: visitor.items,
            complexity_metrics: None,
        })
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_project(
    root_path: &Path,
    toolchain: &str,
) -> Result<ProjectContext, TemplateError> {
    analyze_project_with_cache(root_path, toolchain, None).await
}

// Persistent cache version
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file_with_persistent_cache(
    path: &Path,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Result<FileContext, TemplateError> {
    if let Some(cache) = cache_manager {
        cache
            .get_or_compute_ast(path, || async {
                // Parse the file
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;

                let syntax = syn::parse_file(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Rust file: {e}"))?;

                let mut visitor = RustVisitor::new(content);
                visitor.visit_file(&syntax);

                Ok(FileContext {
                    path: path.display().to_string(),
                    language: "rust".to_string(),
                    items: visitor.items,
                    complexity_metrics: None,
                })
            })
            .await
            .map(|arc| (*arc).clone())
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
    } else {
        // No cache, compute directly
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;

        let syntax =
            syn::parse_file(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        let mut visitor = RustVisitor::new(content);
        visitor.visit_file(&syntax);

        Ok(FileContext {
            path: path.display().to_string(),
            language: "rust".to_string(),
            items: visitor.items,
            complexity_metrics: None,
        })
    }
}

/// Optimized project analysis for dead code detection - focuses only on source files
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_project_for_dead_code(
    root_path: &Path,
    toolchain: &str,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files = scan_rust_files_only(root_path, toolchain, None, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
        graph: None,
    })
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_project_with_cache(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files = scan_and_analyze_files(root_path, toolchain, cache_manager, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    // Build O(1) graph for symbol lookups and PageRank
    let graph = build_context_graph(&files).ok();

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
        graph,
    })
}

