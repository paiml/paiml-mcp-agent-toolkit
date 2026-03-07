/// Extract identifiers from source for better search
pub(super) fn extract_identifiers(source: &str) -> String {
    let mut identifiers = Vec::new();

    for word in source.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let trimmed = word.trim();
        if trimmed.len() >= 3 && !is_keyword(trimmed) {
            identifiers.push(trimmed.to_lowercase());
        }
    }

    identifiers.join(" ")
}

/// Check if word is a language keyword
pub(super) fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "const"
            | "static"
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "return"
            | "break"
            | "continue"
            | "async"
            | "await"
            | "true"
            | "false"
            | "self"
            | "Self"
            | "super"
            | "crate"
            | "where"
            | "move"
            | "ref"
            | "dyn"
            | "box"
            | "in"
            | "as"
            | "unsafe"
            | "extern"
            | "macro"
            | "function"
            | "class"
            | "def"
            | "import"
            | "from"
            | "try"
            | "catch"
            | "throw"
            | "new"
            | "this"
            | "var"
            | "void"
            | "int"
            | "str"
            | "bool"
            | "None"
            | "null"
            | "undefined"
            // Lua keywords
            | "local"
            | "then"
            | "end"
            | "elseif"
            | "repeat"
            | "until"
            | "and"
            | "or"
            | "not"
            | "nil"
            | "goto"
            // C/C++ keywords
            | "char" | "short" | "long" | "float" | "double" | "signed" | "unsigned"
            | "auto" | "register" | "volatile" | "inline" | "virtual" | "override"
            | "switch" | "case" | "default" | "typedef" | "sizeof" | "alignof"
            | "namespace" | "using" | "template" | "typename" | "constexpr"
            | "noexcept" | "nullptr" | "delete" | "operator" | "friend"
            | "private" | "protected" | "public" | "explicit" | "mutable"
            | "static_cast" | "dynamic_cast" | "const_cast" | "reinterpret_cast"
            // CUDA keywords
            | "__global__" | "__device__" | "__host__" | "__shared__"
            | "threadIdx" | "blockIdx" | "blockDim" | "gridDim"
    )
}
