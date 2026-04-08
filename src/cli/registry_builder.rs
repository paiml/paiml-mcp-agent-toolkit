/// Builder for CommandMetadata
#[derive(Debug, Default)]
pub struct CommandMetadataBuilder {
    metadata: CommandMetadata,
}

impl CommandMetadataBuilder {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: CommandMetadata {
                name: name.into(),
                ..Default::default()
            },
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Short description.
    pub fn short_description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.short_description = desc.into();
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Long description.
    pub fn long_description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.long_description = desc.into();
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Alias.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.metadata.aliases.push(alias.into());
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Aliases.
    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata
            .aliases
            .extend(aliases.into_iter().map(Into::into));
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Argument.
    pub fn argument(mut self, arg: ArgumentMetadata) -> Self {
        self.metadata.arguments.push(arg);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Example.
    pub fn example(mut self, example: ExampleMetadata) -> Self {
        self.metadata.examples.push(example);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Mcp.
    pub fn mcp(mut self, mcp: McpToolMetadata) -> Self {
        self.metadata.mcp = Some(mcp);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Subcommand.
    pub fn subcommand(mut self, sub: CommandMetadata) -> Self {
        self.metadata.subcommands.push(sub);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Tags.
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Related.
    pub fn related(mut self, related: impl Into<String>) -> Self {
        self.metadata.related.push(related.into());
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Category.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.metadata.category = category.into();
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Deprecated.
    pub fn deprecated(mut self, info: DeprecationInfo) -> Self {
        self.metadata.deprecated = Some(info);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is mutation.
    pub fn is_mutation(mut self, is_mutation: bool) -> Self {
        self.metadata.is_mutation = is_mutation;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Execution time.
    pub fn execution_time(mut self, time: ExecutionTime) -> Self {
        self.metadata.execution_time = time;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Build and return the final result.
    pub fn build(self) -> CommandMetadata {
        self.metadata
    }
}
