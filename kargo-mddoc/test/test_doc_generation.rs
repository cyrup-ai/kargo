//! Tests for documentation generation feature

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use futures::StreamExt;

use krater::docs::DocGenerator;
use krater::config::Config;
use krater::events::{Event, EventBus};
use krater::up2date::{coordinator::start_update, types::UpdateOptions};

#[tokio::test]
async fn test_doc_generator() -> Result<()> {
    // Create temporary test directory with files
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // Create temporary knowledge base directory
    let kb_dir = TempDir::new()?;
    env::set_var("KNOWLEDGE_BASE_ROOT_DIR", kb_dir.path().to_str().unwrap());

    // Create a Cargo.toml file for a simple project
    let src_dir = root_path.join("src");
    fs::create_dir_all(&src_dir).await?;
    
    let cargo_path = root_path.join("Cargo.toml");
    let cargo_content = r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.0"
tokio = { version = "1.0.0", features = ["full"] }
    "#;
    fs::write(&cargo_path, cargo_content).await?;

    // Create a simple lib.rs file with some documentation
    let lib_path = src_dir.join("lib.rs");
    let lib_content = r#"
//! This is a test crate for documentation generation

/// A test function with documentation
/// 
/// # Examples
/// 
/// ```
/// let result = test_crate::test_function(42);
/// assert_eq!(result, 84);
/// ```
pub fn test_function(x: i32) -> i32 {
    x * 2
}

/// A test struct with documentation
#[derive(Debug)]
pub struct TestStruct {
    /// A field with documentation
    pub field: String,
}

impl TestStruct {
    /// Creates a new TestStruct
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }
}
    "#;
    fs::write(&lib_path, lib_content).await?;

    // Create a DocGenerator and generate docs
    let config = Config::default();
    let events = EventBus::new();
    let doc_generator = DocGenerator::new(config, events);
    
    // Generate documentation
    let output_path = doc_generator.generate_markdown_docs(root_path).await?;
    
    // Check that the documentation file exists
    assert!(output_path.exists(), "Documentation file doesn't exist");

    // Check the markdown content
    let markdown_content = fs::read_to_string(&output_path).await?;
    assert!(markdown_content.contains("This is a test crate"), "Missing crate documentation");
    assert!(markdown_content.contains("test_function"), "Missing function documentation");
    assert!(markdown_content.contains("TestStruct"), "Missing struct documentation");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_doc_generation_with_update() -> Result<()> {
    // Create temporary test directory with files
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // Create temporary knowledge base directory
    let kb_dir = TempDir::new()?;
    env::set_var("KNOWLEDGE_BASE_ROOT_DIR", kb_dir.path().to_str().unwrap());

    // Create a Cargo.toml file
    let src_dir = root_path.join("src");
    fs::create_dir_all(&src_dir).await?;
    
    let cargo_path = root_path.join("Cargo.toml");
    let cargo_content = r#"
[package]
name = "test-updated-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.0"
    "#;
    fs::write(&cargo_path, cargo_content).await?;

    // Create a simple lib.rs file
    let lib_path = src_dir.join("lib.rs");
    let lib_content = r#"
//! This is a test crate for documentation generation

/// A test function with documentation
pub fn test_function(x: i32) -> i32 {
    x * 2
}
    "#;
    fs::write(&lib_path, lib_content).await?;

    // Create update options and event bus
    let options = UpdateOptions {
        update_workspace: true,
        compatible_only: true,
    };
    let events = EventBus::new();

    // Subscribe to events to check for documentation generation
    let mut rx = events.subscribe();
    
    // Spawn a task to listen for events
    let event_task = tokio::spawn(async move {
        let mut doc_generated = false;
        
        while let Ok(event) = rx.recv().await {
            if let Event::Info { message } = &event {
                if message.contains("Generated documentation") {
                    doc_generated = true;
                    break;
                }
            }
        }
        
        doc_generated
    });

    // Start the update process
    let mut session = start_update(root_path.to_path_buf(), options, events);

    // Convert session to stream and wait for all updates to complete
    let mut stream = session.into_stream();
    while let Some(_result) = stream.next().await {}

    // Check that documentation was generated
    let doc_generated = event_task.await?;
    assert!(doc_generated, "Documentation wasn't generated during update");

    // Check that the documentation file exists in the knowledge base
    let doc_path = PathBuf::from(kb_dir.path())
        .join("rust")
        .join("crates")
        .join("test-updated-crate")
        .join("README.md");
    
    assert!(doc_path.exists(), "Documentation file doesn't exist at expected path");

    // Read the documentation file
    let doc_content = fs::read_to_string(doc_path).await?;
    assert!(doc_content.contains("This is a test crate"), "Missing crate documentation");
    assert!(doc_content.contains("test_function"), "Missing function documentation");

    Ok(())
}