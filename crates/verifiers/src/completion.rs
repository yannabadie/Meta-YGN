use serde::{Deserialize, Serialize};

/// What Claude claims to have done (extracted from last_assistant_message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionClaim {
    pub files_mentioned: Vec<String>, // paths extracted from response
    pub claims_completion: bool,      // "done", "finished", "implemented", etc.
    pub claims_tests_pass: bool,      // "tests pass", "all tests green", etc.
    pub claims_compiles: bool,        // "compiles", "builds successfully", etc.
}

/// Result of verifying claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verified: bool,
    pub checks: Vec<Check>,
    pub blocking_issues: Vec<String>, // issues that should block "Done!" claim
    pub warnings: Vec<String>,        // non-blocking concerns
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Extract claims from Claude's response text
pub fn extract_claims(text: &str) -> CompletionClaim {
    let lower = text.to_lowercase();

    // Extract file paths (patterns like `path/to/file.rs`, src/main.py, etc.)
    // Must handle: backtick-quoted, space-delimited, end-of-line, after punctuation
    let file_pattern =
        regex::Regex::new(r"(?:^|[\s`(])((?:[\w.\-]+/)+[\w.\-]+\.\w{1,10})").unwrap();
    let mut files_mentioned: Vec<String> = file_pattern
        .captures_iter(text)
        .map(|c| c[1].to_string())
        .filter(|f| {
            !f.starts_with("http")
                && !f.contains("...")
                && !f.starts_with("e.g.")
                && !f.starts_with("i.e.")
        })
        .collect();
    // Also catch standalone filenames with extension (no directory prefix)
    // e.g., "updated Cargo.toml" or "modified main.rs"
    let standalone_pattern = regex::Regex::new(
        r"(?:^|[\s`(])([A-Z][\w.\-]*\.\w{1,10}|[\w.\-]+\.(?:rs|py|ts|js|toml|json|yaml|yml|md|sh|sql|html|css))\b",
    )
    .unwrap();
    for cap in standalone_pattern.captures_iter(text) {
        let f = cap[1].to_string();
        if !files_mentioned.contains(&f) && !f.starts_with("http") && !f.contains("...") {
            files_mentioned.push(f);
        }
    }
    // Deduplicate
    files_mentioned.sort();
    files_mentioned.dedup();

    let completion_markers = [
        "done",
        "finished",
        "completed",
        "implemented",
        "all set",
        "ready",
        "everything is",
        "that's it",
    ];
    let claims_completion = completion_markers.iter().any(|m| lower.contains(m));

    let test_markers = [
        "tests pass",
        "all tests",
        "test suite passes",
        "tests green",
        "passing tests",
        "tests are passing",
        "test results: ok",
    ];
    let claims_tests_pass = test_markers.iter().any(|m| lower.contains(m));

    let compile_markers = [
        "compiles",
        "builds successfully",
        "cargo check",
        "no errors",
        "compilation successful",
        "build passed",
    ];
    let claims_compiles = compile_markers.iter().any(|m| lower.contains(m));

    CompletionClaim {
        files_mentioned,
        claims_completion,
        claims_tests_pass,
        claims_compiles,
    }
}

/// Verify claims against filesystem reality
/// Returns checks that can be run (file existence only -- compilation/tests are async)
pub fn verify_files_exist(claims: &CompletionClaim, cwd: &str) -> Vec<Check> {
    use std::path::Path;

    claims
        .files_mentioned
        .iter()
        .map(|file| {
            let full_path = Path::new(cwd).join(file);
            let exists = full_path.exists();
            Check {
                name: format!("file_exists:{}", file),
                passed: exists,
                detail: if exists {
                    format!("{} exists", file)
                } else {
                    format!(
                        "{} NOT FOUND -- Claude mentioned this file but it doesn't exist",
                        file
                    )
                },
            }
        })
        .collect()
}

/// Build the full verification result
pub fn verify_completion(text: &str, cwd: &str) -> VerificationResult {
    let claims = extract_claims(text);
    let mut checks = Vec::new();
    let mut blocking_issues = Vec::new();
    let mut warnings = Vec::new();

    // Check 1: If claims completion, verify mentioned files exist
    if claims.claims_completion && !claims.files_mentioned.is_empty() {
        let file_checks = verify_files_exist(&claims, cwd);
        for check in &file_checks {
            if !check.passed {
                blocking_issues.push(check.detail.clone());
            }
        }
        checks.extend(file_checks);
    }

    // Check 2: If claims tests pass but no test command was run recently
    if claims.claims_tests_pass {
        warnings.push("Claude claims tests pass -- verify by running tests yourself".into());
        checks.push(Check {
            name: "test_claim".into(),
            passed: true, // we can't verify without running tests
            detail: "Test pass claim detected -- recommend manual verification".into(),
        });
    }

    // Check 3: If claims compiles but no build was run
    if claims.claims_compiles {
        warnings.push("Claude claims code compiles -- verify with a build command".into());
        checks.push(Check {
            name: "compile_claim".into(),
            passed: true,
            detail: "Compilation claim detected -- recommend manual verification".into(),
        });
    }

    // Check 4: Completion without any files = suspicious
    if claims.claims_completion && claims.files_mentioned.is_empty() {
        warnings
            .push("Claude claims completion but no files were mentioned in the response".into());
    }

    let verified = blocking_issues.is_empty();

    VerificationResult {
        verified,
        checks,
        blocking_issues,
        warnings,
    }
}
