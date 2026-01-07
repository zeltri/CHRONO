# 🎯 Sugerencias Automáticas Basadas en Historial

## ✅ Nueva Característica Implementada

El terminal CHRONO ahora incluye **sugerencias automáticas inteligentes** basadas en tu historial de comandos, similar a Kitty Terminal. Cuando escribes las primeras letras de un comando, el terminal sugiere automáticamente en gris el último comando que usaste que comenzaba igual.

---

## 🚀 Cómo Funciona

### Automático y Transparente

1. **Escribes**: El terminal rastrea cada carácter que escribes
2. **Busca**: Automáticamente busca en tu historial comandos que empiecen igual
3. **Sugiere**: Muestra el resto del comando en gris claro
4. **Aceptas**: Presiona `Tab` o `→` (flecha derecha) para aceptar la sugerencia

### Ejemplo Visual

```
# Historial previo:
$ git status
$ git commit -m "fix: bug"
$ cargo build --release

# Nueva sesión:
$ gi
$ git commit -m "fix: bug"    ← Sugerido en gris
     ^^^^^^^^^^^^^^^^^^^^
     (presiona Tab o → para aceptar)

$ car
$ cargo build --release        ← Sugerido en gris
      ^^^^^^^^^^^^^^^^^
```

---

## ⌨️ Controles

| Tecla                     | Acción                                   |
| ------------------------- | ---------------------------------------- |
| **Tab**                   | Acepta la sugerencia completa            |
| **→** (flecha derecha)    | Acepta la sugerencia completa            |
| **Backspace**             | Borra carácter y actualiza sugerencia    |
| **Enter**                 | Ejecuta comando y lo guarda en historial |
| **Continuar escribiendo** | Actualiza sugerencia en tiempo real      |

---

## 🎨 Características

### ✅ Inteligente

-   **Búsqueda por prefijo**: Solo sugiere comandos que empiecen exactamente igual
-   **Más reciente primero**: Prioriza los comandos más recientes
-   **No duplicados**: No repite sugerencias del mismo comando consecutivo
-   **Actualización en tiempo real**: La sugerencia cambia mientras escribes

### ✅ Eficiente

-   **Historial limitado**: Almacena hasta 1000 comandos
-   **Búsqueda O(n)**: Búsqueda desde el más reciente al más antiguo
-   **Memoria optimizada**: Solo almacena strings de comandos

### ✅ Visual

-   **Color gris claro**: Usa el mismo `fg_suggestion` que las sugerencias ANSI
-   **No invasivo**: Desaparece automáticamente al escribir
-   **Limpieza automática**: Se borra al presionar Enter o cambiar de línea

---

## 🔧 Arquitectura Técnica

### Componentes Nuevos

#### 1. **CommandHistory** (`crates/core/src/history.rs`)

```rust
pub struct CommandHistory {
    commands: Vec<String>,      // Comandos ejecutados
    max_size: usize,             // Máximo 1000
}

// Métodos principales:
- add_command(cmd)              // Agregar al historial
- find_suggestion(prefix)       // Buscar por prefijo
- clear()                       // Limpiar historial
```

#### 2. **Campos en Screen**

```rust
pub struct Screen {
    // ... campos existentes
    command_history: CommandHistory,    // Historial de comandos
    current_command: String,            // Comando siendo escrito
    active_suggestion: Option<String>,  // Sugerencia actual
}
```

#### 3. **Métodos en Screen**

```rust
// Públicos:
- accept_suggestion()           // Aceptar sugerencia (Tab/→)
- get_active_suggestion()       // Obtener sugerencia actual
- handle_backspace()            // Manejar borrado

// Privados:
- update_auto_suggestion()      // Actualizar sugerencia
- clear_auto_suggestion()       // Limpiar sugerencia
```

### Flujo de Datos

```
Usuario escribe "g"
    ↓
write_char('g')
    ↓
current_command = "g"
    ↓
update_auto_suggestion()
    ↓
command_history.find_suggestion("g")
    ↓
Encuentra "it status" (de "git status")
    ↓
Muestra "it status" en gris después del cursor
    ↓
Usuario presiona Tab
    ↓
accept_suggestion()
    ↓
current_command = "git status"
    ↓
Cursor avanza al final
```

---

## 📋 Archivos Modificados

### Nuevos

-   `crates/core/src/history.rs` - Módulo de historial de comandos

### Modificados

-   `crates/core/src/lib.rs` - Exportar `CommandHistory`
-   `crates/core/src/screen.rs` - Integración de historial y sugerencias
-   `crates/ansi/src/handler.rs` - Manejo de backspace mejorado
-   `crates/app/src/main.rs` - Manejo de Tab y → para aceptar sugerencias

---

## 🧪 Cómo Probar

### 1. Compilar y Ejecutar

