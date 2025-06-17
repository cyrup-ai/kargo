use kargo_upgrade::models::{DependencyParser, DependencySource};
use kargo_upgrade::parsers::rust_script_parser::RustScriptParser;

#[test]
fn test_parse_cargo_section() {
    let content = r#"
#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```

fn main() {
    println!("Hello!");
}
"#;

    let source = DependencySource::from_content(content.to_string());
    let parser = RustScriptParser;
    let deps = parser.parse(&source).expect("Failed to parse dependencies");

    assert_eq!(deps.len(), 2);
    assert_eq!(deps[0].name, "anyhow");
    assert_eq!(deps[0].version, "1.0");
    assert_eq!(deps[1].name, "tokio");
    assert_eq!(deps[1].version, "1.0");
}

#[test]
fn test_parse_cargo_deps_line() {
    let content = r#"
#!/usr/bin/env rust-script
// cargo-deps: anyhow="1.0", tokio="1.0", serde

fn main() {
    println!("Hello!");
}
"#;

    let source = DependencySource::from_content(content.to_string());
    let parser = RustScriptParser;
    let deps = parser.parse(&source).expect("Failed to parse dependencies");

    assert_eq!(deps.len(), 3);
    assert_eq!(deps[0].name, "anyhow");
    assert_eq!(deps[0].version, "1.0");
    assert_eq!(deps[1].name, "tokio");
    assert_eq!(deps[1].version, "1.0");
    assert_eq!(deps[2].name, "serde");
    assert_eq!(deps[2].version, "*");
}
