# {{plugin_name}} (Node.js)

{{plugin_description}}

## Development

This is a kargo Node.js plugin that compiles to WASM.

### Prerequisites

```bash
# Install Node.js 16+
# https://nodejs.org/

# Install dependencies
npm install
```

### Building to WASM

```bash
# Using Javy (JavaScript to WASM compiler)
npm run build:wasm
```

This creates `{{plugin_name}}.wasm` in the current directory.

### Testing Locally

```bash
npm test
```

### Installing

Copy the `.wasm` file to one of kargo's plugin directories:
- User plugins: `~/.config/kargo/plugins/`
- Local plugins: `.kargo/plugins/`

## Node.js Plugin Benefits

- Huge ecosystem (npm)
- Familiar JavaScript syntax
- Quick development cycle
- Easy file system and network operations

## Alternative WASM Compilers

- **Javy**: Shopify's JavaScript to WASM compiler (recommended)
- **wasm-pack**: For TypeScript projects
- **AssemblyScript**: TypeScript-like language that compiles to WASM

## License

[Your license here]