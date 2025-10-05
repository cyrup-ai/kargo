use std::env;
use std::path::PathBuf;

fn main() {
    // Get the workspace root by finding Cargo.lock
    let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") else {
        eprintln!("ERROR: CARGO_MANIFEST_DIR environment variable not set");
        eprintln!("This should be set by cargo during build. Cannot proceed.");
        std::process::exit(1);
    };
    let mut workspace_root = PathBuf::from(manifest_dir);

    // Walk up to find workspace root (where Cargo.lock exists)
    while !workspace_root.join("Cargo.lock").exists() {
        workspace_root = workspace_root.parent().unwrap_or_else(|| {
            panic!(
                "Could not find workspace root (Cargo.lock) - searched up from {}",
                workspace_root.display()
            )
        }).to_path_buf();
    }

    // Set KARGO_WORKSPACE_ROOT environment variable at compile time
    println!(
        "cargo:rustc-env=KARGO_WORKSPACE_ROOT={}",
        workspace_root.display()
    );
}
