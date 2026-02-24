/// Language metadata for efficient lookup (Toyota Way: Data-Driven Design)
#[derive(Debug)]
struct LanguageInfo {
    name: &'static str,
    extensions: &'static [&'static str],
}

/// Static language metadata table - eliminates giant match statements (Toyota Way: ≤20 complexity)
static LANGUAGE_INFO: &[LanguageInfo] = &[
    // Systems Programming
    LanguageInfo {
        name: "Rust",
        extensions: &["rs"],
    },
    LanguageInfo {
        name: "C",
        extensions: &["c", "h"],
    },
    LanguageInfo {
        name: "C++",
        extensions: &["cpp", "cc", "cxx", "hpp", "hxx", "C", "H"],
    },
    LanguageInfo {
        name: "Go",
        extensions: &["go"],
    },
    LanguageInfo {
        name: "Zig",
        extensions: &["zig"],
    },
    // JVM Ecosystem
    LanguageInfo {
        name: "Java",
        extensions: &["java"],
    },
    LanguageInfo {
        name: "Kotlin",
        extensions: &["kt", "kts"],
    },
    LanguageInfo {
        name: "Scala",
        extensions: &["scala", "sc"],
    },
    LanguageInfo {
        name: "Groovy",
        extensions: &["groovy", "gvy", "gy", "gsh"],
    },
    LanguageInfo {
        name: "Clojure",
        extensions: &["clj", "cljs", "cljc", "edn"],
    },
    // .NET Ecosystem
    LanguageInfo {
        name: "C#",
        extensions: &["cs"],
    },
    LanguageInfo {
        name: "F#",
        extensions: &["fs", "fsi", "fsx"],
    },
    LanguageInfo {
        name: "Visual Basic",
        extensions: &["vb"],
    },
    // Dynamic Languages
    LanguageInfo {
        name: "Python",
        extensions: &["py", "pyw", "pyi", "pyx", "pxd"],
    },
    LanguageInfo {
        name: "JavaScript",
        extensions: &["js", "jsx", "mjs", "cjs"],
    },
    LanguageInfo {
        name: "TypeScript",
        extensions: &["ts", "tsx", "d.ts"],
    },
    LanguageInfo {
        name: "Ruby",
        extensions: &["rb", "rbw", "rake", "gemspec"],
    },
    LanguageInfo {
        name: "PHP",
        extensions: &["php", "phtml", "php3", "php4", "php5", "phps"],
    },
    LanguageInfo {
        name: "Perl",
        extensions: &["pl", "pm", "t", "pod"],
    },
    LanguageInfo {
        name: "Lua",
        extensions: &["lua"],
    },
    // Functional Languages
    LanguageInfo {
        name: "Haskell",
        extensions: &["hs", "lhs"],
    },
    LanguageInfo {
        name: "Elixir",
        extensions: &["ex", "exs"],
    },
    LanguageInfo {
        name: "Erlang",
        extensions: &["erl", "hrl"],
    },
    LanguageInfo {
        name: "OCaml",
        extensions: &["ml", "mli"],
    },
    LanguageInfo {
        name: "ReasonML",
        extensions: &["re", "rei"],
    },
    LanguageInfo {
        name: "Elm",
        extensions: &["elm"],
    },
    LanguageInfo {
        name: "PureScript",
        extensions: &["purs"],
    },
    // Proof Assistants
    LanguageInfo {
        name: "Lean",
        extensions: &["lean"],
    },
    // Mobile Development
    LanguageInfo {
        name: "Swift",
        extensions: &["swift"],
    },
    LanguageInfo {
        name: "Objective-C",
        extensions: &["m", "mm", "M"],
    },
    LanguageInfo {
        name: "Dart",
        extensions: &["dart"],
    },
    // Shell & Scripting
    LanguageInfo {
        name: "Bash",
        extensions: &["sh", "bash", "zsh"],
    },
    LanguageInfo {
        name: "Zsh",
        extensions: &["zsh"],
    },
    LanguageInfo {
        name: "Fish",
        extensions: &["fish"],
    },
    LanguageInfo {
        name: "PowerShell",
        extensions: &["ps1", "psm1", "psd1"],
    },
    // Data & Config
    LanguageInfo {
        name: "SQL",
        extensions: &["sql", "ddl", "dml"],
    },
    LanguageInfo {
        name: "HCL",
        extensions: &["tf", "tfvars", "hcl"],
    },
    LanguageInfo {
        name: "YAML",
        extensions: &["yml", "yaml"],
    },
    LanguageInfo {
        name: "TOML",
        extensions: &["toml"],
    },
    LanguageInfo {
        name: "JSON",
        extensions: &["json", "jsonc"],
    },
    LanguageInfo {
        name: "XML",
        extensions: &["xml", "xsd", "xsl", "xslt"],
    },
    // Documentation & Markup
    LanguageInfo {
        name: "Markdown",
        extensions: &["md", "markdown", "mdown", "mkd"],
    },
    LanguageInfo {
        name: "LaTeX",
        extensions: &["tex", "latex", "sty", "cls"],
    },
    LanguageInfo {
        name: "AsciiDoc",
        extensions: &["adoc", "asciidoc"],
    },
    // Build Systems
    LanguageInfo {
        name: "Makefile",
        extensions: &["mk", "make"],
    },
    LanguageInfo {
        name: "CMake",
        extensions: &["cmake"],
    },
    LanguageInfo {
        name: "Bazel",
        extensions: &["bazel", "bzl"],
    },
    LanguageInfo {
        name: "Gradle",
        extensions: &["gradle"],
    },
    LanguageInfo {
        name: "Maven",
        extensions: &["pom"],
    },
    // Specialized
    LanguageInfo {
        name: "Solidity",
        extensions: &["sol"],
    },
    LanguageInfo {
        name: "VHDL",
        extensions: &["vhd", "vhdl"],
    },
    LanguageInfo {
        name: "Verilog",
        extensions: &["v", "vh"],
    },
    LanguageInfo {
        name: "R",
        extensions: &["r", "R"],
    },
    LanguageInfo {
        name: "Julia",
        extensions: &["jl"],
    },
    LanguageInfo {
        name: "Matlab",
        extensions: &["m"],
    },
    LanguageInfo {
        name: "Assembly",
        extensions: &["s", "S", "asm"],
    },
    LanguageInfo {
        name: "Unknown",
        extensions: &[],
    },
];
