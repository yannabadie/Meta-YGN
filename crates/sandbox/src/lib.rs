//! Process-based sandbox for executing code snippets with timeout and resource limits.
//!
//! This crate provides the "shadow sandboxing" feature -- the AI tests hypotheses
//! before presenting results to the user.  The initial backend is process-based;
//! a WASM/wasmtime backend can be added later behind a feature gate.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Configuration for sandbox execution.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum execution time (default: 5 s).
    pub timeout: Duration,
    /// Maximum combined stdout + stderr in bytes (default: 64 KB).
    pub max_output_bytes: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_output_bytes: 64 * 1024,
        }
    }
}

/// Result of sandbox execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
}

/// A hypothesis to test in the sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub description: String,
    /// One of `"python"`, `"node"`, `"bash"`.  `"rust"` is not yet supported.
    pub language: String,
    pub code: String,
    pub expected_success: bool,
}

/// Errors that can occur during sandbox execution.
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("execution timed out after {0:?}")]
    Timeout(Duration),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
}

// ---------------------------------------------------------------------------
// ProcessSandbox
// ---------------------------------------------------------------------------

/// Runs code snippets as sub-processes with timeout and output limits.
pub struct ProcessSandbox {
    config: SandboxConfig,
}

impl ProcessSandbox {
    /// Create a new sandbox with the given configuration.
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Create a sandbox with default settings (5 s timeout, 64 KB output).
    pub fn with_defaults() -> Self {
        Self::new(SandboxConfig::default())
    }

