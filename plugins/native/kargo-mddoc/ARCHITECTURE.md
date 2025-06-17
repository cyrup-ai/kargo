# kargo mddoc Architecture

## Overview

`kargo mddoc` is a plugin for the kargo build system that generates Markdown documentation from Rust crates using rustdoc's experimental JSON output format.

## Core Components

### 1. Plugin Interface (`src/plugin.rs`)
- Implements the `PluginCommand` trait from kargo
- Handles command-line argument parsing using clap
- Orchestrates the documentation generation pipeline

### 2. Documentation Generator (`src/kargo/generator.rs`)
- Manages the rustdoc JSON generation process
- Creates temporary Rust projects with target crate as dependency
- Executes `cargo rustdoc --output-format json` with nightly toolchain
- Handles cleanup of temporary files

### 3. Markdown Converter (`src/kargo/markdown.rs`)
- Parses rustdoc JSON using `rustdoc-types` crate
- Converts JSON AST to human-readable Markdown
- Handles cross-references and type resolution

### 4. Toolchain Management (`src/kargo/toolchain.rs`)
- Automatically installs/updates nightly Rust toolchain
- Manages rustdoc components
- Caches updates (only checks once per 24 hours)

## Re-export Handling

### Same-Crate Re-exports
Re-exports within the same crate (e.g., `tokio::spawn` → `tokio::task::spawn`) are handled by:
1. Detecting `ItemEnum::Use` variants in the JSON
2. Following the `id` reference to find the original item
3. Rendering the full documentation at the re-export location

### Foreign-Crate Re-exports (Lazy Loading Design)
For re-exports from external crates, we use a secure lazy-loading approach:

1. **Detection**: Identify foreign re-exports during JSON parsing
2. **Link Generation**: Create special markdown links with custom protocol:
   ```markdown
   [`spawn`](kargo-mddoc://futures/0.3.28/spawn)
   ```
3. **Security**: Links are validated against:
   - Crate name must match `^[a-zA-Z0-9_-]+$`
   - Version must be valid semver
   - Path components sanitized to prevent directory traversal
   - No shell metacharacters allowed
4. **Lazy Resolution**: When user clicks, a handler:
   - Validates the request
   - Checks local cache for existing docs
   - Generates documentation on-demand if needed
   - Returns the specific item documentation

### Security Considerations

1. **Input Validation**:
   - All crate names validated against crates.io naming rules
   - Version strings must be valid semver
   - Path components checked for directory traversal attempts

2. **Process Isolation**:
   - Documentation generation runs in temporary directories
   - No execution of crate code, only documentation extraction
   - Cleanup of temporary files after generation

3. **Link Safety**:
   - Custom protocol prevents accidental web navigation
   - All external links sanitized
   - No JavaScript execution in generated Markdown

## JSON Structure

rustdoc JSON key structures we handle:

```rust
// Re-export representation
ItemEnum::Use(Use {
    source: String,      // e.g., "self::task::spawn" or "futures::spawn"
    name: String,        // e.g., "spawn"
    id: Option<Id>,      // Present for same-crate items
    glob: bool,          // true for `use foo::*`
})
```

## Future Enhancements

1. **Parallel Foreign Documentation**:
   - Batch foreign crate documentation requests
   - Generate multiple crate docs in parallel
   - Cache results for reuse

2. **Incremental Updates**:
   - Detect changes in crate versions
   - Only regenerate changed documentation

3. **Cross-Crate Search**:
   - Index all generated documentation
   - Provide unified search across documented crates