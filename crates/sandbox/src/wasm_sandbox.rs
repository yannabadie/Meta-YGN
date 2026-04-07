//! WASM-based sandbox using Wasmtime for capability-based isolation.
//!
//! This module provides a sandboxed execution environment for WebAssembly modules
//! with fuel-based timeout enforcement and memory limits.  It is gated behind the
//! `wasm` feature.
//!
//! ## Current scope (v2.3)
//! - Accept WAT text, compile to WASM via Wasmtime
//! - Run the `_start` export with fuel-limited execution
//! - Return success/failure with timing
//!
//! ## Not yet implemented
//! - WASI stdout/stderr capture (planned for v2.4)
//! - Network capability control via WASI (planned for v2.4)

use std::time::Instant;

use tracing::{debug, warn};
use wasmtime::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};

use crate::SandboxResult;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the WASM sandbox.
#[derive(Debug, Clone)]
pub struct WasmSandboxConfig {
    /// Maximum execution time in milliseconds (translated to fuel units).
    /// Default: 5000 ms.
    pub timeout_ms: u64,

    /// Maximum memory in bytes that the WASM module may allocate.
    /// Default: 16 MB.
    pub max_memory_bytes: usize,

    /// Whether network access is allowed.
    /// Default: `false`.
    ///
    /// NOTE: Network control is not yet enforced -- WASI integration is
    /// planned for v2.4.  This field exists so callers can set policy now.
    pub allow_network: bool,

    /// Fuel budget for the Wasmtime engine.  Each WASM instruction consumes
    /// roughly one unit of fuel.  If `None`, a budget is derived from
    /// `timeout_ms` (1_000 fuel per ms as a rough heuristic).
    pub fuel_budget: Option<u64>,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5_000,
            max_memory_bytes: 16 * 1024 * 1024, // 16 MB
            allow_network: false,
            fuel_budget: None,
        }
    }
}

impl WasmSandboxConfig {
    /// Effective fuel budget: explicit value or derived from timeout_ms.
    fn effective_fuel(&self) -> u64 {
        self.fuel_budget
            .unwrap_or_else(|| self.timeout_ms.saturating_mul(1_000))
    }
}

// ---------------------------------------------------------------------------
// Store state
// ---------------------------------------------------------------------------

/// Host state held inside the Wasmtime `Store`.
struct SandboxState {
    limits: StoreLimits,
}

// ---------------------------------------------------------------------------
// WasmSandbox
// ---------------------------------------------------------------------------

/// A Wasmtime-backed sandbox that compiles and runs WASM modules with
/// fuel-based execution limits and memory caps.
pub struct WasmSandbox {
    config: WasmSandboxConfig,
    engine: Engine,
}

impl WasmSandbox {
    /// Create a new `WasmSandbox` with the given configuration.
    pub fn new(config: WasmSandboxConfig) -> wasmtime::Result<Self> {
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);

        let engine = Engine::new(&engine_config)
            .map_err(|e| e.context("failed to create Wasmtime engine"))?;

