# Desarrollo

## Compilar el proyecto

```bash
cargo build
```

## Ejecutar

```bash
cargo run -p terminal-app
```

## Tests

```bash
cargo test
```

## Verificar código

```bash
cargo clippy
cargo fmt
```

## Estructura de desarrollo

-   **core**: Implementa el modelo de datos (Screen, Cell, Cursor)
-   **ansi**: Parser ANSI/VT usando vte
-   **pty**: Abstracción de PTY multiplataforma
-   **renderer**: Renderizado CPU (fase 1) y GPU (fase 2)
-   **app**: Binario principal con winit

## Próximos pasos (Fase 2)

-   [ ] Renderizado de texto real con rusttype/fontdue
-   [ ] Soporte completo de Unicode
-   [ ] Width correcto con unicode-width
-   [ ] Combining characters
-   [ ] Ligatures (opcional)
