use anyhow::Result;

/// Init command: create .claude/settings.json for MetaYGN project onboarding.
pub fn cmd_init(force: bool) -> Result<()> {
    let config_dir = std::path::Path::new(".claude");
    let settings_path = config_dir.join("settings.json");

    if settings_path.exists() && !force {
        println!("Configuration already exists at .claude/settings.json");
        println!("Use --force to overwrite.");
        return Ok(());
    }

    std::fs::create_dir_all(config_dir)?;

    let settings = serde_json::json!({
        "enabledPlugins": {
            "aletheia-nexus@local": true
        },
        "outputStyle": "aletheia-proof"
    });

    std::fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

    println!("MetaYGN initialized!");
    println!("  Created: .claude/settings.json");
    println!();
    println!("Next steps:");
    println!("  1. Start the daemon:  aletheia start (or cargo run -p metaygn-daemon)");
    println!("  2. Use Claude Code:   claude --plugin-dir /path/to/MetaYGN");
    println!("  3. Check status:      aletheia status");

    Ok(())
}
