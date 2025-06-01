I'll research the checkmake tool's test suite to integrate its testing strategies into your Makefile linter specification.# Extended Makefile Linter Rust Implementation Specification

## 1. Overview

This specification defines a comprehensive Makefile linter implementation for the PAIML MCP Agent Toolkit, designed to validate both hand-written and generated Makefiles. The linter follows GNU Make 4.4 parsing semantics while providing checkmake-compatible rules with enhanced testing strategies and edge case handling inspired by checkmake's implementation.

## 2. Parser Architecture

### 2.1 Enhanced Lexical Analysis

The lexer must handle GNU Make's context-sensitive tokenization with robust edge case support:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Structural
    Target(String),
    Colon,
    DoubleColon,
    ColonWithSpace,     // Handle ".PHONY : target" syntax
    Semicolon,
    Newline,
    Tab,
    
    // Variables
    VariableName(String),
    ImmediateAssign,    // :=
    DeferredAssign,     // =
    ConditionalAssign,  // ?=
    AppendAssign,       // +=
    ShellAssign,        // !=
    
    // Expansions
    DollarParen(String),     // $(...)
    DollarBrace(String),     // ${...}
    DollarDollar,            // $$
    DollarSingle(char),      // $@, $<, etc.
    
    // Directives
    Include(String),
    Sinclude(String),
    Ifdef(String),
    Ifndef(String),
    Ifeq(String, String),
    Ifneq(String, String),
    Else,
    Endif,
    
    // Special
    Export(Option<String>),
    Override(String),
    Define(String),
    Endef,
    Vpath(String),
    
    // Content
    Text(String),
    RecipeLine(String),
    Comment(String),
    
    // Pattern Rules
    PercentPattern(String),  // %.o: %.c
    StaticPattern(String),   // objects: %.o: %.c
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    in_recipe: bool,
    continuation: bool,
    
    // Enhanced error recovery
    recovery_mode: bool,
    skipped_chars: Vec<(usize, char)>,
}

impl<'a> Lexer<'a> {
    pub fn tokenize_with_recovery(&mut self) -> (Vec<Token>, Vec<LexError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        
        while !self.is_eof() {
            match self.next_token() {
                Ok(token) => tokens.push(token),
                Err(e) => {
                    errors.push(e);
                    self.recovery_mode = true;
                    self.skip_to_recovery_point();
                }
            }
        }
        
        (tokens, errors)
    }
    
    fn skip_to_recovery_point(&mut self) {
        // Skip to next line or known delimiter
        while !self.is_eof() {
            let ch = self.peek_char();
            if ch == '\n' || ch == ':' || ch == '=' {
                break;
            }
            self.skipped_chars.push((self.position, ch));
            self.advance();
        }
        self.recovery_mode = false;
    }
}
```

### 2.2 Parsing State Machine with Autotools Support

```rust
#[derive(Debug, Clone, Copy)]
enum ParseContext {
    TopLevel,
    TargetPrerequisites,
    RecipeBody,
    VariableDefinition,
    ConditionalBlock,
    DefineBlock,
    AutomakeFragment,    // Support for automake-generated content
    PatternRule,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    context_stack: Vec<ParseContext>,
    deferred_expansions: HashMap<String, Vec<ExpansionContext>>,
    
    // Checkmake compatibility
    phony_targets: HashSet<String>,
    rule_targets: HashSet<String>,
    
    // Parser state for complex makefiles
    automake_mode: bool,
    line_map: HashMap<usize, LineInfo>,
}

#[derive(Debug)]
pub struct LineInfo {
    pub original_line: String,
    pub line_number: usize,
    pub file_name: Option<String>,  // For included files
}
```

### 2.3 Extended AST Definition

```rust
#[derive(Debug)]
pub struct Makefile {
    pub statements: Vec<Statement>,
    pub source_map: SourceMap,
    pub metadata: MakefileMetadata,
}

