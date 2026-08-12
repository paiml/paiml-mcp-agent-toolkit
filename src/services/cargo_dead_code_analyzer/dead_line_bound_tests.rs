//! A file cannot contain more dead lines than lines.
//!
//! `estimated_dead_lines` charges 5 lines per dead function from an ITEM COUNT,
//! not a measured span. Four dead one-line functions in a five-line file
//! estimated 20 dead lines, which the project ratio then reported as 400% —
//! and `--fail-on-violation` compared that against its threshold and printed it
//! as a percentage, while the report beside it printed 100.0%.

// `parsing.rs` is `include!`d into `cargo_dead_code_analyzer`, so its items live
// directly on that module rather than a submodule of it.
use crate::services::cargo_dead_code_analyzer::{
    estimated_dead_lines, estimated_dead_lines_bounded, DeadCodeKind, DeadItem,
};

fn functions(n: usize) -> Vec<DeadItem> {
    (0..n)
        .map(|i| DeadItem {
            name: format!("d{i}"),
            kind: DeadCodeKind::Function,
            line: i,
            column: 1,
            message: "never used".to_string(),
        })
        .collect()
}

#[test]
fn the_estimate_never_exceeds_the_file_it_describes() {
    // The reported case: 4 dead one-line fns in a 5-line file.
    let items = functions(4);
    assert_eq!(
        estimated_dead_lines(&items),
        20,
        "the raw estimator is unchanged"
    );
    assert_eq!(
        estimated_dead_lines_bounded(&items, Some(5)),
        5,
        "a 5-line file cannot hold 20 dead lines"
    );
}

#[test]
fn an_estimate_below_the_file_length_is_left_alone() {
    let items = functions(1);
    assert_eq!(estimated_dead_lines_bounded(&items, Some(100)), 5);
}

/// An unknown length must not silently become zero dead lines — that would read
/// as "measured, and clean" for a file nobody could size.
#[test]
fn an_unknown_file_length_keeps_the_raw_estimate() {
    let items = functions(3);
    assert_eq!(
        estimated_dead_lines_bounded(&items, None),
        estimated_dead_lines(&items)
    );
}

#[test]
fn no_items_is_no_dead_lines_at_any_length() {
    assert_eq!(estimated_dead_lines_bounded(&[], Some(0)), 0);
    assert_eq!(estimated_dead_lines_bounded(&[], None), 0);
}
