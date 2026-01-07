#!/bin/bash

# Script de demostración de sugerencias basadas en historial
# Este script muestra cómo funcionan las sugerencias automáticas

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  DEMO: Sugerencias Inteligentes de Historial CHRONO       ║"
echo "╔════════════════════════════════════════════════════════════╗"
echo ""
echo "Este demo creará un historial de comandos y te mostrará"
echo "cómo el terminal sugiere automáticamente basándose en él."
echo ""

# Función para pausar
pause() {
    echo ""
    read -p "Presiona Enter para continuar..."
    echo ""
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Paso 1: Creando historial de comandos"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Vamos a ejecutar varios comandos para crear un historial:"
echo ""

# Comandos de demostración
commands=(
    "ls -la /home"
    "git status"
    "git commit -m 'feat: add new feature'"
    "cargo build --release"
    "cargo test --all"
    "docker ps -a"
    "docker run -it ubuntu bash"
    "npm install --save-dev typescript"
    "npm run dev -- --port 3000"
)

for cmd in "${commands[@]}"; do
    echo "$ $cmd"
    # Simular ejecución (solo mostrar, no ejecutar realmente)
    sleep 0.3
done

echo ""
echo "✅ Historial creado con ${#commands[@]} comandos"
pause

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Paso 2: Cómo funcionan las sugerencias"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Ahora, cuando empieces a escribir, el terminal sugerirá"
echo "automáticamente comandos del historial en GRIS CLARO."
echo ""
echo "Ejemplos de lo que verías:"
echo ""

# Simulaciones visuales
echo "1. Si escribes: l"
echo "   Verías: l\033[2ms -la /home\033[0m"
echo "           ^ Sugerencia en gris"
echo ""

echo "2. Si escribes: git s"
echo "   Verías: git s\033[2mtatus\033[0m"
echo "               ^^^^^^ Sugerencia en gris"
echo ""

echo "3. Si escribes: cargo b"
echo "   Verías: cargo b\033[2muild --release\033[0m"
echo "                 ^^^^^^^^^^^^^^^ Sugerencia en gris"
echo ""

echo "4. Si escribes: doc"
echo "   Verías: doc\033[2mker run -it ubuntu bash\033[0m"
echo "              ^^^^^^^^^^^^^^^^^^^^^^^^ Sugerencia en gris"
echo ""

pause

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Paso 3: Controles"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Para interactuar con las sugerencias:"
echo ""
echo "  ⌨️  TAB            → Acepta la sugerencia completa"
echo "  ⌨️  → (flecha)     → Acepta la sugerencia completa"
echo "  ⌨️  Backspace      → Borra y actualiza sugerencia"
echo "  ⌨️  Enter          → Ejecuta y guarda en historial"
echo "  ⌨️  Seguir escribir → Actualiza sugerencia en tiempo real"
echo ""

pause

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Paso 4: Ejemplo interactivo"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Ahora TÚ pruébalo:"
echo ""
echo "1. Ejecuta algunos comandos (cualquier cosa)"
echo "2. Luego escribe las primeras letras de algo que ejecutaste"
echo "3. Verás la sugerencia en gris"
echo "4. Presiona Tab o → para aceptarla"
echo ""
echo "Comandos sugeridos para probar:"
echo ""
echo "  • ls -la"
echo "  • git status"
echo "  • cargo build"
echo "  • cd /tmp"
echo ""
echo "Luego escribe solo 'l' o 'g' o 'c' y verás las sugerencias!"
echo ""

pause

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Paso 5: Características avanzadas"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✨ Inteligente:"
echo "   • Solo sugiere comandos que empiecen EXACTAMENTE igual"
echo "   • Prioriza los comandos más recientes"
echo "   • No duplica el mismo comando consecutivo"
echo ""
echo "🎨 Visual:"
echo "   • Color gris claro consistente"
echo "   • Desaparece automáticamente al escribir"
echo "   • Se limpia al cambiar de línea"
echo ""
echo "⚡ Eficiente:"
echo "   • Almacena hasta 1000 comandos"
echo "   • Búsqueda ultra-rápida"
echo "   • Sin lag ni retrasos"
echo ""

pause

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  💡 Tips y Trucos"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 Comandos largos repetitivos:"
echo "   $ docker run -it --rm -v \$(pwd):/app node:latest npm install"
echo "   Luego: $ doc[TAB]  ← ¡Recupera todo!"
echo ""
echo "📁 Paths complejos:"
echo "   $ cd /usr/local/share/applications/"
echo "   Luego: $ cd /us[TAB]  ← ¡Completa el path!"
echo ""
echo "🚀 Git commits:"
echo "   $ git commit -m 'feat: add awesome feature'"
echo "   Luego: $ git co[TAB]  ← ¡Repite el commit!"
echo ""

pause

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  ✅ DEMO COMPLETADO                                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Las sugerencias de historial están activas y listas para usar."
echo ""
echo "📖 Documentación completa: HISTORIAL_SUGGESTIONS.md"
echo "🚀 ¡Empieza a escribir y disfruta de las sugerencias!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