#[derive(Debug)]
pub struct MakefileMetadata {
    pub is_automake_generated: bool,
    pub has_gnu_extensions: bool,
    pub make_version_required: Option<Version>,
    pub detected_style: MakefileStyle,
}

#[derive(Debug)]
pub enum MakefileStyle {
    GNU,
    BSD,
    Automake,
    CMakeGenerated,
    Custom,
}

#[derive(Debug)]
pub enum Statement {
    Rule(Rule),
    Variable(Variable),
    Include(Include),
    Conditional(Conditional),
    Export(Export),
    Define(Define),
    Comment(String, Location),
    Empty(Location),
    Directive(Directive),
}

#[derive(Debug)]
pub struct Rule {
    pub targets: Vec<String>,
    pub prerequisites: Prerequisites,
    pub recipe: Option<Recipe>,
    pub location: Location,
    pub is_pattern: bool,          // %.o: %.c
    pub is_double_colon: bool,     // ::
    pub is_static_pattern: bool,   // targets: target-pattern: prereq-patterns
    pub is_suffix_rule: bool,      // .c.o:
}

#[derive(Debug)]
pub struct Recipe {
    pub lines: Vec<RecipeLine>,
    pub silent: bool,               // @command
    pub ignore_errors: bool,        // -command
    pub recursive_make: bool,       // Contains $(MAKE)
    pub line_count: usize,         // For maxbodylength rule
}

#[derive(Debug)]
pub struct RecipeLine {
    pub raw: String,
    pub prefix_modifiers: Vec<RecipeModifier>,
    pub location: Location,
    pub expansions: Vec<ExpansionRef>,
}

#[derive(Debug)]
pub enum RecipeModifier {
    Silent,         // @
    IgnoreError,    // -
    AlwaysExecute,  // +
}
```

## 3. Enhanced Lint Rules

### 3.1 Checkmake-Compatible Rules

#### 3.1.1 MinPhony Rule (Enhanced)

```rust
pub struct MinPhonyRule {
    required_targets: Vec<String>,
    check_implementation: bool,  // Only require .PHONY if target exists
}

impl Default for MinPhonyRule {
    fn default() -> Self {
        Self {
            required_targets: vec!["all".into(), "clean".into(), "test".into()],
            check_implementation: true,
        }
    }
}

impl LintRule for MinPhonyRule {
    fn id(&self) -> &'static str { "minphony" }
    
    fn check(&self, ast: &Makefile, ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut phony_declarations = HashSet::new();
        let mut implemented_targets = HashSet::new();
        
        // Collect .PHONY declarations (handle both ".PHONY:" and ".PHONY :")
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                if rule.targets.iter().any(|t| t == ".PHONY") {
                    phony_declarations.extend(rule.prerequisites.normal.iter().cloned());
                }
                
                // Track all implemented targets
                for target in &rule.targets {
                    if !target.starts_with('.') {
                        implemented_targets.insert(target.clone());
                    }
                }
            }
        }
        
        // Check required targets
        for required in &self.required_targets {
            let is_implemented = implemented_targets.contains(required);
            let is_phony = phony_declarations.contains(required);
            
            if self.check_implementation && !is_implemented {
                // Skip if target not implemented and check_implementation is true
                continue;
            }
            
            if !is_phony {
                violations.push(Violation {
                    rule_id: self.id(),
                    severity: Severity::Warning,
                    location: Location::file_level(),
                    message: format!("Missing required phony target \"{}\"", required),
                    suggestion: Some(format!(".PHONY: {}", required)),
                });
            }
        }
        violations
    }
}
```

#### 3.1.2 PhonyDeclared Rule

```rust
pub struct PhonyDeclaredRule {
    ignore_patterns: Vec<Regex>,
    check_automake: bool,
}

impl Default for PhonyDeclaredRule {
    fn default() -> Self {
        Self {
            ignore_patterns: vec![
                Regex::new(r"^\.").unwrap(),  // Hidden targets
                Regex::new(r"\.(o|a|so)$").unwrap(),  // Object files
            ],
            check_automake: false,  // Disable for automake-generated files
        }
    }
}

