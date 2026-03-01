//! Integration tests for the tool forge.

use std::collections::HashMap;
use std::sync::Arc;

use metaygn_daemon::forge::{ScriptLang, engine::ForgeEngine, list_templates};
use metaygn_sandbox::ProcessSandbox;

/// Helper: create a `ForgeEngine` backed by a default sandbox.
fn engine() -> ForgeEngine {
    let sandbox = Arc::new(ProcessSandbox::with_defaults());
    ForgeEngine::new(sandbox)
}

// -----------------------------------------------------------------------
// Generation tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn generate_grep_checker() {
    let mut eng = engine();
    let params = HashMap::new();
    let spec = eng.generate("grep-pattern-checker", &params).unwrap();

    assert_eq!(spec.name, "grep-pattern-checker");
    assert_eq!(spec.language, ScriptLang::Python);
    assert!(!spec.source_code.is_empty());
    assert!(!spec.content_hash.is_empty());
    assert_eq!(spec.content_hash.len(), 64); // SHA-256 hex
}

#[tokio::test]
async fn generate_unknown_template_fails() {
    let mut eng = engine();
    let params = HashMap::new();
    let result = eng.generate("does-not-exist", &params);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("unknown template"), "error was: {msg}");
}

// -----------------------------------------------------------------------
// Execution tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn execute_json_validator_valid() {
    let mut eng = engine();
    let params = HashMap::new();
    let spec = eng.generate("json-validator", &params).unwrap();

    let input = r#"{"hello": "world", "n": 42}"#;
    let result = eng.execute(&spec, input).await.unwrap();

    assert!(result.success, "stderr: {}", result.stderr);

    let output: serde_json::Value =
        serde_json::from_str(&result.stdout.trim()).expect("stdout should be JSON");
    assert_eq!(output["valid"], true);
    assert_eq!(output["type"], "dict");
}

#[tokio::test]
async fn execute_json_validator_invalid() {
    let mut eng = engine();
    let params = HashMap::new();
    let spec = eng.generate("json-validator", &params).unwrap();

    let input = "not valid json {{{";
    let result = eng.execute(&spec, input).await.unwrap();

    assert!(result.success, "stderr: {}", result.stderr);

    let output: serde_json::Value =
        serde_json::from_str(&result.stdout.trim()).expect("stdout should be JSON");
    assert_eq!(output["valid"], false);
    assert!(output["error"].as_str().is_some());
}

// -----------------------------------------------------------------------
// Cache tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn cache_stores_by_hash() {
    let mut eng = engine();
    let params = HashMap::new();

    // Generate the same template twice — should produce only one cache entry.
    let spec1 = eng.generate("json-validator", &params).unwrap();
    let spec2 = eng.generate("json-validator", &params).unwrap();

    assert_eq!(spec1.content_hash, spec2.content_hash);
    assert_eq!(eng.cache_size(), 1);

    // Verify we can look it up by hash.
    assert!(eng.get_cached(&spec1.content_hash).is_some());
}

// -----------------------------------------------------------------------
// Template catalogue tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn list_templates_returns_all() {
    let names = list_templates();
    assert_eq!(names.len(), 4);
    assert!(names.contains(&"grep-pattern-checker"));
    assert!(names.contains(&"import-validator"));
    assert!(names.contains(&"json-validator"));
    assert!(names.contains(&"file-exists-checker"));
}
