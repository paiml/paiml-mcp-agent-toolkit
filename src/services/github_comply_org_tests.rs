//! GHC-001 `--org` against a fake GitHub. Names: `old*` archived, `bad*` unreadable,
//! `big*` over the issue cap.

use super::org::*;
use super::source::{list_org_repos, Graphql};
use super::*;
use chrono::TimeZone;
use serde_json::{json, Value};

struct Fake(Vec<Vec<&'static str>>);

impl Graphql for Fake {
    fn query(&self, q: &str, vars: &[(&str, &str)]) -> anyhow::Result<Value> {
        let var = |k: &str| vars.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
        let name = var("name").unwrap_or("");
        let pi = json!({"hasNextPage":false,"endCursor":null});
        if q.contains("rateLimit") {
            return Ok(json!({"data":{"rateLimit":{"remaining":4000,"resetAt":"x"}}}));
        } else if q.contains("organization(") {
            let i: usize = var("after").map_or(0, |a| a.parse().unwrap_or(0));
            let nodes: Vec<Value> = self.0[i].iter().map(|n| {
                json!({"nameWithOwner":format!("acme/{n}"),"isArchived":n.starts_with("old")})
            }).collect();
            let pi = json!({"hasNextPage":i + 1 < self.0.len(),"endCursor":(i + 1).to_string()});
            return Ok(
                json!({"data":{"organization":{"repositories":{"pageInfo":pi,"nodes":nodes}}}}),
            );
        } else if name.starts_with("bad") {
            anyhow::bail!("Could not resolve to a Repository");
        }
        let total = if name.starts_with("big") { 101 } else { 3 };
        let (k, t) = if q.contains("issues(") {
            ("issues", total)
        } else {
            ("pullRequests", 0)
        };
        let page = json!({"pageInfo":pi,"nodes":[],"totalCount":t});
        Ok(json!({"data":{"repository":{k:page}}}))
    }
}

fn run(pages: Vec<Vec<&'static str>>) -> OrgReport {
    let now = chrono::Utc
        .with_ymd_and_hms(2026, 11, 5, 12, 0, 0)
        .single()
        .unwrap_or_default();
    let r = evaluate_org(
        &Fake(pages),
        "acme",
        &Policy::default(),
        &Rule::RECONCILER,
        now,
    );
    r.unwrap_or_else(|e| OrgReport {
        org: format!("{e:#}"),
        repos: vec![],
    })
}

#[test]
fn two_pages_are_read_and_archived_repos_are_skipped() {
    let g = Fake(vec![vec!["a", "old"], vec!["b"]]);
    assert_eq!(
        list_org_repos(&g, "acme").unwrap_or_default(),
        ["acme/a", "acme/b"]
    );
}

#[test]
fn clean_org_is_ok_and_nothing_judged_is_not() {
    let r = run(vec![vec!["a", "b"]]);
    assert!(r.ok() && r.counts() == (2, 0, 0));
    assert!(!run(vec![vec!["old"]]).ok());
    assert!(!run(vec![vec!["a", "bad"]]).ok());
}

#[test]
fn violation_and_unreadable_repo_are_counted_apart_and_named() {
    let r = run(vec![vec!["a", "big", "bad"]]);
    assert!(!r.ok());
    assert_eq!(r.counts(), (1, 1, 1));
    let t = r.render_text();
    assert!(t.contains("acme/bad") && t.contains("not_measured: "));
}

/// A canned org-list reply for the pagination-contract tests.
struct Canned(Value);

impl Graphql for Canned {
    fn query(&self, _q: &str, _vars: &[(&str, &str)]) -> anyhow::Result<Value> {
        Ok(self.0.clone())
    }
}

fn reply(page_info: Value) -> Canned {
    let nodes = json!([{"nameWithOwner":"acme/a","isArchived":false}]);
    Canned(json!({"data":{"organization":{"repositories":{"pageInfo":page_info,"nodes":nodes}}}}))
}

#[test]
fn missing_or_non_boolean_has_next_page_is_an_error_not_a_short_list() {
    assert!(list_org_repos(&reply(json!({"endCursor":"c"})), "acme").is_err());
    assert!(list_org_repos(&reply(json!({"hasNextPage":"yes","endCursor":"c"})), "acme").is_err());
    assert!(list_org_repos(&reply(json!({"hasNextPage":false})), "acme").is_ok());
}

#[test]
fn has_next_page_with_null_or_empty_cursor_is_an_error() {
    let null = json!({"hasNextPage":true,"endCursor":null});
    let empty = json!({"hasNextPage":true,"endCursor":""});
    assert!(list_org_repos(&reply(null), "acme").is_err());
    assert!(list_org_repos(&reply(empty), "acme").is_err());
}

#[test]
fn empty_org_says_nothing_was_judged() {
    let t = run(vec![vec!["old"]]).render_text();
    assert!(t.contains("no repository was judged"), "{t}");
}

#[test]
fn org_refuses_an_explicit_format_but_parses_alone() {
    use clap::Parser;
    let ok = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let parse = |a: &[&str]| crate::cli::Cli::try_parse_from(a).is_ok();
            let alone = parse(&["pmat", "comply", "github", "--org", "acme"]);
            let json = parse(&[
                "pmat", "comply", "github", "--org", "acme", "--format", "json",
            ]);
            (alone, json)
        })
        .map(|h| h.join().unwrap_or((false, true)))
        .unwrap_or((false, true));
    assert_eq!(ok, (true, false));
}
