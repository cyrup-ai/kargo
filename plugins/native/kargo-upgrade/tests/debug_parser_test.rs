//! Debug test to understand why cargo-deps parsing returns 0 results

use anyhow::Result;
use tempfile::TempDir;
use tokio::fs;

use kargo_upgrade::models::{DependencyParser, DependencySource};
use kargo_upgrade::parsers::RustScriptParser;

#[tokio::test]
async fn debug_rust_script_parser_cargo_deps() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let script_path = temp_dir.path().join("script.rs");

    let script_content = r#"#!/usr/bin/env rust-script
// cargo-deps: anyhow="1.0.0", tokio="1.0.0", regex

fn main() {
    println!("Hello world!");
}
    "#;

    fs::write(&script_path, script_content).await?;

    println!("\n=== PARSER DEBUG ===");
    println!("Script path: {:?}", script_path);
    println!("Script content:\n{}", script_content);

    // Create the dependency source
    let source = DependencySource::from_path(&script_path).await?;
    
    println!("\nSource created, content length: {}", source.content().len());
    println!("Source content:\n{}", source.content());

    // Parse the dependencies
    let parser = RustScriptParser;
    let dependencies = parser.parse(&source)?;

    println!("\nParsed dependencies count: {}", dependencies.len());
    for (i, dep) in dependencies.iter().enumerate() {
        println!("  Dep {}: name={}, version={}, location={:?}", 
                 i, dep.name, dep.version, dep.location);
    }

    // This should find 3 dependencies: anyhow, tokio, regex
    // But currently it might return 0
    println!("\nExpected: 3 dependencies (anyhow, tokio, regex)");
    println!("Actual: {} dependencies", dependencies.len());

    Ok(())
}
