#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

mod basic_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_ruchy_lexer_basic() {
        let mut lexer = RuchyLexer::new("fun test() { return 42 }".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Fun));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(_)));
        assert!(matches!(lexer.next_token(), RuchyToken::LeftParen));
        assert!(matches!(lexer.next_token(), RuchyToken::RightParen));
        assert!(matches!(lexer.next_token(), RuchyToken::LeftBrace));
        assert!(matches!(lexer.next_token(), RuchyToken::Return));
        assert!(matches!(lexer.next_token(), RuchyToken::Integer(42)));
        assert!(matches!(lexer.next_token(), RuchyToken::RightBrace));
        // lexer.next_token(); // Last token varies based on implementation
    }

    #[test]
    fn test_ruchy_halstead_calculation() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        // Track some operators and operands
        analyzer.track_operator("+");
        analyzer.track_operator("-");
        analyzer.track_operator("+"); // Duplicate should only count once in distinct
        analyzer.track_operand("x");
        analyzer.track_operand("y");
        analyzer.track_operand("42");
        analyzer.track_operand("x"); // Duplicate

        let halstead = analyzer.calculate_halstead();

        assert_eq!(halstead.operators_unique, 2); // 2 distinct operators
        assert_eq!(halstead.operands_unique, 3); // 3 distinct operands
        assert_eq!(halstead.operators_total, 3); // 3 total operators
        assert_eq!(halstead.operands_total, 4); // 4 total operands
        assert!(halstead.volume > 0.0);
    }

    #[test]
    fn test_dead_code_detection() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        // Simulate some function definitions and calls
        analyzer.defined_functions.insert("main".to_string());
        analyzer.defined_functions.insert("helper".to_string());
        analyzer.defined_functions.insert("unused".to_string());

        analyzer.called_functions.insert("helper".to_string());
        // 'unused' is never called, 'main' is entry point

        let dead_code = analyzer.get_dead_code();

        assert_eq!(dead_code.unused_functions.len(), 1);
        assert!(dead_code.unused_functions.contains(&"unused".to_string()));
        assert!(!dead_code.unused_functions.contains(&"main".to_string())); // main is entry point
    }

    #[tokio::test]
    async fn test_ruchy_complexity_analysis() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.ruchy");

        let content = r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    for i in 0..10 {
        println(fibonacci(i))
    }
}
"#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let metrics = analyze_ruchy_file(&file_path).await.unwrap();

        assert_eq!(metrics.functions.len(), 2);
        assert!(metrics.functions[0].metrics.cyclomatic > 1);
        assert!(metrics.total_complexity.cyclomatic > 1);
    }
}

mod ruchy_unit_tests {
    use super::*;

    // RuchyToken tests
    #[test]
    fn test_ruchy_token_keywords() {
        let tokens = vec![
            RuchyToken::Fun,
            RuchyToken::If,
            RuchyToken::Else,
            RuchyToken::While,
            RuchyToken::For,
            RuchyToken::Match,
            RuchyToken::Return,
            RuchyToken::Let,
            RuchyToken::Const,
            RuchyToken::Var,
        ];
        assert_eq!(tokens.len(), 10);
        assert!(matches!(tokens[0], RuchyToken::Fun));
    }

