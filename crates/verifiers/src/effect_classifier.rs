//! AST-based effect classifier for shell commands.
//!
//! Uses tree-sitter-bash to parse commands and classify them by their
//! side-effects (Read, Write, Delete, Execute, Network, Privilege).
//! This replaces fragile regex-based detection that can be trivially bypassed.
//!
//! Feature-gated behind `ast-guard`.

use std::cell::RefCell;

/// The kind of side-effect a shell command produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Read,
    Write,
    Delete,
    Execute,
    Network,
    Privilege,
    Unknown,
}

/// A classified effect extracted from a parsed shell command.
#[derive(Debug, Clone)]
pub struct CommandEffect {
    /// The kind of effect (e.g. Delete, Network).
    pub kind: EffectKind,
    /// The command word (e.g. "rm", "curl").
    pub command: String,
    /// Whether the command operates recursively (e.g. `rm -r`).
    pub recursive: bool,
    /// Whether the command targets root (`/` or `/*`).
    pub targets_root: bool,
    /// Whether input comes from an untrusted source (e.g. piped from a network command).
    pub tainted: bool,
}

thread_local! {
    static BASH_PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_bash::LANGUAGE.into()).ok();
        parser
    });
}

/// Classify a shell command string by its effects.
///
/// Parses the input with tree-sitter-bash and walks the AST to extract
/// effects. For pipelines, commands receiving input from a network command
/// are marked as tainted.
pub fn classify_command(input: &str) -> Vec<CommandEffect> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    BASH_PARSER.with(|parser| {
        let mut parser = parser.borrow_mut();
        let Some(tree) = parser.parse(trimmed, None) else {
            return vec![CommandEffect {
                kind: EffectKind::Unknown,
                command: trimmed.to_string(),
                recursive: false,
                targets_root: false,
                tainted: false,
            }];
        };

        let root = tree.root_node();
        let source = trimmed.as_bytes();

        let mut effects = Vec::new();
        walk_node(root, source, false, &mut effects);

        // Fallback: if AST walk produced nothing, try simple word lookup
        if effects.is_empty() {
            let first_word = trimmed.split_whitespace().next().unwrap_or(trimmed);
            let kind = classify_command_word(first_word);
            if kind != EffectKind::Unknown {
                effects.push(CommandEffect {
                    kind,
                    command: first_word.to_string(),
                    recursive: false,
                    targets_root: false,
                    tainted: false,
                });
            }
        }

        effects
    })
}

/// Walk a tree-sitter node recursively, collecting effects.
fn walk_node(
    node: tree_sitter::Node,
    source: &[u8],
    tainted: bool,
    effects: &mut Vec<CommandEffect>,
) {
    let kind = node.kind();

    match kind {
        "pipeline" => {
            handle_pipeline(node, source, tainted, effects);
        }
        "list" => {
            // `cmd1 && cmd2` or `cmd1 || cmd2` or `cmd1 ; cmd2`
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_node(child, source, tainted, effects);
                }
            }
        }
        "redirected_statement" => {
            handle_redirected_statement(node, source, tainted, effects);
        }
        "command" => {
            if let Some(effect) = classify_command_node(node, source, tainted) {
                effects.push(effect);
            }
        }
        _ => {
            // Recurse into children for compound constructs
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_node(child, source, tainted, effects);
                }
            }
        }
    }
}

/// Handle a pipeline node: `cmd1 | cmd2 | cmd3`
/// If cmd1 is a network command, mark subsequent commands as tainted.
fn handle_pipeline(
    node: tree_sitter::Node,
    source: &[u8],
    tainted: bool,
    effects: &mut Vec<CommandEffect>,
) {
    // Collect the pipeline stages (skip the `|` operator nodes)
    let mut stages = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            let child_kind = child.kind();
            if child_kind != "|" && child_kind != "|&" {
                stages.push(child);
            }
        }
    }

    // Classify first stage
    let mut network_upstream = false;
    if let Some(&first) = stages.first() {
        let mut first_effects = Vec::new();
        walk_node(first, source, tainted, &mut first_effects);
        for eff in &first_effects {
            if eff.kind == EffectKind::Network {
                network_upstream = true;
            }
        }
        effects.extend(first_effects);
    }

    // Classify remaining stages, tainting if upstream was network
    for stage in stages.iter().skip(1) {
        let stage_tainted = tainted || network_upstream;
        let mut stage_effects = Vec::new();
        walk_node(*stage, source, stage_tainted, &mut stage_effects);
        // Check if this stage is also network (for further tainting)
        for eff in &stage_effects {
            if eff.kind == EffectKind::Network {
                network_upstream = true;
            }
        }
        effects.extend(stage_effects);
    }
}

/// Handle a redirected_statement: `cmd > file` or `cmd >> file`
fn handle_redirected_statement(
    node: tree_sitter::Node,
    source: &[u8],
    tainted: bool,
    effects: &mut Vec<CommandEffect>,
) {
    // First, process the inner command/body
    let mut has_output_redirect = false;
    let mut inner_command_node = None;

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "command" | "pipeline" | "list" | "subshell" => {
                    inner_command_node = Some(child);
                }
                "file_redirect" | "heredoc_redirect" => {
                    // Check if this is an output redirect (>, >>)
                    if is_output_redirect(child, source) {
                        has_output_redirect = true;
                    }
                }
                _ => {}
            }
        }
    }

    // Walk the inner command
    if let Some(inner) = inner_command_node {
        walk_node(inner, source, tainted, effects);
    }

    // If there's an output redirect, add a Write effect
    if has_output_redirect {
        // Try to extract the command name from the inner command
        let cmd_name = inner_command_node
            .and_then(|n| extract_command_name(n, source))
            .unwrap_or_else(|| "redirect".to_string());
        effects.push(CommandEffect {
            kind: EffectKind::Write,
            command: cmd_name,
            recursive: false,
            targets_root: false,
            tainted,
        });
    }
}

