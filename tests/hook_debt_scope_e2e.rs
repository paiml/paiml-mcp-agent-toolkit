use std::path::PathBuf;
use std::process::Command;

fn find_pmat() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pmat"))
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn end_to_end_the_hook_allows_the_o11_case_and_refuses_growth() {
    let dir = tempfile::tempdir().expect("tempdir");
    let p = dir.path();
    git(p, &["init", "-q", "--template=", "--initial-branch=main"]);
    git(p, &["config", "user.email", "fixture@example.com"]);
    git(p, &["config", "user.name", "Fixture"]);
    std::fs::create_dir_all(p.join("src")).expect("mkdir");

    // debted > 30 (cyclomatic limit is 30 by default?). The hook's default max_cyclomatic is 30?
    // Let's check src/cli/handlers/complexity_handlers/mod.rs for default.
    // Or I can just write a huge function.
    let mut debted = String::from("fn debted(x: i32) -> i32 {\nlet mut n = x;\n");
    for i in 0..35 {
        debted.push_str(&format!("if x > {} {{ n += 1; }}\n", i));
    }
    debted.push_str("n\n}\n");
    let innocent_head = "fn innocent(x: i32) -> i32 {\nx + 1\n}\n";
    std::fs::write(p.join("src/lib.rs"), format!("{debted}\n{innocent_head}")).expect("write");
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", "pre-existing debt"]);

    // Stage one-line change in innocent
    let innocent_one_line = "fn innocent(x: i32) -> i32 {\nx + 2\n}\n";
    std::fs::write(
        p.join("src/lib.rs"),
        format!("{debted}\n{innocent_one_line}"),
    )
    .expect("write");
    git(p, &["add", "src/lib.rs"]);

    let pmat = find_pmat();
    let out_allowed = Command::new(&pmat)
        .current_dir(p)
        .args([
            "analyze",
            "complexity",
            "--diff-scope",
            "--file",
            "src/lib.rs",
        ])
        .output()
        .expect("pmat");

    let stdout = String::from_utf8_lossy(&out_allowed.stdout);
    let stderr = String::from_utf8_lossy(&out_allowed.stderr);
    assert!(
        out_allowed.status.success(),
        "one-line change should be allowed, got status: {}, out: {}, err: {}",
        out_allowed.status,
        stdout,
        stderr
    );

    // Stage growth in innocent
    let mut innocent_growth = String::from("fn innocent(x: i32) -> i32 {\nlet mut n = x;\n");
    for i in 0..35 {
        innocent_growth.push_str(&format!("if x > {} {{ n += 1; }}\n", i));
    }
    innocent_growth.push_str("n\n}\n");
    std::fs::write(p.join("src/lib.rs"), format!("{debted}\n{innocent_growth}")).expect("write");
    git(p, &["add", "src/lib.rs"]);

    let out_refused = Command::new(&pmat)
        .current_dir(p)
        .args([
            "analyze",
            "complexity",
            "--diff-scope",
            "--file",
            "src/lib.rs",
        ])
        .output()
        .expect("pmat");

    assert!(!out_refused.status.success(), "growth should be refused");
    let out_str = String::from_utf8_lossy(&out_refused.stdout);
    let err_str = String::from_utf8_lossy(&out_refused.stderr);
    let combined = format!("{out_str}\n{err_str}");
    let line = combined
        .lines()
        .find(|l| l.contains("innocent"))
        .unwrap_or_else(|| panic!("a line naming innocent, got:\n{combined}"));
    // render(): "<fn> - <metric> <measured> > <limit> (was <previous>)"
    let nums: Vec<u32> = line
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    assert!(
        nums.len() >= 3,
        "measured, limit and previous must all be printed: {line}"
    );
    let (measured, limit, previous) = (
        nums[nums.len() - 3],
        nums[nums.len() - 2],
        nums[nums.len() - 1],
    );
    assert!(measured > limit, "measured must exceed the limit: {line}");
    assert_eq!(
        previous, 1,
        "innocent was cyclomatic 1 before the growth: {line}"
    );
    assert!(
        line.contains("(was 1)"),
        "previous is rendered as (was 1): {line}"
    );
}
