impl TdgAnalyzerAst {
    fn analyze_javascript_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "typescript-ast")]
        {
            use swc_common::{sync::Lrc, FileName, SourceMap};
            use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
            use swc_ecma_visit::VisitWith;

            let cm: Lrc<SourceMap> = Default::default();
            let fm = cm.new_source_file(
                FileName::Custom("test.js".into()).into(),
                source.to_string(),
            );

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: true,
                    decorators: true,
                    ..Default::default()
                }),
                Default::default(),
                StringInput::from(&*fm),
                None,
            );

            let mut parser = Parser::new_from(lexer);

            match parser.parse_module() {
                Ok(module) => {
                    let mut visitor = JavaScriptComplexityVisitor::new();
                    module.visit_with(&mut visitor);

                    score.structural_complexity = self.score_structural_complexity(
                        visitor.cyclomatic_complexity,
                        visitor.cognitive_complexity,
                        visitor.max_nesting_depth,
                        visitor.max_function_length,
                        tracker,
                    );

                    score.semantic_complexity = self.score_semantic_complexity(
                        visitor.max_params,
                        visitor.async_count,
                        visitor.callback_depth,
                        tracker,
                    );

                    score.duplication_ratio =
                        self.analyze_duplication_ast(source, Language::JavaScript, tracker);

                    score.coupling_score = self.score_coupling(
                        visitor.import_count,
                        visitor.external_calls,
                        visitor.class_count,
                        tracker,
                    );

                    score.doc_coverage = self.score_documentation(
                        visitor.jsdoc_count,
                        visitor.function_count,
                        visitor.comment_lines,
                        visitor.total_lines,
                        tracker,
                    );

                    score.consistency_score = self.score_consistency_javascript(source, tracker);

                    score.entropy_score =
                        self.score_entropy_analysis(source, Language::JavaScript, tracker);
                }
                Err(_) => {
                    self.analyze_heuristic(source, score, tracker)?;
                }
            }
        }
        #[cfg(not(feature = "typescript-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_go_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "go-ast")]
        {
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_go::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Go language: {e}"))?;

            if let Some(tree) = parser.parse(source, None) {
                let mut visitor = GoComplexityVisitor::new(source);
                visitor.analyze_tree(&tree);

                score.structural_complexity = self.score_structural_complexity(
                    visitor.cyclomatic_complexity,
                    visitor.cognitive_complexity,
                    visitor.max_nesting_depth,
                    visitor.max_method_length,
                    tracker,
                );

                score.semantic_complexity = self.score_semantic_complexity(
                    visitor.max_params,
                    visitor.interface_count,
                    0,
                    tracker,
                );

                score.duplication_ratio =
                    self.analyze_duplication_ast(source, Language::Go, tracker);

                score.coupling_score = self.score_coupling(
                    visitor.import_count,
                    visitor.external_calls,
                    visitor.interface_count,
                    tracker,
                );

                score.doc_coverage = self.score_documentation(
                    visitor.documented_functions,
                    visitor.total_functions,
                    visitor.comment_lines,
                    visitor.total_lines,
                    tracker,
                );

                score.consistency_score = self.config.weights.consistency;

                score.entropy_score =
                    self.score_entropy_analysis(source, Language::Go, tracker);
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "go-ast"))]
        {
            self.analyze_tree_sitter_generic(source, Language::Go, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_java_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Java requires tree-sitter-java which we don't have yet
        // Use tree-sitter generic parser as fallback
        self.analyze_tree_sitter_generic(source, Language::Java, score, tracker)
    }

    fn analyze_lua_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "lua-ast")]
        {
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_lua::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Lua language: {e}"))?;

            if let Some(tree) = parser.parse(source, None) {
                let mut visitor = LuaComplexityVisitor::new(source);
                visitor.analyze_tree(&tree);

                score.structural_complexity = self.score_structural_complexity(
                    visitor.cyclomatic_complexity,
                    visitor.cognitive_complexity,
                    visitor.max_nesting_depth,
                    visitor.max_method_length,
                    tracker,
                );

                score.semantic_complexity = self.score_semantic_complexity(
                    visitor.max_params,
                    visitor.metatable_count,
                    0, // Lua has no abstraction levels like generics
                    tracker,
                );

                score.duplication_ratio =
                    self.analyze_duplication_ast(source, Language::Lua, tracker);

                score.coupling_score = self.score_coupling(
                    visitor.import_count,
                    visitor.external_calls,
                    0, // Lua has no explicit interfaces
                    tracker,
                );

                score.doc_coverage = self.score_documentation(
                    visitor.documented_functions,
                    visitor.total_functions,
                    visitor.comment_lines,
                    visitor.total_lines,
                    tracker,
                );

                score.consistency_score = self.score_consistency_lua(source, tracker);

                score.entropy_score = self.score_entropy_analysis(source, Language::Lua, tracker);
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "lua-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_c_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "c-ast")]
        {
            use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

            let mut parser = Parser::new();
            let language = if score.language == Language::Cpp {
                tree_sitter_cpp::LANGUAGE.into()
            } else {
                tree_sitter_c::LANGUAGE.into()
            };

            parser
                .set_language(&language)
                .map_err(|e| anyhow::anyhow!("Failed to set language: {e:?}"))?;

            if let Some(tree) = parser.parse(source, None) {
                let root_node = tree.root_node();

                // Count control flow statements for cyclomatic complexity
                let query_str = r"
                    (if_statement) @if
                    (while_statement) @while
                    (for_statement) @for
                    (do_statement) @do
                    (switch_statement) @switch
                    (case_statement) @case
                    (conditional_expression) @ternary
                ";

                let query = Query::new(&language, query_str)
                    .map_err(|e| anyhow::anyhow!("Query error: {e:?}"))?;

                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, root_node, source.as_bytes());
                let cyclomatic = 1 + matches.count() as u32;

                score.structural_complexity = self.score_structural_complexity(
                    cyclomatic,
                    self.calculate_cognitive_complexity(&root_node),
                    self.calculate_max_nesting(&root_node),
                    self.calculate_max_function_length(&root_node, source),
                    tracker,
                );

                // Count includes for coupling
                let include_query = Query::new(&language, "(preproc_include) @include")
                    .map_err(|e| anyhow::anyhow!("Query error: {e:?}"))?;

                let mut cursor = QueryCursor::new();
                let include_matches = cursor.matches(&include_query, root_node, source.as_bytes());
                let import_count = include_matches.count() as u32;

                score.coupling_score = self.score_coupling(import_count, 0, 0, tracker);

                // Basic documentation coverage
                let comment_query = Query::new(&language, "(comment) @comment")
                    .map_err(|e| anyhow::anyhow!("Query error: {e:?}"))?;

                let mut cursor = QueryCursor::new();
                let comment_matches = cursor.matches(&comment_query, root_node, source.as_bytes());
                let comment_count = comment_matches.count() as u32;

                score.doc_coverage = (comment_count as f32 / source.lines().count() as f32 * 10.0)
                    .min(self.config.weights.documentation);

                score.semantic_complexity = self.config.weights.semantic_complexity;
                score.duplication_ratio =
                    self.analyze_duplication_ast(source, score.language, tracker);
                score.consistency_score = self.config.weights.consistency;

                score.entropy_score =
                    self.score_entropy_analysis(source, score.language, tracker);
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "c-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }
}
