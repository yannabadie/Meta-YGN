//! Tree-sitter based multi-language syntax checking.
//! Feature-gated behind `syntax`.

/// A single syntax error found by tree-sitter parsing.
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// Parse `content` as the language implied by `extension` and return any
/// syntax errors found in the tree-sitter CST.
///
/// Returns an empty vec for unknown extensions or if the parser cannot be
/// initialised. Designed for <10 ms latency on typical source files.
pub fn check_syntax(content: &str, extension: &str) -> Vec<SyntaxError> {
    let language = match extension {
        "rs" => tree_sitter_rust::LANGUAGE,
        "py" => tree_sitter_python::LANGUAGE,
        "js" | "mjs" | "cjs" => tree_sitter_javascript::LANGUAGE,
        "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX,
        _ => return vec![],
    };

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language.into()).is_err() {
        return vec![];
    }

    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => {
            return vec![SyntaxError {
                line: 0,
                column: 0,
                message: "failed to parse".into(),
            }];
        }
    };

    let mut errors = Vec::new();
    collect_errors(tree.root_node(), &mut errors);
    errors
}

/// Walk the CST recursively and collect ERROR / MISSING nodes.
fn collect_errors(node: tree_sitter::Node, errors: &mut Vec<SyntaxError>) {
    if node.is_error() || node.is_missing() {
        let pos = node.start_position();
        errors.push(SyntaxError {
            line: pos.row + 1,
            column: pos.column,
            message: if node.is_missing() {
                format!("missing syntax element at line {}", pos.row + 1)
            } else {
                format!("syntax error at line {}", pos.row + 1)
            },
        });
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_errors(child, errors);
        }
    }
}