impl LintRule for PhonyDeclaredRule {
    fn id(&self) -> &'static str { "phonydeclared" }
    
    fn check(&self, ast: &Makefile, ctx: &LintContext) -> Vec<Violation> {
        if ast.metadata.is_automake_generated && !self.check_automake {
            return Vec::new();
        }
        
        let mut violations = Vec::new();
        let mut phony_targets = HashSet::new();
        
        // Collect .PHONY declarations
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                if rule.targets.iter().any(|t| t == ".PHONY") {
                    phony_targets.extend(rule.prerequisites.normal.iter().cloned());
                }
            }
        }
        
        // Check all non-file-producing targets
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                for target in &rule.targets {
                    if self.should_be_phony(target, &phony_targets, ctx) {
                        violations.push(Violation {
                            rule_id: self.id(),
                            severity: Severity::Warning,
                            location: rule.location.clone(),
                            message: format!("Target \"{}\" should be declared PHONY.", target),
                            suggestion: Some(format!(".PHONY: {}", target)),
                        });
                    }
                }
            }
        }
        violations
    }
}

impl PhonyDeclaredRule {
    fn should_be_phony(&self, target: &str, phony_set: &HashSet<String>, ctx: &LintContext) -> bool {
        // Already declared phony
        if phony_set.contains(target) {
            return false;
        }
        
        // Check ignore patterns
        for pattern in &self.ignore_patterns {
            if pattern.is_match(target) {
                return false;
            }
        }
        
        // Special targets
        if is_special_target(target) {
            return false;
        }
        
        // Likely produces a file
        if target.contains('/') || target.contains('.') {
            return false;
        }
        
        // Pattern rules
        if target.contains('%') {
            return false;
        }
        
        true
    }
}
```

#### 3.1.3 MaxBodyLength Rule

```rust
pub struct MaxBodyLengthRule {
    max_length: usize,
    count_continuations: bool,
}

impl Default for MaxBodyLengthRule {
    fn default() -> Self {
        Self {
            max_length: 5,
            count_continuations: false,  // Count logical lines, not physical
        }
    }
}

impl LintRule for MaxBodyLengthRule {
    fn id(&self) -> &'static str { "maxbodylength" }
    
    fn check(&self, ast: &Makefile, _ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                if let Some(recipe) = &rule.recipe {
                    let line_count = if self.count_continuations {
                        recipe.lines.len()
                    } else {
                        // Count logical lines (commands)
                        self.count_logical_lines(&recipe.lines)
                    };
                    
                    if line_count > self.max_length {
                        violations.push(Violation {
                            rule_id: self.id(),
                            severity: Severity::Info,
                            location: rule.location.clone(),
                            message: format!(
                                "Target body for \"{}\" exceeds allowed length of {} ({}).",
                                rule.targets.join(" "),
                                self.max_length,
                                line_count
                            ),
                            suggestion: Some("Consider breaking complex recipes into functions or separate scripts.".into()),
                        });
                    }
                }
            }
        }
        violations
    }
}

impl MaxBodyLengthRule {
    fn count_logical_lines(&self, lines: &[RecipeLine]) -> usize {
        let mut count = 0;
        let mut in_continuation = false;
        
        for line in lines {
            if !in_continuation {
                count += 1;
            }
            in_continuation = line.raw.trim_end().ends_with('\\');
        }
        count
    }
}
```

#### 3.1.4 TimestampExpanded Rule

```rust
pub struct TimestampExpandedRule {
    timestamp_vars: HashSet<&'static str>,
}

impl Default for TimestampExpandedRule {
    fn default() -> Self {
        Self {
            timestamp_vars: ["TIMESTAMP", "BUILD_TIME", "DATE", "VERSION_DATE"]
                .iter().copied().collect(),
        }
    }
}

impl LintRule for TimestampExpandedRule {
    fn id(&self) -> &'static str { "timestampexpanded" }
    
