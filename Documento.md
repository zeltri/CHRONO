# Documento de Requerimientos

## Proyecto: Emulador de Terminal Gráfico en Rust

---

## 1. Introducción

### 1.1 Propósito

Este documento define de forma completa y estructurada los **requerimientos funcionales y no funcionales** para el desarrollo de un **emulador de terminal gráfico**, escrito íntegramente en **Rust**, con soporte multiplataforma y arquitectura moderna.

El objetivo es construir un **emulador real (no un wrapper)**, comparable en fundamentos a Kitty, Alacritty o WezTerm.

---

### 1.2 Alcance del sistema

El sistema será capaz de:

-   Ejecutar shells reales mediante PTY
-   Interpretar secuencias ANSI / VT
-   Representar una pantalla de terminal fiel
-   Renderizar texto de forma eficiente
-   Manejar entrada de teclado y eventos de ventana
-   Distribuirse como binario nativo (.deb, AppImage, etc.)

Quedan **fuera de alcance inicial**:

-   Multiplexado tipo tmux
-   SSH integrado
-   Scripting embebido
-   Compatibilidad retro con terminales no VT

---

### 1.3 Público objetivo

-   Desarrolladores de software
-   Usuarios técnicos de Linux / macOS / Windows
-   Uso diario como terminal principal (a largo plazo)

---

## 2. Arquitectura General

### 2.1 Principios de diseño

-   Modularidad estricta (crates separados)
-   Separación clara entre lógica y render
-   Performance primero
-   Seguridad de memoria
-   Sin dependencias innecesarias

---

### 2.2 Arquitectura de alto nivel

```
┌────────────┐
│    App     │  (window, input)
└─────┬──────┘
      │
┌─────▼──────┐
│  Renderer  │  (GPU/CPU)
└─────┬──────┘
      │
┌─────▼──────┐
│  Screen    │  (grid + scrollback)
└─────┬──────┘
      │
┌─────▼──────┐
│ ANSI / VT  │  (parser + state)
└─────┬──────┘
      │
┌─────▼──────┐
│   PTY      │  (shell real)
└────────────┘
```

---

### 2.3 Organización de crates

```
terminal-emulator/
├── crates/
│   ├── core/        # Screen, cursor, attributes
│   ├── ansi/        # Parser VT / ANSI
│   ├── pty/         # portable-pty wrapper
│   ├── renderer/    # CPU/GPU rendering
│   └── app/         # binario final
└── Cargo.toml
```

---

## 3. Requerimientos Funcionales

### RF-01: Ejecución de Shell

El sistema **debe**:

-   Crear un PTY usando `portable-pty`
-   Ejecutar el shell por defecto del sistema
-   Redirigir stdin/stdout/stderr

---

### RF-02: Comunicación Bidireccional

-   La entrada del usuario debe enviarse al PTY
-   La salida del PTY debe procesarse en tiempo real
-   Soportar escritura continua (streaming)

---

### RF-03: Interpretación ANSI / VT

El sistema debe:

-   Parsear secuencias ANSI estándar
-   Soportar VT100–VT520 (progresivo)
-   Manejar:

    -   Colores
    -   Cursor
    -   Clear / erase
    -   Scroll

Parser base: `vte`

---

### RF-04: Modelo de Pantalla

Debe existir un **modelo interno de pantalla** que:

-   Represente una grilla bidimensional
-   Almacene por celda:

    -   Carácter
    -   Color foreground/background
    -   Atributos (bold, underline, etc.)

-   Gestione el cursor
-   Soporte scrollback

---

### RF-05: Redimensionamiento

-   La terminal debe reaccionar a resize de ventana
-   Debe recalcular filas/columnas
-   Debe notificar al PTY del nuevo tamaño

---

### RF-06: Entrada de Usuario

Debe soportar:

-   Teclado completo
-   Combinaciones (Ctrl, Alt, Shift)
-   Repetición de teclas
-   Pegado de texto

---

### RF-07: Renderizado

Fases:

-   Fase 1: Render CPU (monoespaciado)
-   Fase 2: Render GPU (`wgpu`)

El renderer debe:

-   Dibujar solo lo necesario (dirty regions)
-   Mantener FPS estable

---

### RF-08: Configuración

El sistema debe:

-   Leer configuración desde archivo (TOML/YAML)
-   Permitir:

    -   Fuente
    -   Tamaño
    -   Colores
    -   Shell

---

### RF-09: Logs y Debug

-   Logs configurables por nivel
-   Modo debug con visualización de eventos ANSI

---

## 4. Requerimientos No Funcionales

### RNF-01: Performance

-   Scroll fluido con miles de líneas
-   Tiempo de arranque < 200ms

---

### RNF-02: Portabilidad

Debe funcionar en:

-   Linux (Wayland / X11)
-   macOS
-   Windows (ConPTY)

---

### RNF-03: Seguridad

-   Sin uso de unsafe innecesario
-   Manejo correcto de memoria

---

### RNF-04: Mantenibilidad

-   Código documentado
-   Tests unitarios en core y ansi

---

### RNF-05: Distribución

El binario debe poder empaquetarse como:

-   .deb
-   AppImage
-   binario standalone

---

## 5. Dependencias Técnicas

### Lenguaje

-   Rust (stable)

### Crates principales

-   portable-pty
-   vte
-   winit
-   wgpu (fase 2)
-   unicode-width
-   rustybuzz (fase 2)

---

## 6. Roadmap de Desarrollo

### Fase 1 – MVP

-   PTY funcional
-   Parser ANSI básico
-   Screen buffer
-   Render CPU

### Fase 2 – Texto real

-   Unicode
-   Width correcto
-   Combining chars

### Fase 3 – GPU

-   Glyph atlas
-   Batch rendering
-   Performance tuning

### Fase 4 – Features avanzadas

-   Tabs / splits
-   Temas
-   Hot reload config

---

## 7. Criterios de Éxito

El proyecto se considera exitoso si:

-   Ejecuta shells reales sin errores
-   Renderiza correctamente salida ANSI
-   Es usable como terminal diaria
-   Puede distribuirse como binario

---

## 8. Consideraciones Finales

Este proyecto es:

-   Complejo
-   De largo plazo
-   Altamente formativo

Se prioriza **calidad, comprensión y arquitectura** sobre velocidad de entrega.
