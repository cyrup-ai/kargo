# rustdoc-md

A tool to generate Markdown documentation for any Rust package and version. This tool extracts documentation from Rust packages and converts it to readable Markdown format.

## Overview

rustdoc-md creates Markdown documentation from any Rust crate's API, which can be used for:

- Creating user-friendly API documentation
- Building custom documentation sites
- Including Rust API documentation in other documents
- Learning and exploring Rust APIs
- And more!

## Requirements

- Rust and Cargo must be installed
- Internet connection (to fetch packages)

The tool will automatically handle:
- Installing or updating the nightly toolchain if needed
- Installing required rustdoc components
- Intelligently caching updates (only updates once every 24 hours)

## Installation

### From crates.io

```bash
cargo install rustdoc-md
```

### From source

```bash
git clone https://github.com/yourusername/rustdoc-md.git
cd rustdoc-md
cargo install --path .
```

## Usage

### Basic Usage

Generate Markdown documentation for the latest version of a package:

```bash
rustdoc-md tokio
```

Generate documentation for a specific version:

```bash
rustdoc-md tokio@1.28.0
```

### Command-line Options

```
USAGE:
    rustdoc-md [OPTIONS] PACKAGE[@VERSION]

ARGS:
    <PACKAGE[@VERSION]>    Package name with optional version (e.g., 'tokio' or 'tokio@1.28.0')

OPTIONS:
    -o, --output <DIR>            Output directory for documentation [default: ./rust_docs]
    -j, --keep-json               Keep JSON documentation files (normally deleted after markdown conversion)
    --json-only                   Skip Markdown generation and only output JSON
    -k, --keep-temp               Keep temporary directory after completion
    --temp-dir <DIR>              Use specific temporary directory
    --skip-component-check        Skip checking/installing rustup components
    --document-private-items      Include private items in documentation
    -v, --verbose                 Enable verbose output
    -h, --help                    Print help information
    -V, --version                 Print version information
```

### Examples

```bash
# Generate markdown for latest serde version
rustdoc-md serde

# Generate docs for a specific version and save to custom directory
rustdoc-md -o ./my_docs tokio@1.28.0

# Keep both JSON and Markdown output
rustdoc-md -j tokio

# Generate only JSON documentation (no Markdown)
rustdoc-md --json-only tokio

# Generate documentation including private items
rustdoc-md --document-private-items tokio
```

## How It Works

1. The tool intelligently manages the Rust toolchain, automatically installing or updating when needed
2. It creates a temporary Rust project with the target package as a dependency
3. It runs the unstable rustdoc JSON generator on the package
4. It converts the JSON documentation to Markdown format
5. It cleans up temporary files (unless instructed to keep them)

### Re-exports and Documentation

The tool properly handles re-exports (e.g., `tokio::spawn` which re-exports `tokio::task::spawn`):

- **Same-crate re-exports**: Automatically resolved by following references within the JSON
- **Foreign-crate re-exports**: Currently shown as simple re-export declarations

rustdoc JSON represents re-exports as `ItemEnum::Use` items containing:
- `source`: The full path being imported
- `name`: The name of the imported item  
- `id`: Optional ID of the imported item (for same-crate re-exports)
- `glob`: Whether this is a glob import (`use foo::*`)

For foreign-crate re-exports, the tool generates special links that enable lazy documentation loading:
- Links use a custom protocol: `kargo-mddoc://crate/version/path`
- Documentation is generated on-demand when links are accessed
- All inputs are validated to prevent security issues:
  - Crate names must match crates.io naming rules
  - Versions must be valid semver
  - Path components are sanitized against directory traversal

## Output

The tool generates documentation files with the following naming convention:

- Markdown: `PACKAGE_NAME.md` or `PACKAGE_NAME-VERSION.md`
- JSON (if kept): `PACKAGE_NAME.json` or `PACKAGE_NAME-VERSION.json`

## Using the Library

This tool can also be used as a library in your Rust projects:

```rust
use rustdoc_md::{Config, DocGenerator, Error};
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    let config = Config {
        package_spec: "tokio@1.28.0".to_string(),
        output_dir: PathBuf::from("./docs"),
        temp_dir: None,
        keep_temp: false,
        skip_component_check: false,
        verbose: true,
        document_private_items: false,
    };

    // Generate JSON
    let mut generator = DocGenerator::new(config)?;
    let json_file = generator.run()?;
    
    // Convert to Markdown
    let md_file = rustdoc_md::markdown::convert_to_markdown(&json_file)?;
    
    println!("Documentation generated at: {}", md_file.display());
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **Internet Connection**: Ensure you have an active internet connection for package fetching
2. **Disk Space**: Make sure you have sufficient disk space for temporary files
3. **Permission errors**: Ensure you have write permissions for the output directory
4. **Package Availability**: Verify the package name and version exist on crates.io

## License

MIT License

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests.