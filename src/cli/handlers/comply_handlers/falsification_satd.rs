#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod satd_manifestation_falsification {
    use super::super::comply_cb_detect::{
        classify_satd_by_pattern_id, classify_satd_manifestation, SATDManifestationType, Severity,
    };

    // Tests 056-070: Verify code vs comment SATD distinction

    #[test]
    fn tp_056_code_satd_is_code_type() {
        // todo!() macro should be classified as Code manifestation
        let result = classify_satd_manifestation("fn foo() { todo!() }");
        assert_eq!(
            result,
            SATDManifestationType::Code,
            "FALSIFIED: todo!() not classified as Code"
        );
    }

    #[test]
    fn tp_057_comment_satd_is_comment_type() {
        // // TODO: comment should be classified as Comment manifestation
        let result = classify_satd_manifestation("// TODO: implement this later");
        assert_eq!(
            result,
            SATDManifestationType::Comment,
            "FALSIFIED: // TODO not classified as Comment"
        );
    }

    #[test]
    fn tp_058_code_satd_escalates_severity() {
        // Code SATD with Medium base -> High effective severity
        let code_type = SATDManifestationType::Code;
        let escalated = code_type.escalate_severity(Severity::Medium);
        assert_eq!(
            escalated,
            Severity::High,
            "FALSIFIED: Code SATD Medium did not escalate to High"
        );
    }

    #[test]
    fn tp_059_comment_satd_no_escalation() {
        // Comment SATD keeps original severity
        let comment_type = SATDManifestationType::Comment;
        let result = comment_type.escalate_severity(Severity::Medium);
        assert_eq!(
            result,
            Severity::Medium,
            "FALSIFIED: Comment SATD should not escalate"
        );
    }

    #[test]
    fn tp_060_unimplemented_is_code_type() {
        let result = classify_satd_manifestation("fn bar() { unimplemented!() }");
        assert_eq!(
            result,
            SATDManifestationType::Code,
            "FALSIFIED: unimplemented!() not classified as Code"
        );
    }

    #[test]
    fn tp_061_fixme_comment_is_comment_type() {
        let result = classify_satd_manifestation("// FIXME: this is broken");
        assert_eq!(
            result,
            SATDManifestationType::Comment,
            "FALSIFIED: // FIXME not classified as Comment"
        );
    }

    #[test]
    fn tp_062_panic_not_implemented_is_code_type() {
        let result = classify_satd_manifestation(r#"fn x() { panic!("not implemented") }"#);
        assert_eq!(
            result,
            SATDManifestationType::Code,
            "FALSIFIED: panic!(\"not implemented\") not classified as Code"
        );
    }

    #[test]
    fn tp_063_hack_comment_is_comment_type() {
        let result = classify_satd_manifestation("// HACK: temporary workaround");
        assert_eq!(
            result,
            SATDManifestationType::Comment,
            "FALSIFIED: // HACK not classified as Comment"
        );
    }

    #[test]
    fn edge_064_doc_comment_with_todo_is_comment() {
        // /// TODO in doc comment is Comment, not Code
        let result = classify_satd_manifestation("/// TODO: document this function");
        assert_eq!(
            result,
            SATDManifestationType::Comment,
            "FALSIFIED: /// TODO doc comment not classified as Comment"
        );
    }

    #[test]
    fn edge_065_python_raise_is_code() {
        // raise NotImplementedError is Code type
        let result = classify_satd_manifestation("raise NotImplementedError('not done')");
        assert_eq!(
            result,
            SATDManifestationType::Code,
            "FALSIFIED: Python NotImplementedError not classified as Code"
        );
    }

    #[test]
    fn edge_066_python_comment_is_comment() {
        // # TODO: is Comment type
        let result = classify_satd_manifestation("# TODO: implement this");
        assert_eq!(
            result,
            SATDManifestationType::Comment,
            "FALSIFIED: Python # TODO not classified as Comment"
        );
    }

    #[test]
    fn edge_067_empty_body_is_code() {
        // fn foo() {} is Code type
        let result = classify_satd_by_pattern_id("CB-050-D");
        assert_eq!(
            result,
            SATDManifestationType::Code,
            "FALSIFIED: Empty function body (CB-050-D) not classified as Code"
        );
    }

    #[test]
    fn edge_068_multiline_comment_is_comment() {
        // /* TODO */ is Comment type
        let result = classify_satd_manifestation("/* TODO: fix this later */");
        assert_eq!(
            result,
            SATDManifestationType::Comment,
            "FALSIFIED: /* TODO */ not classified as Comment"
        );
    }

    #[test]
    fn edge_069_critical_code_satd_stays_critical() {
        // Critical code SATD cannot escalate further
        let code_type = SATDManifestationType::Code;
        let result = code_type.escalate_severity(Severity::Critical);
        assert_eq!(
            result,
            Severity::Critical,
            "FALSIFIED: Critical should stay Critical"
        );
    }

    #[test]
    fn edge_070_low_comment_satd_stays_low() {
        // Low comment SATD stays Low (no escalation)
        let comment_type = SATDManifestationType::Comment;
        let result = comment_type.escalate_severity(Severity::Low);
        assert_eq!(
            result,
            Severity::Low,
            "FALSIFIED: Comment Low should stay Low"
        );
    }
}
