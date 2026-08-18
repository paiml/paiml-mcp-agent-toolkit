impl CliInput {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(command: Commands, command_name: String, raw_args: Vec<String>) -> Self {
        Self {
            command,
            command_name,
            raw_args,
        }
    }

    /// Category **and** wire name for an `analyze` subcommand.
    ///
    /// The table itself lives in [`crate::cli::command_wire_names`], which
    /// compiles under default features; this is the adapter-facing spelling of
    /// it. Keeping the match here meant no shipped build ever type-checked its
    /// exhaustiveness.
    pub(crate) fn classify_analyze_command(
        analyze_cmd: &AnalyzeCommands,
    ) -> (AnalyzeCommandCategory, &'static str) {
        crate::cli::command_wire_names::classify_analyze_command(analyze_cmd)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// From commands.
    pub fn from_commands(command: Commands) -> Self {
        // Toyota Way Extract Method: Get command name using categorized dispatch
        let command_name = Self::get_command_name_by_category(&command);

        Self {
            command,
            command_name,
            raw_args: std::env::args().collect(),
        }
    }

    /// Toyota Way Extract Method: Get command name using categorized dispatch
    fn get_command_name_by_category(command: &Commands) -> String {
        Self::classify_command(command).1.to_string()
    }

    /// Category **and** wire name for a top-level command.
    ///
    /// Delegates to [`crate::cli::command_wire_names::classify_command`]; see
    /// that module for why the table does not live here any more.
    pub(crate) fn classify_command(command: &Commands) -> (CommandCategory, &'static str) {
        crate::cli::command_wire_names::classify_command(command)
    }
}

impl CliAdapter {
    /// Toyota Way Extract Method: Categorize analyze command by type
    /// Single responsibility: classification logic only
    fn get_analyze_command_category(analyze_cmd: &AnalyzeCommands) -> AnalyzeCommandCategory {
        CliInput::classify_analyze_command(analyze_cmd).0
    }
}
