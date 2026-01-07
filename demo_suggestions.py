#!/usr/bin/env python3
"""
Script de demostración avanzado de sugerencias de autocompletado
Simula un shell interactivo con autosuggestions
"""

import sys
import time

# Secuencias ANSI
SUGGESTION_START = "\033[53m"
SUGGESTION_END = "\033[54m"
CURSOR_SAVE = "\033[s"
CURSOR_RESTORE = "\033[u"
CLEAR_LINE = "\033[K"

def print_suggestion(prompt, text, suggestion):
    """Imprime un prompt con sugerencia"""
    sys.stdout.write(prompt)
    sys.stdout.write(text)
    sys.stdout.write(SUGGESTION_START)
    sys.stdout.write(suggestion)
    sys.stdout.write(SUGGESTION_END)
    sys.stdout.write("\n")
    sys.stdout.flush()

def simulate_typing(prompt, full_text, suggestion_map):
    """Simula escritura con sugerencias dinámicas"""
    sys.stdout.write(prompt)
    sys.stdout.flush()
    
    for i, char in enumerate(full_text):
        sys.stdout.write(char)
        sys.stdout.flush()
        time.sleep(0.1)
        
        # Mostrar sugerencia si existe
        typed_so_far = full_text[:i+1]
        if typed_so_far in suggestion_map:
            suggestion = suggestion_map[typed_so_far]
            sys.stdout.write(SUGGESTION_START)
            sys.stdout.write(suggestion)
            sys.stdout.write(SUGGESTION_END)
            sys.stdout.flush()
            time.sleep(0.3)
            # Limpiar sugerencia
            sys.stdout.write("\r" + prompt + full_text[:i+1] + " " * len(suggestion))
            sys.stdout.write("\r" + prompt + full_text[:i+1])
            sys.stdout.flush()
    
    sys.stdout.write("\n")
    sys.stdout.flush()

def main():
    print("\n" + "="*60)
    print("  DEMO: Sugerencias de Autocompletado CHRONO Terminal")
    print("="*60 + "\n")
    
    print("→ Las sugerencias aparecen en GRIS CLARO después del cursor\n")
    time.sleep(1)
    
    # Ejemplo 1: Comandos simples
    print("📝 Ejemplo 1: Comandos básicos\n")
    time.sleep(0.5)
    
    print_suggestion("$ ", "ls", " -la /home/usuario")
    time.sleep(0.8)
    
    print_suggestion("$ ", "git", " status")
    time.sleep(0.8)
    
    print_suggestion("$ ", "cargo", " build --release")
    time.sleep(0.8)
    
    print_suggestion("$ ", "cd", " /usr/local/bin")
    time.sleep(1)
    
    # Ejemplo 2: Comandos largos
    print("\n📝 Ejemplo 2: Comandos más complejos\n")
    time.sleep(0.5)
    
    print_suggestion("$ ", "docker run", " -it --rm ubuntu:latest /bin/bash")
    time.sleep(0.8)
    
    print_suggestion("$ ", "find", " . -name '*.rs' -type f")
    time.sleep(0.8)
    
    print_suggestion("$ ", "grep", " -r 'TODO' src/")
    time.sleep(1)
    
    # Ejemplo 3: Paths
    print("\n📝 Ejemplo 3: Autocompletado de paths\n")
    time.sleep(0.5)
    
    print_suggestion("$ ", "cat ~/.config/", "terminal-emulator/config.toml")
    time.sleep(0.8)
    
    print_suggestion("$ ", "vim /etc/", "hosts")
    time.sleep(0.8)
    
    print_suggestion("$ ", "cd ~/dev/", "terminal/crates/core/")
    time.sleep(1)
    
    # Ejemplo 4: Múltiples opciones (simulado)
    print("\n📝 Ejemplo 4: Comandos con argumentos\n")
    time.sleep(0.5)
    
    print_suggestion("$ ", "git commit", " -m 'feat: add suggestions'")
    time.sleep(0.8)
    
    print_suggestion("$ ", "npm install", " --save-dev typescript")
    time.sleep(0.8)
    
    print_suggestion("$ ", "rustc", " --edition 2021 main.rs")
    time.sleep(1)
    
    print("\n" + "="*60)
    print("  ✅ Demo Completada")
    print("="*60 + "\n")
    
    print("💡 Tip: Las sugerencias usan las secuencias ANSI:")
    print("   • ESC[53m  → Inicia sugerencia (gris claro)")
    print("   • ESC[54m  → Termina sugerencia")
    print()
    print("🧪 Para probar manualmente:")
    print('   echo -e "$ git\\033[53m status\\033[54m"')
    print()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n👋 Demo interrumpida\n")
        sys.exit(0)
