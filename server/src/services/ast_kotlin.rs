//! Kotlin language AST parser implementation
//!
//! This module provides Kotlin language parsing support using tree-sitter-kotlin.

use crate::models::unified_ast::{
    AstDag, AstKind, FunctionKind, Language, TypeKind, UnifiedAstNode,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
use tree_sitter::{Node, Parser};

/// Safety limits to prevent memory exhaustion
const MAX_RECURSION_DEPTH: usize = 1000;
const MAX_PARSING_TIME: Duration = Duration::from_secs(30);
const MAX_STRING_LENGTH: usize = 1024 * 1024; // 1MB string limit
const MAX_NODES: usize = 100_000; // Maximum nodes to prevent memory explosion

/// Kotlin AST parser implementation with memory safety guarantees
pub struct KotlinAstParser {
    parser: Parser,
    max_depth: usize,
    timeout: Duration,
}

impl Default for KotlinAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl KotlinAstParser {
    pub fn new() -> Self {
        Self::with_limits(MAX_RECURSION_DEPTH, MAX_PARSING_TIME)
    }

    pub fn with_limits(max_depth: usize, timeout: Duration) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_kotlin::language())
            .expect("Failed to set Kotlin language");
        Self {
            parser,
            max_depth,
            timeout,
        }
    }

    /// Parse a Kotlin file into an AST DAG with memory safety guarantees
    pub fn parse_file(&mut self, path: &Path, content: &str) -> Result<AstDag> {
        // Input validation
        if content.len() > MAX_STRING_LENGTH {
            return Err(anyhow::anyhow!(
                "File too large: {} bytes exceeds limit of {} bytes",
                content.len(),
                MAX_STRING_LENGTH
            ));
        }

        let mut dag = AstDag::new();
        let start_time = Instant::now();

        let tree = self
            .parser
            .parse(content, None)
            .context("Failed to parse Kotlin file")?;

        let root = tree.root_node();
        let mut ctx = ParseContext {
            content,
            dag: &mut dag,
            path: path.to_string_lossy().into_owned(),
            stack: Vec::with_capacity(self.max_depth),
            node_map: HashMap::with_capacity(1000),
            start_time,
            max_depth: self.max_depth,
            timeout: self.timeout,
            current_depth: 0,
            nodes_created: 0,
        };

        // Visit the root node to start parsing
        if let Err(e) = self.visit_node(&mut ctx, root) {
            return Err(anyhow::anyhow!("Error during AST traversal: {}", e));
        }

        Ok(dag)
    }

    fn visit_node(&self, ctx: &mut ParseContext, node: Node) -> Result<Option<usize>> {
        // Safety checks first
        if ctx.current_depth >= ctx.max_depth {
            return Err(anyhow::anyhow!(
                "Maximum recursion depth exceeded: {} at depth {}",
                ctx.max_depth,
                ctx.current_depth
            ));
        }

        if ctx.start_time.elapsed() > ctx.timeout {
            return Err(anyhow::anyhow!(
                "Parsing timeout exceeded: {:?} at depth {}",
                ctx.timeout,
                ctx.current_depth
            ));
        }

        if ctx.nodes_created >= MAX_NODES {
            return Err(anyhow::anyhow!(
                "Maximum nodes limit exceeded: {} at depth {}",
                MAX_NODES,
                ctx.current_depth
            ));
        }

        // Increment depth counter
        ctx.current_depth += 1;


        let node_id = match node.kind() {
            "class_declaration" => self.process_class(ctx, node)?,
            "object_declaration" => self.process_object(ctx, node)?,
            "function_declaration" => self.process_function(ctx, node)?,
            "enum_class_declaration" => self.process_enum(ctx, node)?,
            _ => None,
        };

        // Also look for class members (methods)
        if node.kind() == "class_body" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "function_declaration" {
                    self.process_function(ctx, child)?;
                }
            }
        }

        // Visit children using iterative approach to prevent stack overflow
        let child_result: Result<()> = {
            // Use a stack for iterative traversal instead of recursion
            let mut work_stack = Vec::new();
            let mut cursor = node.walk();

            // Add all direct children to work stack
            for child in node.children(&mut cursor) {
                work_stack.push((child, ctx.current_depth + 1));
            }


            // Process nodes iteratively
            while let Some((work_node, depth)) = work_stack.pop() {
                // Safety checks
                if ctx.nodes_created >= MAX_NODES {
                    break;
                }

                if depth >= ctx.max_depth {
                    continue;
                }

                if ctx.start_time.elapsed() > ctx.timeout {
                    break;
                }

                // Process the node (only top-level nodes, no deep recursion)
                let kind = work_node.kind();
                
                if kind == "class_declaration"
                    || kind == "object_declaration"
                    || kind == "function_declaration"
                    || kind == "enum_class_declaration"
                {
                    let old_depth = ctx.current_depth;
                    ctx.current_depth = depth;

                    let _ = self.process_node_simple(ctx, work_node);

                    ctx.current_depth = old_depth;
                } else if kind == "class_body" {
                    // Add class body children to work stack to find methods
                    let mut body_cursor = work_node.walk();
                    for body_child in work_node.children(&mut body_cursor) {
                        if body_child.kind() == "function_declaration" {
                            work_stack.push((body_child, depth + 1));
                        }
                    }
                } else {
                    // For other nodes, add their children to continue traversal
                    let mut child_cursor = work_node.walk();
                    for child in work_node.children(&mut child_cursor) {
                        work_stack.push((child, depth + 1));
                    }
                }
            }
            Ok(())
        };

        // Always decrement depth counter before returning
        ctx.current_depth -= 1;

        // Propagate any errors from child processing
        child_result?;

        Ok(node_id)
    }

    fn process_class(&self, ctx: &mut ParseContext, node: Node) -> Result<Option<usize>> {
        // Check if this is actually an enum class by looking at the source
        let source_start = node.start_byte();
        let source_end = (source_start + 20).min(node.end_byte()).min(ctx.content.len());
        let source_prefix = &ctx.content[source_start..source_end];
        let is_enum = source_prefix.starts_with("enum ");
        
        let name = if is_enum {
            // For enums, use special extraction logic
            self.extract_enum_name(ctx, node)
                .unwrap_or_else(|| String::from("AnonymousEnum"))
        } else {
            self.extract_identifier(ctx, node, "simple_identifier")
                .unwrap_or_else(|| String::from("AnonymousClass"))
        };
        
        let kind = if is_enum {
            AstKind::Type(TypeKind::Enum)
        } else {
            AstKind::Type(TypeKind::Class)
        };
        
        let mut ast_node = UnifiedAstNode::new(kind, Language::Kotlin);
        ast_node.source_range = node.start_byte() as u32..node.end_byte() as u32;
        self.set_name_vector(&mut ast_node, &name);

        let node_id = ctx.dag.add_node(ast_node);
        ctx.nodes_created += 1;

        // Process class body to find methods (not for enums)
        if !is_enum {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "class_body" {
                    let mut body_cursor = child.walk();
                    for body_child in child.children(&mut body_cursor) {
                        if body_child.kind() == "function_declaration" {
                            self.process_function(ctx, body_child)?;
                        }
                    }
                }
            }
        }

        Ok(Some(node_id as usize))
    }

    fn process_object(&self, ctx: &mut ParseContext, node: Node) -> Result<Option<usize>> {
        let name = self.extract_identifier(ctx, node, "simple_identifier")
            .unwrap_or_else(|| String::from("AnonymousObject"));
        let mut ast_node = UnifiedAstNode::new(AstKind::Type(TypeKind::Class), Language::Kotlin);
        ast_node.source_range = node.start_byte() as u32..node.end_byte() as u32;
        self.set_name_vector(&mut ast_node, &name);

        let node_id = ctx.dag.add_node(ast_node);
        ctx.nodes_created += 1;
        Ok(Some(node_id as usize))
    }

    fn process_enum(&self, ctx: &mut ParseContext, node: Node) -> Result<Option<usize>> {
        let name = self.extract_identifier(ctx, node, "simple_identifier")
            .unwrap_or_else(|| String::from("AnonymousEnum"));
        let mut ast_node = UnifiedAstNode::new(AstKind::Type(TypeKind::Enum), Language::Kotlin);
        ast_node.source_range = node.start_byte() as u32..node.end_byte() as u32;
        self.set_name_vector(&mut ast_node, &name);

        let node_id = ctx.dag.add_node(ast_node);
        ctx.nodes_created += 1;
        Ok(Some(node_id as usize))
    }

    fn process_function(&self, ctx: &mut ParseContext, node: Node) -> Result<Option<usize>> {
        let name = self.extract_identifier(ctx, node, "simple_identifier")
            .unwrap_or_else(|| String::from("anonymousFunction"));
        let mut ast_node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Kotlin);
        ast_node.source_range = node.start_byte() as u32..node.end_byte() as u32;
        self.set_name_vector(&mut ast_node, &name);

        let node_id = ctx.dag.add_node(ast_node);
        ctx.nodes_created += 1;
        Ok(Some(node_id as usize))
    }

    fn process_node_simple(&self, ctx: &mut ParseContext, node: Node) -> Result<Option<usize>> {
        let kind = node.kind();
        if kind == "class_declaration" {
            self.process_class(ctx, node)
        } else if kind == "object_declaration" {
            self.process_object(ctx, node)
        } else if kind == "function_declaration" {
            self.process_function(ctx, node)
        } else if kind == "enum_class_declaration" {
            self.process_enum(ctx, node)
        } else {
            Ok(None)
        }
    }

    fn set_name_vector(&self, _node: &mut UnifiedAstNode, _name: &str) {
        // Simplified implementation for now
    }

    /// Extract enum name from enum class declaration
    fn extract_enum_name(&self, ctx: &mut ParseContext, node: Node) -> Option<String> {
        // Extract the enum name from source like "enum class Status {"
        let source_text = &ctx.content[node.start_byte()..node.end_byte()];
        if let Some(first_line) = source_text.lines().next() {
            let words: Vec<&str> = first_line.split_whitespace().collect();
            // Look for pattern: enum class Name
            if words.len() >= 3 && words[0] == "enum" && words[1] == "class" {
                let name = words[2].trim_end_matches('{').trim();
                return Some(name.to_string());
            }
        }
        None
    }

    /// Extract identifier from a node by looking for a child with the given kind
    fn extract_identifier(&self, ctx: &mut ParseContext, node: Node, identifier_kind: &str) -> Option<String> {
        // Check if the node is actually an enum by looking at the source
        let source_start = node.start_byte();
        let source_end = (source_start + 20).min(node.end_byte()).min(ctx.content.len());
        let source_prefix = &ctx.content[source_start..source_end];
        let is_enum = source_prefix.starts_with("enum ");
        
        let mut found_identifiers = Vec::new();
        
        // For enum class, let's try a different approach - parse from source
        if is_enum {
            // This is handled by extract_enum_name method, so just return early
            return None;
        }
        
        // Normal identifier extraction for non-enums
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == identifier_kind {
                let start = child.start_byte();
                let end = child.end_byte();
                if start < ctx.content.len() && end <= ctx.content.len() && start < end {
                    let text = ctx.content[start..end].to_string();
                    found_identifiers.push(text);
                }
            }
        }
        
        // For enum class, we need to skip "class" which might be picked up as an identifier
        if is_enum && found_identifiers.len() > 1 {
            // Return the last identifier (should be the actual enum name)
            found_identifiers.into_iter().rev().find(|s| s != "class" && s != "enum")
        } else {
            found_identifiers.into_iter().next()
        }
    }
}

/// Parse context for building the AST with safety limits
struct ParseContext<'a> {
    #[allow(dead_code)]
    content: &'a str,
    dag: &'a mut AstDag,
    #[allow(dead_code)]
    path: String,
    #[allow(dead_code)]
    stack: Vec<usize>,
    #[allow(dead_code)]
    node_map: HashMap<usize, usize>,
    // Safety fields to prevent memory exhaustion
    start_time: Instant,
    max_depth: usize,
    timeout: Duration,
    current_depth: usize,
    nodes_created: usize,
}
