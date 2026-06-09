<div align="center">

# ⚡ CHRONO

### El emulador de terminal que entiende lo que ves

**Construido desde cero en Rust 🦀 — rápido, semántico y hermoso**

[![CI](https://github.com/zeltri/CHRONO/actions/workflows/ci.yml/badge.svg)](https://github.com/zeltri/CHRONO/actions)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-67%20passing-brightgreen.svg)](#-desarrollo)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey.svg)](#%EF%B8%8F-instalación)

[Características](#-características) • [Instalación](#%EF%B8%8F-instalación) • [Uso](#-uso) • [Configuración](#%EF%B8%8F-configuración) • [Arquitectura](#%EF%B8%8F-arquitectura) • [Desarrollo](#-desarrollo)

---

*La mayoría de las terminales dibujan texto.*
***CHRONO lo interpreta:*** *errores en rojo, stack traces clickeables, JSON con sintaxis,
sugerencias de tu historial mientras escribes — sin plugins, sin configurar nada.*

</div>

---

## 🌌 ¿Por qué CHRONO?

| | Terminal clásica | ⚡ CHRONO |
|---|---|---|
| Un error en el output | Texto plano | 🔴 Resaltado automático |
| Un stack trace | Lo copias a mano | 🖱️ Click → se abre en tu editor |
| Un JSON gigante | Muro de texto | 🎨 Syntax highlighting en vivo |
| Repetir un comando | `Ctrl+R` y rezar | 💭 Sugerencia fantasma al estilo Fish |
| `vim`, `htop`, `less` | — | ✅ Alt screen + mouse tracking reales |
| Buscar ese comando de hace rato | Scroll infinito a ciegas | 🧭 `Ctrl+Shift+↑` salta entre prompts |
| Varias sesiones | Varias ventanas | 🗂️ Tabs con `Ctrl+Shift+T` |

Todo sobre una base VT sólida: **CHRONO habla el protocolo de verdad**, no una aproximación.

---

## ✨ Características

### 🧠 Terminal semántica

- 🔴 **Detección de errores y warnings** — `Error:`, `Exception:`, `Traceback`, resaltados al instante
- 🖱️ **Stack traces navegables** — click en `src/main.rs:42:10` y se abre en tu editor (multi-lenguaje: Rust, Python, Java, JS…)
- 🎨 **Output tipado** — logs estructurados (INFO/WARN/ERROR/DEBUG con su color), JSON coloreado, tablas ASCII
- 📁 **Listados inteligentes** — directorios, ejecutables, archivos comprimidos e imágenes, cada uno con su color
- 💭 **Sugerencias de historial** — autocompletado fantasma en gris desde tu `.zsh_history`/`.bash_history`; acepta con `Tab` o `→` ([docs](docs/SUGGESTIONS.md))
- 📖 Más detalles en [Smart Rendering](docs/SMART_RENDERING.md)

### 🖥️ Emulación VT de verdad

- ✅ **Pantalla alternativa** (modos 47/1047/1049) — `vim`, `htop` y `less` entran y salen sin romper tu sesión
- ✅ **Regiones de scroll** (DECSTBM), inserción/borrado de líneas y caracteres (IL/DL/ICH/DCH/ECH)
- ✅ **Borrados completos** — ED 0/1/2/3 y EL 0/1/2 con Background Color Erase
- ✅ **Responde consultas** — DA, DSR y CPR; las apps que preguntan dónde está el cursor obtienen respuesta
- ✅ **Bracketed paste** (2004) — pegar código en vim sin autoindentación fantasma
- ✅ **Cursor guardado/restaurado** (DECSC/DECRC), índices RI/IND/NEL, reset completo (RIS)
- ✅ **Mouse tracking** (modos 9/1000/1002/1003 + SGR 1006) — click y rueda funcionan en `htop`, `vim` y tmux
- ✅ **Hyperlinks OSC 8** — `ls --hyperlink` y compiladores modernos emiten enlaces clickeables de verdad
- ✅ **Título de ventana dinámico** vía OSC 0/2
- 🌈 **Color total** — 16 ANSI + paleta xterm-256 real (cubo 6×6×6 + grises) + true color 24-bit, video inverso incluido
- ✍️ **Estilos reales** — bold, italic, underline, strikethrough y dim, renderizados de verdad
- 🈶 **Unicode completo** — CJK de doble ancho con fallback automático a fuentes del sistema

### 🎨 Experiencia

- 🗂️ **Tabs** — múltiples sesiones en una ventana (`Ctrl+Shift+T`), con barra elegante y click para cambiar
- 📜 **Scrollback navegable** — rueda del mouse y `Shift+PageUp/Down`, con indicador de posición
- 🧭 **Navegación por prompts** — `Ctrl+Shift+↑/↓` salta entre comandos (vía OSC 133)
- 🔀 **Reflow real** — al redimensionar, las líneas se re-envuelven al nuevo ancho en vez de truncarse
- 🖱️ **Selección inteligente** — doble click selecciona palabra, triple click la línea
- 🌃 **Tema OneDark Pro** — paleta profesional lista para vivir en ella
- ✍️ **Cascadia Code** embebida + fallback automático a fuentes del sistema para CJK y símbolos
- 📋 **Clipboard nativo** — `Ctrl+Shift+C/V` con soporte Wayland (`wl-copy`) y X11 (`xclip`/`xsel`)
- ⚡ **Cache de glifos** — cada carácter se rasteriza una sola vez; el render vuela
- 🔒 **Respeta tu sistema** — jamás toca tus dotfiles; toda la configuración va por variables de entorno

### 🗺️ Roadmap

- 🔲 Splits (paneles divididos en una misma tab)
- 🔲 Renderizado GPU con `wgpu` + glyph atlas
- 🔲 Soporte completo Windows + empaquetado (`.deb`, AppImage, Homebrew)
- 🔲 Protocolo de imágenes (Sixel / Kitty graphics)

---

## 🛠️ Instalación

### Requisitos

- **Rust 1.70+** → [rustup.rs](https://rustup.rs/)
- **Linux** o **macOS** (Windows experimental)

### Desde el código fuente

```bash
git clone https://github.com/zeltri/CHRONO.git
cd CHRONO
cargo build --release

./target/release/terminal
```

### Instalación en el sistema

```bash
# A ~/.cargo/bin
cargo install --path crates/app

# O copia directa
sudo cp target/release/terminal /usr/local/bin/chrono
```

---

## 🚀 Uso

```bash
cargo run --release    # desde el repo
terminal               # si lo instalaste
```

### Atajos de teclado

| Atajo | Acción |
| --- | --- |
| `Ctrl+Shift+T` | Nueva tab |
| `Ctrl+Shift+W` | Cerrar tab |
| `Ctrl+Shift+←/→` | Cambiar de tab (o click en la barra) |
| `Ctrl+Shift+↑/↓` | Saltar al prompt anterior / siguiente |
| Rueda / `Shift+PageUp/Down` | Navegar el scrollback |
| Doble / triple click | Seleccionar palabra / línea |
| `Ctrl+Shift+C` | Copiar selección |
| `Ctrl+Shift+V` | Pegar (con bracketed paste si la app lo pide) |
| `Tab` / `→` | Aceptar sugerencia de historial |
| `Shift+click` | Selección normal aunque la app capture el mouse |

### Pruébalo

```bash
# Errores resaltados automáticamente
cargo build            # los errores de compilación se pintan solos

# Stack traces clickeables
python script.py       # click en script.py:15 → se abre en tu editor

# Listados con color semántico
ls -la                 # azul = directorios, verde = ejecutables, rojo = comprimidos

# JSON con sintaxis
curl -s https://api.github.com/repos/zeltri/CHRONO | head -20
```

---

## ⚙️ Configuración

Archivo: `~/.config/terminal-emulator/config.toml` (ver [config.example.toml](config.example.toml))

```toml
[font]
family = "Cascadia Code"
size = 14

[colors]
foreground = [171, 178, 191]  # #abb2bf
background = [40, 44, 52]     # #282c34

[terminal]
scrollback = 10000

[shell]
program = "/bin/zsh"          # o bash, fish...
```

---

## 🏗️ Arquitectura

Cinco crates, responsabilidad única, cero dependencias circulares:

```
CHRONO/
├── crates/
│   ├── core/        🧠  Screen buffer, celdas, cursor, regiones de scroll,
│   │                    alt screen, detección semántica, historial
│   ├── ansi/        📜  Parser VT sobre `vte` — CSI, OSC, SGR, modos privados,
│   │                    y respuestas al PTY (DA/DSR/CPR)
│   ├── pty/         🔌  Shell real vía `portable-pty`, writer thread-safe
│   ├── renderer/    🎨  Rasterización CPU con `fontdue` + cache de glifos,
│   │                    tema OneDark Pro, tab bar, fallback de fuentes
│   └── app/         🪟  Event loop con `winit` + `softbuffer`: tabs,
│   │                    mouse tracking, scrollback, input, clipboard
│
├── .github/         🤖  CI: fmt + clippy + tests en cada push
├── docs/            📖  Guías de smart rendering y sugerencias
├── examples/        🧪  Ejemplos de uso
└── benches/         ⏱️  Benchmarks de renderizado
```

```
            app
         ┌───┼────────┐
     renderer  ansi  pty
         └───┬───┘
            core
```

**Principios:** separación de responsabilidades · Rust seguro en la lógica core · testeable por módulo · el parser nunca bloquea al render.

---

## 🧪 Desarrollo

```bash
cargo test --all                 # 67 tests ✅
cargo clippy --all-targets       # lint
cargo fmt --all                  # formato
cargo bench                      # benchmarks
cargo run --example colored      # demos
```

| Crate | Cobertura |
| --- | --- |
| `terminal-core` | Screen, scroll regions, alt screen, resize, selección, Unicode |
| `terminal-ansi` | SGR, ED/EL, IL/DL/ICH/DCH, DECSTBM, OSC, bracketed paste, CPR |
| `terminal-pty` | Spawn, I/O, resize |
| `terminal-renderer` | Tema, paleta 256, pipeline de glifos |

### Contribuir

Los PRs son bienvenidos: temas nuevos, fixes, rendimiento, tests, docs.
Regla de oro: `cargo fmt` + `cargo clippy` limpios y tests para lo nuevo.

---

## 📚 Referencias

- [XTerm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) — la biblia de las secuencias
- [VT100.net](https://vt100.net/) — especificaciones históricas
- Construido sobre [portable-pty](https://docs.rs/portable-pty/), [vte](https://docs.rs/vte/), [winit](https://docs.rs/winit/), [fontdue](https://docs.rs/fontdue/), [softbuffer](https://docs.rs/softbuffer/)

## 🙏 Agradecimientos

- **Alacritty, Kitty y WezTerm** — los gigantes sobre cuyos hombros nos paramos
- **Microsoft** — por Cascadia Code
- **La comunidad Rust** — por el mejor ecosistema para construir esto

## 📄 Licencia

[MIT](LICENSE) © 2026

---

<div align="center">

**⭐ Si CHRONO te voló la cabeza, una estrella ayuda a que más gente lo descubra ⭐**

*Hecho con ❤️ y 🦀 — porque tu terminal merece entenderte*

</div>
