
struct RustVisitor {
    items: Vec<AstItem>,
    
    source: String,
}

impl RustVisitor {
    fn new(source: String) -> Self {
        Self {
            items: Vec::new(),
            source,
        }
    }

    fn get_line<T: syn::spanned::Spanned>(&self, _span: T) -> usize {
        // For simplicity, return 1. In production, use a proper source map
        1
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
        self.items.push(AstItem::Function {
            name: node.sig.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            is_async: node.sig.asyncness.is_some(),
            line: self.get_line(node.sig.ident.span()),
        });
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let fields_count = match &node.fields {
            syn::Fields::Named(fields) => fields.named.len(),
            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
            syn::Fields::Unit => 0,
        };

        self.items.push(AstItem::Struct {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            fields_count,
            derives: Self::get_derives(&node.attrs),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.items.push(AstItem::Enum {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            variants_count: node.variants.len(),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        self.items.push(AstItem::Trait {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            line: self.get_line(node.ident.span()),
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

        self.items.push(AstItem::Impl {
            type_name,
            trait_name,
            line: 1, // Default line number
        });
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.items.push(AstItem::Module {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            line: self.get_line(node.ident.span()),
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

        self.items.push(AstItem::Use {
            path,
            line: 1, // Default line number
        });
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

