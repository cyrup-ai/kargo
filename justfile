# Kargo Development Commands

# Default command - show available commands
default:
    @just --list

# Create a new native Rust plugin (high performance, full OS access)
new-plugin-native name:
    cargo generate --path plugins/templates/kargo-plugin-template --name {{name}} --destination plugins/native/

# Create a new Rust WASM plugin (sandboxed, safe)
new-plugin-rust-wasm name:
    cargo generate --path plugins/templates/kargo-plugin-template-wasm --name {{name}} --destination plugins/wasm/

# Create a new Python WASM plugin
new-plugin-python name:
    cargo generate --path plugins/templates/kargo-plugin-template-python --name {{name}} --destination plugins/wasm/

# Create a new TypeScript WASM plugin  
new-plugin-typescript name:
    cargo generate --path plugins/templates/kargo-plugin-template-typescript --name {{name}} --destination plugins/wasm/

# Create a new Node.js WASM plugin
new-plugin-node name:
    cargo generate --path plugins/templates/kargo-plugin-template-node --name {{name}} --destination plugins/wasm/

# Create a new Go WASM plugin
new-plugin-go name:
    cargo generate --path plugins/templates/kargo-plugin-template-go --name {{name}} --destination plugins/wasm/

# Build all workspace members
build:
    cargo build --workspace

# Build all in release mode
build-release:
    cargo build --workspace --release

# Check all workspace members
check:
    cargo fmt --all --check
    cargo check --workspace --message-format short --quiet

# Run tests
test:
    cargo nextest run

# Format code
fmt:
    cargo fmt --all

# Install cargo-generate if not present
install-tools:
    @command -v cargo-generate > /dev/null || cargo install cargo-generate

# Build a specific native plugin in release mode
build-plugin-native name:
    cd plugins/native/{{name}} && cargo build --release

# Build a specific WASM plugin in release mode
build-plugin-wasm name:
    cd plugins/wasm/{{name}} && cargo build --release --target wasm32-unknown-unknown

# Build all native plugins
build-plugins-native:
    for dir in plugins/native/*/; do \
        if [ -f "$dir/Cargo.toml" ]; then \
            echo "Building native plugin: $dir"; \
            (cd "$dir" && cargo build --release); \
        fi \
    done

# Build all WASM plugins
build-plugins-wasm:
    for dir in plugins/wasm/*/; do \
        if [ -f "$dir/Cargo.toml" ]; then \
            echo "Building WASM plugin: $dir"; \
            (cd "$dir" && cargo build --release --target wasm32-unknown-unknown); \
        fi \
    done

# Install kargo-cli locally
install:
    cargo install --path kargo-cli

# Clean build artifacts
clean:
    cargo clean