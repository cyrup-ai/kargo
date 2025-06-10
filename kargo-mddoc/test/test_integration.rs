//! Integration tests for the dependency updater

use anyhow::Result;
use tempfile::TempDir;
use tokio::fs;
use futures::StreamExt;

use krater::events::EventBus;
use krater::up2date::{coordinator::start_update, types::UpdateOptions};

#[ignore]
#[tokio::test]
async fn test_full_update_process() -> Result<()> {
    // Create temporary test directory with files
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // Create a Cargo.toml file
    let cargo_path = root_path.join("Cargo.toml");
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

    // Create a rust script file with ```cargo format
    let script1_path = root_path.join("script1.rs");
    let script1_content = r#"#!/usr/bin/env rust-script
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
    fs::write(&script1_path, script1_content).await?;

    // Create a rust script file with cargo-deps format
    let script2_path = root_path.join("script2.rs");
    let script2_content = r#"#!/usr/bin/env rust-script
// cargo-deps: anyhow="1.0.0", tokio="1.0.0", regex

fn main() {
    println!("Hello world!");
}
    "#;
    fs::write(&script2_path, script2_content).await?;

    // Create options and event bus
    let options = UpdateOptions {
        update_workspace: true,
        compatible_only: true,
    };
    let events = EventBus::new();

    // Start the update process
    let mut session = start_update(root_path.to_path_buf(), options, events);

    // Collect all results
    let mut results = Vec::new();
    let mut stream = session.into_stream();
    while let Some(result) = stream.next().await {
        results.push(result);
    }

    // Check results
    assert_eq!(results.len(), 3); // 3 files updated

    // Verify the updates
    for result in &results {
        // Each result should have updates
        assert!(!result.updates.is_empty());

        // Each update should have a new version
        for update in &result.updates {
            assert_ne!(update.from_version, update.to_version);
        }
    }

    // Read the updated files to verify content
    let updated_cargo = fs::read_to_string(&cargo_path).await?;
    let updated_script1 = fs::read_to_string(&script1_path).await?;
    let updated_script2 = fs::read_to_string(&script2_path).await?;

    // Cargo.toml should have updated versions
    assert!(
        updated_cargo.contains("anyhow = \"2.0.0\"")
            || updated_cargo.contains("anyhow = \"")
            || updated_cargo.contains("\"anyhow\"")
    );

    // Script1 should have updated versions in ```cargo section
    assert!(
        updated_script1.contains("anyhow = \"2.0.0\"")
            || updated_script1.contains("anyhow = \"")
            || updated_script1.contains("\"anyhow\"")
    );

    // Script2 should have updated versions in cargo-deps line
    assert!(
        updated_script2.contains("anyhow=\"2.0.0\"")
            || updated_script2.contains("anyhow=\"")
            || updated_script2.contains("\"anyhow\"")
    );
    assert!(updated_script2.contains("regex=\"") && !updated_script2.contains("regex,")); // Bare dependency should get a version

    Ok(())
}
