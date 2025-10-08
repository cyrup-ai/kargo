# Kargo

> A cargo wrapper with zero-knowledge plugins

Kargo is a drop-in replacement for cargo that extends its functionality through a powerful plugin system. Plugins are dynamically loaded native libraries that can add new subcommands without requiring cargo to know about them in advance.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [How It Works](#how-it-works)
- [Plugin Management](#plugin-management)
- [Available Plugins](#available-plugins)
- [Usage Examples](#usage-examples)
- [Building from Source](#building-from-source)
- [Contributing](#contributing)

## Features

- **Zero-Knowledge Plugin System**: Plugins are loaded dynamically without requiring kargo to be recompiled
- **Drop-in Cargo Replacement**: All cargo commands work through kargo
- **Native Performance**: Plugins are compiled native libraries (cdylib), not scripts
- **Easy Plugin Management**: Install plugins directly from GitHub repositories
- **Shell Aliasing**: Start a shell session with `cargo` aliased to `kargo`

## Installation

### From Source

```bash
git clone https://github.com/YOUR_ORG/kargo.git
cd kargo
cargo build --release
```

The binary will be available at `./target/release/kargo`.

### Add to PATH

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export PATH="/path/to/kargo/target/release:$PATH"
```

## How It Works

### Architecture

Kargo operates as a transparent proxy to cargo while adding plugin capabilities:

1. **Command Routing**: When you run `kargo <command>`, it checks if a plugin handles that command
2. **Plugin Priority**: If a plugin exists for the command, it runs the plugin
3. **Cargo Fallback**: If no plugin handles it, the command is forwarded to cargo
4. **Dynamic Loading**: Plugins are loaded at runtime from the platform-specific config directory

### Plugin System

Plugins are Rust libraries that implement the `PluginCommand` trait from `kargo-plugin-api`. They:

- Define their own CLI using clap
- Have full access to the execution context (current directory, config, etc.)
- Run asynchronously with tokio support
- Are completely isolated from each other

### Plugin Discovery

Kargo discovers plugins in platform-specific configuration directories using the `dirs` crate (respects `$XDG_CONFIG_HOME` on Linux):

**Configuration Directory Structure:**

```
<CONFIG_DIR>/kargo/plugins/
├── github.com/
│   └── ORG/
│       └── REPO/
│           └── BRANCH/
│               └── PLUGIN_NAME/
│                   └── VERSION/
│                       └── libplugin.{so,dylib,dll}
└── local/
    └── PLUGIN_NAME/
        └── VERSION/
            └── libplugin.{so,dylib,dll}
```

**Where `<CONFIG_DIR>` is:**
- **Linux**: `$XDG_CONFIG_HOME` (typically `~/.config`)
- **macOS**: `~/Library/Application Support`
- **Windows**: `%APPDATA%` (typically `C:\Users\<User>\AppData\Roaming`)

**Example Paths:**
- Linux: `~/.config/kargo/plugins/`
- macOS: `~/Library/Application Support/kargo/plugins/`
- Windows: `C:\Users\<User>\AppData\Roaming\kargo\plugins\`

## Plugin Management

### Installing Plugins

Kargo provides a built-in `plugin` subcommand for managing plugins:

```bash
# Install from GitHub (org/repo format) - for standalone plugin repos
kargo plugin install ORG/REPO
kargo plugin install ORG/REPO --branch develop

# Install from full GitHub URL - for standalone plugin repos
kargo plugin install https://github.com/ORG/REPO

# Install from local path (REQUIRED for cyrup-ai/kargo plugins)
# First clone the kargo repository:
git clone https://github.com/cyrup-ai/kargo.git
cd kargo

# Then install plugins from local paths:
kargo plugin install ./plugins/native/kargo-mddoc
kargo plugin install ./plugins/native/kargo-turd
kargo plugin install ./plugins/native/kargo-mdlint
kargo plugin install ./plugins/native/kargo-sap
```

### Listing Plugins

```bash
# List all installed plugins
kargo plugin list

# List plugins available in a remote repository (for standalone plugin repos)
kargo plugin list --remote https://github.com/ORG/plugin-repo
```

### Removing Plugins

```bash
# Remove a plugin by identifier
kargo plugin remove kargo-mddoc

# For plugins installed from GitHub (when using standalone repos)
kargo plugin remove ORG/REPO
kargo plugin remove https://github.com/ORG/REPO
```

## Available Plugins

**7 plugins** are planned for this repository. Currently **4 are complete** and installable, with **3 in development**.

### Quick Install (Complete Plugins)

Since the plugins are part of the main kargo repository, clone it first and install from local paths:

```bash
# 1. Clone the kargo repository
git clone https://github.com/cyrup-ai/kargo.git
cd kargo

# 2. Build the workspace (optional but recommended)
cargo build --workspace --release

# 3. Install complete plugins (4 available)
kargo plugin install ./plugins/native/kargo-mddoc
kargo plugin install ./plugins/native/kargo-turd
kargo plugin install ./plugins/native/kargo-mdlint
kargo plugin install ./plugins/native/kargo-sap
```

---

## Complete Plugins (4)

These plugins are fully implemented and ready to use:

### kargo-mddoc

Generate beautiful Markdown documentation from Rust crate APIs.

```bash
# Install
kargo plugin install ./plugins/native/kargo-mddoc

# Usage
kargo mddoc tokio                    # Document the tokio crate
kargo mddoc tokio@1.28.0             # Document specific version
kargo mddoc serde -o ./docs/serde    # Custom output directory
kargo mddoc --help                   # See all options
```

**Features:**
- Converts rustdoc JSON to clean, readable Markdown
- Supports any crate from crates.io
- Customizable output locations
- Option to keep intermediate JSON files

### kargo-turd

Find stubbed, incomplete, and non-production quality code in your Rust projects.

```bash
# Install  
kargo plugin install ./plugins/native/kargo-turd

# Usage
kargo turd                           # Analyze current project
kargo turd --watch .                 # Watch mode: re-analyze on file changes
kargo turd --exclude "tests/**"      # Exclude patterns
kargo turd --help                    # See all options
```

**Features:**
- Detects TODO, FIXME, unimplemented!(), todo!(), unreachable!()
- Finds stub implementations and incomplete code
- Generates task files for LLM-assisted remediation
- Watch mode for continuous analysis
- AST-based analysis for accuracy

### kargo-mdlint

Fast markdown linter powered by mado.

```bash
# Install
kargo plugin install ./plugins/native/kargo-mdlint

# Usage
kargo mdlint README.md               # Lint a single file
kargo mdlint docs/                   # Lint a directory
kargo mdlint --help                  # See all options
```

**Features:**
- Fast markdown validation
- Checks for common markdown issues
- Style violation detection
- Powered by the mado library

### kargo-sap

Smart Agent Protocol - AI-enhanced directory listing optimized for LLM agents.

```bash
# Install
kargo plugin install ./plugins/native/kargo-sap

# Usage
kargo sap                            # List current directory
kargo sap /path/to/dir               # List specific directory
kargo sap --help                     # See all options
```

**Features:**
- Intelligent directory listing for AI agents
- Structured output format
- Optimized for LLM consumption

---

## Plugins In Development (3)

The following plugins are partially implemented and cannot yet be installed. See task files in `/tasks/` for implementation details.

### kargo-kurate

**Status:** Library only, needs plugin interface  
**Task:** `/tasks/PLUG11_kargo_kurate_plugin.md`

Cargo execution orchestrator with LLM-friendly output processing.

**Planned Usage:**
```bash
kargo kurate build --release   # Run cargo with processed output
kargo kurate test             # Run tests with LLM-optimized formatting
```

**Features:**
- Wraps any cargo command
- Processes output for LLM consumption
- Highlights errors and warnings
- Summarizes large outputs
- Async execution support

### kargo-upgrade

**Status:** Library only, needs plugin interface  
**Task:** `/tasks/PLUG12_kargo_upgrade_plugin.md`

Automated dependency upgrade tool.

**Planned Usage:**
```bash
kargo upgrade                     # Check for dependency updates
kargo upgrade --compatible-only   # Only non-breaking updates
kargo upgrade --apply             # Auto-apply updates
```

**Features:**
- Scans Cargo.toml and other dependency sources
- Checks crates.io for latest versions
- Respects semver compatibility
- Supports workspaces
- Can preview or auto-apply updates

### kargo-walk

**Status:** Binary tool, needs conversion to plugin  
**Task:** `/tasks/PLUG13_kargo_walk_plugin.md`

Project discovery and inventory generator.

**Planned Usage:**
```bash
kargo walk                        # Scan current directory
kargo walk /path/to/scan          # Scan specific path
kargo walk --output inventory.yaml # Custom output file
```

**Features:**
- Discovers all Rust projects in directory
- Extracts project metadata
- Checks build status
- Analyzes project relationships
- Generates YAML inventory

**Contributing:** These plugins need to be completed. See the task files for detailed implementation instructions.

## Usage Examples

### Using Kargo as Cargo

Kargo forwards all standard cargo commands:

```bash
kargo build                          # Same as cargo build
kargo test --release                 # Same as cargo test --release
kargo run --bin myapp                # Same as cargo run --bin myapp
```

### Using the Alias Mode

Start a shell with `cargo` aliased to `kargo`:

```bash
kargo --alias
# Now in the new shell, 'cargo' actually runs 'kargo'
cargo build  # Really running kargo build
```

### Combining Plugins with Cargo

```bash
# Generate docs for your dependencies, then build
kargo mddoc serde -o ./docs/serde
kargo build

# Lint your markdown docs, then run tests
kargo mdlint README.md
kargo test

# Find code turds, fix them, then check
kargo turd
# ... fix issues ...
kargo check
```

## Building from Source

### Prerequisites

- Rust 1.75 or later (edition 2024)
- cargo

### Build All Plugins

```bash
# Build kargo and all plugins in release mode
cargo build --workspace --release

# The main binary
./target/release/kargo

# Plugin libraries are in target/release/
# They will be dynamically loaded when installed via `kargo plugin install`
```

### Development Build

```bash
# Build in debug mode
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace --all-targets
```

## Contributing

### Creating a Plugin

1. **Create a new library crate:**

```bash
cargo new --lib kargo-myplugin
```

2. **Add dependencies to `Cargo.toml`:**

```toml
[dependencies]
kargo-plugin-api = { git = "https://github.com/cyrup-ai/kargo" }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
tokio = { version = "1", features = ["full"] }

[lib]
crate-type = ["cdylib", "rlib"]
```

3. **Implement the `PluginCommand` trait:**

```rust
use kargo_plugin_api::{PluginCommand, ExecutionContext, BoxFuture};
use clap::Command;

pub struct MyPlugin;

impl PluginCommand for MyPlugin {
    fn clap(&self) -> Command {
        Command::new("myplugin")
            .about("My awesome plugin")
            .arg(/* ... */)
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture<anyhow::Result<()>> {
        Box::pin(async move {
            // Your plugin logic here
            Ok(())
        })
    }
}

// Export the plugin for dynamic loading
kargo_plugin_api::export_plugin!(MyPlugin);
```

4. **Test locally:**

```bash
cargo build --release
kargo plugin install ./target/release
```

### Plugin Guidelines

- Use async/await for I/O operations
- Use tokio for the async runtime
- Provide helpful error messages with anyhow
- Use clap for CLI argument parsing
- Follow Rust naming conventions
- Write tests for your plugin logic

## License

[Add your license here]

## Authors

Kargo Contributors
