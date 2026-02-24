/// Comprehensive language enumeration supporting 30+ languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    // Systems Programming
    Rust,
    C,
    Cpp,
    Go,
    Zig,

    // JVM Ecosystem
    Java,
    Kotlin,
    Scala,
    Groovy,
    Clojure,

    // .NET Ecosystem
    CSharp,
    FSharp,
    VisualBasic,

    // Dynamic Languages
    Python,
    JavaScript,
    TypeScript,
    Ruby,
    PHP,
    Perl,
    Lua,

    // Functional Languages
    Haskell,
    Elixir,
    Erlang,
    OCaml,
    ReasonML,
    Elm,
    PureScript,

    // Proof Assistants
    Lean,

    // Mobile Development
    Swift,
    ObjectiveC,
    Dart,

    // Shell & Scripting
    Bash,
    Zsh,
    Fish,
    PowerShell,

    // Data & Config
    SQL,
    HCL, // Terraform
    YAML,
    TOML,
    JSON,
    XML,

    // Documentation & Markup
    Markdown,
    LaTeX,
    AsciiDoc,

    // Build Systems
    Makefile,
    CMake,
    Bazel,
    Gradle,
    Maven,

    // Specialized
    Solidity, // Blockchain
    VHDL,     // Hardware
    Verilog,  // Hardware
    R,        // Statistics
    Julia,    // Scientific computing
    Matlab,   // Engineering
    Assembly, // Low-level

    Unknown,
}
