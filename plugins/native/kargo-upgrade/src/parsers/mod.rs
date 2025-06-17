//! Module containing parsers for different dependency sources

mod cargo_parser;
mod rust_script_parser;

pub use cargo_parser::*;
pub use rust_script_parser::*;
