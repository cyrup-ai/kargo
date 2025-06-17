use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),

    #[error("Failed to parse package specification: {0}")]
    PackageSpecParse(String),

    #[error("Toolchain error: {0}")]
    Toolchain(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to find generated documentation")]
    DocNotFound,

    #[error("Failed to parse TOML: {0}")]
    TomlParse(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Failed to check Rustup: {0}")]
    RustupCheckFailed(String),

    #[error("Rustup not found. Please install Rustup from https://rustup.rs")]
    RustupNotFound,

    #[error("Cargo not found. Please install Rust from https://rustup.rs")]
    CargoNotFound,

    #[error("Failed to set up temporary project: {0}")]
    TempProjectSetup(String),

    #[error("Failed to copy documentation: {0}")]
    DocCopyFailed(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Failed to cleanup temporary directory: {0}")]
    CleanupFailed(String),

    #[error("Failed to convert JSON to Markdown: {0}")]
    MarkdownConversionFailed(String),

    #[error("Other error: {0}")]
    Other(String),
}
