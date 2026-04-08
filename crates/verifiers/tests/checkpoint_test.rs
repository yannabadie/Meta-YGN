use metaygn_verifiers::checkpoint::{
    extract_target_files, file_checkpoint, git_checkpoint, CheckpointType,
};

// ---------------------------------------------------------------------------
// extract_target_files
// ---------------------------------------------------------------------------

#[test]
fn extract_target_files_rm() {
    let files = extract_target_files("rm foo.txt bar.txt");
    assert_eq!(files, vec!["foo.txt", "bar.txt"]);
}

#[test]
fn extract_target_files_rm_rf() {
    let files = extract_target_files("rm -rf src/old_module/");
    assert_eq!(files, vec!["src/old_module/"]);
}

#[test]
fn extract_target_files_unlink() {
    let files = extract_target_files("unlink important.dat");
    assert_eq!(files, vec!["important.dat"]);
}

#[test]
fn extract_target_files_ignores_flags() {
    let files = extract_target_files("rm -f -v --verbose file.txt");
    assert_eq!(files, vec!["file.txt"]);
}

#[test]
fn extract_target_files_find_delete() {
    let files = extract_target_files("find /tmp -name '*.log' -delete");
    // For find, we extract the search path as the target.
    assert_eq!(files, vec!["/tmp"]);
}

#[test]
fn extract_target_files_non_destructive_returns_empty() {
    let files = extract_target_files("ls -la");
    assert!(files.is_empty());
}

// ---------------------------------------------------------------------------
// git_checkpoint
// ---------------------------------------------------------------------------

#[test]
fn git_checkpoint_returns_result() {
    // Runs inside the actual repo (which is a git repo).
    let result = git_checkpoint(".");
    assert!(result.created);
    assert!(!result.location.is_empty());
    assert!(matches!(
        result.checkpoint_type,
        CheckpointType::GitStash | CheckpointType::GitCommitRef
    ));
}

#[test]
fn checkpoint_result_has_recovery_message() {
    let result = git_checkpoint(".");
    assert!(!result.message.is_empty());
    // Message should contain recovery instructions.
    assert!(
        result.message.contains("recover")
            || result.message.contains("restore")
            || result.message.contains("git stash")
            || result.message.contains("git checkout"),
        "expected recovery instructions, got: {}",
        result.message
    );
}

// ---------------------------------------------------------------------------
// file_checkpoint
// ---------------------------------------------------------------------------

#[test]
fn file_checkpoint_copies_existing_files() {
    let dir = std::env::temp_dir().join("aletheia_test_checkpoint");
    std::fs::create_dir_all(&dir).unwrap();
    let test_file = dir.join("test.txt");
    std::fs::write(&test_file, "important data").unwrap();

    let result = file_checkpoint(dir.to_str().unwrap(), &["test.txt"]);
    assert!(result.created);
    assert!(result.files_saved >= 1);

    // Cleanup.
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn file_checkpoint_skips_nonexistent_files() {
    let result = file_checkpoint(".", &["nonexistent_file_12345.txt"]);
    assert_eq!(result.files_saved, 0);
}

#[test]
fn file_checkpoint_respects_max_files_limit() {
    let dir = std::env::temp_dir().join("aletheia_test_max_files");
    std::fs::create_dir_all(&dir).unwrap();

    // Create 105 tiny files.
    let mut names: Vec<String> = Vec::new();
    for i in 0..105 {
        let name = format!("f{i}.txt");
        std::fs::write(dir.join(&name), "x").unwrap();
        names.push(name);
    }

    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let result = file_checkpoint(dir.to_str().unwrap(), &refs);

    // Should cap at 100 files.
    assert!(result.files_saved <= 100);

    std::fs::remove_dir_all(&dir).ok();
}