    fn check(&self, ast: &Makefile, _ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        for stmt in &ast.statements {
            if let Statement::Variable(var) = stmt {
                if self.timestamp_vars.contains(var.name.as_str()) {
                    if let VariableValue::Deferred(_) = &var.value {
                        violations.push(Violation {
                            rule_id: self.id(),
                            severity: Severity::Warning,
                            location: var.location.clone(),
                            message: format!(
                                "Timestamp variable \"{}\" should use := instead of = for consistent builds",
                                var.name
                            ),
                            suggestion: Some(format!(
                                "{} := $(shell date)",
                                var.name
                            )),
                        });
                    }
                }
            }
        }
        violations
    }
}
```

### 3.2 Advanced Rules

#### 3.2.1 Recursive Variable in Recipe Rule

```rust
pub struct RecursiveVariableInRecipeRule {
    expensive_functions: HashSet<&'static str>,
}

impl Default for RecursiveVariableInRecipeRule {
    fn default() -> Self {
        Self {
            expensive_functions: ["shell", "wildcard", "realpath", "abspath"]
                .iter().copied().collect(),
        }
    }
}

impl LintRule for RecursiveVariableInRecipeRule {
    fn check(&self, ast: &Makefile, ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut deferred_vars = HashMap::new();
        
        // Collect deferred variable definitions
        for stmt in &ast.statements {
            if let Statement::Variable(var) = stmt {
                if let VariableValue::Deferred(value) = &var.value {
                    deferred_vars.insert(var.name.clone(), (value.clone(), var.location.clone()));
                }
            }
        }
        
        // Check recipe lines
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                if let Some(recipe) = &rule.recipe {
                    for line in &recipe.lines {
                        self.check_recipe_line(line, &deferred_vars, &mut violations);
                    }
                }
            }
        }
        violations
    }
}
```

#### 3.2.2 Portable Makefile Rule

```rust
pub struct PortableMakefileRule {
    gnu_extensions: HashSet<&'static str>,
    posix_compatible: bool,
}

impl Default for PortableMakefileRule {
    fn default() -> Self {
        Self {
            gnu_extensions: [
                "addprefix", "addsuffix", "basename", "dir", "notdir",
                "wildcard", "realpath", "abspath", "if", "or", "and",
                "foreach", "call", "value", "origin", "flavor", "error",
                "warning", "info", "shell", "guile"
            ].iter().copied().collect(),
            posix_compatible: false,
        }
    }
}

impl LintRule for PortableMakefileRule {
    fn id(&self) -> &'static str { "portability" }
    
    fn check(&self, ast: &Makefile, ctx: &LintContext) -> Vec<Violation> {
        if !self.posix_compatible {
            return Vec::new();
        }
        
        let mut violations = Vec::new();
        
        // Check for GNU-specific functions
        for expansion in &ctx.expansions {
            if let Some(func) = self.extract_function_name(&expansion.text) {
                if self.gnu_extensions.contains(func) {
                    violations.push(Violation {
                        rule_id: self.id(),
                        severity: Severity::Info,
                        location: expansion.location.clone(),
                        message: format!("GNU Make extension '{}' reduces portability", func),
                        suggestion: Some("Consider POSIX-compatible alternatives".into()),
                    });
                }
            }
        }
        violations
    }
}
```

## 4. Comprehensive Test Strategy

### 4.1 Test Fixtures Organization

```rust
pub mod test_fixtures {
    use std::collections::HashMap;
    
    pub struct TestFixture {
        pub name: &'static str,
        pub content: &'static str,
        pub expected_violations: Vec<ExpectedViolation>,
        pub tags: Vec<&'static str>,
    }
    
    pub struct ExpectedViolation {
        pub rule_id: &'static str,
        pub line: Option<usize>,
        pub message_contains: &'static str,
    }
    
