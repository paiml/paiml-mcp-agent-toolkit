//! F3 falsifier: a `.txt` verdict (or any non-schema file) cannot satisfy the rule.
use super::*;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn body(seat: &str, verdict: &str, family: &str) -> String {
    format!(
        r#"{{"schema":"quorum-receipt/v1","sha":"{SHA}","seat":"{seat}","verdict":"{verdict}","model":"m-{seat}","family":"{family}","author_family":"claude"}}"#
    )
}

fn name(seat: &str) -> String {
    format!("{SHA}.{seat}.json")
}

fn three_pass() -> Vec<(String, String)> {
    vec![
        (name("a"), body("a", "PASS", "gemini")),
        (name("b"), body("b", "PASS", "openai")),
        (name("c"), body("c", "PASS", "qwen")),
    ]
}

#[test]
fn three_distinct_passing_seats_is_success() {
    let p = judge(SHA, &three_pass());
    assert_eq!(p.state, "success");
    assert_eq!(p.context, CONTEXT);
}

#[test]
fn a_txt_verdict_cannot_satisfy_the_rule() {
    let mut r = three_pass();
    r[2] = (format!("{SHA}.c.txt"), "PASS".to_string());
    let p = judge(SHA, &r);
    assert_eq!(p.state, "failure");
    assert!(p.description.contains("2/3"), "{}", p.description);
}

#[test]
fn a_json_that_is_not_the_schema_is_not_counted() {
    for bad in [
        "PASS",
        "{}",
        &body("c", "PASS", "qwen").replace("\"seat\"", "\"extra\":1,\"seat\""),
        &body("c", "PASS", "qwen").replace("quorum-receipt/v1", "v0"),
        &body("c", "MAYBE", "qwen"),
    ] {
        let mut r = three_pass();
        r[2] = (name("c"), bad.to_string());
        assert_eq!(judge(SHA, &r).state, "failure", "{bad}");
    }
}

#[test]
fn filename_must_agree_with_content() {
    let mut r = three_pass();
    r[2] = (name("zzz"), body("c", "PASS", "qwen")); // seat mismatch
    assert_eq!(judge(SHA, &r).state, "failure");
    let mut r = three_pass();
    r[2] = (name("c").replace("0123", "ffff"), body("c", "PASS", "qwen")); // wrong sha
    assert_eq!(judge(SHA, &r).state, "failure");
}

#[test]
fn a_fail_seat_is_failure_even_with_three_passes() {
    let mut r = three_pass();
    r.push((name("d"), body("d", "FAIL", "gemini")));
    assert_eq!(judge(SHA, &r).state, "failure");
}

#[test]
fn the_authors_family_is_not_a_seat() {
    let mut r = three_pass();
    r[2] = (name("c"), body("c", "PASS", "claude"));
    assert_eq!(judge(SHA, &r).state, "failure");
}

#[test]
fn duplicate_models_count_once() {
    let mut r = three_pass();
    r[2] = (name("c"), body("c", "PASS", "qwen").replace("m-c", "m-a"));
    assert_eq!(judge(SHA, &r).state, "failure");
}

#[test]
fn a_deduplicated_receipt_is_named_in_the_description() {
    let mut r = three_pass();
    r[2] = (name("c"), body("c", "PASS", "qwen").replace("m-c", "m-a"));
    let p = judge(SHA, &r);
    assert_eq!(p.state, "failure");
    assert!(
        p.description.contains("deduped: c/m-a"),
        "{}",
        p.description
    );
}

#[test]
fn a_duplicate_seat_id_is_rejected_not_counted() {
    let mut r = three_pass();
    r.push((
        name("a"),
        body("a", "PASS", "mistral").replace("m-a", "m-z"),
    ));
    let p = judge(SHA, &r);
    assert_eq!(p.state, "success"); // the original three still stand
    assert!(p.description.contains("1 not counted"), "{}", p.description);
    assert!(p.description.contains("dup seat a"), "{}", p.description);
}

#[test]
fn a_malformed_sha_is_a_failure_not_a_panic() {
    assert_eq!(judge("nope", &three_pass()).state, "failure");
}

#[test]
fn gh_args_post_to_the_statuses_endpoint_with_context_quorum() {
    let p = judge(SHA, &three_pass());
    let a = gh_args("paiml/x", SHA, &p);
    assert_eq!(a[0], "api");
    assert!(a.contains(&format!("repos/paiml/x/statuses/{SHA}")));
    assert!(a.contains(&"context=quorum".to_string()));
    assert!(a.contains(&"state=success".to_string()));
}

#[test]
fn load_dir_reads_only_this_shas_json_files() -> std::io::Result<()> {
    let d = std::env::temp_dir().join(format!("qs-{}", std::process::id()));
    std::fs::create_dir_all(&d)?;
    for (n, b) in three_pass() {
        std::fs::write(d.join(n), b)?;
    }
    std::fs::write(d.join(format!("{SHA}.d.txt")), "PASS")?;
    std::fs::write(d.join("other.a.json"), "{}")?;
    let got = load_dir(&d, SHA)?;
    assert_eq!(got.len(), 4); // 3 json + the .txt is handed to judge, which refuses it
    assert!(judge(SHA, &got).description.contains("3/3"));
    std::fs::remove_dir_all(&d)?;
    Ok(())
}
