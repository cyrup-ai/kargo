pub mod config;
pub mod error;
pub mod generator;
pub mod markdown;
pub mod package;
pub mod toolchain;
pub mod utils;
pub mod clap;
pub mod rust2md;

// Re-export main types for easier usage
pub use config::Config;
pub use error::Error;
pub use generator::DocGenerator;
pub use package::PackageSpec;
pub use clap::*;
pub use rust2md::*;

// Version of rustdoc-md for programmatic access
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
