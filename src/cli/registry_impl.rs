impl CommandRegistry {
    /// Create a new empty registry
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            commands: HashMap::new(),
            global_flags: Vec::new(),
            built_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: CommandMetadata) -> &mut Self {
        self.commands.insert(command.name.clone(), command);
        self
    }

    /// Register a global flag
    pub fn register_global_flag(&mut self, flag: FlagMetadata) -> &mut Self {
        self.global_flags.push(flag);
        self
    }

    /// Find a command by path (e.g., "analyze complexity")
    pub fn find_command(&self, path: &str) -> Option<&CommandMetadata> {
        // First try exact match
        if let Some(cmd) = self.commands.get(path) {
            return Some(cmd);
        }

        // Try to find by alias
        for cmd in self.commands.values() {
            if cmd.aliases.iter().any(|a| a == path) {
                return Some(cmd);
            }
        }

        // Try hierarchical lookup (e.g., "analyze complexity" -> analyze -> complexity)
        let parts: Vec<&str> = path.split_whitespace().collect();
        if parts.len() > 1 {
            let parent = parts[0];
            if let Some(parent_cmd) = self.commands.get(parent) {
                let subcommand_path = parts[1..].join(" ");
                return parent_cmd.find_subcommand(&subcommand_path);
            }
        }

        None
    }

    /// Find commands by semantic tags
    pub fn find_by_tag(&self, tag: &str) -> Vec<&CommandMetadata> {
        self.commands
            .values()
            .filter(|cmd| cmd.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Find commands by category
    pub fn find_by_category(&self, category: &str) -> Vec<&CommandMetadata> {
        self.commands
            .values()
            .filter(|cmd| cmd.category == category)
            .collect()
    }

    /// Get all command names (including subcommand paths)
    pub fn all_command_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for (name, cmd) in &self.commands {
            paths.push(name.clone());
            for sub in &cmd.subcommands {
                paths.push(format!("{} {}", name, sub.name));
            }
        }
        paths.sort();
        paths
    }

    /// Validate registry consistency
    pub fn validate(&self) -> Result<(), Vec<RegistryError>> {
        let mut errors = Vec::new();

        // Check for duplicate aliases
        let mut seen_aliases: HashMap<&str, &str> = HashMap::new();
        for (name, cmd) in &self.commands {
            for alias in &cmd.aliases {
                if let Some(existing) = seen_aliases.get(alias.as_str()) {
                    errors.push(RegistryError::DuplicateAlias {
                        alias: alias.clone(),
                        command1: (*existing).to_string(),
                        command2: name.clone(),
                    });
                } else {
                    seen_aliases.insert(alias.as_str(), name.as_str());
                }
            }
        }

        // Check MCP tool name uniqueness
        let mut seen_mcp_tools: HashMap<&str, &str> = HashMap::new();
        for (name, cmd) in &self.commands {
            if let Some(mcp) = &cmd.mcp {
                if let Some(existing) = seen_mcp_tools.get(mcp.tool_name.as_str()) {
                    errors.push(RegistryError::DuplicateMcpTool {
                        tool_name: mcp.tool_name.clone(),
                        command1: (*existing).to_string(),
                        command2: name.clone(),
                    });
                } else {
                    seen_mcp_tools.insert(mcp.tool_name.as_str(), name.as_str());
                }
            }
        }

        // Check related command references
        for (name, cmd) in &self.commands {
            for related in &cmd.related {
                if !self.commands.contains_key(related) {
                    errors.push(RegistryError::InvalidRelatedCommand {
                        command: name.clone(),
                        related: related.clone(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl CommandMetadata {
    /// Create a new command metadata builder
    pub fn builder(name: impl Into<String>) -> CommandMetadataBuilder {
        CommandMetadataBuilder::new(name)
    }

    /// Find a subcommand by name
    pub fn find_subcommand(&self, name: &str) -> Option<&CommandMetadata> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.subcommands
            .iter()
            .find(|sub| sub.name == name || sub.aliases.iter().any(|a| a == name))
    }

    /// Get full command path from root
    pub fn full_path(&self, parent: Option<&str>) -> String {
        match parent {
            Some(p) => format!("{} {}", p, self.name),
            None => self.name.clone(),
        }
    }
}
