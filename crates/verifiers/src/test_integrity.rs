use serde::{Deserialize, Serialize};

/// Analysis of a test file modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIntegrityReport {
    pub is_test_file: bool,
    pub suspicious: bool,
    pub issues: Vec<TestIntegrityIssue>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIntegrityIssue {
    pub issue_type: TestIssueType,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestIssueType {
    AssertionRemoved,      // assert! or assert_eq! line deleted
    AssertionWeakened,     // assert_eq! changed to weaker check
    ExpectedValueChanged, // expected value in assertion changed
    TestFunctionRemoved,  // #[test] function deleted
    TestSkipped,          // #[ignore] or @pytest.mark.skip added
}

/// Check if a file path is a test file
pub fn is_test_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("test")
        || lower.contains("spec")
        || lower.ends_with("_test.rs")
        || lower.ends_with("_test.py")
        || lower.ends_with(".test.ts")
        || lower.ends_with(".test.js")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.js")
        || lower.contains("/tests/")
        || lower.contains("\\tests\\")
        || lower.starts_with("test_")
}

/// Analyze old_string vs new_string in an Edit to detect test weakening
pub fn analyze_test_edit(
    file_path: &str,
    old_string: &str,
    new_string: &str,
) -> TestIntegrityReport {
    if !is_test_file(file_path) {
        return TestIntegrityReport {
            is_test_file: false,
            suspicious: false,
            issues: vec![],
            recommendation: "Not a test file".into(),
        };
    }

    let mut issues = Vec::new();

    // Check 1: Assertions removed (present in old, absent in new)
    let old_asserts = count_assertions(old_string);
    let new_asserts = count_assertions(new_string);
    if new_asserts < old_asserts {
        issues.push(TestIntegrityIssue {
            issue_type: TestIssueType::AssertionRemoved,
            detail: format!(
                "{} assertion(s) removed (was {}, now {})",
                old_asserts - new_asserts,
                old_asserts,
                new_asserts
            ),
        });
    }

    // Check 2: Test functions removed
    let old_tests = count_test_functions(old_string);
    let new_tests = count_test_functions(new_string);
    if new_tests < old_tests {
        issues.push(TestIntegrityIssue {
            issue_type: TestIssueType::TestFunctionRemoved,
            detail: format!(
                "{} test function(s) removed (was {}, now {})",
                old_tests - new_tests,
                old_tests,
                new_tests
            ),
        });
    }

    // Check 3: Tests being skipped/ignored
    let skip_patterns = [
        "#[ignore]",
        "@pytest.mark.skip",
        ".skip(",
        "xit(",
        "xdescribe(",
        "test.skip",
        "@unittest.skip",
    ];
    let old_skips = skip_patterns
        .iter()
        .filter(|p| old_string.contains(*p))
        .count();
    let new_skips = skip_patterns
        .iter()
        .filter(|p| new_string.contains(*p))
        .count();
    if new_skips > old_skips {
        issues.push(TestIntegrityIssue {
            issue_type: TestIssueType::TestSkipped,
            detail: format!("{} new skip/ignore marker(s) added", new_skips - old_skips),
        });
    }

    // Check 4: Expected values changed (heuristic: assert_eq! with different second arg)
    if detect_expected_value_change(old_string, new_string) {
        issues.push(TestIntegrityIssue {
            issue_type: TestIssueType::ExpectedValueChanged,
            detail: "Expected values in assertions appear to have changed".into(),
        });
    }

    let suspicious = !issues.is_empty();
    let recommendation = if suspicious {
        format!(
            "TEST INTEGRITY WARNING: Claude is modifying test assertions instead of fixing the implementation. {} issue(s) detected. Review carefully — the tests may be weakened to hide bugs.",
            issues.len()
        )
    } else {
        "Test modification looks legitimate (no assertions removed or weakened)".into()
    };

    TestIntegrityReport {
        is_test_file: true,
        suspicious,
        issues,
        recommendation,
    }
}

fn count_assertions(text: &str) -> usize {
    let patterns = [
        "assert!(",
        "assert_eq!(",
        "assert_ne!(",
        "debug_assert!(",
        "assertEqual(",
        "assertNotEqual(",
        "assertTrue(",
        "assertFalse(",
        "expect(",
        ".toBe(",
        ".toEqual(",
        ".toMatch(",
        ".should.",
        "assert.equal(",
        "assert.deepEqual(",
    ];
    patterns.iter().map(|p| text.matches(p).count()).sum()
}

fn count_test_functions(text: &str) -> usize {
    let patterns = [
        "#[test]",
        "#[tokio::test]",
        "def test_",
        "it(\"",
        "it('",
        "test(\"",
        "test('",
    ];
    patterns.iter().map(|p| text.matches(p).count()).sum()
}

fn detect_expected_value_change(old: &str, new: &str) -> bool {
    // Simple heuristic: if assert_eq! or assertEqual exists in both
    // but the second argument differs, expected values were changed
    let has_assert_eq_old = old.contains("assert_eq!(") || old.contains("assertEqual(");
    let has_assert_eq_new = new.contains("assert_eq!(") || new.contains("assertEqual(");

    if has_assert_eq_old && has_assert_eq_new {
        // Extract assertion lines and compare
        let old_lines: Vec<&str> = old
            .lines()
            .filter(|l| l.contains("assert_eq!(") || l.contains("assertEqual("))
            .collect();
        let new_lines: Vec<&str> = new
            .lines()
            .filter(|l| l.contains("assert_eq!(") || l.contains("assertEqual("))
            .collect();

        // If same number of assertions but content differs -> suspicious
        if old_lines.len() == new_lines.len() && old_lines != new_lines {
            return true;
        }
    }
    false
}
