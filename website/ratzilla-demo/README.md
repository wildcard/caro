# Caro TUI Demo (Ratzilla)

Interactive terminal UI demo for Caro, powered by [Ratzilla](https://github.com/orhun/ratzilla).

This WebAssembly application provides an authentic terminal experience showcasing Caro's command generation and safety validation directly in the browser.

## Technology

- **Rust** - Memory-safe, blazing-fast implementation
- **Ratzilla** - Terminal UI framework for the web
- **Ratatui** - TUI widget library
- **WebAssembly** - Near-native browser performance
- **WebGL2** - Hardware-accelerated rendering

## Building

### Prerequisites

1. Install Rust: https://rustup.rs/
2. Add the WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. Install Trunk:
   ```bash
   cargo install trunk
   ```

### Development

```bash
cd website/ratzilla-demo

# Start development server with hot reload
trunk serve

# Build for production
trunk build --release
```

The built files will be output to `../public/ratzilla-demo/`.

### Production Build

For production deployment, the built WASM files need to be copied to the website's public directory:

```bash
# From the website directory
cd website

# Build the Ratzilla demo
cd ratzilla-demo
trunk build --release
cd ..

# Build the Astro site (includes the built WASM)
npm run build
```

## Features Demonstrated

### Command Generation
- Natural language to shell command conversion
- Interactive prompt with cursor support
- Autocomplete suggestions from sample prompts

### Safety Validation
- Real-time safety level assessment
- Color-coded risk indicators (Safe, Moderate, Dangerous, Critical)
- Blocked command detection

### Terminal Aesthetics
- Authentic TUI feel with Ratatui widgets
- Smooth animations and transitions
- Keyboard navigation support

## Integration with Astro

The demo is integrated into the Caro website via:

1. **RatzillaDemo.astro** - Astro component that:
   - Loads the WASM module dynamically
   - Shows loading state while WASM compiles
   - Falls back gracefully if WebAssembly is unsupported

2. **try-caro.astro** - Landing page featuring the demo

## File Structure

```
ratzilla-demo/
├── Cargo.toml          # Rust dependencies
├── Trunk.toml          # Trunk build configuration
├── index.html          # Entry point for Trunk
├── src/
│   └── lib.rs          # Main Ratzilla application
└── README.md           # This file
```

## Credits

- [Ratzilla](https://github.com/orhun/ratzilla) by [@orhun](https://github.com/orhun)
- [Ratatui](https://ratatui.rs/) - Rust TUI library
- [Trunk](https://trunkrs.dev/) - WASM web app bundler

## License

This demo is part of the Caro project and is licensed under AGPL-3.0.