    /// Execute a code snippet in a subprocess.
    ///
    /// Supported languages: `"python"`, `"node"`, `"bash"`.
    /// `"rust"` is not yet supported (compilation is too slow for sandboxing).
    pub async fn execute(
        &self,
        language: &str,
        code: &str,
    ) -> Result<SandboxResult, SandboxError> {
        let (program, args) = Self::build_command(language, code)?;

        debug!(language, "sandbox: spawning process");

        let start = Instant::now();

        let mut child = Command::new(&program)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                SandboxError::ExecutionFailed(format!("failed to spawn {program}: {e}"))
            })?;

        // Race the child against the timeout.
        let result = tokio::time::timeout(self.config.timeout, async {
            // Read stdout and stderr concurrently.
            let mut child_stdout = child.stdout.take();
            let mut child_stderr = child.stderr.take();
            let max = self.config.max_output_bytes;

            let stdout_fut = async {
                if let Some(ref mut out) = child_stdout {
                    read_limited(out, max).await
                } else {
                    Vec::new()
                }
            };

            let stderr_fut = async {
                if let Some(ref mut err) = child_stderr {
                    read_limited(err, max).await
                } else {
                    Vec::new()
                }
            };

            let (stdout_buf, stderr_buf) = tokio::join!(stdout_fut, stderr_fut);

            let status = child.wait().await.map_err(|e| {
                SandboxError::ExecutionFailed(format!("failed to wait for child: {e}"))
            })?;

            Ok::<_, SandboxError>((status, stdout_buf, stderr_buf))
        })
        .await;

        let elapsed = start.elapsed();
        let duration_ms = elapsed.as_millis() as u64;

        match result {
            Ok(Ok((status, stdout_buf, stderr_buf))) => {
                let stdout = truncate_to_string(&stdout_buf, self.config.max_output_bytes);
                let stderr = truncate_to_string(&stderr_buf, self.config.max_output_bytes);
                let exit_code = status.code();
                let success = status.success();

                debug!(
                    language,
                    exit_code = ?exit_code,
                    duration_ms,
                    "sandbox: execution completed"
                );

                Ok(SandboxResult {
                    success,
                    exit_code,
                    stdout,
                    stderr,
                    duration_ms,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout -- try to kill the child.
                warn!(language, duration_ms, "sandbox: execution timed out, killing child");
                let _ = child.kill().await;

                Ok(SandboxResult {
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    duration_ms,
                    timed_out: true,
                })
            }
        }
    }

    /// Test a hypothesis by executing its code and returning the result.
    ///
    /// The caller can compare `result.success` with `hypothesis.expected_success`
    /// to decide whether the hypothesis was confirmed.
    pub async fn test_hypothesis(&self, hypothesis: &Hypothesis) -> SandboxResult {
        match self.execute(&hypothesis.language, &hypothesis.code).await {
            Ok(result) => result,
            Err(SandboxError::Timeout(d)) => SandboxResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("execution timed out after {d:?}"),
                duration_ms: d.as_millis() as u64,
                timed_out: true,
            },
            Err(SandboxError::UnsupportedLanguage(lang)) => SandboxResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("unsupported language: {lang}"),
                duration_ms: 0,
                timed_out: false,
            },
            Err(SandboxError::ExecutionFailed(msg)) => SandboxResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: msg,
                duration_ms: 0,
                timed_out: false,
            },
        }
    }

    /// Build the command + arguments for a given language.
    fn build_command(language: &str, code: &str) -> Result<(String, Vec<String>), SandboxError> {
        match language {
            "python" => {
                // On Windows, `python3` may not exist; try `python` as fallback.
                let program = if cfg!(windows) {
                    find_python_command()
                } else {
                    "python3".to_string()
                };
                Ok((program, vec!["-c".to_string(), code.to_string()]))
            }
            "node" => Ok(("node".to_string(), vec!["-e".to_string(), code.to_string()])),
            "bash" => {
                let program = if cfg!(windows) {
                    find_bash_command()
                } else {
                    "bash".to_string()
                };
                Ok((program, vec!["-c".to_string(), code.to_string()]))
            }
            "rust" => Err(SandboxError::UnsupportedLanguage(
                "rust (compilation too slow for sandbox)".to_string(),
            )),
            other => Err(SandboxError::UnsupportedLanguage(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read from an async reader up to `max_bytes`, returning the collected buffer.
async fn read_limited<R: tokio::io::AsyncRead + Unpin>(reader: &mut R, max_bytes: usize) -> Vec<u8> {
    let mut buf = vec![0u8; max_bytes + 1];
    let mut total = 0;
    loop {
        let n = reader.read(&mut buf[total..]).await.unwrap_or(0);
        if n == 0 {
            break;
        }
        total += n;
        if total >= max_bytes {
            total = max_bytes;
            break;
        }
    }
    buf.truncate(total);
    buf
}

/// Try to find the Python command on Windows (python3, then python).
fn find_python_command() -> String {
    // On Windows, `python` is the standard command name.
    "python".to_string()
}

/// Find Git Bash on Windows (avoids WSL's bash.exe which may not work).
fn find_bash_command() -> String {
    // Prefer Git Bash over WSL bash.
    let git_bash = r"C:\Program Files\Git\usr\bin\bash.exe";
    if std::path::Path::new(git_bash).exists() {
        return git_bash.to_string();
    }
    let git_bash_x86 = r"C:\Program Files (x86)\Git\usr\bin\bash.exe";
    if std::path::Path::new(git_bash_x86).exists() {
        return git_bash_x86.to_string();
    }
    // Fall back to whatever is on PATH.
    "bash".to_string()
}

/// Convert a byte buffer to a UTF-8 string, truncating to `max_bytes`.
fn truncate_to_string(buf: &[u8], max_bytes: usize) -> String {
    let truncated = if buf.len() > max_bytes {
        &buf[..max_bytes]
    } else {
        buf
    };
    String::from_utf8_lossy(truncated).to_string()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.max_output_bytes, 64 * 1024);
    }

    #[test]
    fn build_command_python() {
        let (prog, args) = ProcessSandbox::build_command("python", "print('hi')").unwrap();
        if cfg!(windows) {
            assert_eq!(prog, "python");
        } else {
            assert_eq!(prog, "python3");
        }
        assert_eq!(args, vec!["-c", "print('hi')"]);
    }

    #[test]
    fn build_command_node() {
        let (prog, args) = ProcessSandbox::build_command("node", "console.log(1)").unwrap();
        assert_eq!(prog, "node");
        assert_eq!(args, vec!["-e", "console.log(1)"]);
    }

    #[test]
    fn build_command_bash() {
        let (prog, args) = ProcessSandbox::build_command("bash", "echo hi").unwrap();
        if cfg!(windows) {
            // On Windows we prefer Git Bash over WSL bash.
            assert!(
                prog.contains("bash"),
                "expected a bash path, got: {prog}"
            );
        } else {
            assert_eq!(prog, "bash");
        }
        assert_eq!(args, vec!["-c", "echo hi"]);
    }

    #[test]
    fn build_command_rust_unsupported() {
        let result = ProcessSandbox::build_command("rust", "fn main() {}");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::UnsupportedLanguage(_)));
    }

    #[test]
    fn build_command_unknown_language() {
        let result = ProcessSandbox::build_command("cobol", "DISPLAY 'HI'");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::UnsupportedLanguage(_)));
    }

    #[test]
    fn truncate_to_string_within_limit() {
        let data = b"hello world";
        let result = truncate_to_string(data, 1024);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn truncate_to_string_exceeds_limit() {
        let data = b"hello world, this is a long string";
        let result = truncate_to_string(data, 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn sandbox_result_serialization() {
        let result = SandboxResult {
            success: true,
            exit_code: Some(0),
            stdout: "hello".to_string(),
            stderr: String::new(),
            duration_ms: 42,
            timed_out: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SandboxResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.success, true);
        assert_eq!(deserialized.exit_code, Some(0));
        assert_eq!(deserialized.stdout, "hello");
    }

    #[test]
    fn hypothesis_serialization() {
        let h = Hypothesis {
            description: "test".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            expected_success: true,
        };
        let json = serde_json::to_string(&h).unwrap();
        let deserialized: Hypothesis = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.description, "test");
        assert_eq!(deserialized.expected_success, true);
    }
}
