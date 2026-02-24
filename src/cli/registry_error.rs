/// Registry validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateAlias {
        alias: String,
        command1: String,
        command2: String,
    },
    DuplicateMcpTool {
        tool_name: String,
        command1: String,
        command2: String,
    },
    InvalidRelatedCommand {
        command: String,
        related: String,
    },
    InvalidExample {
        command: String,
        example: String,
        reason: String,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateAlias {
                alias,
                command1,
                command2,
            } => {
                write!(
                    f,
                    "Duplicate alias '{}' in commands '{}' and '{}'",
                    alias, command1, command2
                )
            }
            Self::DuplicateMcpTool {
                tool_name,
                command1,
                command2,
            } => {
                write!(
                    f,
                    "Duplicate MCP tool '{}' in commands '{}' and '{}'",
                    tool_name, command1, command2
                )
            }
            Self::InvalidRelatedCommand { command, related } => {
                write!(
                    f,
                    "Command '{}' references non-existent related command '{}'",
                    command, related
                )
            }
            Self::InvalidExample {
                command,
                example,
                reason,
            } => {
                write!(
                    f,
                    "Invalid example '{}' in command '{}': {}",
                    example, command, reason
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}
