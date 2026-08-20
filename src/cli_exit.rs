//! Process exit codes, declared by the site that raises the error.
//!
//! This module exists because the exit code used to be a property of the
//! WORDING of a user-facing sentence. `src/bin/pmat.rs` lowercased
//! `error.to_string()` and grepped it: "quality gate" or "violation" gave 3,
//! "config" or "parse" gave 4, "analysis" or "complexity" gave 5, "permission"
//! or "access" gave 126. Rewording a message changed the exit code, and CI
//! branches on exit codes.
//!
//! Two measured instances, both live before this module:
//!
//! * `pmat serve --transport http` with no token exits 4, and the help text at
//!   `commands_enum/definition.rs:933` documents that as a contract. The 4 came
//!   from the word "configured" inside a subordinate clause explaining pmcp's
//!   behaviour — "pmcp serves every request when no auth provider is
//!   configured" — in a message whose subject is a MISSING TOKEN. Nothing in
//!   that sentence was written to select an exit code.
//!
//! * `analyze complexity` over a directory of unanalyzable files exits 5,
//!   because its refusal contains "no complexity measurement was taken". Point
//!   it at an absent path and the refusal reads "Path not found", matching no
//!   keyword, so the same class of failure exits 1.
//!
//! A code is now stated where the error is raised, or it is `GeneralError`.

/// Process exit codes.
///
/// `CommandNotFound = 127` and `InvalidExitArg = 128` were removed along with
/// the substring classifier: no error text could reach them, and the
/// `SPECIFICATION.md Section 23` they cited does not exist in this tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success.
    Success = 0,
    /// Any failure that does not declare a more specific code.
    GeneralError = 1,
    /// Command-line misuse. Clap owns this one.
    MisuseError = 2,
    /// A quality gate found blocking violations.
    QualityGateFailure = 3,
    /// Configuration is missing or unusable.
    ConfigurationError = 4,
    /// An analysis could not produce a measurement.
    AnalysisError = 5,
    /// Permission denied.
    PermissionDenied = 126,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

/// An error that DECLARES its process exit code.
///
/// `Display` delegates to the wrapped error, so attaching a code changes not
/// one byte of what the user reads. That separation is the point: the message
/// and the exit code stop being the same channel.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct ExitCoded {
    /// The code this failure exits with.
    pub code: ExitCode,
    /// The underlying error, whose `Display` is preserved verbatim.
    #[source]
    pub source: anyhow::Error,
}

/// Attach an exit code to an error at the site that raises it.
#[must_use]
pub fn with_code(code: ExitCode, source: anyhow::Error) -> anyhow::Error {
    anyhow::Error::new(ExitCoded { code, source })
}

/// Configuration is missing or unusable — exit 4.
#[must_use]
pub fn configuration_error(source: anyhow::Error) -> anyhow::Error {
    with_code(ExitCode::ConfigurationError, source)
}

/// An analysis could not produce a measurement — exit 5.
#[must_use]
pub fn analysis_error(source: anyhow::Error) -> anyhow::Error {
    with_code(ExitCode::AnalysisError, source)
}

/// A quality gate found blocking violations — exit 3.
#[must_use]
pub fn quality_gate_failure(source: anyhow::Error) -> anyhow::Error {
    with_code(ExitCode::QualityGateFailure, source)
}

/// The declared code for an error, or `GeneralError` when none was declared.
///
/// Walks the whole cause chain, so a declaring error keeps its code after
/// `.context(...)` wraps it.
#[must_use]
pub fn code_for(error: &anyhow::Error) -> ExitCode {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<ExitCoded>())
        .map_or(ExitCode::GeneralError, |coded| coded.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point: the code does not move when the words do.
    #[test]
    fn the_exit_code_is_independent_of_the_message() {
        for text in [
            "PMAT_MCP_HTTP_TOKEN is not set, no auth provider is configured",
            "the token is absent",
            "quality gate violation complexity permission access parse",
        ] {
            let e = configuration_error(anyhow::anyhow!("{text}"));
            assert_eq!(
                code_for(&e),
                ExitCode::ConfigurationError,
                "wording must not change the code: {text}"
            );
            assert_eq!(e.to_string(), text, "the user's message must be unchanged");
        }
    }

    /// The counter-test. An undeclared error must NOT pick up a code from words
    /// that used to trigger the classifier — otherwise the substring behaviour
    /// survives under a new name.
    #[test]
    fn undeclared_errors_are_general_whatever_they_say() {
        for text in [
            "quality gate failed with 3 violations",
            "failed to parse config",
            "analysis of complexity aborted",
            "permission denied: access refused",
        ] {
            assert_eq!(
                code_for(&anyhow::anyhow!("{text}")),
                ExitCode::GeneralError,
                "an undeclared error must be GeneralError: {text}"
            );
        }
    }

    /// A declared code survives being wrapped in context.
    #[test]
    fn context_does_not_lose_the_declared_code() {
        let inner = analysis_error(anyhow::anyhow!("no measurement was taken"));
        let wrapped = inner.context("while analysing the project");
        assert_eq!(code_for(&wrapped), ExitCode::AnalysisError);
    }
}