/// Check if a file_redirect node is an output redirect (>, >>)
fn is_output_redirect(node: tree_sitter::Node, source: &[u8]) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            let text = node_text(child, source);
            if text == ">" || text == ">>" || text == "&>" || text == "&>>" {
                return true;
            }
        }
    }
    // Also check the node text itself for cases where the operator
    // is directly part of the redirect
    let full = node_text(node, source);
    full.contains('>') && !full.starts_with('<')
}

/// Extract the command name from a command node.
fn extract_command_name(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    if node.kind() == "command" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i)
                && child.kind() == "command_name"
            {
                return Some(node_text(child, source).to_string());
            }
        }
    }
    None
}

/// Classify a single `command` AST node into a CommandEffect.
fn classify_command_node(
    node: tree_sitter::Node,
    source: &[u8],
    tainted: bool,
) -> Option<CommandEffect> {
    // Extract command_name child
    let mut cmd_name = None;
    let mut args: Vec<String> = Vec::new();

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "command_name" => {
                    cmd_name = Some(node_text(child, source).to_string());
                }
                "word" | "string" | "raw_string" | "concatenation"
                | "simple_expansion" | "expansion" | "number" => {
                    args.push(node_text(child, source).to_string());
                }
                _ => {}
            }
        }
    }

    let cmd = cmd_name?;
    let kind = classify_command_word(&cmd);

    let mut recursive = false;
    let mut targets_root = false;

    // Special flag detection
    match cmd.as_str() {
        "rm" | "rmdir" => {
            for arg in &args {
                if arg.starts_with('-') && arg.contains('r') {
                    recursive = true;
                }
                if is_root_target(arg) {
                    targets_root = true;
                }
            }
        }
        "find" => {
            let mut reclassified_kind = kind;
            for arg in &args {
                if arg == "-delete" {
                    reclassified_kind = EffectKind::Delete;
                }
                if arg == "-exec" || arg == "-execdir" {
                    reclassified_kind = EffectKind::Delete;
                }
                if is_root_target(arg) {
                    targets_root = true;
                }
            }
            return Some(CommandEffect {
                kind: reclassified_kind,
                command: cmd,
                recursive,
                targets_root,
                tainted,
            });
        }
        "chmod" | "chown" | "chgrp" => {
            for arg in &args {
                if is_root_target(arg) {
                    targets_root = true;
                }
            }
        }
        _ => {}
    }

    Some(CommandEffect {
        kind,
        command: cmd,
        recursive,
        targets_root,
        tainted,
    })
}

/// Classify a command word into an EffectKind.
fn classify_command_word(cmd: &str) -> EffectKind {
    match cmd {
        // Delete
        "rm" | "rmdir" | "unlink" | "shred" => EffectKind::Delete,

        // Write
        "mv" | "cp" | "tee" | "dd" | "mkfs" | "mkdir" | "touch" => EffectKind::Write,

        // Read
        "cat" | "less" | "more" | "head" | "tail" | "ls" | "pwd" | "date" | "whoami" | "grep"
        | "find" | "diff" | "wc" | "sort" | "uniq" | "file" | "stat" | "du" | "df"
        | "readlink" | "realpath" | "basename" | "dirname" | "id" | "env" | "printenv"
        | "uname" | "hostname" | "uptime" | "free" | "lsof" | "ps" | "top" | "htop" => {
            EffectKind::Read
        }

        // Network
        "curl" | "wget" | "ssh" | "scp" | "rsync" | "nc" | "ncat" | "ping" | "dig" | "nslookup"
        | "traceroute" | "telnet" | "ftp" | "sftp" => EffectKind::Network,

        // Privilege
        "sudo" | "su" | "chmod" | "chown" | "chgrp" => EffectKind::Privilege,

        // Execute
        "bash" | "sh" | "eval" | "exec" | "source" | "python" | "python3" | "node" | "perl"
        | "ruby" | "php" => EffectKind::Execute,

        _ => EffectKind::Unknown,
    }
}

/// Check if a path targets root.
fn is_root_target(arg: &str) -> bool {
    arg == "/" || arg == "/*" || arg == "/."
}

/// Extract text content of a tree-sitter node.
fn node_text<'a>(node: tree_sitter::Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_known_commands() {
        assert_eq!(classify_command_word("rm"), EffectKind::Delete);
        assert_eq!(classify_command_word("curl"), EffectKind::Network);
        assert_eq!(classify_command_word("sudo"), EffectKind::Privilege);
        assert_eq!(classify_command_word("cat"), EffectKind::Read);
        assert_eq!(classify_command_word("bash"), EffectKind::Execute);
        assert_eq!(classify_command_word("dd"), EffectKind::Write);
        assert_eq!(classify_command_word("unknown_cmd"), EffectKind::Unknown);
    }
}
