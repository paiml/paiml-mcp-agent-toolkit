// language_simple_rules.rs — LanguageRules impl methods
// Included from language_simple.rs — NO use imports, NO #! inner attributes

impl LanguageRules {
    #[must_use]
    pub fn for_language(language: Language) -> Self {
        match language {
            Language::Rust => Self::rust_rules(),
            Language::Python => Self::python_rules(),
            Language::JavaScript => Self::javascript_rules(),
            Language::TypeScript => Self::typescript_rules(),
            Language::Go => Self::go_rules(),
            Language::Ruchy => Self::ruchy_rules(),
            Language::Lua => Self::lua_rules(),
            Language::Sql => Self::sql_rules(),
            Language::Scala => Self::scala_rules(),
            Language::Yaml => Self::yaml_rules(),
            Language::Markdown => Self::markdown_rules(),
            _ => Self::rust_rules(), // Default
        }
    }

    #[must_use]
    pub fn rust_rules() -> Self {
        LanguageRules {
            language: Language::Rust,
            function_style: NamingStyle::SnakeCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }

    #[must_use]
    pub fn python_rules() -> Self {
        LanguageRules {
            language: Language::Python,
            function_style: NamingStyle::SnakeCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }

    #[must_use]
    pub fn javascript_rules() -> Self {
        LanguageRules {
            language: Language::JavaScript,
            function_style: NamingStyle::CamelCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::CamelCase,
        }
    }

    #[must_use]
    pub fn typescript_rules() -> Self {
        LanguageRules {
            language: Language::TypeScript,
            function_style: NamingStyle::CamelCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::CamelCase,
        }
    }

    #[must_use]
    pub fn go_rules() -> Self {
        LanguageRules {
            language: Language::Go,
            function_style: NamingStyle::PascalCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::PascalCase,
            variable_style: NamingStyle::CamelCase,
        }
    }

    #[must_use]
    pub fn ruchy_rules() -> Self {
        LanguageRules {
            language: Language::Ruchy,
            function_style: NamingStyle::SnakeCase, // fun hello_world()
            type_style: NamingStyle::PascalCase,    // struct Point, enum Color
            constant_style: NamingStyle::ScreamingSnakeCase, // const MAX_SIZE
            variable_style: NamingStyle::SnakeCase, // let my_variable
        }
    }

    #[must_use]
    pub fn lua_rules() -> Self {
        LanguageRules {
            language: Language::Lua,
            function_style: NamingStyle::SnakeCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }

    #[must_use]
    pub fn sql_rules() -> Self {
        LanguageRules {
            language: Language::Sql,
            function_style: NamingStyle::SnakeCase,
            type_style: NamingStyle::SnakeCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }

    #[must_use]
    pub fn scala_rules() -> Self {
        LanguageRules {
            language: Language::Scala,
            function_style: NamingStyle::CamelCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::PascalCase,
            variable_style: NamingStyle::CamelCase,
        }
    }

    #[must_use]
    pub fn yaml_rules() -> Self {
        LanguageRules {
            language: Language::Yaml,
            function_style: NamingStyle::KebabCase,
            type_style: NamingStyle::KebabCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::KebabCase,
        }
    }

    #[must_use]
    pub fn markdown_rules() -> Self {
        LanguageRules {
            language: Language::Markdown,
            function_style: NamingStyle::KebabCase,
            type_style: NamingStyle::PascalCase,
            constant_style: NamingStyle::ScreamingSnakeCase,
            variable_style: NamingStyle::SnakeCase,
        }
    }
}