    lazy_static! {
        pub static ref FIXTURES: HashMap<&'static str, TestFixture> = {
            let mut m = HashMap::new();
            
            // Checkmake compatibility fixtures
            m.insert("missing_phony", TestFixture {
                name: "missing_phony",
                content: include_str!("fixtures/missing_phony.make"),
                expected_violations: vec![
                    ExpectedViolation {
                        rule_id: "minphony",
                        line: None,
                        message_contains: "Missing required phony target \"all\"",
                    },
                    ExpectedViolation {
                        rule_id: "minphony",
                        line: None,
                        message_contains: "Missing required phony target \"test\"",
                    },
                    ExpectedViolation {
                        rule_id: "phonydeclared",
                        line: Some(18),
                        message_contains: "Target \"all\" should be declared PHONY",
                    },
                ],
                tags: vec!["checkmake", "phony"],
            });
            
            // Edge cases
            m.insert("phony_with_space", TestFixture {
                name: "phony_with_space",
                content: r#"
.PHONY : all clean test

all:
	@echo "Building..."

clean:
	rm -f *.o

test:
	cargo test
"#,
                expected_violations: vec![],
                tags: vec!["edge-case", "phony", "whitespace"],
            });
            
            // Automake-generated
            m.insert("automake_generated", TestFixture {
                name: "automake_generated",
                content: include_str!("fixtures/automake_generated.make"),
                expected_violations: vec![
                    // Automake files have different expectations
                ],
                tags: vec!["automake", "generated"],
            });
            
            m
        };
    }
}
```

### 4.2 Parser Test Suite

```rust
#[cfg(test)]
mod parser_tests {
    use super::*;
    
    #[test]
    fn test_phony_declaration_variants() {
        let cases = vec![
            (".PHONY: all", true),
            (".PHONY : all", true),
            (".PHONY: all clean test", true),
            (".PHONY:all", true),  // No space
            (".PHONY\t:\tall", true),  // Tab
        ];
        
        for (input, should_parse) in cases {
            let mut parser = Parser::new(input);
            let result = parser.parse();
            
            if should_parse {
                assert!(result.is_ok(), "Failed to parse: {}", input);
                let ast = result.unwrap();
                assert!(has_phony_declaration(&ast), "No .PHONY found in: {}", input);
            }
        }
    }
    
    #[test]
    fn test_complex_recipe_line_counting() {
        let makefile = r#"
complex-target:
	@echo "Line 1"
	@for i in 1 2 3; do \
		echo "Still line 2"; \
		echo "Still line 2"; \
	done
	@echo "Line 3"
	@if [ -f foo ]; then \
		echo "Line 4"; \
	fi
"#;
        
        let ast = Parser::new(makefile).parse().unwrap();
        let rule = find_rule(&ast, "complex-target").unwrap();
        let recipe = rule.recipe.as_ref().unwrap();
        
        assert_eq!(count_logical_lines(&recipe.lines), 4);
        assert_eq!(recipe.lines.len(), 8);  // Physical lines
    }
    
    #[test]
    fn test_gnu_extension_detection() {
        let makefile = r#"
SOURCES := $(wildcard *.c)
OBJECTS := $(patsubst %.c,%.o,$(SOURCES))
PREFIX := $(addprefix build/,$(OBJECTS))

portable:
	echo $(SOURCES)
"#;
        
        let ast = Parser::new(makefile).parse().unwrap();
        assert!(ast.metadata.has_gnu_extensions);
    }
}
```

### 4.3 Rule Test Suite

```rust
#[cfg(test)]
mod rule_tests {
    use super::*;
    
    #[test]
    fn test_minphony_with_implementation_check() {
        let makefile = r#"
# No 'all' target implemented
.PHONY: clean

clean:
	rm -f *.o
"#;
        
        let ast = Parser::new(makefile).parse().unwrap();
        let rule = MinPhonyRule {
            required_targets: vec!["all".into(), "clean".into()],
            check_implementation: true,
        };
        
        let violations = rule.check(&ast, &LintContext::default());
        
        // Should not complain about missing 'all' since it's not implemented
        assert_eq!(violations.len(), 0);
    }
    
