<div align="center">

# 🚀 CHRONO Terminal Emulator

**Un emulador de terminal moderno, rápido y elegante escrito en Rust**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Configuration](#-configuration) • [Development](#-development)

</div>

---

## 📖 Overview

CHRONO es un emulador de terminal construido desde cero en Rust, diseñado para desarrolladores que buscan una terminal moderna con:

-   🎨 **OneDark Pro Theme** - Colores oscuros innovadores y profesionales
-   ⚡ **Alto Rendimiento** - Arquitectura modular optimizada
-   🔍 **Context-Aware** - Detección inteligente de errores, warnings y archivos
-   🎯 **Stack Traces Navegables** - Click para abrir archivos en tu editor
-   🌈 **Full ANSI Support** - Colores 24-bit, estilos y secuencias VT
-   📦 **Arquitectura Modular** - Crates independientes y testeables
-   💡 **Smart Suggestions** - Autocompletado visual en gris claro (Fish/Zsh style)

---

## ✨ Features

### ✅ Implementado (Fases 1-2)

#### Core Functionality

-   ✅ **PTY Real** - Ejecución de shell mediante `portable-pty`
-   ✅ **ANSI/VT Parser** - Interpretación completa de secuencias de escape
-   ✅ **Screen Buffer** - Grid bidimensional con scrollback configurable
-   ✅ **Unicode Support** - Caracteres CJK, emojis y ancho variable

#### Terminal Features

-   ✅ **Colores ANSI** - 16 colores indexados + 256 colores + RGB true color (24-bit)
-   ✅ **Atributos de Texto** - Bold, italic, underline, strikethrough, inverse
-   ✅ **Movimiento de Cursor** - Todas las secuencias VT de navegación
-   ✅ **Borrado** - Screen clear, line clear, scrolling regions

#### Input System

-   ✅ **Teclado Completo** - Teclas especiales, modificadores, funciones F1-F12
-   ✅ **Clipboard Integration** - Copy/paste con `Ctrl+Shift+C/V`
-   ✅ **Shortcuts** - `Ctrl+C`, `Ctrl+D`, `Ctrl+L`, etc.

#### Smart Features (NEW! ⭐)

-   ✅ **Autocompletado Visual** - Sugerencias en gris claro al estilo Fish/Zsh
    -   🔹 Secuencias ANSI personalizadas (ESC[53m/54m)
    -   🔹 Limpieza automática al escribir
    -   🔹 Color gris sutil y configurable
    -   🔹 API pública completa en Rust
    -   📖 [Ver guía completa](SUGGESTIONS_GUIDE.md)
-   ✅ **Sugerencias Inteligentes de Historial** - Como Kitty Terminal 🆕
    -   🔹 Busca automáticamente en comandos previos
    -   🔹 Sugiere basándose en primeras letras
    -   🔹 Acepta con Tab o flecha derecha
    -   🔹 Actualización en tiempo real
    -   📖 [Ver documentación](HISTORIAL_SUGGESTIONS.md)

#### Smart Features

-   ✅ **Context Detection** - Detección automática de:
    -   🔴 **Errores** (Error:, Exception:, Traceback:)
    -   🟡 **Warnings** (Warning:, WARN:)
    -   📁 **File Listings** (ls output con colorización inteligente)
    -   📍 **Stack Traces** (file.rs:42:10 con click navigation)
-   ✅ **File Type Colors** - Colorización semántica de directorios, ejecutables, archivos
-   ✅ **Smart Rendering** - Diferente renderizado según contexto de línea

#### Rendering & UI

-   ✅ **CPU Renderer** - Optimizado con `fontdue` para rasterización
-   ✅ **Modern Theme** - OneDark Pro color palette
-   ✅ **Dynamic Resize** - Redimensionamiento suave de ventana
-   ✅ **Animated Cursor** - Cursor pulsante moderno
-   ✅ **Cascadia Code** - Fuente monoespaciada con ligaduras

#### Configuration

-   ✅ **TOML Config** - `~/.config/terminal-emulator/config.toml`
-   ✅ **Customizable** - Font, colors, size, scrollback

### 🚧 Roadmap (Fase 3)

-   🔲 **GPU Rendering** - Aceleración con `wgpu` + glyph atlas
-   🔲 **Tabs/Splits** - Múltiples sesiones en una ventana
-   🔲 **Image Protocol** - Sixel o protocolo similar
-   🔲 **Multiplataforma** - Soporte completo Windows + macOS
-   🔲 **Packaging** - `.deb`, `.rpm`, `AppImage`, Homebrew formula

---

## 🛠️ Installation

### Prerequisites

-   **Rust 1.70+** - [Install Rust](https://rustup.rs/)
-   **Linux** or **macOS** (Windows experimental)
-   **Cascadia Code Font** - [Download](https://github.com/microsoft/cascadia-code/releases)

### Build from Source

```bash
# Clone repository
git clone https://github.com/yoezequiel/CHRONO.git
cd CHRONO

# Build release
cargo build --release

# Binary location
./target/release/terminal
```

### Install System-wide

```bash
# Install to ~/.cargo/bin
cargo install --path crates/app

# Or copy binary
sudo cp target/release/terminal /usr/local/bin/
```

---

## 🚀 Usage

### Run Terminal

```bash
# From source
cargo run

# Or installed binary
terminal
```

### Keyboard Shortcuts

| Shortcut       | Action                  |
| -------------- | ----------------------- |
| `Ctrl+Shift+C` | Copy selection          |
| `Ctrl+Shift+V` | Paste from clipboard    |
| `Ctrl+C`       | Send SIGINT (interrupt) |
| `Ctrl+D`       | Send EOF                |
| `Ctrl+L`       | Clear screen            |
| `Ctrl+Z`       | Suspend process         |

### Smart Features Usage

**Error Detection:**

```bash
# Errors are automatically highlighted in red
echo "Error: Something went wrong"
rustc bad_file.rs  # Compilation errors highlighted
```

**Stack Trace Navigation:**

```bash
# Click on file references to open in editor
cargo build  # Click on src/main.rs:42:10
python script.py  # Click on script.py line 15
```

**Colorized File Listings:**

```bash
ls -la  # Directories in blue, executables in green, archives in red
```

---

## ⚙️ Configuration

Configuration file: `~/.config/terminal-emulator/config.toml`

### Example Config

```toml
[font]
family = "Cascadia Code"
size = 14

[colors]
# OneDark Pro theme (default)
foreground = [171, 178, 191]  # #abb2bf
background = [40, 44, 52]     # #282c34

[terminal]
rows = 24
cols = 80
scrollback = 10000  # Lines of scrollback buffer

[shell]
# Auto-detect or specify shell
program = "/bin/zsh"  # or "/bin/bash", "fish", etc.
```

### Theme Customization

```toml
[colors.ansi]
# Override specific ANSI colors
black = [40, 44, 52]
red = [224, 108, 117]
green = [152, 195, 121]
yellow = [229, 192, 123]
blue = [97, 175, 239]
magenta = [198, 120, 221]
cyan = [86, 182, 194]
white = [171, 178, 191]
```

---

## 🏗️ Architecture

### Workspace Structure

```
CHRONO/
├── crates/
│   ├── core/           # Terminal core logic
│   │   ├── screen.rs      # Screen buffer & grid
│   │   ├── cell.rs        # Cell attributes
│   │   ├── cursor.rs      # Cursor state
│   │   ├── context.rs     # Context detection
│   │   └── attributes.rs  # Colors & styles
│   │
│   ├── ansi/           # ANSI/VT parser
│   │   ├── handler.rs     # VTE handler
│   │   └── lib.rs         # Parser wrapper
│   │
│   ├── pty/            # PTY interface
│   │   └── lib.rs         # Shell spawning
│   │
│   ├── renderer/       # Rendering engine
│   │   ├── cpu.rs         # CPU renderer
│   │   └── theme.rs       # Color themes
│   │
│   └── app/            # Main application
│       ├── main.rs        # Event loop
│       └── config.rs      # Config management
│
├── examples/           # Usage examples
│   ├── simple.rs          # Basic usage
│   └── colored.rs         # Color demonstrations
│
├── benches/            # Performance benchmarks
│   └── rendering.rs       # Rendering benchmarks
│
└── .github/
    └── workflows/
        └── ci.yml         # CI/CD pipeline
```

### Module Dependencies

```
app
 ├─ renderer → core
 ├─ ansi → core
 ├─ pty
 └─ core
```

### Key Design Principles

1. **Separation of Concerns** - Each crate has a single responsibility
2. **No Circular Dependencies** - Clean dependency tree
3. **Zero-Copy Where Possible** - Minimize allocations
4. **Safe Rust** - No unsafe code in core logic
5. **Testable** - Unit tests for each module

---

## 🧪 Development

### Running Tests

```bash
# All tests
cargo test --all

# Specific crate
cargo test -p terminal-core
cargo test -p terminal-renderer

# With output
cargo test -- --nocapture

# Watch mode (requires cargo-watch)
cargo watch -x test
```

### Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench rendering

# With baseline comparison
cargo bench -- --save-baseline main
```

### Running Examples

```bash
# Simple example
cargo run --example simple

# Colored output example
cargo run --example colored
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Generate docs
cargo doc --no-deps --all-features --open
```

### Development Workflow

1. **Create Feature Branch**

    ```bash
    git checkout -b feature/my-feature
    ```

2. **Write Code + Tests**

    ```bash
    # Implement feature
    # Add tests in same file or tests/ directory
    cargo test -p <crate-name>
    ```

3. **Format & Lint**

    ```bash
    cargo fmt --all
    cargo clippy --fix --allow-dirty
    ```

4. **Commit & Push**

    ```bash
    git add .
    git commit -m "feat: add my feature"
    git push origin feature/my-feature
    ```

5. **Create Pull Request**

---

## 📊 Testing Coverage

| Crate               | Tests        | Coverage           |
| ------------------- | ------------ | ------------------ |
| `terminal-core`     | 15 tests     | Core functionality |
| `terminal-ansi`     | 10 tests     | ANSI parsing       |
| `terminal-pty`      | 5 tests      | PTY operations     |
| `terminal-renderer` | 9 tests      | Rendering pipeline |
| **Total**           | **39 tests** | ✅ Passing         |

### Test Breakdown

-   **Screen Management**: Creation, resize, scrollback, clearing
-   **ANSI Parsing**: Colors, SGR, cursor movement, text attributes
-   **Context Detection**: Error/warning patterns, file references, stack traces
-   **Rendering**: Theme colors, glyph rendering, buffer management
-   **PTY**: Shell spawning, I/O operations, resizing

---

## 🤝 Contributing

Contributions are welcome! This project was developed with AI assistance - see [AGENTS.md](AGENTS.md) for the development methodology.

### Guidelines

1. **Follow Rust conventions** - Use `cargo fmt` and `cargo clippy`
2. **Write tests** - Add tests for new features
3. **Document code** - Use doc comments (`///` and `//!`)
4. **Update README** - If adding user-facing features
5. **Be respectful** - Follow code of conduct

### Areas for Contribution

-   🎨 **Themes** - New color schemes
-   🐛 **Bug Fixes** - Report or fix issues
-   📖 **Documentation** - Improve docs
-   ⚡ **Performance** - Optimization PRs welcome
-   🧪 **Tests** - Increase coverage
-   🌍 **Localization** - i18n support

---

## 📚 Resources

### Documentation

-   [Development Guide](DEVELOPMENT.md) - Detailed development setup
-   [AI Development Process](AGENTS.md) - How this was built with AI
-   [Requirements Document](Documento.md) - Original specifications

### Rust Terminal Ecosystem

-   [VT100 Sequences](https://vt100.net/) - Terminal escape codes
-   [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code) - Standard reference
-   [XTerm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) - Extended sequences

### Dependencies

-   [portable-pty](https://docs.rs/portable-pty/) - Cross-platform PTY
-   [vte](https://docs.rs/vte/) - VT escape sequence parser
-   [winit](https://docs.rs/winit/) - Window creation and events
-   [fontdue](https://docs.rs/fontdue/) - Font rasterization
-   [softbuffer](https://docs.rs/softbuffer/) - CPU rendering

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2026 Terminal Emulator Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction...
```

---

## 🙏 Acknowledgments

-   **Rust Community** - For excellent tooling and libraries
-   **Terminal Emulator Pioneers** - Alacritty, Kitty, WezTerm for inspiration
-   **AI Assistance** - GitHub Copilot (Claude Sonnet 4.5) for development
-   **Font Creators** - Microsoft for Cascadia Code

---

## 🔗 Links

-   **Repository**: [github.com/yoezequiel/CHRONO](https://github.com/yoezequiel/CHRONO)
-   **Issues**: [github.com/yoezequiel/CHRONO/issues](https://github.com/yoezequiel/CHRONO/issues)
-   **Discussions**: [github.com/yoezequiel/CHRONO/discussions](https://github.com/yoezequiel/CHRONO/discussions)

---

<div align="center">

**⭐ Si te gusta este proyecto, considera darle una estrella! ⭐**

Made with ❤️ and 🦀 Rust

</div>
