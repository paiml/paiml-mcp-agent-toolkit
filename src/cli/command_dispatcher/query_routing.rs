//! `Commands::Query` routing — extracted from `command_routing.rs` for CB-040 file health.
//!
//! Owns the full 50-field `Query` dispatch: contract delegations, gaps, asset
//! contracts, and the mainline `handle_query` call.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::Commands;
use crate::cli::handlers;

impl CommandDispatcher {
    /// Route `Commands::Query` to the appropriate query handler.
    pub(super) async fn route_query_command(command: Commands) -> anyhow::Result<()> {
        let Commands::Query {
            query,
            limit,
            min_grade,
            max_complexity,
            language,
            path,
            project_path,
            format,
            include_source,
            rebuild_index,
            exclude_tests,
            rank_by,
            min_pagerank,
            include_project,
            churn,
            duplicates,
            entropy,
            faults,
            coverage,
            uncovered_only,
            coverage_diff,
            coverage_file,
            coverage_gaps,
            include_excluded,
            definition_type,
            summary,
            git_history,
            regex,
            literal,
            raw,
            case_sensitive,
            ignore_case,
            exclude,
            exclude_file,
            files_with_matches,
            count,
            after_context,
            before_context,
            context_lines,
            ptx_flow,
            ptx_diagnostics,
            suggest_rename,
            apply,
            no_docs,
            docs_only,
            extract_candidates,
            max_module_lines,
            contracts,
            contract_gaps,
            min_level,
            max_level,
            contract_score,
            asset_contracts,
        } = command
        else {
            unreachable!("route_query_command called with non-Query variant");
        };

        // Delegate to pv query for contract searches
        if contracts {
            return crate::cli::command_dispatcher::contract_query_handlers::handle_pv_query_delegation(&query, limit, &format);
        }
        // Contract gaps: show functions without bindings
        if contract_gaps {
            return crate::cli::command_dispatcher::contract_query_handlers::handle_contract_gaps(
                &project_path,
                limit,
                &format,
            );
        }
        // Asset contracts: validate non-code assets
        if asset_contracts {
            return crate::cli::command_dispatcher::contract_query_handlers::handle_asset_contracts(
                &project_path,
                &format,
            );
        }
        // min-level/max-level/contract-score require ContractIndex integration
        // into the query pipeline — not yet implemented. Error explicitly.
        if min_level.is_some() || max_level.is_some() || contract_score {
            eprintln!("error: --min-level, --max-level, --contract-score not yet implemented.");
            eprintln!("  Use --contract-gaps to find functions without bindings.");
            eprintln!("  Use --asset-contracts to check non-code asset compliance.");
            eprintln!("  Track at: docs/specifications/components/commit-level-contract-enforcement.md R-6");
            std::process::exit(2);
        }
        // Default is to show code; --summary disables it
        let show_code = !summary;
        let effective_docs = !no_docs;
        handlers::handle_query(
            query,
            limit,
            min_grade,
            max_complexity,
            language,
            path,
            project_path,
            format,
            include_source,
            rebuild_index,
            exclude_tests,
            rank_by,
            min_pagerank,
            include_project,
            churn,
            duplicates,
            entropy,
            faults,
            coverage,
            uncovered_only,
            coverage_diff,
            coverage_file,
            coverage_gaps,
            include_excluded,
            definition_type,
            show_code,
            git_history,
            regex,
            literal,
            raw,
            case_sensitive,
            ignore_case,
            exclude,
            exclude_file,
            files_with_matches,
            count,
            after_context,
            before_context,
            context_lines,
            ptx_flow,
            ptx_diagnostics,
            suggest_rename,
            apply,
            effective_docs,
            docs_only,
            extract_candidates,
            max_module_lines,
        )
        .await
    }
}