    #[test]
    fn test_maxbodylength_with_continuations() {
        let makefile = r#"
target:
	@echo "Line 1"
	@echo "Line 2" && \
		echo "Still line 2" && \
		echo "Still line 2"
	@echo "Line 3"
	@echo "Line 4"
	@echo "Line 5"
	@echo "Line 6"
"#;
        
        let ast = Parser::new(makefile).parse().unwrap();
        let rule = MaxBodyLengthRule {
            max_length: 5,
            count_continuations: false,
        };
        
        let violations = rule.check(&ast, &LintContext::default());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("exceeds allowed length of 5"));
    }
}
```

### 4.4 Fuzzing Infrastructure

```rust
#[cfg(fuzzing)]
mod fuzz_tests {
    use arbitrary::{Arbitrary, Unstructured};
    use std::fmt;
    
    #[derive(Debug, Clone)]
    struct FuzzMakefile {
        statements: Vec<FuzzStatement>,
    }
    
    #[derive(Debug, Clone)]
    enum FuzzStatement {
        Target {
            name: FuzzString,
            deps: Vec<FuzzString>,
            recipe_lines: Vec<FuzzString>,
            double_colon: bool,
        },
        Variable {
            name: FuzzString,
            value: FuzzString,
            assignment_type: AssignmentType,
        },
        PhonyDecl {
            targets: Vec<FuzzString>,
        },
        Include {
            paths: Vec<FuzzString>,
        },
    }
    
    #[derive(Debug, Clone)]
    struct FuzzString(String);
    
    impl Arbitrary for FuzzString {
        fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
            let chars = vec![
                'a', 'b', 'c', '_', '-', '.', '/', '$', '(', ')', 
                ' ', '\t', '\n', '\\', ':', '=', '%', '@', '#'
            ];
            
            let len: usize = u.int_in_range(0..=100)?;
            let mut s = String::new();
            
            for _ in 0..len {
                let idx = u.int_in_range(0..=chars.len()-1)?;
                s.push(chars[idx]);
            }
            
            Ok(FuzzString(s))
        }
    }
    
    impl fmt::Display for FuzzMakefile {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for stmt in &self.statements {
                match stmt {
                    FuzzStatement::Target { name, deps, recipe_lines, double_colon } => {
                        write!(f, "{}", name.0)?;
                        write!(f, "{} ", if *double_colon { "::" } else { ":" })?;
                        for (i, dep) in deps.iter().enumerate() {
                            if i > 0 { write!(f, " ")?; }
                            write!(f, "{}", dep.0)?;
                        }
                        writeln!(f)?;
                        for line in recipe_lines {
                            writeln!(f, "\t{}", line.0)?;
                        }
                    }
                    FuzzStatement::Variable { name, value, assignment_type } => {
                        write!(f, "{} ", name.0)?;
                        write!(f, "{} ", assignment_type)?;
                        writeln!(f, "{}", value.0)?;
                    }
                    FuzzStatement::PhonyDecl { targets } => {
                        write!(f, ".PHONY:")?;
                        for target in targets {
                            write!(f, " {}", target.0)?;
                        }
                        writeln!(f)?;
                    }
                    FuzzStatement::Include { paths } => {
                        write!(f, "include")?;
                        for path in paths {
                            write!(f, " {}", path.0)?;
                        }
                        writeln!(f)?;
                    }
                }
            }
            Ok(())
        }
    }
    
    fuzz_target!(|data: &[u8]| {
        if let Ok(makefile) = FuzzMakefile::arbitrary(&mut Unstructured::new(data)) {
            let content = makefile.to_string();
            
            // Parser should not panic
            let _ = Parser::new(&content).parse();
            
            // If parsing succeeds, linting should not panic
            if let Ok(ast) = Parser::new(&content).parse() {
                let ctx = LintContext::default();
                for rule in get_all_rules() {
                    let _ = rule.check(&ast, &ctx);
                }
            }
        }
    });
}
```

### 4.5 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_real_world_makefiles() {
        let test_cases = vec![
            ("rust_project", include_str!("fixtures/real/rust_makefile.make")),
            ("autotools", include_str!("fixtures/real/autotools_generated.make")),
            ("kernel_module", include_str!("fixtures/real/kernel_module.make")),
            ("cmake_wrapper", include_str!("fixtures/real/cmake_wrapper.make")),
        ];
        
        for (name, content) in test_cases {
            println!("Testing real-world Makefile: {}", name);
            
            let result = Parser::new(content).parse();
            assert!(result.is_ok(), "Failed to parse {}: {:?}", name, result);
            
            let ast = result.unwrap();
            let ctx = LintContext::default();
            let mut total_violations = 0;
            
            for rule in get_all_rules() {
                let violations = rule.check(&ast, &ctx);
                total_violations += violations.len();
                
                for v in &violations {
                    println!("  {} ({}): {}", v.rule_id, v.severity, v.message);
                }
            }
            
            // Real-world makefiles might have violations, but shouldn't crash
            println!("  Total violations: {}", total_violations);
        }
    }
    
    #[test]
    fn test_incremental_parsing_performance() {
        let large_makefile = generate_large_makefile(1000);  // 1000 targets
        let mut parser = IncrementalParser::new();
        
        // Initial parse
        let start = std::time::Instant::now();
        let ast1 = parser.parse_incremental(&large_makefile).unwrap();
        let initial_time = start.elapsed();
        
        // Make small change
        let modified = large_makefile.replace("target500:", "target500_modified:");
        
        let start = std::time::Instant::now();
        let ast2 = parser.parse_incremental(&modified).unwrap();
        let incremental_time = start.elapsed();
        
        println!("Initial parse: {:?}", initial_time);
        println!("Incremental parse: {:?}", incremental_time);
        
        // Incremental should be significantly faster
        assert!(incremental_time < initial_time / 10);
    }
}
```

