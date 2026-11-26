//! Shared fixtures: a GitHub that answers from a queue, and tickets built from parts.

use super::findings::{PrState, TicketRec};
use super::rubric::Ticket;
use crate::services::github_comply::source::Graphql;
use crate::services::github_comply::IssueRec;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};
use std::cell::RefCell;

/// A GitHub that pops one canned reply per query and records the queries.
pub(super) struct Fake {
    replies: RefCell<Vec<Result<Value, String>>>,
    pub(super) seen: RefCell<Vec<String>>,
}

impl Fake {
    pub(super) fn new(replies: Vec<Result<Value, String>>) -> Self {
        Fake {
            replies: RefCell::new(replies.into_iter().rev().collect()),
            seen: RefCell::new(vec![]),
        }
    }
}

impl Graphql for Fake {
    fn query(&self, query: &str, _vars: &[(&str, &str)]) -> anyhow::Result<Value> {
        self.seen.borrow_mut().push(query.to_string());
        let next = self.replies.borrow_mut().pop();
        next.unwrap_or_else(|| Err("unexpected query".to_string()))
            .map_err(|e| anyhow::anyhow!(e))
    }
}

pub(super) fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap()
}

pub(super) fn rate(remaining: u64) -> Result<Value, String> {
    Ok(json!({"data":{"rateLimit":{"remaining":remaining,"resetAt":"2026-11-26T13:00:00Z"}}}))
}

/// The seven rubric items as switches; `ticket_from` builds a ticket passing exactly
/// the items whose bit is set (bit order = `Item::ALL`).
pub(super) fn ticket_from(mask: u8) -> Ticket {
    let body: String = BODY_PARTS
        .iter()
        .filter(|(bits, _)| mask & bits != 0)
        .map(|(_, text)| *text)
        .collect();
    let on = |bit: u8| mask & (1 << bit) != 0;
    let title = ["ticket score", "Add a ticket score"][usize::from(on(6))];
    Ticket {
        title: title.to_string(),
        body,
        labels: vec!["kind:code".to_string(); usize::from(on(5))],
        parent_open: on(2),
        has_milestone: on(2),
    }
}

/// The body text each rubric switch adds: (the mask bits that ask for it, the text).
const BODY_PARTS: [(u8, &str); 5] = [
    (
        1 << 4,
        "## Observed\nit breaks\n\n## Expected\nit works\n\n",
    ),
    (
        1 << 3,
        "Touches src/services/ticket_quality/rubric.rs only.\n\n",
    ),
    (0b11, "## Acceptance\n"),
    (1 << 0, "- [ ] the score is deterministic\n"),
    (1 << 1, "- proved by `cargo test --lib ticket_quality`\n"),
];

/// A plain open ticket: parented, with a milestone, scoring 100, touched today.
pub(super) fn rec(number: u64, title: &str) -> TicketRec {
    let full = ticket_from(0b111_1111);
    TicketRec {
        rec: IssueRec {
            number,
            title: title.to_string(),
            milestone: Some("m1".to_string()),
            parent: Some(1),
            labels: full.labels,
            sub_issues: 0,
            sub_issues_closed: Some(0),
        },
        body: full.body,
        updated_at: now(),
        parent_open: Some(true),
        linked_prs: Vec::<(u64, PrState)>::new(),
    }
}

/// One issue node as `TICKETS_QUERY` returns it.
pub(super) fn node(number: u64) -> Value {
    json!({"number":number,"title":format!("Add thing {number}"),"body":"see src/a.rs and src/b.rs",
           "updatedAt":"2026-11-26T10:00:00Z","milestone":{"title":"m1"},
           "parent":{"number":1,"state":"OPEN"},"subIssues":{"totalCount":0},
           "subIssuesSummary":{"completed":0},"labels":{"nodes":[{"name":"kind:code"}]},
           "closedByPullRequestsReferences":{"nodes":[{"number":7,"state":"MERGED"}]}})
}

pub(super) fn page(total: u64, nodes: Vec<Value>) -> Result<Value, String> {
    Ok(json!({"data":{"repository":{"issues":{"totalCount":total,
        "pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":nodes}}}}))
}
