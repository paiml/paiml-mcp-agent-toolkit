/// Builder for CommandMetadata
#[derive(Debug, Default)]
pub struct CommandMetadataBuilder {
    metadata: CommandMetadata,
}

impl CommandMetadataBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: CommandMetadata {
                name: name.into(),
                ..Default::default()
            },
        }
    }

    pub fn short_description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.short_description = desc.into();
        self
    }

    pub fn long_description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.long_description = desc.into();
        self
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.metadata.aliases.push(alias.into());
        self
    }

    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata
            .aliases
            .extend(aliases.into_iter().map(Into::into));
        self
    }

    pub fn argument(mut self, arg: ArgumentMetadata) -> Self {
        self.metadata.arguments.push(arg);
        self
    }

    pub fn example(mut self, example: ExampleMetadata) -> Self {
        self.metadata.examples.push(example);
        self
    }

    pub fn mcp(mut self, mcp: McpToolMetadata) -> Self {
        self.metadata.mcp = Some(mcp);
        self
    }

    pub fn subcommand(mut self, sub: CommandMetadata) -> Self {
        self.metadata.subcommands.push(sub);
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn related(mut self, related: impl Into<String>) -> Self {
        self.metadata.related.push(related.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.metadata.category = category.into();
        self
    }

    pub fn deprecated(mut self, info: DeprecationInfo) -> Self {
        self.metadata.deprecated = Some(info);
        self
    }

    pub fn is_mutation(mut self, is_mutation: bool) -> Self {
        self.metadata.is_mutation = is_mutation;
        self
    }

    pub fn execution_time(mut self, time: ExecutionTime) -> Self {
        self.metadata.execution_time = time;
        self
    }

    pub fn build(self) -> CommandMetadata {
        self.metadata
    }
}
