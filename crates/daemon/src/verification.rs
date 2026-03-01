//! In-process file content validation (Tier 1).
//! Validates JSON, YAML, and TOML files without spawning a subprocess.

/// Validate file content based on file extension.
/// Returns Some(error_message) if validation fails, None if valid or unknown extension.
pub fn validate_file_content(file_path: &str, content: &str) -> Option<String> {
    let ext = file_path.rsplit('.').next().unwrap_or("");
    match ext {
        "json" => validate_json(content),
        "yaml" | "yml" => validate_yaml(content),
        "toml" => validate_toml(content),
        _ => None,
    }
}

fn validate_json(content: &str) -> Option<String> {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(_) => None,
        Err(e) => Some(format!("JSON parse error: {e}")),
    }
}

fn validate_yaml(content: &str) -> Option<String> {
    match serde_yaml::from_str::<serde_yaml::Value>(content) {
        Ok(_) => None,
        Err(e) => Some(format!("YAML parse error: {e}")),
    }
}

fn validate_toml(content: &str) -> Option<String> {
    match toml::from_str::<toml::Value>(content) {
        Ok(_) => None,
        Err(e) => Some(format!("TOML parse error: {e}")),
    }
}