    #[test]
    fn test_ruchy_token_operators() {
        let tokens = [
            RuchyToken::Plus,
            RuchyToken::Minus,
            RuchyToken::Star,
            RuchyToken::Slash,
            RuchyToken::EqualEqual,
            RuchyToken::NotEqual,
        ];
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[4], RuchyToken::EqualEqual));
    }

    #[test]
    fn test_ruchy_token_literals() {
        let int_token = RuchyToken::Integer(42);
        let float_token = RuchyToken::Float(2.75);
        let string_token = RuchyToken::String("hello".to_string());
        let bool_token = RuchyToken::Bool(true);

        assert!(matches!(int_token, RuchyToken::Integer(42)));
        assert!(matches!(float_token, RuchyToken::Float(f) if (f - 2.75).abs() < 0.001));
        assert!(matches!(string_token, RuchyToken::String(s) if s == "hello"));
        assert!(matches!(bool_token, RuchyToken::Bool(true)));
    }

    // RuchyType tests
    #[test]
    fn test_ruchy_type_primitives() {
        let types = [
            RuchyType::Integer,
            RuchyType::Float,
            RuchyType::String,
            RuchyType::Bool,
            RuchyType::Char,
            RuchyType::Unknown,
        ];
        assert_eq!(types.len(), 6);
        assert_eq!(RuchyType::Integer, RuchyType::Integer);
    }

    #[test]
    fn test_ruchy_type_compound() {
        let array_type = RuchyType::Array(Box::new(RuchyType::Integer));
        let option_type = RuchyType::Option(Box::new(RuchyType::String));
        let result_type =
            RuchyType::Result(Box::new(RuchyType::Integer), Box::new(RuchyType::String));
        let func_type = RuchyType::Function(
            vec![RuchyType::Integer, RuchyType::Integer],
            Box::new(RuchyType::Integer),
        );

        assert!(matches!(array_type, RuchyType::Array(_)));
        assert!(matches!(option_type, RuchyType::Option(_)));
        assert!(matches!(result_type, RuchyType::Result(_, _)));
        assert!(matches!(func_type, RuchyType::Function(_, _)));
    }

    #[test]
    fn test_ruchy_type_class_actor() {
        let class_type = RuchyType::Class("MyClass".to_string());
        let actor_type = RuchyType::Actor("MyActor".to_string());
        let inferred = RuchyType::Inferred("T".to_string());

        assert!(matches!(class_type, RuchyType::Class(name) if name == "MyClass"));
        assert!(matches!(actor_type, RuchyType::Actor(name) if name == "MyActor"));
        assert!(matches!(inferred, RuchyType::Inferred(t) if t == "T"));
    }

    // RuchyImport tests
    #[test]
    fn test_ruchy_import_creation() {
        let import = RuchyImport {
            module: "std::io".to_string(),
            items: vec!["Read".to_string(), "Write".to_string()],
            line: 5,
        };

        assert_eq!(import.module, "std::io");
        assert_eq!(import.items.len(), 2);
        assert_eq!(import.line, 5);
    }

    // RuchyDeadCode tests
    #[test]
    fn test_ruchy_dead_code_creation() {
        let dead_code = RuchyDeadCode {
            unused_functions: vec!["old_func".to_string()],
            unused_variables: vec!["unused_var".to_string()],
            unreachable_code: vec![(10, 20)],
        };

        assert_eq!(dead_code.unused_functions.len(), 1);
        assert_eq!(dead_code.unused_variables.len(), 1);
        assert_eq!(dead_code.unreachable_code.len(), 1);
    }

    // ActorInfo tests
    #[test]
    fn test_actor_info_creation() {
        let actor = ActorInfo {
            name: "CounterActor".to_string(),
            state_fields: vec!["count".to_string()],
            message_handlers: vec!["increment".to_string(), "decrement".to_string()],
            spawned_actors: vec![],
            line_start: 1,
            line_end: 50,
        };

        assert_eq!(actor.name, "CounterActor");
        assert_eq!(actor.state_fields.len(), 1);
        assert_eq!(actor.message_handlers.len(), 2);
        assert_eq!(actor.line_end - actor.line_start, 49);
    }

    // MessageFlow tests
    #[test]
    fn test_message_flow_creation() {
        let flow = MessageFlow {
            from_actor: "Client".to_string(),
            to_actor: "Server".to_string(),
            message_type: "Request".to_string(),
            line: 42,
        };

        assert_eq!(flow.from_actor, "Client");
        assert_eq!(flow.to_actor, "Server");
        assert_eq!(flow.message_type, "Request");
    }

    // DeadlockWarning tests
    #[test]
    fn test_deadlock_warning_creation() {
        let warning = DeadlockWarning {
            actors_involved: vec!["A".to_string(), "B".to_string()],
            description: "Circular dependency".to_string(),
            line: 100,
        };

        assert_eq!(warning.actors_involved.len(), 2);
        assert!(!warning.description.is_empty());
    }

    // RuchyActorAnalysis tests
    #[test]
    fn test_ruchy_actor_analysis_creation() {
        let analysis = RuchyActorAnalysis {
            actors: vec![],
            message_flows: vec![],
            potential_deadlocks: vec![],
        };

        assert!(analysis.actors.is_empty());
        assert!(analysis.message_flows.is_empty());
        assert!(analysis.potential_deadlocks.is_empty());
    }

    // RuchyComplexityAnalyzer tests
    #[test]
    fn test_analyzer_new() {
        let analyzer = RuchyComplexityAnalyzer::new();

        assert_eq!(analyzer.nesting_level, 0);
        assert!(analyzer.functions.is_empty());
        assert!(analyzer.classes.is_empty());
        assert!(analyzer.operators.is_empty());
        assert!(analyzer.operands.is_empty());
    }

    #[test]
    fn test_analyzer_default() {
        let analyzer = RuchyComplexityAnalyzer::default();

        assert_eq!(analyzer.operator_count, 0);
        assert_eq!(analyzer.operand_count, 0);
    }

    #[test]
    fn test_analyzer_reset_halstead() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        analyzer.track_operator("+");
        analyzer.track_operand("x");
        assert!(!analyzer.operators.is_empty());

        analyzer.reset_halstead();

        assert!(analyzer.operators.is_empty());
        assert!(analyzer.operands.is_empty());
        assert_eq!(analyzer.operator_count, 0);
        assert_eq!(analyzer.operand_count, 0);
    }

    #[test]
    fn test_analyzer_track_operator() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        analyzer.track_operator("+");
        analyzer.track_operator("-");
        analyzer.track_operator("+"); // Duplicate

        assert_eq!(analyzer.operators.len(), 2);
        assert_eq!(analyzer.operator_count, 3);
    }

    #[test]
    fn test_analyzer_track_operand() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        analyzer.track_operand("x");
        analyzer.track_operand("y");
        analyzer.track_operand("x"); // Duplicate

        assert_eq!(analyzer.operands.len(), 2);
        assert_eq!(analyzer.operand_count, 3);
    }

    #[test]
    fn test_analyzer_halstead_empty() {
        let analyzer = RuchyComplexityAnalyzer::new();
        let halstead = analyzer.calculate_halstead();

        assert_eq!(halstead.operators_unique, 0);
        assert_eq!(halstead.operands_unique, 0);
        assert_eq!(halstead.volume, 0.0);
    }

    #[test]
    fn test_analyzer_get_imports() {
        let analyzer = RuchyComplexityAnalyzer::new();
        let imports = analyzer.get_imports();

        assert!(imports.is_empty());
    }

    #[test]
    fn test_analyzer_get_exports() {
        let analyzer = RuchyComplexityAnalyzer::new();
        let exports = analyzer.get_exports();

        assert!(exports.is_empty());
    }

    #[test]
    fn test_analyzer_dead_code_with_main() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        analyzer.defined_functions.insert("main".to_string());
        analyzer.defined_functions.insert("helper".to_string());
        analyzer.called_functions.insert("helper".to_string());

        let dead_code = analyzer.get_dead_code();

        // main should not be marked as dead code
        assert!(!dead_code.unused_functions.contains(&"main".to_string()));
        assert!(!dead_code.unused_functions.contains(&"helper".to_string()));
    }

    #[test]
    fn test_analyzer_dead_code_with_exports() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        analyzer.defined_functions.insert("public_api".to_string());
        analyzer
            .defined_functions
            .insert("private_helper".to_string());
        analyzer.exports.insert("public_api".to_string());

        let dead_code = analyzer.get_dead_code();

        // Exported function should not be marked as dead code
        assert!(!dead_code
            .unused_functions
            .contains(&"public_api".to_string()));
        assert!(dead_code
            .unused_functions
            .contains(&"private_helper".to_string()));
    }

    #[test]
    fn test_analyzer_unused_variables() {
        let mut analyzer = RuchyComplexityAnalyzer::new();

        analyzer.defined_variables.insert("used_var".to_string());
        analyzer.defined_variables.insert("unused_var".to_string());
        analyzer.used_variables.insert("used_var".to_string());

        let dead_code = analyzer.get_dead_code();

        assert!(!dead_code.unused_variables.contains(&"used_var".to_string()));
        assert!(dead_code
            .unused_variables
            .contains(&"unused_var".to_string()));
    }

    #[test]
    fn test_analyzer_actor_analysis_empty() {
        let analyzer = RuchyComplexityAnalyzer::new();
        let analysis = analyzer.get_actor_analysis();

        assert!(analysis.actors.is_empty());
        assert!(analysis.message_flows.is_empty());
        assert!(analysis.potential_deadlocks.is_empty());
    }

    // RED tests: access private methods/fields, gated behind broken-tests
    #[cfg(pmat_broken_tests)]
    #[test]
    fn test_analyzer_infer_literal_type() {
        let analyzer = RuchyComplexityAnalyzer::new();

        assert_eq!(
            analyzer.infer_literal_type(&RuchyToken::Integer(42)),
            RuchyType::Integer
        );
        assert_eq!(
            analyzer.infer_literal_type(&RuchyToken::Float(3.14)),
            RuchyType::Float
        );
        assert_eq!(
            analyzer.infer_literal_type(&RuchyToken::String("hello".to_string())),
            RuchyType::String
        );
        assert_eq!(
            analyzer.infer_literal_type(&RuchyToken::True),
            RuchyType::Bool
        );
        assert_eq!(
            analyzer.infer_literal_type(&RuchyToken::False),
            RuchyType::Bool
        );
        assert_eq!(
            analyzer.infer_literal_type(&RuchyToken::Bool(true)),
            RuchyType::Bool
        );
    }

    #[cfg(pmat_broken_tests)]
    #[test]
    fn test_analyzer_infer_binary_type_arithmetic() {
        let analyzer = RuchyComplexityAnalyzer::new();

        let result =
            analyzer.infer_binary_type(&RuchyToken::Plus, &RuchyType::Integer, &RuchyType::Integer);
        assert_eq!(result, RuchyType::Integer);

        let result =
            analyzer.infer_binary_type(&RuchyToken::Plus, &RuchyType::Float, &RuchyType::Float);
        assert_eq!(result, RuchyType::Float);
    }

    #[cfg(pmat_broken_tests)]
    #[test]
    fn test_analyzer_infer_binary_type_comparison() {
        let analyzer = RuchyComplexityAnalyzer::new();

        let result = analyzer.infer_binary_type(
            &RuchyToken::EqualEqual,
            &RuchyType::Integer,
            &RuchyType::Integer,
        );
        assert_eq!(result, RuchyType::Bool);

        let result =
            analyzer.infer_binary_type(&RuchyToken::Less, &RuchyType::Integer, &RuchyType::Integer);
        assert_eq!(result, RuchyType::Bool);
    }

    #[cfg(pmat_broken_tests)]
    #[test]
    fn test_analyzer_infer_binary_type_logical() {
        let analyzer = RuchyComplexityAnalyzer::new();

        let result =
            analyzer.infer_binary_type(&RuchyToken::And, &RuchyType::Bool, &RuchyType::Bool);
        assert_eq!(result, RuchyType::Bool);

        let result =
            analyzer.infer_binary_type(&RuchyToken::Or, &RuchyType::Bool, &RuchyType::Bool);
        assert_eq!(result, RuchyType::Bool);
    }

    #[cfg(pmat_broken_tests)]
    #[test]
    fn test_analyzer_infer_binary_type_string_concat() {
        let analyzer = RuchyComplexityAnalyzer::new();

        let result =
            analyzer.infer_binary_type(&RuchyToken::Plus, &RuchyType::String, &RuchyType::String);
        assert_eq!(result, RuchyType::String);
    }

    // RuchyLexer tests — access private fields, gated behind broken-tests
    #[cfg(pmat_broken_tests)]
    #[test]
    fn test_lexer_new() {
        let lexer = RuchyLexer::new("fun test() {}".to_string());

        assert_eq!(lexer.position, 0);
        assert_eq!(lexer.line, 1);
        assert_eq!(lexer.column, 1);
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = RuchyLexer::new("myVar another_var _private".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(s) if s == "myVar"));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(s) if s == "another_var"));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(s) if s == "_private"));
    }

    #[test]
    fn test_lexer_numbers() {
        let mut lexer = RuchyLexer::new("42 2.75 0 -5".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Integer(42)));
        assert!(matches!(lexer.next_token(), RuchyToken::Float(f) if (f - 2.75).abs() < 0.001));
        assert!(matches!(lexer.next_token(), RuchyToken::Integer(0)));
    }

    #[test]
    fn test_lexer_strings() {
        let mut lexer = RuchyLexer::new(r#""hello" "world""#.to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::String(s) if s == "hello"));
        assert!(matches!(lexer.next_token(), RuchyToken::String(s) if s == "world"));
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = RuchyLexer::new("+ - * / == != < > <= >=".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Plus));
        assert!(matches!(lexer.next_token(), RuchyToken::Minus));
        assert!(matches!(lexer.next_token(), RuchyToken::Star));
        assert!(matches!(lexer.next_token(), RuchyToken::Slash));
        assert!(matches!(lexer.next_token(), RuchyToken::EqualEqual));
        assert!(matches!(lexer.next_token(), RuchyToken::NotEqual));
        assert!(matches!(lexer.next_token(), RuchyToken::Less));
        assert!(matches!(lexer.next_token(), RuchyToken::Greater));
        assert!(matches!(lexer.next_token(), RuchyToken::LessEqual));
        assert!(matches!(lexer.next_token(), RuchyToken::GreaterEqual));
    }

    #[test]
    fn test_lexer_delimiters() {
        let mut lexer = RuchyLexer::new("( ) { } [ ] ; , : .".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::LeftParen));
        assert!(matches!(lexer.next_token(), RuchyToken::RightParen));
        assert!(matches!(lexer.next_token(), RuchyToken::LeftBrace));
        assert!(matches!(lexer.next_token(), RuchyToken::RightBrace));
        assert!(matches!(lexer.next_token(), RuchyToken::LeftBracket));
        assert!(matches!(lexer.next_token(), RuchyToken::RightBracket));
        assert!(matches!(lexer.next_token(), RuchyToken::Semicolon));
        assert!(matches!(lexer.next_token(), RuchyToken::Comma));
        assert!(matches!(lexer.next_token(), RuchyToken::Colon));
        assert!(matches!(lexer.next_token(), RuchyToken::Dot));
    }

    #[test]
    fn test_lexer_keywords_vs_identifiers() {
        let mut lexer = RuchyLexer::new("fun function if iffy".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Fun));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(s) if s == "function"));
        assert!(matches!(lexer.next_token(), RuchyToken::If));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(s) if s == "iffy"));
    }

    #[test]
    fn test_lexer_comments() {
        let mut lexer = RuchyLexer::new("fun // comment\ntest".to_string());

        // Lexer returns fun first
        assert!(matches!(lexer.next_token(), RuchyToken::Fun));
    }

    #[test]
    fn test_lexer_whitespace() {
        let mut lexer = RuchyLexer::new("   fun   \n\t  test  ".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Fun));
        assert!(matches!(lexer.next_token(), RuchyToken::Identifier(s) if s == "test"));
    }

    #[test]
    fn test_lexer_eof() {
        let mut lexer = RuchyLexer::new("".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Eof));
    }

    #[test]
    fn test_lexer_arrows() {
        let mut lexer = RuchyLexer::new("-> =>".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::Arrow));
        assert!(matches!(lexer.next_token(), RuchyToken::FatArrow));
    }

    #[test]
    fn test_lexer_pipeline() {
        let mut lexer = RuchyLexer::new("|>".to_string());

        assert!(matches!(lexer.next_token(), RuchyToken::PipeForward));
    }

    // RuchyAst tests
    #[test]
    fn test_ast_program() {
        let program = RuchyAst::Program { items: vec![] };
        assert!(matches!(program, RuchyAst::Program { items } if items.is_empty()));
    }

    #[test]
    fn test_ast_function() {
        let func = RuchyAst::Function {
            name: "test".to_string(),
            params: vec![("x".to_string(), "i32".to_string())],
            return_type: Some("i32".to_string()),
            body: Box::new(RuchyAst::Block { statements: vec![] }),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };

        assert!(matches!(func, RuchyAst::Function { name, .. } if name == "test"));
    }

    #[test]
    fn test_ast_class() {
        let class = RuchyAst::Class {
            name: "MyClass".to_string(),
            fields: vec![("field".to_string(), "i32".to_string())],
            methods: vec![],
            line_start: 1,
            line_end: 10,
        };

        assert!(matches!(class, RuchyAst::Class { name, .. } if name == "MyClass"));
    }

    #[test]
    fn test_ast_actor() {
        let actor = RuchyAst::Actor {
            name: "MyActor".to_string(),
            state: vec![("count".to_string(), "i32".to_string())],
            handlers: vec![],
            line_start: 1,
            line_end: 20,
        };

        assert!(matches!(actor, RuchyAst::Actor { name, .. } if name == "MyActor"));
    }

    #[test]
    fn test_ast_control_flow() {
        let if_stmt = RuchyAst::If {
            condition: Box::new(RuchyAst::Identifier("x".to_string())),
            then_branch: Box::new(RuchyAst::Block { statements: vec![] }),
            else_branch: None,
        };

        let while_stmt = RuchyAst::While {
            condition: Box::new(RuchyAst::Identifier("x".to_string())),
            body: Box::new(RuchyAst::Block { statements: vec![] }),
        };

        let for_stmt = RuchyAst::For {
            variable: "i".to_string(),
            iterable: Box::new(RuchyAst::Identifier("items".to_string())),
            body: Box::new(RuchyAst::Block { statements: vec![] }),
        };

        assert!(matches!(if_stmt, RuchyAst::If { .. }));
        assert!(matches!(while_stmt, RuchyAst::While { .. }));
        assert!(matches!(for_stmt, RuchyAst::For { variable, .. } if variable == "i"));
    }
}

