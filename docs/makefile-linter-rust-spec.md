# Makefile Linter Rust Implementation Specification

## 1. Overview

This specification defines a Makefile linter implementation for the PAIML MCP Agent Toolkit, designed to validate both hand-written and generated Makefiles. The linter follows GNU Make 4.4 parsing semantics while providing checkmake-compatible rules.

## 2. Parser Architecture

### 2.1 Lexical Analysis

The lexer must handle GNU Make's context-sensitive tokenization:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Structural
    Target(String),
    Colon,
    DoubleColon,
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
    
    // Content
    Text(String),
    RecipeLine(String),
    Comment(String),
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    in_recipe: bool,
    continuation: bool,
}
```

### 2.2 Parsing State Machine

GNU Make parsing requires stateful context switching:

```rust
#[derive(Debug, Clone, Copy)]
enum ParseContext {
    TopLevel,
    TargetPrerequisites,
    RecipeBody,
    VariableDefinition,
    ConditionalBlock,
    DefineBlock,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    context_stack: Vec<ParseContext>,
    deferred_expansions: HashMap<String, Vec<ExpansionContext>>,
}

#[derive(Debug)]
pub struct ExpansionContext {
    variable: String,
    line: usize,
    immediate: bool,
    recursive_depth: usize,
}
```

### 2.3 AST Definition

```rust
#[derive(Debug)]
pub struct Makefile {
    pub statements: Vec<Statement>,
    pub source_map: SourceMap,
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
}

#[derive(Debug)]
pub struct Prerequisites {
    pub normal: Vec<String>,
    pub order_only: Vec<String>,    // After |
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: VariableValue,
    pub assignment_type: AssignmentType,
    pub export: bool,
    pub override_: bool,
    pub location: Location,
}

#[derive(Debug)]
pub enum VariableValue {
    Immediate(String),
    Deferred(String),
    Shell(String),
}

#[derive(Debug)]
pub struct Recipe {
    pub lines: Vec<RecipeLine>,
    pub silent: bool,               // @command
    pub ignore_errors: bool,        // -command
    pub recursive_make: bool,       // Contains $(MAKE)
}
```

## 3. Lint Rules

### 3.1 Rule Categories

Based on checkmake's implementation with additions for generated Makefiles:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Style,
}

#[derive(Debug, Clone, Copy)]
pub enum RuleCategory {
    Correctness,    // Missing .PHONY, undefined variables
    Performance,    // Recursive variable expansion in recipes
    Style,          // Indentation, naming conventions
    Portability,    // GNU-specific features
    Generated,      // Rules specific to tool-generated Makefiles
}

pub trait LintRule: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> RuleCategory;
    fn default_severity(&self) -> Severity;
    fn check(&self, ast: &Makefile, context: &LintContext) -> Vec<Violation>;
}
```

### 3.2 Core Rules (from checkmake)

#### 3.2.1 Missing .PHONY Declaration

```rust
pub struct PhonyRule;

impl LintRule for PhonyRule {
    fn id(&self) -> &'static str { "missing-phony" }
    
    fn check(&self, ast: &Makefile, _ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut phony_targets = HashSet::new();
        
        // Collect declared .PHONY targets
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                if rule.targets.iter().any(|t| t == ".PHONY") {
                    phony_targets.extend(rule.prerequisites.normal.iter().cloned());
                }
            }
        }
        
        // Check non-file-producing targets
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                for target in &rule.targets {
                    if !target.contains('/') && 
                       !target.contains('.') &&
                       !phony_targets.contains(target) &&
                       !is_special_target(target) {
                        violations.push(Violation {
                            rule_id: self.id(),
                            severity: Severity::Warning,
                            location: rule.location.clone(),
                            message: format!("Target '{}' should be declared .PHONY", target),
                            suggestion: Some(format!(".PHONY: {}", target)),
                        });
                    }
                }
            }
        }
        violations
    }
}
```

#### 3.2.2 Undefined Variable Usage

