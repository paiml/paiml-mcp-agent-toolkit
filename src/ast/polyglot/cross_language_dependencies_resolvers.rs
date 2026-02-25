// Language-specific name resolvers
// Included by cross_language_dependencies.rs — no `use` imports allowed

/// Java-to-Kotlin name resolver
pub struct JavaKotlinResolver;

impl NameResolver for JavaKotlinResolver {
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        _source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool {
        // Only handle Java->Kotlin and Kotlin->Java
        if !((source_language == Language::Java && target_language == Language::Kotlin)
            || (source_language == Language::Kotlin && target_language == Language::Java))
        {
            return false;
        }

        // Direct name match
        if reference.target_name == target.name {
            return true;
        }

        // FQN match
        if reference.target_name == target.fqn {
            return true;
        }

        // Java package name conversion (com.example -> com.example)
        let src_parts: Vec<&str> = reference.target_name.split('.').collect();
        let tgt_parts: Vec<&str> = target.fqn.split('.').collect();

        if src_parts.len() == tgt_parts.len() {
            // Check if all parts except the last match exactly
            if src_parts[0..src_parts.len() - 1] == tgt_parts[0..tgt_parts.len() - 1] {
                // Check if the last part (class/method name) matches
                if src_parts.last().expect("internal error")
                    == tgt_parts.last().expect("internal error")
                {
                    return true;
                }
            }
        }

        false
    }
}

/// Java-to-Scala name resolver
pub struct JavaScalaResolver;

impl NameResolver for JavaScalaResolver {
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        _source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool {
        // Only handle Java->Scala and Scala->Java
        if !((source_language == Language::Java && target_language == Language::Scala)
            || (source_language == Language::Scala && target_language == Language::Java))
        {
            return false;
        }

        // Direct name match
        if reference.target_name == target.name {
            return true;
        }

        // FQN match
        if reference.target_name == target.fqn {
            return true;
        }

        // Scala package name conversion (com.example -> com.example)
        let src_parts: Vec<&str> = reference.target_name.split('.').collect();
        let tgt_parts: Vec<&str> = target.fqn.split('.').collect();

        if src_parts.len() == tgt_parts.len() {
            // Check if all parts except the last match exactly
            if src_parts[0..src_parts.len() - 1] == tgt_parts[0..tgt_parts.len() - 1] {
                // Check if the last part (class/method name) matches
                if src_parts.last().expect("internal error")
                    == tgt_parts.last().expect("internal error")
                {
                    return true;
                }
            }
        }

        false
    }
}

/// TypeScript-to-Java name resolver
pub struct TypeScriptJavaResolver;

impl NameResolver for TypeScriptJavaResolver {
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        _source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool {
        // Only handle TypeScript->Java and Java->TypeScript
        if !((source_language == Language::TypeScript && target_language == Language::Java)
            || (source_language == Language::Java && target_language == Language::TypeScript))
        {
            return false;
        }

        // Direct name match
        if reference.target_name == target.name {
            return true;
        }

        // Handle common naming conventions in web applications
        // e.g., UserService.ts might reference com.example.UserService
        if target.fqn.ends_with(&reference.target_name) {
            return true;
        }

        // Handle TypeScript interface to Java class mapping (IUser -> User)
        if source_language == Language::TypeScript
            && reference.target_name.starts_with('I')
            && reference.target_name.len() > 1
        {
            let name_without_i = reference.target_name.get(1..).unwrap_or_default();
            if name_without_i == target.name {
                return true;
            }
            if target.fqn.ends_with(name_without_i) {
                return true;
            }
        }

        false
    }
}
