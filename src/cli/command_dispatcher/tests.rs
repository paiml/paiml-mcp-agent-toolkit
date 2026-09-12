// Was quarantined under a false "File splitting broke syntax" claim (#1321).
// Lifting the hub alone showed what was actually wrong, and it was never syntax:
//
//   98 x `CommandDispatcher`, 19 x `OutputFormat`, 8 x `PathBuf`, 8 x
//        `DemoProtocol` not in scope — the split moved the hub away from the
//        `use super::*` that had reached them
//   14 x `crate::demo` gone — those tests are now behind `#[cfg(feature =
//        "demo")]`, mirroring `demo_commands.rs`'s own gate
//    2 x compiler errors for `WorkCommands` fields added since the split —
//        which the fix traced to THREE initialisers and three fields:
//        `Start` gained `level`, `Validate` gained `check_base`, `Migrate`
//        gained `levels`. (`agent` moved in the same hunk; it was not added.)
//
// Three further changes were needed to make them RUN, and each carries its own
// evidence at the site rather than being summarised here:
//   - `execute_report_command` call sites updated for the signature #918
//     (issue #706) gave it — `Vec<AnalysisType>` in place of `Vec<String>`,
//     and the hand-rolled hyphen matcher deleted
//   - two assertions corrected where the production behaviour changed, each
//     citing the commit that changed it
//   - three tests `#[ignore]`d because they reach a handler's `process::exit`,
//     each naming #1331, which is what must change for them to run
#[cfg(test)]
#[path = "command_dispatcher_tests.rs"]
mod command_dispatcher_tests;
