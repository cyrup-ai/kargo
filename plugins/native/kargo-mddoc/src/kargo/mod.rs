pub mod clap;
pub mod config;
pub mod error;
pub mod generator;
pub mod markdown;
pub mod multipage_markdown;
pub mod package;
pub mod rust2md;
pub mod toolchain;
pub mod utils;

// Re-export main types for easier usage
#[allow(unused_imports)]
pub use clap::*;
pub use config::Config;
pub use error::Error;
pub use generator::DocGenerator;
pub use package::PackageSpec;
pub use rust2md::*;

// Version of rustdoc-md for programmatic access
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
