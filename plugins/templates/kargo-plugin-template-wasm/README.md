# {{plugin_name}} (WASM)

{{plugin_description}}

## Development

This is a kargo WASM plugin created from the kargo-plugin-template-wasm.

### Prerequisites

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Optionally install wasm-opt for optimization
cargo install wasm-opt
```

### Building

```bash
cargo build --release --target wasm32-unknown-unknown
```

The plugin will be built as: `target/wasm32-unknown-unknown/release/kargo_{{plugin_name}}_wasm.wasm`

### Optimizing (Optional)

```bash
wasm-opt -Oz \
  target/wasm32-unknown-unknown/release/kargo_{{plugin_name}}_wasm.wasm \
  -o kargo_{{plugin_name}}.wasm
```

### Installing

Copy the `.wasm` file to one of kargo's plugin directories:
- User plugins: `~/.config/kargo/plugins/`
- Local plugins: `.kargo/plugins/`
- Or set `KARGO_PLUGIN_PATH` environment variable

### Testing

```bash
# After installing the plugin
kargo {{plugin_name}} --help
```

## Benefits of WASM

- **Safety**: Runs in sandboxed environment
- **Portability**: Same binary works on all platforms
- **Isolation**: No direct access to filesystem or network
- **Memory safety**: Cannot corrupt host memory

## License

[Your license here]