## 5. Configuration System

### 5.1 Rule Configuration

```toml
# .makefilelint.toml
[global]
style = "gnu"  # gnu, bsd, posix
strict = false
ignore_patterns = ["vendor/", "third_party/", "*.generated.mk"]

[rules.minphony]
severity = "warning"
required_targets = ["all", "clean", "test", "install"]
check_implementation = true

[rules.phonydeclared]
severity = "warning"
ignore_patterns = ["^\\.", ".*\\.o$", ".*\\.a$"]
check_automake = false

[rules.maxbodylength]
severity = "info"
max_length = 10
count_continuations = false

[rules.timestampexpanded]
severity = "warning"
timestamp_vars = ["TIMESTAMP", "BUILD_TIME", "DATE", "VERSION"]

[rules.undefined-variable]
severity = "error"
allow_undefined = ["DESTDIR", "PREFIX"]  # Common in install targets

[rules.recursive-expansion]
severity = "warning"
expensive_functions = ["shell", "wildcard", "find"]

[rules.portability]
enabled = false
severity = "info"
posix_compatible = true

# Custom rules for generated makefiles
[rules.template-marker]
enabled = true
severity = "error"
markers = ["{{", "}}", "<%", "%>"]

[rules.project-structure]
enabled = true
severity = "info"
expected_targets = ["build", "test", "clean", "install", "fmt", "lint"]
language = "rust"  # rust, c, cpp, go
```

## 6. Performance Optimizations

### 6.1 Parallel Linting

```rust
pub struct ParallelLinter {
    thread_pool: ThreadPool,
    rule_executor: Arc<RuleExecutor>,
}

impl ParallelLinter {
    pub fn lint_files(&self, paths: Vec<PathBuf>) -> LintResults {
        let (tx, rx) = mpsc::channel();
        let paths = Arc::new(paths);
        
        for idx in 0..paths.len() {
            let tx = tx.clone();
            let paths = Arc::clone(&paths);
            let executor = Arc::clone(&self.rule_executor);
            
            self.thread_pool.execute(move || {
                let path = &paths[idx];
                let result = executor.lint_file(path);
                tx.send((idx, result)).unwrap();
            });
        }
        
        drop(tx);
        
        let mut results = LintResults::new();
        for (idx, result) in rx {
            results.add_file_result(paths[idx].clone(), result);
        }
        
        results
    }
}
```

