# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kargo is a cargo wrapper with a plugin system that extends cargo functionality through native Rust plugins and WASM plugins. The CLI dynamically discovers, builds, and loads plugins at runtime.

### Architecture

**Core Components:**
- `kargo-cli/`: Main binary that wraps cargo and manages plugins
- `kargo-plugin/`: Plugin framework with multiple subcrates:
  - `kargo-plugin-api/`: Trait definitions and types for plugin communication
  - `kargo-plugin-native/`: Native plugin trait and metadata
  - `kargo-plugin-macros/`: Proc macros for plugin development
  - `kargo-plugin-builder/`: Build-time support for plugins
  - `kargo-plugin-wasm/`: WASM plugin support via Extism
- `plugins/native/`: Native Rust plugins with full system access
- `plugins/wasm/`: Sandboxed WASM plugins

**Plugin Loading Flow:**
1. `PluginManager::new()` discovers plugins in workspace at compile time via `build.rs`
2. Plugins are auto-discovered in `plugins/native/` and `plugins/wasm/` directories
3. Each plugin directory with `Cargo.toml` is built with `cargo build --release --lib`
4. Native plugins are loaded via `libloading` and must export `kargo_plugin_create` symbol
5. Trait scanner (`trait_scanner.rs`) verifies plugins implement `PluginCommand` trait
6. Plugins extend the CLI by returning `clap::Command` from their `clap()` method

**Command Dispatch:**
- Unknown subcommands are proxied to cargo (e.g., `kargo build` → `cargo build`)
- Known plugin subcommands execute via `PluginCommand::run()` with `ExecutionContext`
- Verbosity controlled via `-v` flags (off/error/warn/info/debug/trace)

### Workspace Structure

This is a Cargo workspace with resolver = "2". All plugins and framework crates share workspace dependencies defined in root `Cargo.toml`.

## Development Commands

### Building and Testing

```bash
# Build entire workspace
cargo build --workspace
just build

# Build release binaries
cargo build --workspace --release
just build-release

# Run tests (uses cargo-nextest)
cargo nextest run
just test

# Run tests for a single package
cargo test -p kargo-mddoc
cargo nextest run -p kargo-mddoc

# Check formatting and build
just check

# Format code
cargo fmt --all
just fmt

# Run clippy
cargo clippy --workspace
```

### Plugin Development

```bash
# Create a new native plugin (full OS access, high performance)
just new-plugin-native <name>

# Build a specific native plugin
just build-plugin-native <name>
cd plugins/native/<name> && cargo build --release

# Build all native plugins
just build-plugins-native

# Create WASM plugins (sandboxed)
just new-plugin-rust-wasm <name>
just new-plugin-python <name>
just new-plugin-node <name>
just new-plugin-go <name>

# Install kargo locally
cargo install --path kargo-cli
just install
```

### Running Kargo

```bash
# Run the CLI during development (from workspace root)
cargo run -p kargo-cli -- <subcommand> [args]

# After installation
kargo <subcommand> [args]

# Proxy to cargo
kargo cargo build
kargo build  # Also proxies to cargo if no plugin handles it

# Run with verbose logging
kargo -vvv <subcommand>  # info level
kargo -vvvv <subcommand> # debug level
```

## Native Plugin Implementation

All native plugins must:
1. Depend on `kargo-plugin-api = "0.1.0"`
2. Implement the `PluginCommand` trait:
   - `fn clap(&self) -> clap::Command` - Return CLI definition
   - `fn run(&self, ctx: ExecutionContext) -> BoxFuture` - Execute plugin logic
3. Export a `kargo_plugin_create` function:
   ```rust
   #[no_mangle]
   pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
       Box::new(MyPlugin)
   }
   ```
4. Use `crate-type = ["cdylib"]` in their `Cargo.toml`

**Key Plugin Examples:**
- `kargo-mddoc`: Generates Markdown documentation from Rust crates using rustdoc JSON
- `kargo-upgrade`: Updates dependencies in Cargo.toml and Rust scripts
- `kargo-mdlint`: Lints Markdown files
- `kargo-walk`: File tree walker
- `kargo-sap`: Search and process files
- `kargo-kurate`: Crate information tool

## Important Implementation Details

### Plugin Discovery
- `PluginManager` uses `CARGO_MANIFEST_DIR` at compile time to discover plugins
- The `build.rs` in `kargo-cli` sets `KARGO_WORKSPACE_ROOT` environment variable
- Plugins are rebuilt only if source files are newer than the compiled artifact
- Library artifacts are located in `target/release/lib<name>.{dylib,so,dll}`

### Argument Handling
- `gather_raw_args()` in `cli.rs` captures raw CLI arguments for plugins
- Plugins receive `ExecutionContext` with matched args, current dir, and config dir
- Config directory defaults to `~/.config/kargo` on Unix-like systems

### Logging
- Uses `env_logger` with `RUST_LOG` environment variable
- Verbosity levels: off (default) → error (-v) → warn (-vv) → info (-vvv) → debug (-vvvv) → trace (-vvvvv+)
- Plugin loading uses `log::info!()` for discovery and load status

## Common Gotchas

1. **Plugin not loading?**
   - Check that `kargo_plugin_create` is exported with `#[no_mangle]` and `extern "C"`
   - Verify `crate-type = ["cdylib"]` is set
   - Run with `-vvv` to see plugin discovery logs

2. **Compilation errors in plugins?**
   - Ensure plugin depends on correct version of `kargo-plugin-api`
   - Check workspace dependency versions match

3. **Tests not found?**
   - Use `cargo nextest run` instead of `cargo test` (preferred in this project)
   - Or use `cargo test` with appropriate filters

4. **Working directory issues?**
   - Plugins receive `ExecutionContext.current_dir` with the invocation directory
   - Use this instead of `std::env::current_dir()` when possible

## Project-Specific Patterns

- **Edition 2024**: All crates use Rust Edition 2024
- **Async runtime**: Uses `tokio` with "full" features
- **Error handling**: Primarily uses `anyhow::Result<()>`
- **Parallel processing**: `rayon` for CPU-bound parallelism
- **File walking**: `jwalk` for efficient directory traversal
- **TOML editing**: `toml_edit` for preserving formatting
- **Process management**: `tokio::process::Command` for async process spawning
