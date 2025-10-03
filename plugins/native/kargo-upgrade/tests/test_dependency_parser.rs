//! Tests for dependency parsers

use anyhow::Result;
use tempfile::TempDir;
use tokio::fs;

use kargo_upgrade::models::{DependencyLocation, DependencyParser, DependencySource};
use kargo_upgrade::parsers::{CargoParser, RustScriptParser};

#[tokio::test]
async fn test_cargo_parser() -> Result<()> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let cargo_path = temp_dir.path().join("Cargo.toml");

    // Create test Cargo.toml file
    let cargo_content = r#"
[package]
name = "test-cargo"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.0"
tokio = { version = "1.0.0", features = ["full"] }
    "#;

    fs::write(&cargo_path, cargo_content).await?;

    // Create the dependency source
    let source = DependencySource::from_path(&cargo_path).await?;

    // Parse the dependencies
    let parser = CargoParser;
    let dependencies = parser.parse(&source)?;

    // Verify the results
    assert_eq!(dependencies.len(), 2);

    // Check for specific dependencies
    // The parser only finds the two direct dependencies
    let deps_names: Vec<_> = dependencies.iter().map(|d| d.name.clone()).collect();
    
    // Debug what names we're actually getting
    println!("Found dependency names: {:?}", deps_names);
    
    assert!(deps_names.contains(&"anyhow".to_string()));
    
    let anyhow_dep = dependencies
        .iter()
        .find(|d| d.name == "anyhow")
        .expect("TEST FAILURE: 'anyhow' dependency not found in parsed results");
    assert_eq!(anyhow_dep.version, "1.0.0");
    assert!(matches!(
        anyhow_dep.location,
        DependencyLocation::CargoTomlDirect
    ));

    let tokio_dep = dependencies
        .iter()
        .find(|d| d.name == "tokio")
        .expect("TEST FAILURE: 'tokio' dependency not found in parsed results");
    assert_eq!(tokio_dep.version, "1.0.0");
    assert!(matches!(
        tokio_dep.location,
        DependencyLocation::CargoTomlDirect
    ));

    Ok(())
}

#[tokio::test]
async fn test_rust_script_parser_cargo_format() -> Result<()> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let script_path = temp_dir.path().join("script.rs");

    // Create test rust script file with ```cargo format
    let script_content = r#"#!/usr/bin/env rust-script
//! This is a test rust script

```cargo
[dependencies]
anyhow = "1.0.0"
tokio = { version = "1.0.0", features = ["full"] }
```

fn main() {
    println!("Hello world!");
}
    "#;

    fs::write(&script_path, script_content).await?;

    // Create the dependency source
    let source = DependencySource::from_path(&script_path).await?;

    // Parse the dependencies
    let parser = RustScriptParser;
    let dependencies = parser.parse(&source)?;

    // Verify the results
    assert_eq!(dependencies.len(), 2);

    // Check for specific dependencies
    let anyhow_dep = dependencies
        .iter()
        .find(|d| d.name == "anyhow")
        .expect("TEST FAILURE: 'anyhow' dependency not found in rust-script parsed results");
    assert_eq!(anyhow_dep.version, "1.0.0");
    assert!(matches!(
        anyhow_dep.location,
        DependencyLocation::RustScriptCargo { .. }
    ));

    let tokio_dep = dependencies
        .iter()
        .find(|d| d.name == "tokio")
        .expect("TEST FAILURE: 'tokio' dependency not found in rust-script parsed results");
    assert_eq!(tokio_dep.version, "1.0.0");
    assert!(matches!(
        tokio_dep.location,
        DependencyLocation::RustScriptCargo { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn test_rust_script_parser_cargo_deps_format() -> Result<()> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let script_path = temp_dir.path().join("script.rs");

    // Create test rust script file with cargo-deps format
    let script_content = r#"#!/usr/bin/env rust-script
// cargo-deps: anyhow="1.0.0", tokio="1.0.0", regex

fn main() {
    println!("Hello world!");
}
    "#;

    fs::write(&script_path, script_content).await?;

    // Create the dependency source
    let source = DependencySource::from_path(&script_path).await?;

    // Parse the dependencies
    let parser = RustScriptParser;
    let dependencies = parser.parse(&source)?;

    // Verify the results
    assert_eq!(dependencies.len(), 3);

    // Check for specific dependencies
    let anyhow_dep = dependencies
        .iter()
        .find(|d| d.name == "anyhow")
        .expect("TEST FAILURE: 'anyhow' dependency not found");
    assert_eq!(anyhow_dep.version, "1.0.0");

    let tokio_dep = dependencies
        .iter()
        .find(|d| d.name == "tokio")
        .expect("TEST FAILURE: 'tokio' dependency not found");
    assert_eq!(tokio_dep.version, "1.0.0");

    let regex_dep = dependencies
        .iter()
        .find(|d| d.name == "regex")
        .expect("TEST FAILURE: 'regex' dependency not found");
    assert_eq!(regex_dep.version, "*");

    Ok(())
}
