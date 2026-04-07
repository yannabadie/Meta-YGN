use metaygn_core::context::LoopContext;
use metaygn_core::stages::assess::AssessStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::state::RiskLevel;

/// Helper: build a `HookInput` with only the prompt set.
fn input_with_prompt(prompt: &str) -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        prompt: Some(prompt.to_string()),
        ..Default::default()
    }
}

/// Helper: build a `HookInput` with prompt and tool_name set.
fn input_with_prompt_and_tool(prompt: &str, tool_name: &str) -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        prompt: Some(prompt.to_string()),
        tool_name: Some(tool_name.to_string()),
        ..Default::default()
    }
}

/// Helper: build a `HookInput` with only tool_name set (no prompt).
fn input_with_tool_only(tool_name: &str) -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        tool_name: Some(tool_name.to_string()),
        ..Default::default()
    }
}

/// Helper: run the assess stage and return (risk, difficulty).
fn assess(input: HookInput) -> (RiskLevel, f32) {
    let stage = AssessStage;
    let mut ctx = LoopContext::new(input);
    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue, "assess stage should always return Continue");
    (ctx.risk, ctx.difficulty)
}

/// Helper: run the assess stage and return only the risk level.
fn assess_risk(input: HookInput) -> RiskLevel {
    assess(input).0
}

/// Helper: run the assess stage and return only the difficulty score.
fn assess_difficulty(input: HookInput) -> f32 {
    assess(input).1
}

// ─── Stage metadata ───────────────────────────────────────────────────────────

#[test]
fn stage_name_is_assess() {
    let stage = AssessStage;
    assert_eq!(stage.name(), "assess");
}

#[test]
fn stage_always_returns_continue() {
    let input = input_with_prompt("anything at all");
    let stage = AssessStage;
    let mut ctx = LoopContext::new(input);
    assert_eq!(stage.run(&mut ctx), StageResult::Continue);
}

// ─── High-risk: destructive command keywords ─────────────────────────────────