### 6.2 Caching Strategy

```rust
pub struct LintCache {
    cache_dir: PathBuf,
    hash_algo: Blake3,
}

impl LintCache {
    pub fn get_cached_result(&self, path: &Path) -> Option<CachedLintResult> {
        let content = std::fs::read_to_string(path).ok()?;
        let hash = self.hash_algo.hash(content.as_bytes());
        
        let cache_path = self.cache_path_for_hash(&hash);
        if cache_path.exists() {
            let cached: CachedLintResult = bincode::deserialize(
                &std::fs::read(&cache_path).ok()?
            ).ok()?;
            
            // Verify file hasn't changed
            if cached.file_hash == hash && cached.file_mtime == get_mtime(path) {
                return Some(cached);
            }
        }
        None
    }
}
```

## 7. Error Recovery and Diagnostics

### 7.1 Enhanced Error Messages

```rust
pub struct DiagnosticEngine {
    source_map: SourceMap,
    color_output: bool,
}

impl DiagnosticEngine {
    pub fn format_violation(&self, violation: &Violation, source: &str) -> String {
        let mut output = String::new();
        
        // Error location
        writeln!(&mut output, "{}:{}:{}: {}: {}",
            violation.location.file,
            violation.location.line,
            violation.location.column,
            violation.severity,
            violation.message
        ).unwrap();
        
        // Source snippet with context
        if let Some(snippet) = self.get_source_snippet(source, &violation.location) {
            writeln!(&mut output, "{}", snippet).unwrap();
        }
        
        // Suggestion
        if let Some(suggestion) = &violation.suggestion {
            writeln!(&mut output, "hint: {}", suggestion).unwrap();
        }
        
        output
    }
    
    fn get_source_snippet(&self, source: &str, location: &Location) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = location.line.saturating_sub(1);
        
        if line_idx >= lines.len() {
            return None;
        }
        
        let mut snippet = String::new();
        
        // Context lines
        let start = line_idx.saturating_sub(2);
        let end = (line_idx + 3).min(lines.len());
        
        for i in start..end {
            let line_num = i + 1;
            let is_error_line = i == line_idx;
            
            write!(&mut snippet, "{:4} | {}", line_num, lines[i]).unwrap();
            
            if is_error_line {
                writeln!(&mut snippet).unwrap();
                write!(&mut snippet, "     | ").unwrap();
                
                // Error indicator
                for _ in 0..location.column.saturating_sub(1) {
                    write!(&mut snippet, " ").unwrap();
                }
                write!(&mut snippet, "^").unwrap();
                
                if let Some(len) = location.length {
                    for _ in 1..len {
                        write!(&mut snippet, "~").unwrap();
                    }
                }
            }
            writeln!(&mut snippet).unwrap();
        }
        
        Some(snippet)
    }
}
```

## 8. Implementation Timeline

1. **Phase 1** (3 days): Enhanced parser with checkmake compatibility
    - Implement lexer with edge case handling
    - Build robust parser with error recovery
    - Add automake/autotools detection

2. **Phase 2** (2 days): Core checkmake rules implementation
    - MinPhony, PhonyDeclared, MaxBodyLength, TimestampExpanded
    - Rule configuration system
    - Basic test suite

3. **Phase 3** (2 days): Advanced rules and testing
    - Recursive variable detection
    - Portability checking
    - Comprehensive test fixtures
    - Fuzzing infrastructure

4. **Phase 4** (2 days): Performance and integration
    - Incremental parsing
    - Parallel linting
    - Caching system
    - CLI integration

5. **Phase 5** (1 day): Polish and documentation
    - Error diagnostics
    - Configuration examples
    - Performance benchmarks
    - User documentation

Total: 10 days for production-ready implementation with comprehensive testing.