```rust
pub struct UndefinedVariableRule {
    builtin_vars: HashSet<&'static str>,
}

impl UndefinedVariableRule {
    pub fn new() -> Self {
        let builtins = [
            "MAKE", "MAKEFLAGS", "MAKECMDGOALS", "CURDIR", "SHELL",
            "CC", "CXX", "AR", "RM", "CP", "MV", "MKDIR",
            "@", "<", "^", "+", "*", "?", "%", "|",
        ];
        Self { builtin_vars: builtins.into_iter().collect() }
    }
}

impl LintRule for UndefinedVariableRule {
    fn check(&self, ast: &Makefile, ctx: &LintContext) -> Vec<Violation> {
        let mut defined = self.builtin_vars.clone();
        let mut violations = Vec::new();
        
        // Two-pass: collect definitions, then check usage
        for stmt in &ast.statements {
            if let Statement::Variable(var) = stmt {
                defined.insert(&var.name);
            }
        }
        
        // Check expansions
        for expansion in &ctx.expansions {
            if !defined.contains(expansion.name.as_str()) {
                violations.push(Violation {
                    rule_id: self.id(),
                    severity: Severity::Error,
                    location: expansion.location.clone(),
                    message: format!("Undefined variable: {}", expansion.name),
                    suggestion: None,
                });
            }
        }
        violations
    }
}
```

#### 3.2.3 Tab vs Space in Recipes

```rust
pub struct IndentationRule;

impl LintRule for IndentationRule {
    fn check(&self, ast: &Makefile, _ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                if let Some(recipe) = &rule.recipe {
                    for line in &recipe.lines {
                        if line.raw.starts_with(' ') {
                            violations.push(Violation {
                                rule_id: self.id(),
                                severity: Severity::Error,
                                location: line.location.clone(),
                                message: "Recipe lines must start with tab, not spaces".into(),
                                suggestion: Some(line.raw.replacen(' ', "\t", 1)),
                            });
                        }
                    }
                }
            }
        }
        violations
    }
}
```

### 3.3 Performance Rules

#### 3.3.1 Recursive Variable Expansion in Recipes

```rust
pub struct RecursiveExpansionRule;

impl LintRule for RecursiveExpansionRule {
    fn check(&self, ast: &Makefile, ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Find variables using = (deferred)
        let deferred_vars: HashSet<_> = ast.statements.iter()
            .filter_map(|s| match s {
                Statement::Variable(v) if matches!(v.value, VariableValue::Deferred(_)) => {
                    Some(v.name.as_str())
                }
                _ => None,
            })
            .collect();
        
        // Check recipe lines for $(shell ...) with deferred vars
        for expansion in &ctx.recipe_expansions {
            if expansion.is_shell_call && 
               expansion.contains_vars.iter().any(|v| deferred_vars.contains(v.as_str())) {
                violations.push(Violation {
                    rule_id: self.id(),
                    severity: Severity::Warning,
                    location: expansion.location.clone(),
                    message: "Shell call with recursively expanded variable in recipe".into(),
                    suggestion: Some("Use := for immediate expansion".into()),
                });
            }
        }
        violations
    }
}
```

### 3.4 Generated Makefile Rules

Since this tool generates Makefiles, we need special rules:

#### 3.4.1 Template Marker Validation

```rust
pub struct TemplateMarkerRule;

impl LintRule for TemplateMarkerRule {
    fn check(&self, ast: &Makefile, _ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Check for incomplete template expansions
        for stmt in &ast.statements {
            if let Statement::Comment(text, loc) = stmt {
                if text.contains("{{") && text.contains("}}") {
                    violations.push(Violation {
                        rule_id: self.id(),
                        severity: Severity::Error,
                        location: loc.clone(),
                        message: "Unexpanded template marker found".into(),
                        suggestion: None,
                    });
                }
            }
        }
        violations
    }
}
```

#### 3.4.2 Project Structure Assumptions

```rust
pub struct ProjectStructureRule {
    expected_targets: Vec<&'static str>,
}

impl LintRule for ProjectStructureRule {
    fn check(&self, ast: &Makefile, _ctx: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut found_targets = HashSet::new();
        
        for stmt in &ast.statements {
            if let Statement::Rule(rule) = stmt {
                for target in &rule.targets {
                    found_targets.insert(target.as_str());
                }
            }
        }
        
        // For Rust projects, expect certain targets
        if !found_targets.contains("test") {
            violations.push(Violation {
                rule_id: self.id(),
                severity: Severity::Info,
                location: Location::file_level(),
                message: "Missing 'test' target".into(),
                suggestion: Some("test:\n\tcargo test".into()),
            });
        }
        violations
    }
}
```

## 4. Integration Architecture

### 4.1 CLI Integration

```rust
#[derive(Debug, Parser)]
pub struct MakefileLintArgs {
    /// Makefile path (defaults to ./Makefile)
    #[arg(default_value = "Makefile")]
    pub path: PathBuf,
    
    /// Output format
    #[arg(long, value_enum, default_value = "summary")]
    pub format: LintOutputFormat,
    
    /// Fail on warnings
    #[arg(long)]
    pub strict: bool,
    
    /// Rule configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,
    
    /// Disable specific rules
    #[arg(long)]
    pub disable: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LintOutputFormat {
    Summary,
    Json,
    Sarif,
    Gcc,  // file:line:column: severity: message
}
```

