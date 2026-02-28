//! Static verification-script templates.
//!
//! Each [`Template`] carries a name, language tag, human description, source
//! code (with optional `{{param}}` placeholders), and the list of parameter
//! names the caller may substitute.
//!
//! Templates are defined as `&'static` data — no heap allocation, no runtime
//! generation.  The [`get_template`] and [`list_templates`] helpers provide
//! convenient access.

use super::engine::ScriptLang;

// ---------------------------------------------------------------------------
// Template type
// ---------------------------------------------------------------------------

/// A static verification-script template.
#[derive(Debug, Clone)]
pub struct Template {
    pub name: &'static str,
    pub language: ScriptLang,
    pub description: &'static str,
    /// The template source code.  `{{param}}` placeholders are replaced at
    /// generation time.
    pub source: &'static str,
    /// Names of parameters that may appear as `{{name}}` in `source`.
    pub params: &'static [&'static str],
}

// ---------------------------------------------------------------------------
// Template catalogue
// ---------------------------------------------------------------------------

/// All built-in verification templates.
pub static TEMPLATES: &[Template] = &[
    // 1. grep-pattern-checker — regex search over text
    Template {
        name: "grep-pattern-checker",
        language: ScriptLang::Python,
        description: "Search for a regex pattern in given text",
        source: r#"import re, sys, json
input_data = json.loads(sys.stdin.read())
pattern = input_data.get("pattern", "")
text = input_data.get("text", "")
matches = re.findall(pattern, text)
print(json.dumps({"found": len(matches) > 0, "count": len(matches), "matches": matches[:10]}))"#,
        params: &[],
    },
    // 2. import-validator — check Python imports
    Template {
        name: "import-validator",
        language: ScriptLang::Python,
        description: "Check if Python imports are valid",
        source: r#"import sys, json
input_data = json.loads(sys.stdin.read())
modules = input_data.get("modules", [])
results = {}
for mod in modules:
    try:
        __import__(mod)
        results[mod] = "ok"
    except ImportError as e:
        results[mod] = f"error: {e}"
print(json.dumps({"results": results, "all_valid": all(v == "ok" for v in results.values())}))"#,
        params: &[],
    },
    // 3. json-validator — validate JSON structure
    Template {
        name: "json-validator",
        language: ScriptLang::Python,
        description: "Validate JSON structure",
        source: r#"import sys, json
input_data = sys.stdin.read()
try:
    parsed = json.loads(input_data)
    print(json.dumps({"valid": True, "type": type(parsed).__name__, "keys": list(parsed.keys()) if isinstance(parsed, dict) else None}))
except json.JSONDecodeError as e:
    print(json.dumps({"valid": False, "error": str(e)}))"#,
        params: &[],
    },
    // 4. file-exists-checker — check if files exist (bash)
    Template {
        name: "file-exists-checker",
        language: ScriptLang::Bash,
        description: "Check if files exist",
        source: r#"#!/bin/bash
while IFS= read -r file; do
    if [ -f "$file" ]; then
        echo "OK: $file"
    else
        echo "MISSING: $file"
    fi
done"#,
        params: &[],
    },
];

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

/// Look up a template by name.
pub fn get_template(name: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.name == name)
}

/// Return the names of all available templates.
pub fn list_templates() -> Vec<&'static str> {
    TEMPLATES.iter().map(|t| t.name).collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_has_four_entries() {
        assert_eq!(TEMPLATES.len(), 4);
    }

    #[test]
    fn get_known_template() {
        let t = get_template("json-validator").expect("template should exist");
        assert_eq!(t.language, ScriptLang::Python);
    }

    #[test]
    fn get_unknown_template_returns_none() {
        assert!(get_template("nope").is_none());
    }

    #[test]
    fn list_returns_all_names() {
        let names = list_templates();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"grep-pattern-checker"));
        assert!(names.contains(&"import-validator"));
        assert!(names.contains(&"json-validator"));
        assert!(names.contains(&"file-exists-checker"));
    }
}
