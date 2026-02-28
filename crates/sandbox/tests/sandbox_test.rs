use std::time::Duration;

use metaygn_sandbox::{Hypothesis, ProcessSandbox, SandboxConfig, SandboxError};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check whether a command is available on the system.
fn command_exists(cmd: &str) -> bool {
    // On Windows, "bash" resolves to WSL's bash.exe which may not work.
    // Check for Git Bash specifically.
    if cfg!(windows) && cmd == "bash" {
        return std::path::Path::new(r"C:\Program Files\Git\usr\bin\bash.exe").exists()
            || std::path::Path::new(r"C:\Program Files (x86)\Git\usr\bin\bash.exe").exists();
    }
    std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Return the python command available on this system, or None.
fn python_cmd() -> Option<&'static str> {
    if cfg!(windows) {
        if command_exists("python") {
            Some("python")
        } else if command_exists("python3") {
            Some("python3")
        } else {
            None
        }
    } else if command_exists("python3") {
        Some("python3")
    } else if command_exists("python") {
        Some("python")
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Python tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_python_hello() {
    if python_cmd().is_none() {
        eprintln!("SKIP: python not available");
        return;
    }

    let sandbox = ProcessSandbox::with_defaults();
    let result = sandbox.execute("python", r#"print("hello")"#).await.unwrap();

    assert!(result.success, "expected success, got: {:?}", result);
    assert_eq!(result.exit_code, Some(0));
    assert!(
        result.stdout.trim().contains("hello"),
        "stdout should contain 'hello', got: {:?}",
        result.stdout
    );
    assert!(!result.timed_out);
}

#[tokio::test]
async fn execute_python_error() {
    if python_cmd().is_none() {
        eprintln!("SKIP: python not available");
        return;
    }

    let sandbox = ProcessSandbox::with_defaults();
    let result = sandbox
        .execute("python", "raise Exception('boom')")
        .await
        .unwrap();

    assert!(!result.success, "expected failure, got: {:?}", result);
    assert!(
        result.stderr.contains("Exception") || result.stderr.contains("boom"),
        "stderr should contain traceback, got: {:?}",
        result.stderr
    );
    assert!(!result.timed_out);
}

// ---------------------------------------------------------------------------
// Bash tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_bash_echo() {
    if !command_exists("bash") {
        eprintln!("SKIP: bash not available");
        return;
    }

    let sandbox = ProcessSandbox::with_defaults();
    let result = sandbox.execute("bash", "echo hello_from_bash").await.unwrap();

    assert!(result.success, "expected success, got: {:?}", result);
    assert_eq!(result.exit_code, Some(0));
    assert!(
        result.stdout.contains("hello_from_bash"),
        "stdout should contain output, got: {:?}",
        result.stdout
    );
}

// ---------------------------------------------------------------------------
// Node tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_node_hello() {
    if !command_exists("node") {
        eprintln!("SKIP: node not available");
        return;
    }

    let sandbox = ProcessSandbox::with_defaults();
    let result = sandbox
        .execute("node", "console.log('hello_node')")
        .await
        .unwrap();

    assert!(result.success, "expected success, got: {:?}", result);
    assert!(
        result.stdout.contains("hello_node"),
        "stdout should contain output, got: {:?}",
        result.stdout
    );
}

// ---------------------------------------------------------------------------
// Timeout test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_timeout() {
    if !command_exists("bash") {
        eprintln!("SKIP: bash not available");
        return;
    }

    let sandbox = ProcessSandbox::new(SandboxConfig {
        timeout: Duration::from_secs(1),
        max_output_bytes: 64 * 1024,
    });

    let result = sandbox.execute("bash", "sleep 10").await.unwrap();

    assert!(result.timed_out, "expected timeout, got: {:?}", result);
    assert!(!result.success);
    // Duration should be roughly 1 second (allow up to 3s for CI slack).
    assert!(
        result.duration_ms >= 900 && result.duration_ms < 3000,
        "duration_ms should be ~1000, got: {}",
        result.duration_ms
    );
}

// ---------------------------------------------------------------------------
// Hypothesis tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hypothesis_success_matches() {
    if python_cmd().is_none() {
        eprintln!("SKIP: python not available");
        return;
    }

    let sandbox = ProcessSandbox::with_defaults();
    let hypothesis = Hypothesis {
        description: "Python print should succeed".to_string(),
        language: "python".to_string(),
        code: "print('hypothesis works')".to_string(),
        expected_success: true,
    };

    let result = sandbox.test_hypothesis(&hypothesis).await;

    assert!(result.success, "expected success, got: {:?}", result);
    assert_eq!(
        result.success, hypothesis.expected_success,
        "hypothesis expectation should match"
    );
    assert!(result.stdout.contains("hypothesis works"));
}

#[tokio::test]
async fn hypothesis_failure_matches() {
    if python_cmd().is_none() {
        eprintln!("SKIP: python not available");
        return;
    }

    let sandbox = ProcessSandbox::with_defaults();
    let hypothesis = Hypothesis {
        description: "Python exception should fail".to_string(),
        language: "python".to_string(),
        code: "raise ValueError('nope')".to_string(),
        expected_success: false,
    };

    let result = sandbox.test_hypothesis(&hypothesis).await;

    assert!(!result.success, "expected failure, got: {:?}", result);
    assert_eq!(
        result.success, hypothesis.expected_success,
        "hypothesis expectation should match"
    );
}

// ---------------------------------------------------------------------------
// Unsupported language
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unsupported_language_errors() {
    let sandbox = ProcessSandbox::with_defaults();
    let result = sandbox.execute("cobol", "DISPLAY 'HI'").await;

    assert!(result.is_err(), "expected error for unsupported language");
    match result.unwrap_err() {
        SandboxError::UnsupportedLanguage(lang) => {
            assert_eq!(lang, "cobol");
        }
        other => panic!("expected UnsupportedLanguage, got: {other:?}"),
    }
}

#[tokio::test]
async fn unsupported_language_rust() {
    let sandbox = ProcessSandbox::with_defaults();
    let result = sandbox.execute("rust", "fn main() {}").await;

    assert!(result.is_err(), "expected error for rust");
    assert!(matches!(
        result.unwrap_err(),
        SandboxError::UnsupportedLanguage(_)
    ));
}

// ---------------------------------------------------------------------------
// Hypothesis with unsupported language goes through test_hypothesis gracefully
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hypothesis_unsupported_language_returns_failure() {
    let sandbox = ProcessSandbox::with_defaults();
    let hypothesis = Hypothesis {
        description: "COBOL should not work".to_string(),
        language: "cobol".to_string(),
        code: "DISPLAY 'HI'".to_string(),
        expected_success: false,
    };

    let result = sandbox.test_hypothesis(&hypothesis).await;

    // test_hypothesis should not panic; it returns a SandboxResult with success=false.
    assert!(!result.success);
    assert!(result.stderr.contains("unsupported language"));
}

// ---------------------------------------------------------------------------
// Output truncation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn output_truncation() {
    if python_cmd().is_none() {
        eprintln!("SKIP: python not available");
        return;
    }

    let sandbox = ProcessSandbox::new(SandboxConfig {
        timeout: Duration::from_secs(5),
        max_output_bytes: 32, // very small limit
    });

    // Python prints 100 'A' characters.
    let result = sandbox
        .execute("python", "print('A' * 100)")
        .await
        .unwrap();

    assert!(result.success);
    // stdout should be truncated to roughly max_output_bytes.
    assert!(
        result.stdout.len() <= 33, // allow +1 for rounding
        "stdout should be truncated, got {} bytes",
        result.stdout.len()
    );
}
