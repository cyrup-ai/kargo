use krater::config::Config;
use krater::docs::DocGenerator;
use krater::events::EventBus;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_extract_package_name() {
    let config = Config::default();
    let events = EventBus::new();
    let generator = DocGenerator::new(config, events);
    
    let content = r#"
        [package]
        name = "test-crate"
        version = "0.1.0"
        edition = "2021"
    "#;
    
    let name = generator.extract_package_name(content).expect("Failed to extract package name from Cargo.toml content");
    assert_eq!(name, "test-crate");
}

#[test]
fn test_get_output_path() {
    let config = Config::default();
    let events = EventBus::new();
    let generator = DocGenerator::new(config, events);
    
    // Temporarily set KNOWLEDGE_BASE_ROOT_DIR
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let kb_root = temp_dir.path().to_str().expect("Failed to convert temp directory path to string");
    std::env::set_var("KNOWLEDGE_BASE_ROOT_DIR", kb_root);
    
    let output_path = generator.get_output_path("test-package").expect("Failed to get output path for documentation");
    
    // Check path structure
    assert!(output_path.to_str().expect("Failed to convert output path to string").contains("rust/crates/test-package/README.md"));
    
    // Clean up
    std::env::remove_var("KNOWLEDGE_BASE_ROOT_DIR");
}