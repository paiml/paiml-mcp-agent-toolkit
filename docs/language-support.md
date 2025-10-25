# Language Support

## Supported Languages

PMAT provides comprehensive support for a wide range of programming languages, with varying levels of analysis capabilities based on language-specific parsers and analyzers.

### First-Class Language Support

These languages have dedicated parsers and full analysis capabilities:

#### Rust
- **Parser**: Native syn-based AST analysis
- **Features**: Full cargo integration, macro expansion, trait resolution
- **Metrics**: McCabe cyclomatic complexity, cognitive complexity, maintainability index
- **Dead Code**: Function-level detection with call graph analysis
- **SATD**: Self-admitted technical debt detection with severity classification

#### TypeScript/JavaScript
- **Parser**: SWC-based AST analysis
- **Features**: Module resolution, React component detection, JSX/TSX support
- **Metrics**: Cyclomatic complexity, cognitive complexity, Halstead metrics
- **Dead Code**: Function and class-level detection
- **SATD**: Comment-based technical debt detection

#### Python
- **Parser**: Tree-sitter-based AST analysis
- **Features**: Module imports, class hierarchy, virtual environment detection
- **Metrics**: McCabe cyclomatic complexity, cognitive complexity
- **Dead Code**: Function-level detection with import tracking
- **SATD**: Technical debt comment detection with PEP-compliance validation

#### C/C++
- **Parser**: Tree-sitter with additional goto tracking
- **Features**: Preprocessor directive handling, header resolution
- **Metrics**: Cyclomatic complexity, nesting depth
- **Dead Code**: Function-level detection
- **SATD**: Comment-based technical debt detection

#### Java
- **Parser**: Tree-sitter-based AST analysis
- **Features**: Package structure, inheritance hierarchy
- **Metrics**: Cyclomatic complexity, cognitive complexity
- **Dead Code**: Method-level detection with class hierarchy analysis
- **SATD**: Javadoc and comment-based technical debt detection

### Second-Tier Language Support

These languages have dedicated parsers but fewer advanced analysis features:

- **Kotlin**: Tree-sitter-based analysis
- **Scala**: Tree-sitter-based analysis
- **Go**: Tree-sitter-based analysis
- **Ruby**: Tree-sitter-based analysis
- **C#**: Tree-sitter-based analysis
- **Swift**: Tree-sitter-based analysis
- **PHP**: Tree-sitter-based analysis
- **Ruchy**: Custom parser with advanced analysis

### Basic Support

These languages have basic parsing support:

- Shell scripts (Bash, Zsh, etc.)
- PowerShell
- Perl
- Lua
- YAML/TOML/JSON
- SQL
- And many others...

## Cross-Language Analysis

PMAT supports cross-language analysis for polyglot codebases. See [Cross-Language Analysis](./cross-language-analysis.md) for details.

## Language Feature Configuration

PMAT uses Rust's feature flag system to customize language support. See [Polyglot AST Feature Flags](./polyglot-ast-feature-flags.md) for details on how to enable or disable specific language support.

## Language-Specific Analysis Capabilities

| Language | AST | Complexity | Dead Code | SATD | Documentation | Cross-References |
|----------|-----|------------|-----------|------|---------------|------------------|
| Rust | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TypeScript | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| JavaScript | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Python | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| C/C++ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Java | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Kotlin | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| Scala | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| Go | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Ruby | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| C# | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| Swift | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| PHP | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| Shell | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ❌ |
| WASM | ✅ | ✅ | ❌ | ❌ | ❌ | ⚠️ |

Legend:
- ✅ Full support
- ⚠️ Partial support
- ❌ Not supported

## Adding Language Support

For developers interested in adding support for additional languages, please see the [Language Support Extension Guide](./extending-language-support.md).