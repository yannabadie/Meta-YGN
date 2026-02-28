use metaygn_verifiers::completion::{extract_claims, verify_completion};

#[test]
fn extract_claims_detects_done() {
    let claims = extract_claims("I'm done! Everything has been implemented.");
    assert!(
        claims.claims_completion,
        "Expected claims_completion = true for 'I'm done!'"
    );
}

#[test]
fn extract_claims_detects_files() {
    let claims = extract_claims("Created `src/main.rs` and updated `lib/utils.py` for the feature.");
    assert!(
        claims.files_mentioned.iter().any(|f| f == "src/main.rs"),
        "Expected src/main.rs in files_mentioned, got: {:?}",
        claims.files_mentioned
    );
    assert!(
        claims.files_mentioned.iter().any(|f| f == "lib/utils.py"),
        "Expected lib/utils.py in files_mentioned, got: {:?}",
        claims.files_mentioned
    );
}

#[test]
fn extract_claims_no_false_positive() {
    let claims = extract_claims("Let me think about this approach before proceeding.");
    assert!(
        !claims.claims_completion,
        "Expected claims_completion = false for a thinking message"
    );
    assert!(!claims.claims_tests_pass);
    assert!(!claims.claims_compiles);
}

#[test]
fn verify_missing_file_blocks() {
    // Claim a file that definitely doesn't exist
    let text = "Done! I created `nonexistent/fake/file.rs` for you.";
    let result = verify_completion(text, ".");
    assert!(
        !result.verified,
        "Expected verified = false when claimed file doesn't exist"
    );
    assert!(
        !result.blocking_issues.is_empty(),
        "Expected at least one blocking issue for missing file"
    );
    assert!(
        result.blocking_issues[0].contains("NOT FOUND"),
        "Expected blocking issue to mention NOT FOUND, got: {}",
        result.blocking_issues[0]
    );
}

#[test]
fn verify_existing_file_passes() {
    // Cargo.toml definitely exists at the repo root
    let text = "Done! I updated `Cargo.toml` with the new dependency.";
    let result = verify_completion(text, env!("CARGO_MANIFEST_DIR"));
    assert!(
        result.verified,
        "Expected verified = true when claimed file exists. Blocking: {:?}",
        result.blocking_issues
    );
    assert!(
        result.blocking_issues.is_empty(),
        "Expected no blocking issues when file exists"
    );
}

#[test]
fn verify_completion_without_files_warns() {
    let text = "Done! Everything is working now.";
    let result = verify_completion(text, ".");
    assert!(
        result.verified,
        "Expected verified = true (no blocking issues) for completion without files"
    );
    assert!(
        result.warnings.iter().any(|w| w.contains("no files were mentioned")),
        "Expected warning about no files mentioned, got: {:?}",
        result.warnings
    );
}

#[test]
fn claims_tests_pass_warns() {
    let text = "All tests pass and the implementation is complete.";
    let result = verify_completion(text, ".");
    assert!(
        result.warnings.iter().any(|w| w.contains("tests pass")),
        "Expected warning about test pass claim, got: {:?}",
        result.warnings
    );
    // Should still be verified (tests claim is a warning, not a blocker)
    assert!(
        result.verified,
        "Expected verified = true (test claims are warnings, not blockers)"
    );
}
