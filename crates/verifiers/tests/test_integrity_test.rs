use metaygn_verifiers::test_integrity::{TestIssueType, analyze_test_edit, is_test_file};

#[test]
fn detects_test_files() {
    assert!(is_test_file("tests/foo_test.rs"));
    assert!(is_test_file("src/tests/widget.rs"));
    assert!(is_test_file("test_parser.py"));
    assert!(is_test_file("components/Button.test.ts"));
    assert!(is_test_file("components/Button.spec.js"));
    assert!(is_test_file("crates/core/tests/integration.rs"));
}

#[test]
fn non_test_file_passes() {
    assert!(!is_test_file("src/main.rs"));
    assert!(!is_test_file("src/lib.rs"));
    assert!(!is_test_file("Cargo.toml"));
    assert!(!is_test_file("src/parser.rs"));
}

#[test]
fn assertion_removed_is_suspicious() {
    let old = r#"
#[test]
fn it_works() {
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}
"#;
    let new = r#"
#[test]
fn it_works() {
    // check passes
}
"#;

    let report = analyze_test_edit("tests/my_test.rs", old, new);
    assert!(report.is_test_file);
    assert!(report.suspicious);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.issue_type == TestIssueType::AssertionRemoved)
    );
}

#[test]
fn test_function_removed_is_suspicious() {
    let old = r#"
#[test]
fn first_test() {
    assert!(true);
}

#[test]
fn second_test() {
    assert_eq!(1 + 1, 2);
}
"#;
    let new = r#"
#[test]
fn first_test() {
    assert!(true);
}
"#;

    let report = analyze_test_edit("tests/my_test.rs", old, new);
    assert!(report.is_test_file);
    assert!(report.suspicious);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.issue_type == TestIssueType::TestFunctionRemoved)
    );
}

#[test]
fn skip_added_is_suspicious() {
    let old = r#"
#[test]
fn important_test() {
    assert_eq!(compute(), 42);
}
"#;
    let new = r#"
#[test]
#[ignore]
fn important_test() {
    assert_eq!(compute(), 42);
}
"#;

    let report = analyze_test_edit("tests/my_test.rs", old, new);
    assert!(report.is_test_file);
    assert!(report.suspicious);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.issue_type == TestIssueType::TestSkipped)
    );
}

#[test]
fn expected_value_changed_is_suspicious() {
    let old = r#"
#[test]
fn it_returns_correct_value() {
    assert_eq!(compute(), 42);
}
"#;
    let new = r#"
#[test]
fn it_returns_correct_value() {
    assert_eq!(compute(), 17);
}
"#;

    let report = analyze_test_edit("tests/my_test.rs", old, new);
    assert!(report.is_test_file);
    assert!(report.suspicious);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.issue_type == TestIssueType::ExpectedValueChanged)
    );
}

#[test]
fn legitimate_edit_passes() {
    let old = r#"
#[test]
fn it_works() {
    assert!(result.is_ok());
}
"#;
    let new = r#"
#[test]
fn it_works() {
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_ne!(result.unwrap(), 0);
}
"#;

    let report = analyze_test_edit("tests/my_test.rs", old, new);
    assert!(report.is_test_file);
    assert!(
        !report.suspicious,
        "Adding assertions should not be suspicious, got: {:?}",
        report.issues
    );
}