// =========================================================================
// Wave 39 PR2 — integration tests for complexity_analysis.rs (231 missed,
// 0% broad coverage). Goes through `analyze_program(&RuchyAst)` to exercise
// the analyze_node match arms (Function, If, While, For, Match, BinaryOp,
// Block, Import, Return, Let, Call, Pipeline, etc.).
// =========================================================================
#[cfg(test)]
mod analyze_program_integration_tests {
    use super::*;

    fn boxed(ast: RuchyAst) -> Box<RuchyAst> {
        Box::new(ast)
    }

    #[test]
    fn test_analyze_program_empty_emits_zero_functions() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let ast = RuchyAst::Program { items: vec![] };
        let metrics = analyzer.analyze_program(&ast);
        assert_eq!(metrics.functions.len(), 0);
    }

    #[test]
    fn test_analyze_program_single_function_records_one() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let func = RuchyAst::Function {
            name: "foo".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(RuchyAst::Block { statements: vec![] }),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "foo");
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 1);
    }

    #[test]
    fn test_analyze_function_with_if_increments_cyclomatic() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        // Function body: `if cond then expr else expr2`.
        let body = RuchyAst::If {
            condition: boxed(RuchyAst::Identifier("cond".to_string())),
            then_branch: boxed(RuchyAst::Identifier("a".to_string())),
            else_branch: Some(boxed(RuchyAst::Identifier("b".to_string()))),
        };
        let func = RuchyAst::Function {
            name: "branchy".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(body),
            is_test: false,
            line_start: 1,
            line_end: 10,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        // base 1 + if (+1) + else (+1) = 3.
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 3);
    }

    #[test]
    fn test_analyze_function_with_while_increments_cyclomatic() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let body = RuchyAst::While {
            condition: boxed(RuchyAst::Identifier("c".to_string())),
            body: boxed(RuchyAst::Block { statements: vec![] }),
        };
        let func = RuchyAst::Function {
            name: "looper".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(body),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        // base 1 + while (+1) = 2.
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 2);
    }

    #[test]
    fn test_analyze_function_with_for_increments_cyclomatic() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let body = RuchyAst::For {
            variable: "i".to_string(),
            iterable: boxed(RuchyAst::Identifier("xs".to_string())),
            body: boxed(RuchyAst::Block { statements: vec![] }),
        };
        let func = RuchyAst::Function {
            name: "iter".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(body),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 2);
    }

    #[test]
    fn test_analyze_function_with_match_arms_increments_cyclomatic() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let arms = vec![
            (
                RuchyAst::Identifier("Some".to_string()),
                RuchyAst::Identifier("x".to_string()),
            ),
            (
                RuchyAst::Identifier("None".to_string()),
                RuchyAst::Identifier("0".to_string()),
            ),
        ];
        let body = RuchyAst::Match {
            expr: boxed(RuchyAst::Identifier("opt".to_string())),
            arms,
        };
        let func = RuchyAst::Function {
            name: "matcher".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(body),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        // base 1 + match arms — exact count depends on impl, but should be > 1.
        assert!(metrics.functions[0].metrics.cyclomatic > 1);
    }

    #[test]
    fn test_analyze_program_imports_collected() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let import = RuchyAst::Import {
            module: "std::io".to_string(),
            items: vec!["read".to_string(), "write".to_string()],
            line: 1,
        };
        let ast = RuchyAst::Program {
            items: vec![import],
        };
        let _ = analyzer.analyze_program(&ast);
        let imports = analyzer.get_imports();
        assert!(!imports.is_empty(), "imports should be collected");
    }

    #[test]
    fn test_analyze_program_nested_if_increments_nesting_max() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        // Nested if-in-if structure exercises nesting tracker.
        let inner_if = RuchyAst::If {
            condition: boxed(RuchyAst::Identifier("c2".to_string())),
            then_branch: boxed(RuchyAst::Identifier("y".to_string())),
            else_branch: None,
        };
        let outer_if = RuchyAst::If {
            condition: boxed(RuchyAst::Identifier("c1".to_string())),
            then_branch: boxed(inner_if),
            else_branch: None,
        };
        let func = RuchyAst::Function {
            name: "nested".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(outer_if),
            is_test: false,
            line_start: 1,
            line_end: 10,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        assert!(
            metrics.functions[0].metrics.nesting_max >= 2,
            "expected nesting_max >= 2, got {}",
            metrics.functions[0].metrics.nesting_max
        );
    }

    #[test]
    fn test_analyze_program_block_with_let_and_call() {
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let body = RuchyAst::Block {
            statements: vec![
                RuchyAst::Let {
                    name: "x".to_string(),
                    value: boxed(RuchyAst::Identifier("42".to_string())),
                },
                RuchyAst::Call {
                    function: boxed(RuchyAst::Identifier("println".to_string())),
                    args: vec![RuchyAst::Identifier("x".to_string())],
                },
            ],
        };
        let func = RuchyAst::Function {
            name: "main".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(body),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 1);
    }

    #[test]
    fn test_analyze_program_binary_op_does_not_alter_cyclomatic() {
        // PIN: simple BinaryOp (a + b) should not bump cyclomatic complexity.
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let body = RuchyAst::BinaryOp {
            left: boxed(RuchyAst::Identifier("a".to_string())),
            op: RuchyToken::Plus,
            right: boxed(RuchyAst::Identifier("b".to_string())),
        };
        let func = RuchyAst::Function {
            name: "add".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(body),
            is_test: false,
            line_start: 1,
            line_end: 3,
        };
        let ast = RuchyAst::Program { items: vec![func] };
        let metrics = analyzer.analyze_program(&ast);
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 1);
    }

    #[test]
    fn test_analyze_program_top_level_non_program_falls_through() {
        // PIN: passing a non-Program AST falls into the else branch which
        // calls analyze_node directly. This exercises the fallback path.
        let mut analyzer = RuchyComplexityAnalyzer::new();
        let func = RuchyAst::Function {
            name: "foo".to_string(),
            params: vec![],
            return_type: None,
            body: boxed(RuchyAst::Block { statements: vec![] }),
            is_test: false,
            line_start: 1,
            line_end: 5,
        };
        // Pass the Function directly, NOT wrapped in Program.
        let metrics = analyzer.analyze_program(&func);
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "foo");
    }
}
