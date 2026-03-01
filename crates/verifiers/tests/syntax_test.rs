#![cfg(feature = "syntax")]

use metaygn_verifiers::syntax::check_syntax;

#[test]
fn valid_rust() {
    let errors = check_syntax("fn main() { println!(\"hello\"); }", "rs");
    assert!(errors.is_empty(), "valid Rust should have no errors");
}

#[test]
fn invalid_rust() {
    let errors = check_syntax("fn main( { }", "rs");
    assert!(!errors.is_empty(), "invalid Rust should have errors");
}

#[test]
fn valid_python() {
    let errors = check_syntax("def hello():\n    print('hi')\n", "py");
    assert!(errors.is_empty(), "valid Python should have no errors");
}

#[test]
fn invalid_python() {
    let errors = check_syntax("def hello(\n    print('hi')\n", "py");
    assert!(!errors.is_empty(), "invalid Python should have errors");
}

#[test]
fn valid_javascript() {
    let errors = check_syntax("function hello() { return 42; }", "js");
    assert!(errors.is_empty(), "valid JS should have no errors");
}

#[test]
fn invalid_javascript() {
    let errors = check_syntax("function hello( { return 42; }", "js");
    assert!(!errors.is_empty(), "invalid JS should have errors");
}

#[test]
fn valid_typescript() {
    let errors = check_syntax("function hello(): number { return 42; }", "ts");
    assert!(errors.is_empty(), "valid TS should have no errors");
}

#[test]
fn valid_tsx() {
    let errors = check_syntax("const App = () => <div>hello</div>;", "tsx");
    assert!(errors.is_empty(), "valid TSX should have no errors");
}

#[test]
fn unknown_extension_returns_empty() {
    let errors = check_syntax("anything goes here!", "xyz");
    assert!(
        errors.is_empty(),
        "unknown extension should return no errors"
    );
}

#[test]
fn error_has_correct_line_info() {
    // Line 2 has the syntax error (missing closing paren)
    let code = "fn main() {\n    let x = foo(;\n}\n";
    let errors = check_syntax(code, "rs");
    assert!(!errors.is_empty(), "should detect syntax error");
    // The error should be on line 2
    assert!(
        errors.iter().any(|e| e.line == 2),
        "error should be reported on line 2, got lines: {:?}",
        errors.iter().map(|e| e.line).collect::<Vec<_>>()
    );
}
