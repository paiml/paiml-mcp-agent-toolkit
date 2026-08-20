// Tests for MakefileParser
// Included by parser.rs - shares parent module scope (no `use` imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_new() {
        let parser = MakefileParser::new("test input");
        assert_eq!(parser.cursor, 0);
        assert_eq!(parser.line, 1);
        assert_eq!(parser.column, 1);
        assert!(parser.errors.is_empty());
    }

    #[test]
    fn test_parse_empty_file() {
        let mut parser = MakefileParser::new("");
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.nodes.len(), 0);
    }

    #[test]
    fn test_parse_simple_rule() {
        let input = "test: dep1 dep2\n\techo hello";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(!ast.nodes.is_empty());

        // Check rule node
        let rule_node = &ast.nodes[0];
        assert_eq!(rule_node.kind, MakefileNodeKind::Rule);
        if let NodeData::Rule {
            targets,
            prerequisites,
            ..
        } = &rule_node.data
        {
            assert_eq!(targets, &vec!["test".to_string()]);
            assert_eq!(prerequisites, &vec!["dep1".to_string(), "dep2".to_string()]);
        } else {
            panic!("Expected Rule node data");
        }
    }

    #[test]
    fn test_parse_variable() {
        let input = "CC = gcc\nCFLAGS := -Wall\nLDFLAGS += -lm";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let vars = ast.get_variables();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].0, "CC");
        assert_eq!(vars[0].2, "gcc");
        assert_eq!(vars[1].0, "CFLAGS");
        assert_eq!(vars[1].2, "-Wall");
    }

    #[test]
    fn test_parse_comment() {
        let input = "# This is a comment\ntest:\n\techo test";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        // Find comment node
        let comment_node = ast
            .nodes
            .iter()
            .find(|n| n.kind == MakefileNodeKind::Comment);
        assert!(comment_node.is_some());
        if let NodeData::Text(text) = &comment_node.unwrap().data {
            assert!(text.contains("This is a comment"));
        }
    }

    #[test]
    fn test_parse_pattern_rule() {
        let input = "%.o: %.c\n\tgcc -c $< -o $@";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        assert!(ast.has_pattern_rules());
    }

    #[test]
    fn test_parse_phony_rule() {
        let input = ".PHONY: clean test\nclean:\n\trm -f *.o";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let phony_targets = ast.get_phony_targets();
        assert_eq!(phony_targets.len(), 2);
        assert!(phony_targets.contains(&"clean".to_string()));
        assert!(phony_targets.contains(&"test".to_string()));
    }

    #[test]
    fn test_parse_double_colon_rule() {
        let input = "all:: target1\nall:: target2";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let all_rules = ast.find_rules_by_target("all");
        assert_eq!(all_rules.len(), 2);
    }

    #[test]
    fn test_parse_include() {
        let input = "include config.mk\n-include optional.mk";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let include_nodes: Vec<_> = ast
            .nodes
            .iter()
            .filter(|n| n.kind == MakefileNodeKind::Include)
            .collect();
        assert_eq!(include_nodes.len(), 2);
    }

    #[test]
    fn test_parse_recipe_with_prefixes() {
        let input = "test:\n\t@echo Starting test\n\t-rm -f temp\n\t+make subtarget";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        // Find recipe node
        let recipe_node = ast
            .nodes
            .iter()
            .find(|n| n.kind == MakefileNodeKind::Recipe);
        assert!(recipe_node.is_some());

        if let NodeData::Recipe { lines } = &recipe_node.unwrap().data {
            assert_eq!(lines.len(), 3);
            assert!(lines[0].prefixes.silent); // @ prefix
            assert!(lines[1].prefixes.ignore_error); // - prefix
            assert!(lines[2].prefixes.always_exec); // + prefix
        }
    }

    #[test]
    fn test_parse_automatic_variables() {
        let input = "%.o: %.c\n\tgcc -c $< -o $@\n\techo $^ $?";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        assert!(ast.uses_automatic_variables());
    }

    #[test]
    fn test_parse_errors() {
        // Test invalid variable name
        let input = "= value";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_err());

        // Test that parse returns Ok but with empty AST for unknown lines
        let input2 = "unknown line";
        let mut parser2 = MakefileParser::new(input2);
        let result2 = parser2.parse();
        assert!(result2.is_ok());
        let ast = result2.unwrap();
        assert_eq!(ast.nodes.len(), 0); // Unknown lines are skipped
    }

    #[test]
    fn test_skip_functions() {
        // skip_spaces should NOT skip tabs (only spaces and carriage returns)
        let mut parser = MakefileParser::new("  \t  text after spaces");
        parser.skip_spaces();
        assert_eq!(parser.peek(), Some('\t'));

        let mut parser2 = MakefileParser::new("line1\nline2");
        parser2.skip_to_next_line();
        assert_eq!(parser2.peek(), Some('l'));
        assert_eq!(parser2.line, 2);
    }

    #[test]
    fn test_at_end() {
        let mut parser = MakefileParser::new("abc");
        assert!(!parser.at_end());
        parser.cursor = 3;
        assert!(parser.at_end());
    }

    #[test]
    fn test_advance() {
        let mut parser = MakefileParser::new("a\nb");
        assert_eq!(parser.line, 1);
        assert_eq!(parser.column, 1);

        parser.advance();
        assert_eq!(parser.column, 2);

        parser.advance(); // newline
        assert_eq!(parser.line, 2);
        assert_eq!(parser.column, 1);
    }

    #[test]
    fn test_starts_with() {
        let parser = MakefileParser::new("include file.mk");
        assert!(parser.starts_with("include"));
        assert!(!parser.starts_with("exclude"));
    }

    /// Every recipe line of `rule`, in order.
    fn recipe_text(ast: &MakefileAst, rule_idx: usize) -> Vec<String> {
        ast.nodes[rule_idx]
            .children
            .iter()
            .filter_map(|idx| match &ast.nodes[*idx].data {
                NodeData::Recipe { lines } => Some(lines),
                _ => None,
            })
            .flatten()
            .map(|line| line.text.clone())
            .collect()
    }

    /// GNU make: "Blank lines ... may appear among the recipe lines; they are
    /// ignored." `make` runs both commands below; this parser used to stop the
    /// recipe at the blank line and then reject `\t@echo two` as a recipe with
    /// no rule above it.
    #[test]
    fn a_blank_line_does_not_end_a_recipe() {
        let input = "all:\n\t@echo one\n\n\t@echo two\n";
        let ast = MakefileParser::new(input)
            .parse()
            .expect("GNU make accepts a blank line inside a recipe");
        assert_eq!(
            recipe_text(&ast, 0),
            vec!["echo one".to_string(), "echo two".to_string()],
            "both commands belong to `all`"
        );
    }

    /// The shape that made `pmat analyze makefile Makefile` exit 4 on pmat's own
    /// Makefile: comment lines between the target line and its first command.
    #[test]
    fn comment_lines_do_not_end_a_recipe() {
        let input = "all:\n# why this target exists\n# and a second line of it\n\t@echo one\n\t@echo two\n";
        let ast = MakefileParser::new(input)
            .parse()
            .expect("GNU make accepts comment lines among recipe lines");
        assert_eq!(
            recipe_text(&ast, 0),
            vec!["echo one".to_string(), "echo two".to_string()],
            "the commands still belong to `all`"
        );
        // The comments reached the AST before this fix (via the top-level
        // comment branch, as a side effect of the recipe being cut short); they
        // must still be there.
        let comments = ast
            .nodes
            .iter()
            .filter(|node| node.kind == MakefileNodeKind::Comment)
            .count();
        assert_eq!(comments, 2, "comment nodes must not be swallowed");
    }

    /// Mixed blanks and comments, and a rule that follows the recipe: the
    /// recipe must resume across the ignorable lines and still stop at `next:`.
    #[test]
    fn blanks_and_comments_interleaved_with_recipe_lines() {
        let input = "\
first:
\t@echo a

# a note

\t@echo b
\t@echo c

next:
\t@echo d
";
        let ast = MakefileParser::new(input)
            .parse()
            .expect("blank and comment lines are ignored among recipe lines");
        let rules: Vec<usize> = ast
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.kind == MakefileNodeKind::Rule)
            .map(|(idx, _)| idx)
            .collect();
        assert_eq!(rules.len(), 2, "two rules");
        assert_eq!(
            recipe_text(&ast, rules[0]),
            vec![
                "echo a".to_string(),
                "echo b".to_string(),
                "echo c".to_string()
            ]
        );
        assert_eq!(recipe_text(&ast, rules[1]), vec!["echo d".to_string()]);
    }

    /// The blank/comment rule must not resurrect a genuinely orphaned recipe:
    /// GNU make rejects this file with "recipe commences before first target",
    /// and so must the parser.
    #[test]
    fn a_recipe_with_no_rule_above_it_is_still_an_error() {
        for input in [
            "\techo orphan\n",
            "# just a comment\n\n\techo orphan\n",
            "VAR = 1\n\n\techo orphan\n",
        ] {
            let errors = MakefileParser::new(input)
                .parse()
                .expect_err("a recipe with no rule above it is a parse error");
            assert!(
                errors.iter().any(|e| matches!(
                    e,
                    ParseError::InvalidSyntax(msg) if msg == "Recipe without rule"
                )),
                "expected `Recipe without rule` for {input:?}, got {errors:?}"
            );
        }
    }

    /// Dogfood: pmat must parse the Makefile it ships with. GNU make runs this
    /// file; `pmat analyze makefile Makefile` answered
    /// `[InvalidSyntax("Recipe without rule") x12]` and exit 4.
    #[test]
    fn pmats_own_makefile_parses() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Makefile");
        let source = std::fs::read_to_string(&path).expect("pmat ships a Makefile");
        if let Err(errors) = MakefileParser::new(&source).parse() {
            panic!(
                "{} does not parse, but GNU make runs it: {errors:?}",
                path.display()
            );
        }
    }
}

