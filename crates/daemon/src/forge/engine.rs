//! Forge engine — generates, caches, and executes verification tools.
//!
//! The engine pairs the static [`Template`] catalogue with the
//! [`ProcessSandbox`] to produce and run ephemeral verification scripts.
//! Scripts are content-hashed (SHA-256) so repeated invocations with identical
//! source code are served from cache.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use metaygn_sandbox::ProcessSandbox;

use super::templates::get_template;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Language of a generated tool script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptLang {
    Python,
    Bash,
}

impl ScriptLang {
    /// Convert to the string tag expected by [`ProcessSandbox::execute`].
    pub fn as_sandbox_tag(&self) -> &'static str {
        match self {
            ScriptLang::Python => "python",
            ScriptLang::Bash => "bash",
        }
    }
}

/// A fully-resolved verification tool ready for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub language: ScriptLang,
    pub source_code: String,
    pub description: String,
    /// SHA-256 hex digest of `source_code` (used as cache key).
    pub content_hash: String,
}

/// Result of executing a verification tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeResult {
    pub tool_name: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// ForgeEngine
// ---------------------------------------------------------------------------

/// The forge: generate, cache, and execute verification scripts.
pub struct ForgeEngine {
    /// Cache: `content_hash` -> `ToolSpec`.
    cache: HashMap<String, ToolSpec>,
    sandbox: Arc<ProcessSandbox>,
}

impl ForgeEngine {
    /// Create a new engine backed by the given sandbox.
    pub fn new(sandbox: Arc<ProcessSandbox>) -> Self {
        Self {
            cache: HashMap::new(),
            sandbox,
        }
    }

    // -- generation --------------------------------------------------------

    /// Generate a [`ToolSpec`] from a named template, substituting `params`.
    ///
    /// The resulting spec is automatically cached by content hash.
    pub fn generate(
        &mut self,
        template_name: &str,
        params: &HashMap<String, String>,
    ) -> Result<ToolSpec> {
        let tmpl = get_template(template_name)
            .with_context(|| format!("unknown template: {template_name}"))?;

        // Substitute {{param}} placeholders.
        let mut source = tmpl.source.to_string();
        for (key, value) in params {
            let placeholder = format!("{{{{{key}}}}}"); // "{{key}}"
            source = source.replace(&placeholder, value);
        }

        let content_hash = sha256_hex(&source);

        // Return from cache if we already have this exact source.
        if let Some(cached) = self.cache.get(&content_hash) {
            return Ok(cached.clone());
        }

        let spec = ToolSpec {
            name: template_name.to_string(),
            language: tmpl.language,
            source_code: source,
            description: tmpl.description.to_string(),
            content_hash: content_hash.clone(),
        };

        self.cache.insert(content_hash, spec.clone());
        Ok(spec)
    }

    // -- execution ---------------------------------------------------------

    /// Execute a [`ToolSpec`] inside the sandbox.
    ///
    /// `input` is piped to the script via stdin emulation (the input is
    /// embedded into the script itself since the sandbox API does not expose
    /// raw stdin).
    pub async fn execute(&self, spec: &ToolSpec, input: &str) -> Result<ForgeResult> {
        let code = Self::inject_stdin(spec.language, &spec.source_code, input);

        let result = self
            .sandbox
            .execute(spec.language.as_sandbox_tag(), &code)
            .await
            .with_context(|| format!("sandbox execution failed for tool {}", spec.name))?;

        Ok(ForgeResult {
            tool_name: spec.name.clone(),
            success: result.success,
            stdout: result.stdout,
            stderr: result.stderr,
            duration_ms: result.duration_ms,
        })
    }

    /// Generate from a template and immediately execute.
    pub async fn forge_and_run(
        &mut self,
        template_name: &str,
        params: &HashMap<String, String>,
        input: &str,
    ) -> Result<ForgeResult> {
        let spec = self.generate(template_name, params)?;
        self.execute(&spec, input).await
    }

    // -- cache accessors ---------------------------------------------------

    /// Retrieve a cached tool spec by its content hash.
    pub fn get_cached(&self, hash: &str) -> Option<&ToolSpec> {
        self.cache.get(hash)
    }

    /// Number of entries in the cache.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    // -- internal helpers --------------------------------------------------

    /// Inject `input` as emulated stdin into the script source.
    ///
    /// For Python we override `sys.stdin` with a `StringIO` containing the
    /// input data (properly escaped).  For Bash we prepend an `echo` with a
    /// heredoc that pipes into the original script.
    fn inject_stdin(lang: ScriptLang, source: &str, input: &str) -> String {
        if input.is_empty() {
            return source.to_string();
        }

        match lang {
            ScriptLang::Python => {
                // Escape the input for embedding as a Python raw-ish string.
                let escaped = input
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r");

                format!("import sys, io\nsys.stdin = io.StringIO('{escaped}')\n{source}")
            }
            ScriptLang::Bash => {
                // Use a heredoc to feed stdin into the script via a pipe.
                // The delimiter _FORGE_EOF_ is unlikely to appear in user data.
                format!(
                    "cat <<'_FORGE_EOF_' | bash -c {source_quoted}\n{input}\n_FORGE_EOF_",
                    source_quoted = shell_quote(source),
                    input = input,
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Compute the SHA-256 hex digest of a string.
fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Simple single-quote shell escaping for embedding a string in `'...'`.
fn shell_quote(s: &str) -> String {
    // Replace every ' with '\'' (end quote, escaped quote, restart quote).
    let escaped = s.replace('\'', "'\\''");
    format!("'{escaped}'")
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_deterministic() {
        let a = sha256_hex("hello");
        let b = sha256_hex("hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64); // 256 bits = 64 hex chars
    }

    #[test]
    fn sha256_hex_different_inputs() {
        assert_ne!(sha256_hex("a"), sha256_hex("b"));
    }

    #[test]
    fn inject_stdin_python_empty() {
        let code = "print('hi')";
        let result = ForgeEngine::inject_stdin(ScriptLang::Python, code, "");
        assert_eq!(result, code);
    }

    #[test]
    fn inject_stdin_python_with_data() {
        let code = "import sys\nprint(sys.stdin.read())";
        let result = ForgeEngine::inject_stdin(ScriptLang::Python, code, "{\"key\":\"val\"}");
        assert!(result.contains("io.StringIO"));
        assert!(result.contains("import sys"));
    }

    #[test]
    fn shell_quote_basic() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    #[test]
    fn shell_quote_with_single_quotes() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn script_lang_sandbox_tag() {
        assert_eq!(ScriptLang::Python.as_sandbox_tag(), "python");
        assert_eq!(ScriptLang::Bash.as_sandbox_tag(), "bash");
    }
}
