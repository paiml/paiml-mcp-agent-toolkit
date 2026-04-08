impl UnifiedNode {
    /// Create a new basic unified node with minimal information
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(kind: NodeKind, name: &str, language: Language) -> Self {
        let id = format!("{}:{}:{}", language.name(), kind.as_str(), name);
        Self {
            id,
            kind,
            name: name.to_string(),
            fqn: name.to_string(),
            language,
            file_path: std::path::PathBuf::new(),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references: Vec::new(),
            type_info: None,
            signature: None,
            documentation: None,
            original_item: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new unified node from an AST item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_ast_item(
        item: &AstItem,
        language: Language,
        file_path: &Path,
        id_prefix: Option<&str>,
    ) -> Self {
        // Generate a unique ID
        let prefix = id_prefix.unwrap_or(language.name());

        // Extract name based on AstItem type
        let (name, line, visibility, namespace) = match item {
            AstItem::Function {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Struct {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Enum {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Trait {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Module {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Use { path, line } => {
                (path.clone(), *line, "public".to_string(), None::<String>)
            }
            AstItem::Impl {
                type_name, line, ..
            } => (
                type_name.clone(),
                *line,
                "public".to_string(),
                None::<String>,
            ),
            AstItem::Import { module, line, .. } => {
                (module.clone(), *line, "public".to_string(), None::<String>)
            }
        };

        let kind = NodeKind::from_ast_item(item);
        let id = format!("{}:{}:{}", prefix, kind.as_str(), name);

        // Create position info
        let position = SourcePosition {
            start_line: line,
            start_col: 0,
            end_line: line + 1, // Approximate
            end_col: 0,
        };

        // Extract attributes
        let mut attributes = HashMap::new();
        attributes.insert("access".to_string(), visibility);

        // Get special attributes from item type
        match item {
            AstItem::Function { is_async, .. } => {
                if *is_async {
                    attributes.insert("modifier:async".to_string(), "true".to_string());
                }
            }
            AstItem::Struct { derives, .. } => {
                for derive in derives {
                    attributes.insert(format!("derive:{}", derive), "true".to_string());
                }
            }
            _ => {}
        }

        // Create the FQN with namespace if available
        let fqn = if let Some(ns) = namespace {
            if !ns.is_empty() {
                format!("{}.{}", ns, name)
            } else {
                name.clone()
            }
        } else {
            match item {
                AstItem::Struct { name, .. }
                | AstItem::Enum { name, .. }
                | AstItem::Trait { name, .. } => {
                    // For top-level types, use file path + name
                    let file_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if file_name != name {
                        format!("{}.{}", file_name, name)
                    } else {
                        name.clone()
                    }
                }
                _ => name.clone(),
            }
        };

        UnifiedNode {
            id,
            kind,
            name,
            fqn,
            language,
            file_path: file_path.to_path_buf(),
            position,
            attributes,
            children: Vec::new(),   // To be populated later
            parent: None,           // To be populated later
            references: Vec::new(), // To be populated later
            type_info: None,        // To be populated later
            signature: None,        // We'll need to extract this separately
            documentation: None,    // We'll need to extract this separately
            original_item: Some(item.clone()),
            metadata: HashMap::new(),
        }
    }

    // Helper to extract name from any AstItem
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Extract name from item.
    pub fn extract_name_from_item(item: &AstItem) -> String {
        match item {
            AstItem::Function { name, .. } => name.clone(),
            AstItem::Struct { name, .. } => name.clone(),
            AstItem::Enum { name, .. } => name.clone(),
            AstItem::Trait { name, .. } => name.clone(),
            AstItem::Impl { type_name, .. } => type_name.clone(),
            AstItem::Use { path, .. } => path.clone(),
            AstItem::Module { name, .. } => name.clone(),
            AstItem::Import { module, .. } => module.clone(),
        }
    }

    /// Get the access/visibility modifier of this node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn access(&self) -> Option<&str> {
        self.attributes.get("access").map(AsRef::as_ref)
    }

    /// Check if this node has a specific modifier
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_modifier(&self, modifier: &str) -> bool {
        self.attributes
            .contains_key(&format!("modifier:{}", modifier))
    }

    /// Check if this node is abstract
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_abstract(&self) -> bool {
        self.has_modifier("abstract")
    }

    /// Check if this node is static
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_static(&self) -> bool {
        self.has_modifier("static")
    }

    /// Check if this node is final
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_final(&self) -> bool {
        self.has_modifier("final")
    }

    /// Add a child node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_child(&mut self, child_id: String) {
        self.children.push(child_id);
    }

    /// Set the parent node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_parent(&mut self, parent_id: String) {
        self.parent = Some(parent_id);
    }

    /// Add a reference to another node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_reference(
        &mut self,
        kind: ReferenceKind,
        target_name: String,
        target_id: Option<String>,
    ) {
        let reference = NodeReference {
            kind,
            target_id: target_id.unwrap_or_default(),
            target_name,
            target_language: None, // To be resolved later
        };
        self.references.push(reference);
    }

    /// Get all references of a specific kind
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_references_by_kind(&self, kind: ReferenceKind) -> Vec<&NodeReference> {
        self.references.iter().filter(|r| r.kind == kind).collect()
    }

    /// Set type information
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_type_info(&mut self, type_info: TypeInfo) {
        self.type_info = Some(type_info);
    }

    /// Add language-specific metadata
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}