### 4.2 Configuration

```toml
# .makefilelint.toml
[rules]
missing-phony = "warning"
undefined-variable = "error"
recursive-expansion = "warning"
tab-indentation = "error"

[rules.naming-convention]
enabled = true
target-pattern = "^[a-z][a-z0-9-]*$"
variable-pattern = "^[A-Z][A-Z0-9_]*$"

[ignore]
patterns = ["vendor/", "third_party/"]
```

## 5. Performance Considerations

### 5.1 Parser Optimization

```rust
pub struct MakefileParser {
    // Pre-allocated buffers
    token_buffer: Vec<Token>,
    string_pool: StringPool,
    
    // Caching for variable expansion
    expansion_cache: HashMap<String, String>,
    
    // Statistics
    stats: ParserStats,
}

#[derive(Debug, Default)]
pub struct ParserStats {
    pub lines_parsed: usize,
    pub tokens_created: usize,
    pub expansions_cached: usize,
    pub parse_time_ns: u64,
}
```

### 5.2 Incremental Parsing

```rust
pub struct IncrementalParser {
    last_ast: Option<Makefile>,
    last_checksum: [u8; 32],
    change_regions: Vec<ChangeRegion>,
}

impl IncrementalParser {
    pub fn parse_incremental(&mut self, input: &str) -> Result<Makefile, ParseError> {
        let checksum = blake3::hash(input.as_bytes());
        
        if let Some(last) = &self.last_ast {
            if checksum == self.last_checksum {
                return Ok(last.clone());
            }
            
            // Diff and parse only changed regions
            let changes = self.compute_changes(input);
            if changes.len() < input.len() / 10 {
                return self.patch_ast(last, &changes);
            }
        }
        
        // Full parse
        let ast = Parser::new(input).parse()?;
        self.last_ast = Some(ast.clone());
        self.last_checksum = checksum.into();
        Ok(ast)
    }
}
```

## 6. Error Recovery

GNU Make continues parsing after errors, so must we:

```rust
pub struct ErrorRecoveryParser {
    errors: Vec<ParseError>,
    recovery_points: Vec<RecoveryPoint>,
}

#[derive(Debug)]
enum RecoveryPoint {
    NextStatement,      // Skip to next line starting at column 0
    NextTarget,         // Skip to next ':'
    EndOfBlock,         // Skip to next endif/endef
}

impl ErrorRecoveryParser {
    fn parse_with_recovery(&mut self, input: &str) -> (Option<Makefile>, Vec<ParseError>) {
        let mut partial_ast = Makefile::default();
        
        loop {
            match self.parse_statement() {
                Ok(stmt) => partial_ast.statements.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    if !self.recover() {
                        break;
                    }
                }
            }
        }
        
        (Some(partial_ast), self.errors)
    }
}
```

## 7. Testing Strategy

### 7.1 Parser Test Cases

```rust
#[cfg(test)]
mod parser_tests {
    use super::*;
    
    #[test]
    fn test_gnu_make_compatibility() {
        // Test cases from GNU Make test suite
        let cases = include_str!("../test-data/gnu-make-tests.mk");
        // ...
    }
    
    #[test]
    fn test_generated_makefile_patterns() {
        // Test our own generated Makefiles
        let rust_makefile = include_str!("../templates/rust-makefile.mk");
        let ast = Parser::new(rust_makefile).parse().unwrap();
        assert!(ast.statements.len() > 10);
    }
}
```

### 7.2 Fuzzing

```rust
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Arbitrary)]
struct FuzzMakefile {
    statements: Vec<FuzzStatement>,
}

#[derive(Debug, Arbitrary)]
enum FuzzStatement {
    Target { 
        name: String, 
        deps: Vec<String>,
        double_colon: bool,
    },
    Variable {
        name: String,
        value: String,
        immediate: bool,
    },
    Include {
        path: String,
    },
}
```

## 8. Implementation Timeline

1. **Phase 1** (2 days): Core parser with error recovery
2. **Phase 2** (1 day): AST and basic lint rules
3. **Phase 3** (1 day): Integration with CLI and caching
4. **Phase 4** (1 day): Generated Makefile-specific rules

Total: 5 days for production-ready implementation.