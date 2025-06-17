//! Tests for dependency writers

use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use krater::up2date::models::{
    Dependency, DependencyLocation, DependencySource, DependencyUpdate, DependencyWriter,
};
use krater::up2date::writers::{CargoWriter, RustScriptWriter};

#[ignore]
#[tokio::test]
async fn test_cargo_writer() -> Result<()> {
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

[dev-dependencies]
tempfile = "3.0.0"
    "#;

    fs::write(&cargo_path, cargo_content).await?;

    // Create the dependency source
    let mut source = DependencySource::from_path(&cargo_path).await?;

    // Create test dependencies and updates
    let dependencies = vec![
        Dependency {
            name: "anyhow".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::CargoTomlDirect,
        },
        Dependency {
            name: "tokio".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::CargoTomlDirect,
        },
    ];

    let updates = vec![
        DependencyUpdate {
            name: "anyhow".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            dependency: dependencies[0].clone(),
        },
        DependencyUpdate {
            name: "tokio".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            dependency: dependencies[1].clone(),
        },
    ];

    // Apply updates
    let writer = CargoWriter;
    writer.apply_updates(&mut source, &updates)?;

    // Write back to disk
    writer.write(&source)?;

    // Read the updated file
    let updated_content = fs::read_to_string(&cargo_path).await?;

    // Verify the updates
    assert!(updated_content.contains("anyhow = \"2.0.0\""));
    assert!(updated_content.contains("version = \"2.0.0\""));
    assert!(updated_content.contains("tempfile = \"3.0.0\"")); // Not changed

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_rust_script_writer_cargo_format() -> Result<()> {
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
    let mut source = DependencySource::from_path(&script_path).await?;

    // Create test dependencies and updates
    // We need the correct section range, which would normally come from the parser
    let content = source.content();
    let section_start = content.find("[dependencies]").expect("Failed to find [dependencies] section in rust-script content");
    let section_end = content[section_start..].find("```").map(|pos| pos + section_start).expect("Failed to find closing ``` for cargo section in rust-script");
    let section_range = (section_start, section_end);

    let dependencies = vec![
        Dependency {
            name: "anyhow".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::RustScriptCargo { section_range },
        },
        Dependency {
            name: "tokio".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::RustScriptCargo { section_range },
        },
    ];

    let updates = vec![
        DependencyUpdate {
            name: "anyhow".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            dependency: dependencies[0].clone(),
        },
        DependencyUpdate {
            name: "tokio".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            dependency: dependencies[1].clone(),
        },
    ];

    // Apply updates
    let writer = RustScriptWriter;
    writer.apply_updates(&mut source, &updates)?;

    // Write back to disk
    writer.write(&source)?;

    // Read the updated file
    let updated_content = fs::read_to_string(&script_path).await?;

    // Verify the updates
    assert!(updated_content.contains("anyhow = \"2.0.0\""));
    assert!(updated_content.contains("version = \"2.0.0\""));

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_rust_script_writer_cargo_deps_format() -> Result<()> {
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
    let mut source = DependencySource::from_path(&script_path).await?;

    // Create test dependencies and updates
    // We need the correct line range, which would normally come from the parser
    let content = source.content();
    let cargo_deps_start = content.find("anyhow").expect("Failed to find 'anyhow' in cargo-deps line");
    let line_end = content[cargo_deps_start..].find("\n").map(|pos| pos + cargo_deps_start).expect("Failed to find end of cargo-deps line");
    let line_range = (cargo_deps_start, line_end);

    let dependencies = vec![
        Dependency {
            name: "anyhow".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::RustScriptDeps { line_range },
        },
        Dependency {
            name: "tokio".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::RustScriptDeps { line_range },
        },
        Dependency {
            name: "regex".to_string(),
            version: "".to_string(), // No version specified
            location: DependencyLocation::RustScriptDeps { line_range },
        },
    ];

    let updates = vec![
        DependencyUpdate {
            name: "anyhow".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            dependency: dependencies[0].clone(),
        },
        DependencyUpdate {
            name: "tokio".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            dependency: dependencies[1].clone(),
        },
        DependencyUpdate {
            name: "regex".to_string(),
            from_version: "none".to_string(),
            to_version: "1.5.0".to_string(),
            dependency: dependencies[2].clone(),
        },
    ];

    // Apply updates
    let writer = RustScriptWriter;
    writer.apply_updates(&mut source, &updates)?;

    // Write back to disk
    writer.write(&source)?;

    // Read the updated file
    let updated_content = fs::read_to_string(&script_path).await?;

    // Verify the updates
    assert!(updated_content.contains("anyhow=\"2.0.0\""));
    assert!(updated_content.contains("tokio=\"2.0.0\""));
    assert!(updated_content.contains("regex=\"1.5.0\"")); // Added version

    Ok(())
}