        Ok(Self { config, engine })
    }

    /// Create a sandbox with default settings.
    pub fn with_defaults() -> wasmtime::Result<Self> {
        Self::new(WasmSandboxConfig::default())
    }

    /// Return a reference to the current configuration.
    pub fn config(&self) -> &WasmSandboxConfig {
        &self.config
    }

    /// Compile and execute a WAT (WebAssembly Text) module.
    ///
    /// The module must export a `_start` function with signature `() -> ()`.
    /// Execution is bounded by the configured fuel budget and memory limit.
    ///
    /// Returns a [`SandboxResult`] with timing and success/failure info.
    /// Stdout and stderr are empty strings for now (WASI capture is planned
    /// for v2.4).
    pub fn execute_wat(&self, wat_source: &str) -> SandboxResult {
        let start = Instant::now();

        match self.execute_wat_inner(wat_source) {
            Ok(()) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                debug!(duration_ms, "wasm sandbox: execution succeeded");
                SandboxResult {
                    success: true,
                    exit_code: Some(0),
                    stdout: String::new(),
                    stderr: String::new(),
                    duration_ms,
                    timed_out: false,
                }
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let msg = format!("{e:#}");
                let timed_out = msg.contains("fuel") || msg.contains("all fuel consumed");

                if timed_out {
                    warn!(duration_ms, "wasm sandbox: execution ran out of fuel");
                } else {
                    warn!(duration_ms, error = %msg, "wasm sandbox: execution failed");
                }

                SandboxResult {
                    success: false,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: msg,
                    duration_ms,
                    timed_out,
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    fn execute_wat_inner(&self, wat_source: &str) -> wasmtime::Result<()> {
        // 1. Compile the WAT text into a module.
        let module = Module::new(&self.engine, wat_source)
            .map_err(|e| e.context("failed to compile WAT module"))?;

        // 2. Build a store with fuel + memory limits.
        let limits = StoreLimitsBuilder::new()
            .memory_size(self.config.max_memory_bytes)
            .build();

        let mut store = Store::new(&self.engine, SandboxState { limits });
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(self.config.effective_fuel())
            .map_err(|e| e.context("failed to set fuel on store"))?;

        // 3. Link (no host imports for now) and instantiate.
        let linker: Linker<SandboxState> = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| e.context("failed to instantiate WASM module"))?;

        // 4. Call `_start` if it exists; otherwise treat as success (the
        //    module may just define data or be a no-op).
        if let Ok(start_fn) =
            instance.get_typed_func::<(), ()>(&mut store, "_start")
        {
            start_fn
                .call(&mut store, ())
                .map_err(|e| e.context("_start function trapped"))?;
        } else {
            debug!("wasm sandbox: module has no _start export, treating as success");
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = WasmSandboxConfig::default();
        assert_eq!(cfg.timeout_ms, 5_000);
        assert_eq!(cfg.max_memory_bytes, 16 * 1024 * 1024);
        assert!(!cfg.allow_network);
        assert!(cfg.fuel_budget.is_none());
        // Derived fuel: 5_000 * 1_000 = 5_000_000
        assert_eq!(cfg.effective_fuel(), 5_000_000);
    }

    #[test]
    fn sandbox_creation_succeeds() {
        let sandbox = WasmSandbox::with_defaults();
        assert!(sandbox.is_ok());
    }

    #[test]
    fn execute_minimal_wat() {
        let sandbox = WasmSandbox::with_defaults().unwrap();
        let result = sandbox.execute_wat(r#"(module (func (export "_start")))"#);
        assert!(result.success, "expected success, got: {:?}", result);
        assert_eq!(result.exit_code, Some(0));
        assert!(!result.timed_out);
    }

    #[test]
    fn execute_empty_module() {
        let sandbox = WasmSandbox::with_defaults().unwrap();
        // Module with no _start -- should still succeed.
        let result = sandbox.execute_wat("(module)");
        assert!(result.success, "expected success, got: {:?}", result);
    }

    #[test]
    fn invalid_wat_returns_failure() {
        let sandbox = WasmSandbox::with_defaults().unwrap();
        let result = sandbox.execute_wat("not valid wat at all");
        assert!(!result.success);
        assert!(!result.stderr.is_empty());
    }

    #[test]
    fn fuel_exhaustion_detected() {
        let config = WasmSandboxConfig {
            fuel_budget: Some(10), // very small budget
            ..Default::default()
        };
        let sandbox = WasmSandbox::new(config).unwrap();
        // An infinite loop should exhaust fuel quickly.
        let result = sandbox.execute_wat(
            r#"(module
                (func (export "_start")
                    (loop $inf
                        (br $inf)
                    )
                )
            )"#,
        );
        assert!(!result.success);
        assert!(result.timed_out, "expected timed_out flag for fuel exhaustion");
    }

    #[test]
    fn custom_timeout_config() {
        let config = WasmSandboxConfig {
            timeout_ms: 1_000,
            ..Default::default()
        };
        assert_eq!(config.effective_fuel(), 1_000_000);
        let sandbox = WasmSandbox::new(config).unwrap();
        assert_eq!(sandbox.config().timeout_ms, 1_000);
    }

    #[test]
    fn network_disabled_by_default() {
        let config = WasmSandboxConfig::default();
        assert!(
            !config.allow_network,
            "network must be denied by default"
        );
    }

    #[test]
    fn explicit_fuel_budget_overrides_derived() {
        let config = WasmSandboxConfig {
            timeout_ms: 10_000,
            fuel_budget: Some(42),
            ..Default::default()
        };
        assert_eq!(config.effective_fuel(), 42);
    }
}
