#!/bin/bash

# Script de demostración del sistema de renderizado inteligente de CHRONO Terminal
# Este script muestra diferentes tipos de output que serán automáticamente
# detectados y coloreados por el terminal.

echo "=================================="
echo "CHRONO Terminal - Demo Interactivo"
echo "=================================="
echo ""

# === LOGS ===
echo "📋 Demostración de Logs:"
echo ""
echo "INFO: Servidor iniciado en puerto 8080"
echo "DEBUG: Variable contador = 42"
echo "WARN: Función deprecada en uso"
echo "ERROR: Conexión a base de datos falló"
echo "FATAL: Error crítico del sistema"
echo "[INF] Formato alternativo de log"
echo "✓ Operación completada con éxito"
echo ""

# === JSON ===
echo "📊 Demostración de JSON:"
echo ""
cat << 'EOF'
{
  "name": "CHRONO Terminal",
  "version": "0.1.0",
  "features": {
    "smart_detection": true,
    "json_colors": true,
    "log_colors": true
  },
  "performance": {
    "fps": 60,
    "latency_ms": 1,
    "memory_mb": null
  },
  "tags": ["terminal", "rust", "modern"]
}
EOF
echo ""

# === ERRORES ===
echo "❌ Demostración de Errores:"
echo ""
echo "error: could not compile \`main.rs\` due to previous error"
echo "Exception in thread \"main\" java.lang.NullPointerException"
echo "Traceback (most recent call last):"
echo "TypeError: Cannot read property 'name' of undefined"
echo "✗ Build failed with 3 errors"
echo ""

# === STACK TRACES ===
echo "🔍 Demostración de Stack Traces:"
echo ""
echo "Stack trace:"
echo "   at src/main.rs:42:5"
echo "   at handler.rs:15:10"
echo "   --> core/screen.rs:128:22"
echo "  File \"app.py\", line 42, in process_data"
echo "    at Object.<anonymous> (server.js:10:5)"
echo "    at com.example.Main.method(Main.java:42)"
echo ""

# === TABLAS ===
echo "📋 Demostración de Tablas:"
echo ""
echo "+------------+---------+--------+"
echo "| Nombre     | Estado  | Puerto |"
echo "+------------+---------+--------+"
echo "| web-server | running | 8080   |"
echo "| database   | running | 5432   |"
echo "| cache      | stopped | 6379   |"
echo "+------------+---------+--------+"
echo ""

# === WARNINGS ===
echo "⚠️  Demostración de Warnings:"
echo ""
echo "warning: unused variable \`x\`"
echo "caution: This operation may take a while"
echo "⚠ Configuration file not found, using defaults"
echo ""

# === SUCCESS ===
echo "✅ Demostración de Mensajes de Éxito:"
echo ""
echo "✓ All tests passed (42 tests)"
echo "✔ Build successful in 2.3s"
echo "✅ Deployment completed"
echo "success: Configuration saved"
echo ""

# === COMBINACIÓN ===
echo "🎨 Ejemplo Realista - Log de Aplicación:"
echo ""
echo "2026-01-08 12:00:00 INFO: Application starting..."
echo "2026-01-08 12:00:01 DEBUG: Loading configuration from config.json"
cat << 'EOF'
{
  "port": 8080,
  "debug": true,
  "database": {
    "host": "localhost",
    "port": 5432
  }
}
EOF
echo "2026-01-08 12:00:02 INFO: Database connection established"
echo "2026-01-08 12:00:03 WARN: Cache service unavailable, using fallback"
echo "2026-01-08 12:00:04 INFO: Server listening on http://localhost:8080"
echo "2026-01-08 12:00:05 ERROR: Failed to load plugin 'example-plugin'"
echo "   at plugin_loader.rs:45:12"
echo "   --> src/main.rs:120:5"
echo "2026-01-08 12:00:06 INFO: Recovered from error, continuing..."
echo "✓ Startup completed in 6.2s"
echo ""

echo "=================================="
echo "Demo completada!"
echo ""
echo "Nota: Si ves colores diferentes para logs, JSON, errores, etc.,"
echo "significa que el renderizado inteligente está funcionando. ✨"
echo ""
echo "Para configurar: edita ~/.config/terminal-emulator/config.toml"
echo "Documentación: docs/SMART_RENDERING.md"
echo "=================================="
