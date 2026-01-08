# Sistema de Detección y Renderizado Inteligente

## 📖 Descripción General

CHRONO Terminal ahora incluye un sistema avanzado de detección y renderizado inteligente que automáticamente identifica y colorea diferentes tipos de contenido en el output:

-   🎨 **Logs estructurados** (INFO, WARN, ERROR, DEBUG, TRACE, FATAL)
-   ❌ **Mensajes de error** y excepciones
-   ⚠️ **Advertencias** y cautiones
-   ✅ **Mensajes de éxito**
-   📊 **JSON** con sintaxis coloreada
-   📋 **Tablas ASCII** con formato mejorado
-   🔍 **Stack traces** multi-lenguaje (Rust, Python, Java, JavaScript, etc.)

## 🎯 Características

### Detección de Logs

El sistema detecta automáticamente niveles de log comunes y los colorea apropiadamente:

```bash
INFO: Server started on port 8080     # Azul
WARN: Deprecated function used        # Amarillo
ERROR: Connection failed              # Rojo
DEBUG: Variable value = 42            # Cyan
TRACE: Entering function              # Gris
FATAL: System crash detected          # Rojo intenso
```

**Patrones detectados:**

-   `INFO`, `[INF]`, `ℹ`
-   `WARN`, `WARNING`, `[WRN]`
-   `ERROR`, `[ERR]`, `✗`
-   `DEBUG`, `[DBG]`
-   `TRACE`, `[TRC]`
-   `FATAL`, `CRITICAL`

### Detección de Errores

Identifica mensajes de error de múltiples fuentes:

```bash
error: could not compile `main.rs`
Exception in thread "main": NullPointerException
Traceback (most recent call last):
✗ Error: Connection timeout
```

### JSON Coloreado

Sintaxis highlight automático para JSON:

```json
{
    "name": "John", // Keys en azul, strings en verde
    "age": 30, // Números en naranja
    "active": true, // Booleanos en magenta
    "role": null // null en cyan
}
```

**Colores aplicados:**

