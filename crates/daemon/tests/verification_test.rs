use metaygn_daemon::verification::validate_file_content;

#[test]
fn valid_json() {
    assert!(validate_file_content("test.json", r#"{"key": "value"}"#).is_none());
}

#[test]
fn invalid_json() {
    let result = validate_file_content("test.json", "{bad json");
    assert!(result.is_some());
    assert!(result.unwrap().contains("JSON parse error"));
}

#[test]
fn valid_yaml() {
    assert!(validate_file_content("config.yaml", "key: value\nlist:\n  - item1").is_none());
}

#[test]
fn invalid_yaml() {
    let result = validate_file_content("config.yml", "key: [invalid: yaml: here");
    assert!(result.is_some());
}

#[test]
fn valid_toml() {
    assert!(validate_file_content("Cargo.toml", "[package]\nname = \"test\"").is_none());
}

#[test]
fn invalid_toml() {
    let result = validate_file_content("config.toml", "[invalid\ntoml");
    assert!(result.is_some());
    assert!(result.unwrap().contains("TOML parse error"));
}

#[test]
fn unknown_extension_returns_none() {
    assert!(validate_file_content("file.rs", "fn main() {}").is_none());
    assert!(validate_file_content("file.py", "import os").is_none());
}
