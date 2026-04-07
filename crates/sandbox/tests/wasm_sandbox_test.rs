#![cfg(feature = "wasm")]
//! Integration tests for the WASM sandbox (requires `wasm` feature).

use metaygn_sandbox::wasm_sandbox::{WasmSandbox, WasmSandboxConfig};

#[test]
fn default_config() {
    let cfg = WasmSandboxConfig::default();
    assert_eq!(cfg.timeout_ms, 5_000);
    assert_eq!(cfg.max_memory_bytes, 16 * 1024 * 1024);
    assert!(!cfg.allow_network);
    assert!(cfg.fuel_budget.is_none());
}

#[test]
fn sandbox_creation_succeeds() {
    let sandbox = WasmSandbox::with_defaults();
    assert!(sandbox.is_ok(), "sandbox creation failed: {:?}", sandbox.err());
}

#[test]
fn execute_minimal_wat() {
    let sandbox = WasmSandbox::with_defaults().unwrap();
    let result = sandbox.execute_wat(r#"(module (func (export "_start")))"#);
    assert!(result.success, "expected success, stderr: {}", result.stderr);
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.timed_out);
    // duration_ms should be non-negative (trivially true for u64, but
    // we assert it was measured).
    assert!(result.duration_ms < 10_000, "execution took too long");
}

#[test]
fn network_disabled_by_default() {
    let cfg = WasmSandboxConfig::default();
    assert!(
        !cfg.allow_network,
        "network must be denied by default for sandbox safety"
    );
}

#[test]
fn custom_timeout_config() {
    let config = WasmSandboxConfig {
        timeout_ms: 2_000,
        ..Default::default()
    };
    let sandbox = WasmSandbox::new(config).unwrap();
    assert_eq!(sandbox.config().timeout_ms, 2_000);
}

#[test]
fn fuel_exhaustion_traps_infinite_loop() {
    let config = WasmSandboxConfig {
        fuel_budget: Some(10),
        ..Default::default()
    };
    let sandbox = WasmSandbox::new(config).unwrap();
    let result = sandbox.execute_wat(
        r#"(module
            (func (export "_start")
                (loop $inf (br $inf))
            )
        )"#,
    );
    assert!(!result.success, "infinite loop should fail");
    assert!(result.timed_out, "should be flagged as timed_out (fuel exhaustion)");
}

#[test]
fn invalid_wat_returns_error() {
    let sandbox = WasmSandbox::with_defaults().unwrap();
    let result = sandbox.execute_wat("THIS IS NOT WAT");
    assert!(!result.success);
    assert!(!result.stderr.is_empty(), "stderr should contain error details");
}

#[test]
fn empty_module_succeeds() {
    let sandbox = WasmSandbox::with_defaults().unwrap();
    let result = sandbox.execute_wat("(module)");
    assert!(result.success, "empty module should succeed, stderr: {}", result.stderr);
}