#[test]
fn high_risk_keyword_delete() {
    assert_eq!(assess_risk(input_with_prompt("delete the database")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_rm_space() {
    // "rm " (with trailing space) is a high-risk pattern
    assert_eq!(assess_risk(input_with_prompt("rm -rf /tmp/files")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_rm_tab() {
    // "rm\t" (with trailing tab) is a high-risk pattern
    assert_eq!(assess_risk(input_with_prompt("rm\t-rf /tmp")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_drop() {
    assert_eq!(assess_risk(input_with_prompt("drop table users")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_force() {
    assert_eq!(assess_risk(input_with_prompt("force push to main")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_deploy() {
    assert_eq!(assess_risk(input_with_prompt("deploy to production")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_push() {
    assert_eq!(assess_risk(input_with_prompt("push to remote")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_credential() {
    assert_eq!(assess_risk(input_with_prompt("update the credential store")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_secret() {
    assert_eq!(assess_risk(input_with_prompt("rotate the api secret")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_vulnerability() {
    assert_eq!(assess_risk(input_with_prompt("found a vulnerability")), RiskLevel::High);
}

#[test]
fn high_risk_keyword_exploit() {
    assert_eq!(assess_risk(input_with_prompt("test the exploit path")), RiskLevel::High);
}

// ─── High-risk: keywords matched via tool_name ───────────────────────────────

#[test]
fn high_risk_keyword_in_tool_name() {
    // "delete" in tool_name is matched via combined = "{tool} {prompt}".to_lowercase()
    assert_eq!(assess_risk(input_with_prompt_and_tool("do something", "delete_file")), RiskLevel::High);
}

#[test]
fn high_risk_deploy_in_tool_name() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("run it", "deploy_service")), RiskLevel::High);
}

// ─── High-risk: bash tool with no prompt ─────────────────────────────────────

#[test]
fn bash_tool_no_prompt_is_high_risk() {
    // Bash tool with no command (empty prompt) defaults to High risk
    let input = input_with_tool_only("Bash");
    assert_eq!(assess_risk(input), RiskLevel::High);
}

#[test]
fn bash_tool_no_prompt_field_is_high_risk() {
    // Bash tool with prompt = None
    let input = HookInput {
        hook_event_name: HookEvent::PreToolUse,
        tool_name: Some("bash".into()),
        ..Default::default()
    };
    assert_eq!(assess_risk(input), RiskLevel::High);
}

// ─── Low-risk: safe bash commands ────────────────────────────────────────────

#[test]
fn low_risk_bash_ls() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("ls -la", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_cat() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("cat /tmp/file.txt", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_head() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("head -n 10 file.rs", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_tail() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("tail -f logs.txt", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_echo() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("echo hello world", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_pwd() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("pwd", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_date() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("date", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_whoami() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("whoami", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_find() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("find . -name '*.rs'", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_grep() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("grep -r 'TODO' src/", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_rg() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("rg pattern src/", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_which() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("which cargo", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_env() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("env", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_printenv() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("printenv HOME", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_cargo_test() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("cargo test --all", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_cargo_check() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("cargo check", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_cargo_build() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("cargo build --release", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_cargo_clippy() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("cargo clippy", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_cargo_fmt() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("cargo fmt --check", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_npm_test() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("npm test", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_npm_run() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("npm run lint", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_git_status() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("git status", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_git_log() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("git log --oneline", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_git_diff() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("git diff HEAD", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_git_branch() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("git branch -a", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_wc() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("wc -l src/main.rs", "Bash")), RiskLevel::Low);
}

#[test]
fn low_risk_bash_uname() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("uname -a", "Bash")), RiskLevel::Low);
}

// ─── Medium-risk: bash with unknown commands ─────────────────────────────────

#[test]
fn medium_risk_bash_unknown_command() {
    // Bash tool with a command that doesn't match any safe prefix => Medium
    assert_eq!(assess_risk(input_with_prompt_and_tool("curl https://example.com", "Bash")), RiskLevel::Medium);
}

#[test]
fn medium_risk_bash_sed() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("sed -i 's/foo/bar/' file.txt", "Bash")), RiskLevel::Medium);
}

#[test]
fn medium_risk_bash_chmod() {
    assert_eq!(assess_risk(input_with_prompt_and_tool("chmod 755 script.sh", "Bash")), RiskLevel::Medium);
}

// ─── Medium-risk: Write tool ─────────────────────────────────────────────────

#[test]
fn medium_risk_write_tool() {
    let input = input_with_prompt_and_tool("write the file", "Write");
    assert_eq!(assess_risk(input), RiskLevel::Medium);
}

#[test]
fn medium_risk_write_tool_case_insensitive() {
    let input = input_with_prompt_and_tool("create a file", "write");
    assert_eq!(assess_risk(input), RiskLevel::Medium);
}

// ─── Medium-risk: medium keywords ────────────────────────────────────────────

#[test]
fn medium_risk_keyword_edit() {
    assert_eq!(assess_risk(input_with_prompt("edit the config")), RiskLevel::Medium);
}

#[test]
fn medium_risk_keyword_replace() {
    assert_eq!(assess_risk(input_with_prompt("replace the string")), RiskLevel::Medium);
}

#[test]
fn medium_risk_keyword_modify() {
    assert_eq!(assess_risk(input_with_prompt("modify the settings")), RiskLevel::Medium);
}

#[test]
fn medium_risk_keyword_update() {
    assert_eq!(assess_risk(input_with_prompt("update the dependency")), RiskLevel::Medium);
}

#[test]
fn medium_risk_keyword_install() {
    assert_eq!(assess_risk(input_with_prompt("install the package")), RiskLevel::Medium);
}

#[test]
fn medium_risk_keyword_create() {
    assert_eq!(assess_risk(input_with_prompt("create a new module")), RiskLevel::Medium);
}

// ─── Low-risk: no risk indicators ────────────────────────────────────────────

#[test]
fn low_risk_benign_prompt_no_tool() {
    assert_eq!(assess_risk(input_with_prompt("explain how iterators work")), RiskLevel::Low);
}

#[test]
fn low_risk_read_tool() {
    let input = input_with_prompt_and_tool("read the file", "Read");
    assert_eq!(assess_risk(input), RiskLevel::Low);
}

#[test]
fn low_risk_glob_tool() {
    let input = input_with_prompt_and_tool("find files", "Glob");
    assert_eq!(assess_risk(input), RiskLevel::Low);
}

#[test]
fn low_risk_grep_tool() {
    let input = input_with_prompt_and_tool("search for pattern", "Grep");
    assert_eq!(assess_risk(input), RiskLevel::Low);
}

// ─── High-risk takes priority over safe bash ─────────────────────────────────

#[test]
fn high_risk_keyword_overrides_safe_bash_prefix() {
    // "git push" starts with "git" but "push" is a high-risk keyword in the combined text
    // combined = "Bash git push origin main".to_lowercase() contains "push"
    let input = input_with_prompt_and_tool("git push origin main", "Bash");
    assert_eq!(assess_risk(input), RiskLevel::High);
}

#[test]
fn high_risk_delete_in_bash_safe_command() {
    // Even though "grep" is a safe prefix, "delete" in the prompt triggers high risk
    // because high-risk keywords are checked on the combined text BEFORE bash safe check
    let input = input_with_prompt_and_tool("grep delete_user src/", "Bash");
    assert_eq!(assess_risk(input), RiskLevel::High);
}

// ─── Difficulty: prompt length ───────────────────────────────────────────────

#[test]
fn difficulty_zero_for_empty_prompt() {
    let d = assess_difficulty(input_with_prompt(""));
    assert!(d < 0.01, "empty prompt should have near-zero difficulty, got {d}");
}

#[test]
fn difficulty_low_for_short_prompt() {
    let d = assess_difficulty(input_with_prompt("hello world"));
    assert!(d < 0.1, "short prompt should have low difficulty, got {d}");
}

#[test]
fn difficulty_increases_with_length() {
    let short = assess_difficulty(input_with_prompt("fix the bug"));
    let long_text = "word ".repeat(100);
    let long = assess_difficulty(input_with_prompt(&long_text));
    assert!(long > short, "longer prompt should have higher difficulty: short={short}, long={long}");
}

#[test]
fn difficulty_base_capped_at_06() {
    // 300 words >> 200-word cap, so base = min(300/200, 0.6) = 0.6
    let very_long = "word ".repeat(300);
    let d = assess_difficulty(input_with_prompt(&very_long));
    // Without any complexity keywords, difficulty should be exactly 0.6
    assert!((d - 0.6).abs() < 0.01, "base difficulty capped at 0.6, got {d}");
}

// ─── Difficulty: complexity keywords ─────────────────────────────────────────

#[test]
fn difficulty_increases_with_complexity_keywords() {
    let without = assess_difficulty(input_with_prompt("implement a simple feature"));
    let with_one = assess_difficulty(input_with_prompt("implement an async feature"));
    assert!(with_one > without, "complexity keyword should increase difficulty: without={without}, with={with_one}");
}

#[test]
fn difficulty_keyword_complex() {
    let d = assess_difficulty(input_with_prompt("complex logic here"));
    assert!(d >= 0.1, "should detect 'complex' keyword, got {d}");
}

#[test]
fn difficulty_keyword_concurrent() {
    // "concurren" is the keyword (prefix match for concurrent/concurrency)
    let d = assess_difficulty(input_with_prompt("concurrent access pattern"));
    assert!(d >= 0.1, "should detect 'concurren' keyword, got {d}");
}

#[test]
fn difficulty_keyword_async() {
    let d = assess_difficulty(input_with_prompt("async runtime setup"));
    assert!(d >= 0.1, "should detect 'async' keyword, got {d}");
}

#[test]
fn difficulty_keyword_parallel() {
    let d = assess_difficulty(input_with_prompt("parallel processing pipeline"));
    assert!(d >= 0.1, "should detect 'parallel' keyword, got {d}");
}

#[test]
fn difficulty_keyword_distributed() {
    let d = assess_difficulty(input_with_prompt("distributed system design"));
    assert!(d >= 0.1, "should detect 'distributed' keyword, got {d}");
}

#[test]
fn difficulty_keyword_performance() {
    let d = assess_difficulty(input_with_prompt("performance optimization needed"));
    assert!(d >= 0.1, "should detect 'performance' keyword, got {d}");
}

#[test]
fn difficulty_keyword_optimize() {
    let d = assess_difficulty(input_with_prompt("optimize the query"));
    assert!(d >= 0.1, "should detect 'optimize' keyword, got {d}");
}

#[test]
fn difficulty_keyword_scale() {
    let d = assess_difficulty(input_with_prompt("scale the service horizontally"));
    assert!(d >= 0.1, "should detect 'scale' keyword, got {d}");
}

#[test]
fn difficulty_keyword_migration() {
    let d = assess_difficulty(input_with_prompt("migration of the database"));
    assert!(d >= 0.1, "should detect 'migration' keyword, got {d}");
}

#[test]
fn difficulty_keyword_backward() {
    let d = assess_difficulty(input_with_prompt("backward compatible change"));
    assert!(d >= 0.1, "should detect 'backward' keyword, got {d}");
}

#[test]
fn difficulty_keyword_compatibility() {
    let d = assess_difficulty(input_with_prompt("ensure compatibility with v1"));
    assert!(d >= 0.1, "should detect 'compatibility' keyword, got {d}");
}

#[test]
fn difficulty_keyword_recursive() {
    let d = assess_difficulty(input_with_prompt("recursive tree traversal"));
    assert!(d >= 0.1, "should detect 'recursive' keyword, got {d}");
}

#[test]
fn difficulty_keyword_cryptograph() {
    // "cryptograph" is the keyword (prefix match for cryptography/cryptographic)
    let d = assess_difficulty(input_with_prompt("cryptographic hash function"));
    assert!(d >= 0.1, "should detect 'cryptograph' keyword, got {d}");
}

#[test]
fn difficulty_multiple_keywords_stack() {
    let one = assess_difficulty(input_with_prompt("async task"));
    let three = assess_difficulty(input_with_prompt("async distributed parallel task"));
    assert!(three > one, "multiple keywords should stack: one={one}, three={three}");
}

#[test]
fn difficulty_capped_at_1() {
    // Use a long prompt (high base) plus many complexity keywords to try to exceed 1.0
    let many_keywords = "word ".repeat(200)
        + " complex concurrent async parallel distributed performance optimize scale migration backward compatibility recursive cryptograph";
    let d = assess_difficulty(input_with_prompt(&many_keywords));
    assert!(d <= 1.0, "difficulty should be capped at 1.0, got {d}");
    assert!((d - 1.0).abs() < 0.01, "should be very close to 1.0 with max keywords, got {d}");
}

#[test]
fn difficulty_case_insensitive() {
    let d = assess_difficulty(input_with_prompt("ASYNC PARALLEL Processing"));
    assert!(d >= 0.2, "keyword matching should be case insensitive, got {d}");
}

// ─── Difficulty: uses tool_input too ─────────────────────────────────────────

#[test]
fn difficulty_includes_tool_input() {
    let mut input = input_with_prompt("simple task");
    input.tool_input = Some(serde_json::json!({"command": "complex distributed async system"}));
    let d = assess_difficulty(input);
    let simple_d = assess_difficulty(input_with_prompt("simple task"));
    assert!(d > simple_d, "tool_input keywords should contribute to difficulty: with={d}, without={simple_d}");
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn no_prompt_no_tool_defaults_to_low_risk() {
    let input = HookInput {
        hook_event_name: HookEvent::PreToolUse,
        ..Default::default()
    };
    assert_eq!(assess_risk(input), RiskLevel::Low);
}

#[test]
fn empty_prompt_no_tool_is_low_risk() {
    let input = input_with_prompt("");
    assert_eq!(assess_risk(input), RiskLevel::Low);
}

#[test]
fn default_context_risk_is_low_before_assess() {
    let input = input_with_prompt("anything");
    let ctx = LoopContext::new(input);
    assert_eq!(ctx.risk, RiskLevel::Low, "default risk before assess runs should be Low");
}

#[test]
fn default_context_difficulty_is_05_before_assess() {
    let input = input_with_prompt("anything");
    let ctx = LoopContext::new(input);
    assert!((ctx.difficulty - 0.5).abs() < 0.01, "default difficulty before assess runs should be 0.5");
}

#[test]
fn assess_overwrites_default_difficulty() {
    let input = input_with_prompt("hello");
    let d = assess_difficulty(input);
    // "hello" is 1 word => base = 1/200 = 0.005, no keywords => ~0.005
    // This is different from the default 0.5
    assert!(d < 0.1, "assess should overwrite default difficulty with computed value, got {d}");
}

// ─── Case sensitivity of tool_name matching ──────────────────────────────────

#[test]
fn bash_tool_case_insensitive() {
    // tool_lower = tool.to_lowercase(), so "BASH", "Bash", "bash" all match
    let upper = input_with_prompt_and_tool("ls -la", "BASH");
    assert_eq!(assess_risk(upper), RiskLevel::Low);

    let mixed = input_with_prompt_and_tool("ls -la", "Bash");
    assert_eq!(assess_risk(mixed), RiskLevel::Low);

    let lower = input_with_prompt_and_tool("ls -la", "bash");
    assert_eq!(assess_risk(lower), RiskLevel::Low);
}

#[test]
fn write_tool_case_insensitive() {
    let upper = input_with_prompt_and_tool("some content", "WRITE");
    assert_eq!(assess_risk(upper), RiskLevel::Medium);

    let lower = input_with_prompt_and_tool("some content", "write");
    assert_eq!(assess_risk(lower), RiskLevel::Medium);
}

// ─── Safe bash prefix matching is starts_with ────────────────────────────────

#[test]
fn safe_bash_requires_starts_with() {
    // "my_ls command" does NOT start with "ls", so it's not safe => Medium
    let input = input_with_prompt_and_tool("my_ls command", "Bash");
    assert_eq!(assess_risk(input), RiskLevel::Medium);
}

#[test]
fn safe_bash_leading_whitespace_is_trimmed() {
    // cmd_trimmed = cmd_lower.trim(), so leading spaces are handled
    let input = input_with_prompt_and_tool("  ls -la", "Bash");
    assert_eq!(assess_risk(input), RiskLevel::Low);
}
