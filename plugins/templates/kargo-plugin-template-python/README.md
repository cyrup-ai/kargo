# {{plugin_name}} (Python)

{{plugin_description}}

## Development

This is a kargo Python plugin that compiles to WASM.

### Prerequisites

```bash
pip install -r requirements.txt
```

### Building to WASM

```bash
# Install Extism PDK
pip install -r requirements.txt

# Build the plugin
python build.py

# Or directly with extism-pdk
extism-pdk-python build {{plugin_name}}.py -o {{plugin_name}}.wasm
```

### Testing Locally

```bash
python {{plugin_name}}.py
```

### Installing

Copy the `.wasm` file to one of kargo's plugin directories:
- User plugins: `~/.config/kargo/plugins/`
- Local plugins: `.kargo/plugins/`

## Python Plugin Benefits

- Rapid development
- Rich ecosystem of libraries
- Easy to prototype
- Runs safely in WASM sandbox

## License

[Your license here]