```bash
cargo build && cargo run
```

### 2. Crear Historial

```bash
# Dentro del terminal, ejecuta varios comandos:
$ ls -la
$ git status
$ cargo build
$ git commit -m "test"
```

### 3. Probar Sugerencias

```bash
# Escribe las primeras letras:
$ l        ← Debería sugerir "s -la"
$ g        ← Debería sugerir "it commit -m "test""
$ ca       ← Debería sugerir "rgo build"
```

### 4. Aceptar Sugerencias

-   Presiona `Tab` o `→` para aceptar
-   Continúa escribiendo para actualizar
-   Presiona `Backspace` para retroceder

---

## 🎯 Casos de Uso

### Comandos Largos Repetitivos

```bash
$ docker run -it --rm -v $(pwd):/app node:latest npm install
# Luego solo:
$ doc[TAB]  ← Completa todo el comando
```

### Paths Complejos

```bash
$ cd /usr/local/share/applications/
# Luego:
$ cd /us[TAB]  ← Completa el path
```

### Git Commits

```bash
$ git commit -m "feat: add new feature"
# Luego:
$ git co[TAB]  ← Completa el commit anterior
```

### Scripts con Argumentos

```bash
$ npm run dev -- --port 3000 --host 0.0.0.0
# Luego:
$ npm[TAB]  ← Recupera el comando completo
```

---

## 🔄 Diferencias con Sugerencias ANSI

El terminal ahora tiene **DOS sistemas de sugerencias**:

### 1. Sugerencias ANSI (ESC[53m/54m)

-   **Origen**: Shells externos (Fish, Zsh)
-   **Control**: El shell decide qué mostrar
-   **Uso**: Shells modernos con autosuggestions

### 2. Sugerencias de Historial (NUEVO)

-   **Origen**: Terminal mismo (basado en historial)
-   **Control**: El terminal decide basándose en comandos previos
-   **Uso**: Cualquier shell, automático

### Pueden Coexistir

-   Las sugerencias de historial se muestran cuando escribes
-   Las sugerencias ANSI se muestran cuando el shell las envía
-   Ambas usan el mismo color gris para consistencia visual

---

## ⚙️ Configuración

### Cambiar Tamaño del Historial

Edita `crates/core/src/screen.rs` línea ~72:

```rust
command_history: CommandHistory::new(1000),  // Actual
command_history: CommandHistory::new(5000),  // Más historial
command_history: CommandHistory::new(100),   // Menos historial
```

### Cambiar Color de Sugerencias

Las sugerencias usan el mismo color que antes:
Edita `crates/renderer/src/theme.rs` línea ~43:

```rust
fg_suggestion: (75, 82, 95),    // Gris claro actual
fg_suggestion: (100, 107, 120), // Más claro
fg_suggestion: (60, 67, 80),    // Más oscuro
```

---

## 🐛 Limitaciones Conocidas

1. **Sin persistencia**: El historial se pierde al cerrar el terminal
2. **Una sugerencia a la vez**: Solo muestra el comando más reciente
3. **Prefijo exacto**: Solo busca comandos que empiecen exactamente igual
4. **Sin fuzzy matching**: No hay búsqueda difusa o parcial
5. **Memoria por sesión**: El historial es por ventana, no global

---

## 🚀 Mejoras Futuras (Opcionales)

-   [ ] Persistir historial a disco (`~/.chrono_history`)
-   [ ] Historial global compartido entre ventanas
-   [ ] Fuzzy matching para búsqueda más flexible
-   [ ] Múltiples sugerencias con selector
-   [ ] Frecuencia de uso en ranking
-   [ ] Sugerencias por directorio/contexto
-   [ ] Integración con `history` command de shell

---

## 📊 Performance

-   **Espacio**: ~80 bytes por comando × 1000 = ~80KB máximo
-   **Búsqueda**: O(n) lineal, típicamente < 1ms para 1000 comandos
-   **Inserción**: O(1) amortizado
-   **Renderizado**: Sin overhead adicional (usa mismo pipeline)

---

## ✨ Resumen

| Característica               | Estado |
| ---------------------------- | ------ |
| Historial de comandos        | ✅     |
| Búsqueda por prefijo         | ✅     |
| Sugerencias en gris          | ✅     |
| Aceptar con Tab              | ✅     |
| Aceptar con →                | ✅     |
| Actualización en tiempo real | ✅     |
| Limpieza automática          | ✅     |
| Sin duplicados consecutivos  | ✅     |
| Límite de historial          | ✅     |
| Tests unitarios              | ✅     |

---

**¡Disfruta de las sugerencias inteligentes en CHRONO Terminal! 🎉**

_Implementado el 7 de enero de 2026_
_Compatible con el sistema de sugerencias ANSI existente_
