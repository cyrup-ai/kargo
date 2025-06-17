# {{plugin_name}} (Go)

{{plugin_description}}

## Development

This is a kargo Go plugin that compiles to WASM using TinyGo.

### Prerequisites

```bash
# Install Go
# https://golang.org/doc/install

# Install TinyGo
# https://tinygo.org/getting-started/install/
```

### Building

```bash
make build
```

This creates `{{plugin_name}}.wasm` in the current directory.

### Installing

```bash
make install
```

Or manually copy to:
- User plugins: `~/.config/kargo/plugins/`
- Local plugins: `.kargo/plugins/`

### Testing

```bash
go test ./...
```

## Go Plugin Benefits

- Strong typing
- Excellent performance
- Good standard library
- Compiles to efficient WASM with TinyGo

## License

[Your license here]