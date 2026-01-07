# Terminal EmulatorUn emulador de terminal moderno escrito en Rust, con soporte ANSI completo y rendering CPU optimizado.## Características✅ **Completado (Fase 1)**- Ejecución de shell a través de PTY- Comunicación bidireccional con subprocesos- Parser ANSI/VT con soporte para: - Colores básicos (16 colores indexados) - Colores RGB verdaderos (24-bit) - Atributos de texto (negrita, cursiva, subrayado, invertido) - Movimiento de cursor - Borrado de pantalla y líneas- Buffer de pantalla con scrollback- Redimensionamiento de ventana- Entrada completa de teclado: - Teclas especiales (Enter, Tab, Escape, Backspace, Delete) - Teclas de función (F1-F12) - Navegación (Home, End, PageUp, PageDown, flechas) - Modificadores (Ctrl, Alt, Shift) - Combinaciones especiales (Ctrl+C, Ctrl+D, Ctrl+L, etc.)- Portapapeles (Ctrl+Shift+V para pegar)- Rendering CPU con la fuente Cascadia Code- Sistema de configuración TOML- Soporte para caracteres Unicode de ancho variable (CJK, emojis)🚧 **Pendiente (Fase 2)**- Rendering GPU con wgpu- Tests completos- Empaquetado para distribución (.deb, AppImage)- Soporte completo para Windows y macOS## Requisitos- Rust 1.70 o superior- Linux (Ubuntu, Debian, Fedora, etc.) o macOS- Fuente Cascadia Code instalada en el sistema## Instalación`bash# Clonar el repositoriogit clone <repository-url>cd terminal# Compilar en modo releasecargo build --release# El binario estará en target/release/terminal`

## Uso

```bash
# Ejecutar directamente
cargo run

# O ejecutar el binario compilado
./target/release/terminal
```

### Configuración

El emulador busca la configuración en `~/.config/terminal-emulator/config.toml`.

Archivo de configuración de ejemplo:

```toml
[font]
family = "Cascadia Code"
size = 14

[colors]
foreground = [204, 204, 204]
background = [26, 27, 38]

[terminal]
rows = 24
cols = 80
scrollback = 10000
```

## Atajos de Teclado

| Atajo          | Acción                               |
| -------------- | ------------------------------------ |
| `Ctrl+Shift+V` | Pegar desde portapapeles             |
| `Ctrl+C`       | Enviar SIGINT (interrumpir programa) |
| `Ctrl+D`       | Enviar EOF                           |
| `Ctrl+L`       | Limpiar pantalla                     |

## Arquitectura

El proyecto está organizado en crates modulares:

-   **`core`**: Estructuras de datos centrales (Screen, Cell, Cursor, Attributes)
-   **`ansi`**: Parser ANSI/VT usando la biblioteca `vte`
-   **`pty`**: Wrapper para PTY usando `portable-pty`
-   **`renderer`**: Sistema de rendering CPU con `fontdue` y `softbuffer`
-   **`app`**: Aplicación principal con loop de eventos `winit`

## Tests

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests de un crate específico
cargo test -p terminal-core
cargo test -p terminal-ansi

# Ejecutar tests con output detallado
cargo test -- --nocapture
```

### Cobertura de Tests

-   **terminal-core**: 15 tests (Screen, Cell, Cursor, Scrollback, Context detection)
-   **terminal-ansi**: 10 tests (Parser ANSI, SGR, Colores, Movimiento)

## Desarrollo

### Estructura de Archivos

```
terminal/
├── crates/
│   ├── core/          # Modelo de datos
│   │   ├── attributes.rs
│   │   ├── cell.rs
│   │   ├── cursor.rs
│   │   ├── screen.rs
│   │   └── tests.rs
│   ├── ansi/          # Parser ANSI
│   │   ├── handler.rs
│   │   └── tests.rs
│   ├── pty/           # Gestión de PTY
│   ├── renderer/      # Rendering
│   │   └── cpu.rs
│   └── app/           # Aplicación
│       ├── config.rs
│       └── main.rs
├── fonts/
│   └── CascadiaCode.ttf
├── Cargo.toml
└── README.md
```

### Agregar Nuevas Funcionalidades

1. **Parser ANSI**: Modificar `crates/ansi/src/handler.rs`
2. **Rendering**: Modificar `crates/renderer/src/cpu.rs`
3. **Configuración**: Modificar `crates/app/src/config.rs`
4. **Tests**: Agregar en `crates/*/src/tests.rs`

## Depuración

Para habilitar logs de depuración:

```bash
RUST_LOG=trace cargo run
```

Niveles de log disponibles: `error`, `warn`, `info`, `debug`, `trace`

## Licencia

MIT

## Referencias

-   [VT100 Sequences](https://vt100.net/)
-   [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)
-   [Cascadia Code Font](https://github.com/microsoft/cascadia-code)
