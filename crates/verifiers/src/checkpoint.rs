use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The kind of checkpoint that was created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointType {
    /// A `git stash create` ref (working-tree untouched).
    GitStash,
    /// A `git rev-parse HEAD` ref (nothing to stash).
    GitCommitRef,
    /// Files copied to a timestamped directory.
    FileCopy,
    /// Nothing to checkpoint.
    Skipped,
}

/// Outcome of a checkpoint operation.
#[derive(Debug, Clone)]
pub struct CheckpointResult {
    pub checkpoint_type: CheckpointType,
    pub created: bool,
    /// Stash SHA, HEAD commit, or checkpoint directory path.
    pub location: String,
    pub files_saved: usize,
    /// Human-readable recovery instruction.
    pub message: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of files we will checkpoint in a single operation.
const MAX_FILES: usize = 100;

/// Maximum total bytes we will copy (50 MB).
const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;

/// Sub-directory under the project root where checkpoint artefacts live.
const CHECKPOINT_DIR: &str = ".claude/aletheia";

/// Log file name inside `CHECKPOINT_DIR`.
const LOG_FILE: &str = "checkpoints.log";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a git checkpoint (stash or commit ref).
///
/// Runs `git stash create` first.  If the stash is empty (nothing to stash),
/// falls back to `git rev-parse HEAD` so we always record the current state.
///
/// The result is appended to the checkpoint log at
/// `<cwd>/.claude/aletheia/checkpoints.log`.
pub fn git_checkpoint(cwd: &str) -> CheckpointResult {
    // Try `git stash create` — this is a plumbing command that does NOT modify
    // the working tree.
    let stash = Command::new("git")
        .args(["stash", "create"])
        .current_dir(cwd)
        .output();

    match stash {
        Ok(output) if output.status.success() => {
            let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !sha.is_empty() {
                append_log(cwd, "git-stash", &sha, "git stash create");
                return CheckpointResult {
                    checkpoint_type: CheckpointType::GitStash,
                    created: true,
                    location: sha.clone(),
                    files_saved: 0,
                    message: format!(
                        "To restore working-tree changes: git stash apply {sha}"
                    ),
                };
            }
            // Empty output — nothing to stash; fall through to HEAD.
        }
        Ok(_) | Err(_) => {
            // git stash create failed (maybe not a git repo?); fall through.
        }
    }

    // Fall back to recording current HEAD.
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(cwd)
        .output();

    match head {
        Ok(output) if output.status.success() => {
            let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !sha.is_empty() {
                append_log(cwd, "git-commit-ref", &sha, "git rev-parse HEAD");
                return CheckpointResult {
                    checkpoint_type: CheckpointType::GitCommitRef,
                    created: true,
                    location: sha.clone(),
                    files_saved: 0,
                    message: format!(
                        "To recover to this commit: git checkout {sha}"
                    ),
                };
            }
        }
        _ => {}
    }

    CheckpointResult {
        checkpoint_type: CheckpointType::Skipped,
        created: false,
        location: String::new(),
        files_saved: 0,
        message: "No git checkpoint created (not a git repository or git unavailable).".into(),
    }
}

/// Copy files that would be affected by a destructive command.
///
/// `files` are paths relative to `cwd`.  Each file is copied into a
/// timestamped sub-directory under `<cwd>/.claude/aletheia/checkpoints/`.
///
/// Limits: at most [`MAX_FILES`] files and [`MAX_TOTAL_BYTES`] total.
pub fn file_checkpoint(cwd: &str, files: &[&str]) -> CheckpointResult {
    let cwd_path = PathBuf::from(cwd);
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
    let dest_dir = cwd_path
        .join(CHECKPOINT_DIR)
        .join("checkpoints")
        .join(&timestamp);

    let mut saved: usize = 0;
    let mut total_bytes: u64 = 0;

    for rel in files.iter().take(MAX_FILES) {
        let src = cwd_path.join(rel);
        if !src.exists() || !src.is_file() {
            continue;
        }

        // Ensure the source is under the project directory.
        match (src.canonicalize(), cwd_path.canonicalize()) {
            (Ok(abs_src), Ok(abs_cwd)) if abs_src.starts_with(&abs_cwd) => {}
            _ => continue,
        }

        let meta = match fs::metadata(&src) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if total_bytes + meta.len() > MAX_TOTAL_BYTES {
            break;
        }

        // Mirror the relative path inside the checkpoint directory.
        let target = dest_dir.join(rel);
        if let Some(parent) = target.parent() {
            if fs::create_dir_all(parent).is_err() {
                continue;
            }
        }

        if fs::copy(&src, &target).is_ok() {
            saved += 1;
            total_bytes += meta.len();
        }
    }

    if saved == 0 {
        return CheckpointResult {
            checkpoint_type: CheckpointType::Skipped,
            created: false,
            location: String::new(),
            files_saved: 0,
            message: "No files to checkpoint (none exist or all skipped).".into(),
        };
    }

    let location = dest_dir.to_string_lossy().to_string();
    append_log(cwd, "file-copy", &location, &format!("{saved} files"));

    CheckpointResult {
        checkpoint_type: CheckpointType::FileCopy,
        created: true,
        location: location.clone(),
        files_saved: saved,
        message: format!(
            "To recover files: copy from {location}"
        ),
    }
}

/// Parse a shell command and extract file paths that would be destroyed.
///
/// Recognised commands: `rm`, `unlink`, `find … -delete`.
/// Flags (tokens starting with `-`) are ignored.
pub fn extract_target_files(command: &str) -> Vec<String> {
    let tokens: Vec<&str> = shell_split(command);
    if tokens.is_empty() {
        return Vec::new();
    }

    let bin = base_command(tokens[0]);

    match bin {
        "rm" => extract_rm_targets(&tokens[1..]),
        "unlink" => extract_non_flag_args(&tokens[1..]),
        "find" => extract_find_targets(&tokens),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Helpers — command parsing
// ---------------------------------------------------------------------------

/// Minimal shell-style tokeniser.  Handles single and double quotes so that
/// paths with spaces survive, but does NOT handle backslash escapes or
/// variable expansion — good enough for the patterns we need to match.
fn shell_split(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            i += 1; // skip opening quote
            let start = i;
            while i < len && bytes[i] != quote {
                i += 1;
            }
            tokens.push(&input[start..i]);
            if i < len {
                i += 1; // skip closing quote
            }
        } else {
            let start = i;
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(&input[start..i]);
        }
    }

    tokens
}

/// Return the base binary name (strip any leading path).
fn base_command(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

/// Extract non-flag arguments (anything not starting with `-`).
fn extract_non_flag_args(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .filter(|t| !t.starts_with('-'))
        .map(|t| (*t).to_string())
        .collect()
}

/// Extract `rm` targets: skip flags (short `-f`, `-r`, `-v`, combined `-rf`,
/// and long `--verbose`, `--force`, etc.).
fn extract_rm_targets(tokens: &[&str]) -> Vec<String> {
    extract_non_flag_args(tokens)
}

/// For `find <path> … -delete`, extract the search path (first non-flag
/// argument after `find`).
fn extract_find_targets(tokens: &[&str]) -> Vec<String> {
    // Only recognise find as destructive if it contains `-delete` or
    // `-exec rm`.
    let joined = tokens.join(" ");
    let is_destructive = joined.contains("-delete") || joined.contains("-exec rm");
    if !is_destructive {
        return Vec::new();
    }

    // The search path is usually the first argument after `find` that does
    // not start with `-`.
    tokens
        .iter()
        .skip(1) // skip "find" itself
        .take_while(|t| !t.starts_with('-'))
        .map(|t| (*t).to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Helpers — logging
// ---------------------------------------------------------------------------

/// Append a line to the checkpoint log (best-effort, never fatal).
fn append_log(cwd: &str, kind: &str, location: &str, command: &str) {
    let log_dir = Path::new(cwd).join(CHECKPOINT_DIR);
    if fs::create_dir_all(&log_dir).is_err() {
        return;
    }
    let log_path = log_dir.join(LOG_FILE);
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let line = format!("{timestamp} {kind} {location} {command}\n");

    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

// ---------------------------------------------------------------------------
// Unit tests (in-module, fast, no I/O)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_split_basic() {
        assert_eq!(shell_split("rm -rf foo"), vec!["rm", "-rf", "foo"]);
    }

    #[test]
    fn shell_split_quoted() {
        assert_eq!(
            shell_split("rm 'my file.txt'"),
            vec!["rm", "my file.txt"]
        );
    }

    #[test]
    fn base_command_strips_path() {
        assert_eq!(base_command("/usr/bin/rm"), "rm");
        assert_eq!(base_command("rm"), "rm");
    }
}
