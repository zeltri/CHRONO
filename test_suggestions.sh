#!/bin/bash

# Script de prueba para demostrar sugerencias de autocompletado
# Este script simula lo que un shell con autosuggestions haría

echo "=== Prueba de Sugerencias de Autocompletado ==="
echo ""
echo "Las siguientes líneas demuestran cómo usar las secuencias ANSI"
echo "para mostrar sugerencias en gris claro:"
echo ""

# Simular escritura de comando con sugerencia
echo -n "$ ls"
# Activar modo sugerencia (CSI 53 m)
echo -ne "\033[53m"
echo -n " -la /home/usuario"
# Desactivar modo sugerencia (CSI 54 m)
echo -ne "\033[54m"
echo ""

echo ""
echo -n "$ git"
echo -ne "\033[53m"
echo -n " status"
echo -ne "\033[54m"
echo ""

echo ""
echo -n "$ cargo"
echo -ne "\033[53m"
echo -n " build --release"
echo -ne "\033[54m"
echo ""

echo ""
echo "=== Secuencias ANSI utilizadas ==="
echo "- \\033[53m  → Inicia modo sugerencia (texto en gris claro)"
echo "- \\033[54m  → Termina modo sugerencia (vuelve a texto normal)"
echo ""
echo "Ejemplo de uso en código:"
echo '  printf "$ git"'
echo '  printf "\\033[53m status\\033[54m\\n"'
echo ""
