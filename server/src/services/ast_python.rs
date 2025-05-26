use crate::models::error::TemplateError;
use crate::services::context::{AstItem, FileContext};
use rustpython_parser::ast;
use std::path::Path;

pub async fn analyze_python_file(path: &Path) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    let ast = rustpython_parser::parse(
        &content,
        rustpython_parser::Mode::Module,
        path.to_str().unwrap_or("<unknown>"),
    )
    .map_err(|e| TemplateError::InvalidUtf8(format!("Python parse error: {:?}", e)))?;

    let mut items = Vec::new();

    if let ast::Mod::Module(module) = ast {
        for stmt in module.body {
            extract_python_items(&stmt, &mut items);
        }
    }

    Ok(FileContext {
        path: path.display().to_string(),
        language: "python".to_string(),
        items,
    })
}

fn extract_python_items(stmt: &ast::Stmt, items: &mut Vec<AstItem>) {
    match stmt {
        ast::Stmt::FunctionDef(func) => {
            items.push(AstItem::Function {
                name: func.name.to_string(),
                visibility: if func.name.starts_with('_') {
                    "private".to_string()
                } else {
                    "public".to_string()
                },
                is_async: matches!(stmt, ast::Stmt::AsyncFunctionDef(_)),
                line: 1, // rustpython-parser doesn't provide line numbers easily
            });
        }
        ast::Stmt::AsyncFunctionDef(func) => {
            items.push(AstItem::Function {
                name: func.name.to_string(),
                visibility: if func.name.starts_with('_') {
                    "private".to_string()
                } else {
                    "public".to_string()
                },
                is_async: true,
                line: 1, // rustpython-parser doesn't provide line numbers easily
            });
        }
        ast::Stmt::ClassDef(class) => {
            // Count methods and attributes
            let mut _methods_count = 0;
            let mut attributes_count = 0;

            for stmt in &class.body {
                match stmt {
                    ast::Stmt::FunctionDef(_) | ast::Stmt::AsyncFunctionDef(_) => {
                        _methods_count += 1
                    }
                    ast::Stmt::AnnAssign(_) | ast::Stmt::Assign(_) => attributes_count += 1,
                    _ => {}
                }
            }

            // Python classes are similar to structs
            items.push(AstItem::Struct {
                name: class.name.to_string(),
                visibility: "public".to_string(),
                fields_count: attributes_count,
                derives: vec![], // Python doesn't have derives like Rust
                line: 1,         // rustpython-parser doesn't provide line numbers easily
            });

            // Also extract methods from the class
            for stmt in &class.body {
                extract_python_items(stmt, items);
            }
        }
        ast::Stmt::Import(import) => {
            for alias in &import.names {
                items.push(AstItem::Use {
                    path: alias.name.to_string(),
                    line: 1, // rustpython-parser doesn't provide line numbers easily
                });
            }
        }
        ast::Stmt::ImportFrom(import_from) => {
            if let Some(module) = &import_from.module {
                let base_path = module.to_string();
                for alias in &import_from.names {
                    items.push(AstItem::Use {
                        path: format!("{}.{}", base_path, alias.name),
                        line: 1, // rustpython-parser doesn't provide line numbers easily
                    });
                }
            }
        }
        _ => {
            // Handle other statement types if needed
        }
    }
}
