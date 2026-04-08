use anyhow::Result;
use serde_json::Value;
use std::process::Command;

use crate::util::{http_client, read_daemon_port};

/// Doctor command: check MetaYGN installation health.
pub async fn cmd_doctor() -> Result<()> {
    println!("Aletheia Doctor\n");
    let mut issues = 0;

    // 1. Check daemon
    if let Some(port) = read_daemon_port() {
        let client = http_client()?;
        match client
            .get(format!("http://127.0.0.1:{port}/health"))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let body: Value = resp.json().await.unwrap_or_default();
                let version = body.get("version").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  Daemon:     RUNNING (port {port}, version {version})");
            }
            _ => {
                println!("  Daemon:     NOT RESPONDING (port file exists but daemon unreachable)");
                issues += 1;
            }
        }
    } else {
        println!("  Daemon:     STOPPED");
        issues += 1;
    }

    // 2. Check plugin.json
    let plugin_path = std::path::Path::new(".claude-plugin/plugin.json");
    if plugin_path.exists() {
        if let Ok(content) = std::fs::read_to_string(plugin_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  Plugin:     VALID (version {version})");
            } else {
                println!("  Plugin:     INVALID JSON");
                issues += 1;
            }
        }
    } else {
        println!("  Plugin:     NOT FOUND (.claude-plugin/plugin.json missing)");
        issues += 1;
    }

    // 3. Check hooks
    let hooks_path = std::path::Path::new("hooks/hooks.json");
    if hooks_path.exists()
        && let Ok(content) = std::fs::read_to_string(hooks_path)
        && let Ok(json) = serde_json::from_str::<Value>(&content)
    {
        let count = json
            .get("hooks")
            .and_then(|v| v.as_object())
            .map(|obj| obj.len())
            .unwrap_or(0);
        println!("  Hooks:      {count}/8 configured");
        if count == 0 {
            issues += 1;
        }
    } else {
        println!("  Hooks:      NOT FOUND");
        issues += 1;
    }

    // 4. Check skills
    let skills_dir = std::path::Path::new("skills");
    if skills_dir.exists() {
        let count = std::fs::read_dir(skills_dir)
            .map(|d| {
                d.filter(|e| e.as_ref().map(|e| e.path().is_dir()).unwrap_or(false))
                    .count()
            })
            .unwrap_or(0);
        println!("  Skills:     {count}/9 present");
    } else {
        println!("  Skills:     NOT FOUND");
    }

    // 5. Check Codex compatibility assets
    let codex_assets_ready = std::path::Path::new("AGENTS.md").exists()
        && std::path::Path::new("docs/codex-bootstrap-prompt.txt").exists()
        && (std::path::Path::new("scripts/install-codex.ps1").exists()
            || std::path::Path::new("scripts/install-codex.sh").exists());
    if codex_assets_ready {
        println!("  Codex:      ASSETS READY (AGENTS + bootstrap + install script)");
    } else {
        println!("  Codex:      ASSETS INCOMPLETE");
        issues += 1;
    }

    match Command::new("codex").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let configured = Command::new("codex")
                .args(["mcp", "get", "aletheia"])
                .output()
                .map(|out| out.status.success())
                .unwrap_or(false);
            if configured {
                println!("  Codex MCP:  REGISTERED (server 'aletheia')");
            } else {
                println!("  Codex MCP:  NOT REGISTERED (run scripts/install-codex.*)");
                issues += 1;
            }
        }
        _ => {
            println!("  Codex MCP:  CLI NOT FOUND (optional)");
        }
    }

    // 6. Check agents
    let agents_dir = std::path::Path::new("agents");
    if agents_dir.exists() {
        let count = std::fs::read_dir(agents_dir)
            .map(|d| {
                d.filter(|e| {
                    e.as_ref()
                        .map(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                        .unwrap_or(false)
                })
                .count()
            })
            .unwrap_or(0);
        println!("  Agents:     {count}/6 present");
    } else {
        println!("  Agents:     NOT FOUND");
    }

    // 7. Check database
    let home = dirs::home_dir().unwrap_or_default();
    let db_path = home.join(".claude").join("aletheia").join("metaygn.db");
    if db_path.exists() {
        let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
        println!("  Database:   {} ({} KB)", db_path.display(), size / 1024);
    } else {
        println!("  Database:   NOT FOUND (start daemon first)");
    }

    println!();
    if issues == 0 {
        println!("  Status: ALL CHECKS PASSED");
    } else {
        println!("  Status: {} ISSUE(S) FOUND", issues);
    }

    Ok(())
}
