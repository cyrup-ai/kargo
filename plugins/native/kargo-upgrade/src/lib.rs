pub mod crates_io;
pub mod finder;
pub mod models;
pub mod parsers;
pub mod plugin;
pub mod types;
pub mod updater;
pub mod updaters;
pub mod writers;

// Re-export plugin for convenience
pub use plugin::UpgradePlugin;
