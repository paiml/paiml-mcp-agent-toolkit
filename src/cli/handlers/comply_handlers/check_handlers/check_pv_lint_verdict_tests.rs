#[cfg(test)]
mod pv_lint_verdict_tests {
    //! ONT-11 (#1422, ONT-001 R-21 (a)): CB-1201 reads pv's exit code as its
    //! verdict. An Unknown (exit 2) or an error (exit 3) is never a pass and
    //! never a Warn; a reject (exit 1) keeps the `pv_lint_is_error` policy.
    use super::*;

    const PASS_JSON: &str = r#"{"passed":true,"findings":[]}"#;
    const REJECT_JSON: &str =
        r#"{"passed":false,"findings":[{"severity":"error","message":"bad shape"}]}"#;

    #[test]
    fn ont11_exit_2_is_unknown_with_the_decline_line_verbatim() {
        assert_eq!(
            classify_pv_lint(Some(2), "", "note: first\ndecline: NotArmed\n"),
            PvLintRun::Unknown("decline: NotArmed".into())
        );
    }

    #[test]
    fn ont11_exit_2_is_unknown_even_when_stdout_says_passed() {
        // The JSON is not the verdict when the exit code says Unknown.
        assert_eq!(
            classify_pv_lint(Some(2), PASS_JSON, "decline: Prose"),
            PvLintRun::Unknown("decline: Prose".into())
        );
    }

    #[test]
    fn ont11_exit_2_without_a_decline_line_is_still_unknown() {
        assert!(matches!(
            classify_pv_lint(Some(2), "", ""),
            PvLintRun::Unknown(l) if l.starts_with("decline:")
        ));
    }

    #[test]
    fn ont11_exit_3_is_an_error_with_the_error_line() {
        assert_eq!(
            classify_pv_lint(Some(3), "decline: x\nerror: cannot read contracts/x.yaml", ""),
            PvLintRun::Errored("error: cannot read contracts/x.yaml".into())
        );
        assert!(matches!(
            classify_pv_lint(Some(3), PASS_JSON, ""),
            PvLintRun::Errored(l) if l.starts_with("error:")
        ));
    }

    #[test]
    fn ont11_exit_0_and_1_are_judged_by_the_json() {
        assert_eq!(
            classify_pv_lint(Some(0), PASS_JSON, ""),
            PvLintRun::Judged {
                passed: true,
                detail: None
            }
        );
        assert_eq!(
            classify_pv_lint(Some(1), REJECT_JSON, ""),
            PvLintRun::Judged {
                passed: false,
                detail: Some("bad shape".into())
            }
        );
    }

    #[test]
    fn ont11_a_missing_pv_is_unknown_not_a_warn() {
        let e = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        assert_eq!(
            pv_lint_spawn_error(&e),
            PvLintRun::Unknown("decline: pv not found".into())
        );
        let e = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        assert!(matches!(
            pv_lint_spawn_error(&e),
            PvLintRun::Unknown(l) if l.starts_with("decline: pv could not be run")
        ));
    }
}