-   **Llaves/Brackets** `{}[]`: Gris secundario
-   **Keys**: Azul (#61afef)
-   **Strings**: Verde (#98c379)
-   **Números**: Naranja (#d19a66)
-   **Booleanos**: Magenta (#c678dd)
-   **null**: Cyan (#56b6c2)

### Stack Traces Inteligentes

Detecta stack traces de múltiples lenguajes y los resalta en cyan:

**Rust:**

```
at src/main.rs:42:5
   --> src/handler.rs:15:10
```

**Python:**

```
  File "script.py", line 42, in main
```

**Java:**

```
at com.example.Main.method(Main.java:42)
```

**JavaScript/Node.js:**

```
at Object.<anonymous> (app.js:10:5)
```

### Detección de Tablas

Reconoce y formatea tablas ASCII comunes:

```
+--------+--------+--------+
| Header | Header | Header |  # Encabezado en cyan
+--------+--------+--------+
| Data   | Data   | Data   |  # Datos con formato
+--------+--------+--------+
```

**Separadores reconocidos:**

-   `+---+---+`
-   `|---|---|`
-   `=========`
-   Caracteres Unicode: `─`, `━`, `═`

### Mensajes de Éxito

Identifica indicadores de éxito y los colorea en verde:

```bash
✓ Tests passed
✔ Build successful
✅ All checks completed
success: Operation completed
```

## ⚙️ Configuración

El sistema es completamente configurable vía `config.toml`:

```toml
[rendering]
# Habilita detección inteligente de contenido
smart_detection = true

# Habilita sintaxis coloreada para JSON
json_colors = true

# Habilita colores automáticos para logs
log_colors = true

# Habilita detección y formateo de tablas ASCII
table_detection = true
```

### Configuración por Defecto

Por defecto, **todas las características están habilitadas**. Esto proporciona la mejor experiencia en modo interactivo.

### Desactivar para Scripts

Si estás usando la terminal en scripts o pipelines donde quieres output plano:

```toml
[rendering]
smart_detection = false
```

O usa variables de entorno:

```bash
NO_COLOR=1 ./tu-script.sh
```

## 🎨 Paleta de Colores

El sistema usa la paleta **OneDark Pro**:

| Tipo          | Color    | Hex     |
| ------------- | -------- | ------- |
| Log INFO      | Azul     | #61afef |
| Log WARN      | Amarillo | #e5c07b |
| Log ERROR     | Rojo     | #e06c75 |
| Log DEBUG     | Cyan     | #56b6c2 |
| Éxito         | Verde    | #98c379 |
| JSON Keys     | Azul     | #61afef |
| JSON Strings  | Verde    | #98c379 |
| JSON Numbers  | Naranja  | #d19a66 |
| JSON Booleans | Magenta  | #c678dd |
| Stack Traces  | Cyan     | #56b6c2 |

## 🔧 Arquitectura

### Módulo `detector.rs`

Ubicación: `crates/core/src/detector.rs`

**Componentes principales:**

1. **`ContentDetector`**: Detector stateful con soporte para multi-línea
2. **`ContentType`**: Enum que representa tipos de contenido detectados
3. **`LogLevel`**: Niveles de log soportados
4. **`JsonFragment`**: Información sobre tokens JSON
5. **`TableInfo`**: Metadata de tablas detectadas

**API Principal:**

```rust
let mut detector = ContentDetector::new();

// Detectar tipo de línea
let content_type = detector.detect_line("INFO: Server started");

// Parsear JSON
let fragments = detector.parse_json_fragments(r#"{"key": "value"}"#);
```

### Integración con Renderer

El `CpuRenderer` integra automáticamente el detector:

```rust
// En render()
let content_type = self.content_detector.detect_line(&line_text);

// Aplicar colores según tipo
let fg = self.get_content_color(cell, line_context, content_type);
```

## 📊 Performance

El sistema de detección está optimizado para:

-   **Mínima latencia**: Detección en una sola pasada por línea
-   **Zero-copy**: No copia strings innecesariamente
-   **Lazy evaluation**: Solo analiza lo que se renderiza
-   **Caching**: Estado mantenido para tablas multi-línea

**Impacto en performance:**

-   < 1ms de overhead por frame típico
-   Compatible con 60+ FPS
-   No bloquea el PTY o input

## 🧪 Testing

Ejecuta los tests del detector:

```bash
cargo test -p terminal-core detector
```

Tests incluidos:

-   ✅ Detección de niveles de log
-   ✅ Identificación de stack traces multi-lenguaje
-   ✅ Parsing de JSON
-   ✅ Detección de tablas
-   ✅ Identificación de errores

## 🚀 Ejemplos de Uso

### Ver JSON coloreado

```bash
cat data.json
echo '{"name": "John", "age": 30}' | jq
```

### Ver logs de aplicaciones

```bash
npm run dev
cargo run
python app.py
```

### Ver stack traces

```bash
cargo build  # Rust compilation errors
pytest      # Python test failures
npm test    # JavaScript test output
```

### Ver tablas

```bash
docker ps
kubectl get pods
ls -la | column -t
```

## 🔮 Futuras Mejoras

Planeadas para futuras versiones:

-   [ ] Detección de URLs y email (clickeable)
-   [ ] Sintaxis highlight para YAML/TOML
-   [ ] Detección de números de commit Git
-   [ ] Resaltado de diffs
-   [ ] Detección de IPs y puertos
-   [ ] Formato mejorado para Markdown en terminal
-   [ ] Integración con language servers para hints

## 📝 Notas Técnicas

### Compatibilidad con Scripts

El sistema **no interfiere** con scripts porque:

1. Solo afecta el renderizado visual
2. No modifica el output del PTY
3. Respeta variables de entorno como `NO_COLOR`
4. Puede desactivarse completamente

### Compatibilidad ANSI

El sistema **complementa** las secuencias ANSI existentes:

-   Los colores ANSI explícitos se respetan siempre
-   La detección inteligente actúa como fallback
-   Los estilos (bold, italic) se preservan

### Multi-idioma

El detector es agnóstico del lenguaje de programación y funciona con:

-   ✅ Rust
-   ✅ Python
-   ✅ JavaScript/TypeScript
-   ✅ Java
-   ✅ C/C++
-   ✅ Go
-   ✅ Ruby
-   ✅ PHP
-   ✅ Y más...

## 💡 Tips

1. **JSON inválido**: El parser es tolerante y colorea lo que puede
2. **Performance**: Desactiva `smart_detection` si notas lag con output masivo
3. **Combinación**: El sistema funciona junto con aliases y pipes normales
4. **Debugging**: Activa logs con `RUST_LOG=debug` para ver detecciones

---

**Documentación generada para CHRONO Terminal v0.1.0**
