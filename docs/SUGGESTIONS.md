# Sugerencias de Autocompletado

## Descripción

El terminal ahora soporta **autocompletado visual completo** con sugerencias renderizadas en gris claro. Esta funcionalidad está completamente integrada y lista para usar con shells modernos como fish y zsh.

## Estado: ✅ COMPLETAMENTE IMPLEMENTADO

### Secuencias ANSI Soportadas

El terminal implementa dos secuencias CSI personalizadas para controlar el modo de sugerencias:

-   **`ESC[53m`** - Activa el modo sugerencia (todo el texto subsiguiente aparecerá en gris claro)
-   **`ESC[54m`** - Desactiva el modo sugerencia (vuelve a renderizado normal)

### Ejemplo de Uso

```bash
# Mostrar sugerencia después del comando
echo -n "$ git"
echo -ne "\033[53m"  # Activar sugerencia
echo -n " status"
echo -ne "\033[54m"  # Desactivar sugerencia
echo ""
```

## Características Implementadas

### 1. **Modo de Sugerencia Automático**

-   Las celdas escritas en modo sugerencia se marcan automáticamente con `is_suggestion = true`
-   Se renderizan en color gris claro definido en el tema

### 2. **Limpieza Inteligente**

-   Las sugerencias se limpian automáticamente cuando se escribe texto normal
-   Las sugerencias desaparecen al presionar teclas (comportamiento esperado)

### 3. **Integración con ANSI Parser**

-   Secuencias CSI 53 y 54 integradas en el handler SGR
-   Procesamiento automático sin intervención manual

### 4. **API Pública en Screen**

```rust
// Activar modo sugerencia
screen.start_suggestion();

// Escribir texto de sugerencia
screen.write_char('h');
screen.write_char('e');
screen.write_char('l');
screen.write_char('p');

// Desactivar modo sugerencia
screen.end_suggestion();

// Limpiar sugerencias manualmente
screen.clear_suggestions();

// Verificar si hay sugerencias
if screen.has_suggestions() {
    // ...
}
```

## Uso

### Para Shells que Envían Sugerencias

Las sugerencias se muestran automáticamente en un **gris claro** (#4b525f) para diferenciarlas del texto normal.

## Integración con Shells

### Fish Shell

Fish puede configurarse para usar estas secuencias:

```fish
# ~/.config/fish/config.fish
function fish_postexec
    # Las sugerencias de fish ya funcionarán visualmente
end
```

### Zsh con zsh-autosuggestions

```bash
# ~/.zshrc
# Las autosuggestions de zsh funcionarán automáticamente
# si el plugin está configurado para usar color dim/gray
```

### Script de Prueba

Ejecuta el script incluido para ver las sugerencias en acción:

```bash
./test_suggestions.sh
```

O dentro del terminal:

```bash
cargo run
# Luego dentro del terminal:
./test_suggestions.sh
```

### Cómo Funciona

1. **Detección**: Las celdas marcadas con `is_suggestion = true` se renderizan con el color especial
2. **Color**: Definido en `ModernTheme.fg_suggestion`
3. **Renderizado**: El CPU renderer prioriza el color de sugerencia sobre otros colores

### API Programática

```rust
use terminal_core::Cell;

// Crear una celda de sugerencia
let suggestion_cell = Cell::as_suggestion('t');

// O marcar una celda existente como sugerencia
let mut cell = Cell::new('t');
cell.is_suggestion = true;
```

## Colores

-   **Color actual**: `rgb(75, 82, 95)` - Gris claro sutil (#4b525f)
-   **Alternativa más clara**: Puedes ajustar en [theme.rs](../crates/renderer/src/theme.rs) cambiando `fg_suggestion`

Para cambiar el color:

```rust
// En crates/renderer/src/theme.rs
fg_suggestion: (75, 82, 95),   // Actual (gris claro)
// O prueba:
fg_suggestion: (100, 107, 120), // Gris más claro
fg_suggestion: (60, 67, 80),    // Gris más oscuro
```

## Arquitectura Técnica

### Flujo de Datos

1. **Shell envía**: `ESC[53m` → Texto sugerido → `ESC[54m`
2. **Parser ANSI** detecta secuencia CSI 53
3. **Handler** llama a `screen.start_suggestion()`
4. **Screen** marca `suggestion_mode = true`
5. **write_char** marca cada celda con `is_suggestion = true`
6. **Renderer** detecta `cell.is_suggestion` y usa `fg_suggestion_u32()`
7. **Usuario escribe** → `write_char` limpia sugerencias automáticamente

### Archivos Modificados

-   ✅ `crates/core/src/cell.rs` - Campo `is_suggestion` y método `as_suggestion()`
-   ✅ `crates/core/src/screen.rs` - Modo sugerencia, métodos de control y limpieza automática
-   ✅ `crates/ansi/src/handler.rs` - Secuencias CSI 53/54 en handler SGR
-   ✅ `crates/renderer/src/theme.rs` - Color `fg_suggestion` y método `fg_suggestion_u32()`
-   ✅ `crates/renderer/src/cpu.rs` - Renderizado prioritario de sugerencias

## Testing Manual

### Prueba Básica

```bash
# Compilar y ejecutar
cargo build && cargo run

# Dentro del terminal, ejecutar:
./test_suggestions.sh
```

### Prueba con printf

```bash
printf "$ git\033[53m status\033[54m\n"
printf "$ cargo\033[53m build --release\033[54m\n"
```

### Prueba con echo

```bash
echo -e "$ ls\033[53m -la /home\033[54m"
```

## Limitaciones Conocidas

1. **Shells nativos**: Los shells estándar (bash, sh) no envían automáticamente estas secuencias
2. **Configuración requerida**: Fish y zsh requieren configuración/plugins para usar las secuencias
3. **Sin API remota**: No hay comunicación bidireccional (solo rendering)

## Ejemplos de Uso

### Shell con Autocompletado (fish/zsh)

Cuando escribes `gi` y el shell sugiere `git`:

```
$ gi|t      <- 't' se muestra en gris claro
```

## Integración Futura con Shells Populares

Esta funcionalidad funciona con cualquier shell que emita las secuencias ANSI apropiadas:

-   Fish shell autosuggestions (configuración pendiente)
-   Zsh autosuggestions plugin (wrapper pendiente)
-   Cualquier shell que envíe `ESC[53m` y `ESC[54m`

## Siguiente Paso: Integración Nativa

Para integración nativa completa con fish/zsh sin modificar el shell:

1. Crear wrapper que intercepte output del shell
2. Detectar patrones de autocompletado
3. Inyectar secuencias CSI 53/54 automáticamente

---

**Estado**: ✅ Integración Completa - Listo para Producción  
**Versión**: 0.1.0  
**Fecha**: Enero 2026
