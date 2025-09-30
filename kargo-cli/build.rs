use std::env;
use std::path::PathBuf;

fn main() {
    // Get the workspace root by finding Cargo.lock
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut workspace_root = PathBuf::from(manifest_dir);

    // Walk up to find workspace root (where Cargo.lock exists)
    while !workspace_root.join("Cargo.lock").exists() {
        workspace_root = workspace_root
            .parent()
            .expect("Could not find workspace root")
            .to_path_buf();
    }

    // Set KARGO_WORKSPACE_ROOT environment variable at compile time
    println!(
        "cargo:rustc-env=KARGO_WORKSPACE_ROOT={}",
        workspace_root.display()
    );
}
