# {{plugin_name}}

{{plugin_description}}

## Development

This is a kargo plugin created from the kargo-plugin-template.

### Building

```bash
cargo build --release
```

The plugin will be built as a dynamic library in `target/release/`:
- Linux: `libkargo_{{plugin_name}}.so`
- macOS: `libkargo_{{plugin_name}}.dylib`
- Windows: `kargo_{{plugin_name}}.dll`

### Installing

Copy the built library to one of kargo's plugin directories:
- User plugins: `~/.config/kargo/plugins/`
- Local plugins: `.kargo/plugins/`
- Or set `KARGO_PLUGIN_PATH` environment variable

### Testing

```bash
# After installing the plugin
kargo {{plugin_name}} --help
```

## Usage

```bash
kargo {{plugin_name}} [OPTIONS]
```

### Options

- `-e, --example <VALUE>` - An example argument

## License

[Your license here]