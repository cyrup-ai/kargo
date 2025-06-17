//! Module for dependency writers

mod cargo_writer;
mod rust_script_writer;

pub use cargo_writer::*;
pub use rust_script_writer::*;
