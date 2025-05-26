use crate::models::error::TemplateError;
use crate::services::context::{AstItem, FileContext};
use std::path::Path;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{Visit, VisitWith};

pub async fn analyze_typescript_file(path: &Path) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Custom(path.display().to_string()), content);

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: path.extension().and_then(|s| s.to_str()) == Some("tsx"),
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("TypeScript parse error: {:?}", e)))?;

    let mut visitor = TypeScriptVisitor::new();
    module.visit_with(&mut visitor);

    Ok(FileContext {
        path: path.display().to_string(),
        language: if path.extension().and_then(|s| s.to_str()) == Some("tsx") {
            "tsx".to_string()
        } else {
            "typescript".to_string()
        },
        items: visitor.items,
    })
}

pub async fn analyze_javascript_file(path: &Path) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Custom(path.display().to_string()), content);

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("JavaScript parse error: {:?}", e)))?;

    let mut visitor = TypeScriptVisitor::new();
    module.visit_with(&mut visitor);

    Ok(FileContext {
        path: path.display().to_string(),
        language: if path.extension().and_then(|s| s.to_str()) == Some("jsx") {
            "jsx".to_string()
        } else {
            "javascript".to_string()
        },
        items: visitor.items,
    })
}

struct TypeScriptVisitor {
    items: Vec<AstItem>,
}

impl TypeScriptVisitor {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl Visit for TypeScriptVisitor {
    fn visit_fn_decl(&mut self, node: &FnDecl) {
        self.items.push(AstItem::Function {
            name: node.ident.sym.to_string(),
            visibility: "public".to_string(), // JS/TS doesn't have visibility modifiers like Rust
            is_async: node.function.is_async,
            line: 1, // SWC doesn't provide line numbers easily
        });
    }

    fn visit_class_decl(&mut self, node: &ClassDecl) {
        let mut fields_count = 0;
        let mut _methods_count = 0;

        for member in &node.class.body {
            match member {
                ClassMember::Constructor(_) => _methods_count += 1,
                ClassMember::Method(_) => _methods_count += 1,
                ClassMember::PrivateMethod(_) => _methods_count += 1,
                ClassMember::ClassProp(_) => fields_count += 1,
                ClassMember::PrivateProp(_) => fields_count += 1,
                _ => {}
            }
        }

        self.items.push(AstItem::Struct {
            name: node.ident.sym.to_string(),
            visibility: "public".to_string(),
            fields_count,
            derives: vec![], // TypeScript uses decorators, not derives
            line: 1,
        });
    }

    fn visit_var_decl(&mut self, node: &VarDecl) {
        // Track exported const functions (common pattern in JS/TS)
        for decl in &node.decls {
            if let Pat::Ident(ident) = &decl.name {
                if let Some(init) = &decl.init {
                    if let Expr::Arrow(arrow) = &**init {
                        self.items.push(AstItem::Function {
                            name: ident.id.sym.to_string(),
                            visibility: "public".to_string(),
                            is_async: arrow.is_async,
                            line: 1,
                        });
                    } else if let Expr::Fn(fn_expr) = &**init {
                        if let Some(ident) = &fn_expr.ident {
                            self.items.push(AstItem::Function {
                                name: ident.sym.to_string(),
                                visibility: "public".to_string(),
                                is_async: fn_expr.function.is_async,
                                line: 1,
                            });
                        }
                    }
                }
            }
        }
    }

    fn visit_ts_interface_decl(&mut self, node: &TsInterfaceDecl) {
        // Treat interfaces as traits
        self.items.push(AstItem::Trait {
            name: node.id.sym.to_string(),
            visibility: "public".to_string(),
            line: 1,
        });
    }

    fn visit_ts_type_alias_decl(&mut self, node: &TsTypeAliasDecl) {
        // Treat type aliases as a special kind of struct
        self.items.push(AstItem::Struct {
            name: node.id.sym.to_string(),
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        });
    }

    fn visit_ts_enum_decl(&mut self, node: &TsEnumDecl) {
        self.items.push(AstItem::Enum {
            name: node.id.sym.to_string(),
            visibility: "public".to_string(),
            variants_count: node.members.len(),
            line: 1,
        });
    }

    fn visit_import_decl(&mut self, node: &ImportDecl) {
        self.items.push(AstItem::Use {
            path: node.src.value.to_string(),
            line: 1,
        });
    }

    fn visit_export_decl(&mut self, node: &ExportDecl) {
        // Visit the declaration inside the export
        match &node.decl {
            Decl::Fn(fn_decl) => self.visit_fn_decl(fn_decl),
            Decl::Class(class_decl) => self.visit_class_decl(class_decl),
            Decl::Var(var_decl) => self.visit_var_decl(var_decl),
            Decl::TsInterface(interface) => self.visit_ts_interface_decl(interface),
            Decl::TsTypeAlias(type_alias) => self.visit_ts_type_alias_decl(type_alias),
            Decl::TsEnum(enum_decl) => self.visit_ts_enum_decl(enum_decl),
            _ => {}
        }
    }
}
