use std::fs;
use std::path::PathBuf;

/// Historial de comandos para sugerencias automáticas
#[derive(Debug, Clone)]
pub struct CommandHistory {
    /// Lista de comandos ejecutados (más reciente al final)
    commands: Vec<String>,
    /// Máximo de comandos a almacenar
    max_size: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: Vec::new(),
            max_size,
        }
    }

    /// Crea un historial cargando desde archivo de zsh/bash
    pub fn from_shell_history(max_size: usize) -> Self {
        let mut history = Self::new(max_size);
        history.load_from_shell();
        history
    }

    /// Carga el historial desde ~/.zsh_history o ~/.bash_history
    fn load_from_shell(&mut self) {
        // Detectar el shell actual desde la variable de entorno SHELL
        let shell = std::env::var("SHELL").unwrap_or_default();
        let is_zsh = shell.contains("zsh");
        let is_bash = shell.contains("bash");

        eprintln!(
            "[HISTORIAL] Shell detectado: {} (zsh={}, bash={})",
            shell, is_zsh, is_bash
        );

        // Priorizar el shell actual del usuario
        if is_zsh {
            // Usuario usa zsh, intentar cargar .zsh_history
            let zsh_path = Self::get_home_path(".zsh_history");
            if let Some(path) = zsh_path {
                eprintln!("[HISTORIAL] Intentando cargar: {:?}", path);
                match Self::read_file_lossy(&path) {
                    Ok(content) => {
                        self.parse_zsh_history(&content);
                        eprintln!(
                            "[HISTORIAL] ✓ Cargados {} comandos desde zsh",
                            self.commands.len()
                        );
                        return;
                    }
                    Err(e) => {
                        eprintln!("[HISTORIAL] ✗ Error leyendo archivo zsh: {}", e);
                    }
                }
            }
        } else if is_bash {
            // Usuario usa bash, intentar cargar .bash_history
            let bash_path = Self::get_home_path(".bash_history");
            if let Some(path) = bash_path {
                eprintln!("[HISTORIAL] Intentando cargar: {:?}", path);
                match Self::read_file_lossy(&path) {
                    Ok(content) => {
                        self.parse_bash_history(&content);
                        eprintln!(
                            "[HISTORIAL] ✓ Cargados {} comandos desde bash",
                            self.commands.len()
                        );
                        return;
                    }
                    Err(e) => {
                        eprintln!("[HISTORIAL] ✗ Error leyendo archivo bash: {}", e);
                    }
                }
            }
        }

        // Fallback: intentar primero zsh, luego bash
        let zsh_path = Self::get_home_path(".zsh_history");
        if let Some(path) = zsh_path {
            eprintln!("[HISTORIAL] Fallback - Intentando cargar: {:?}", path);
            if let Ok(content) = Self::read_file_lossy(&path) {
                self.parse_zsh_history(&content);
                eprintln!(
                    "[HISTORIAL] ✓ Cargados {} comandos desde zsh (fallback)",
                    self.commands.len()
                );
                return;
            }
        }

        // Si no hay zsh, intentar bash
        let bash_path = Self::get_home_path(".bash_history");
        if let Some(path) = bash_path {
            eprintln!("[HISTORIAL] Fallback - Intentando cargar: {:?}", path);
            if let Ok(content) = Self::read_file_lossy(&path) {
                self.parse_bash_history(&content);
                eprintln!(
                    "[HISTORIAL] ✓ Cargados {} comandos desde bash (fallback)",
                    self.commands.len()
                );
            }
        }
    }

    /// Lee un archivo de texto, reemplazando caracteres inválidos UTF-8
    fn read_file_lossy(path: &PathBuf) -> Result<String, std::io::Error> {
        let bytes = fs::read(path)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Verifica si un comando contiene caracteres UTF-8 válidos
    /// Rechaza comandos con el carácter de reemplazo UTF-8 (�)
    fn is_valid_utf8_command(cmd: &str) -> bool {
        !cmd.contains('\u{FFFD}') // U+FFFD es el carácter de reemplazo UTF-8
    }

    /// Obtiene la ruta del archivo en el home del usuario
    fn get_home_path(filename: &str) -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(filename))
    }

    /// Parsea el historial de zsh (formato con timestamps o simple)
    fn parse_zsh_history(&mut self, content: &str) {
        let mut all_commands = Vec::new();

        for line in content.lines() {
            // El formato de zsh puede ser:
            // 1. Con timestamp: ": timestamp:0;comando"
            // 2. Sin timestamp (simple): "comando"
            // 3. Líneas que continúan de comandos multilínea

            let command = if line.starts_with(':') && line.contains(';') {
                // Formato con timestamp: buscar el ; y tomar lo que sigue
                line.split_once(';').map(|(_, cmd)| cmd).unwrap_or(line)
            } else {
                // Formato simple o continuación
                line
            };

            let command = command.trim();
            if !command.is_empty() && Self::is_valid_utf8_command(command) {
                all_commands.push(command.to_string());
            }
        }

        // all_commands ahora tiene: [más antiguo, ..., más reciente]
        // Eliminar duplicados manteniendo SOLO la última (más reciente) ocurrencia
        use std::collections::HashMap;
        let mut last_index: HashMap<String, usize> = HashMap::new();

        for (i, cmd) in all_commands.iter().enumerate() {
            last_index.insert(cmd.clone(), i);
        }

        // Construir lista final con comandos únicos en orden
        for (i, cmd) in all_commands.into_iter().enumerate() {
            // Solo agregar si este índice es la última ocurrencia del comando
            if last_index.get(&cmd) == Some(&i) {
                self.commands.push(cmd);
            }
        }

        // Mantener solo los últimos max_size comandos
        if self.commands.len() > self.max_size {
            let start = self.commands.len() - self.max_size;
            self.commands = self.commands[start..].to_vec();
        }

        // Debug: mostrar últimos 10 comandos únicos
        eprintln!("[HISTORIAL] Últimos 10 comandos únicos (más reciente al final del array):");
        for (i, cmd) in self.commands.iter().rev().take(10).enumerate() {
            eprintln!("  {} posiciones desde el final: {}", i, cmd);
        }
    }

    /// Parsea el historial de bash (líneas simples)
    fn parse_bash_history(&mut self, content: &str) {
        for line in content.lines() {
            let command = line.trim();
            if !command.is_empty() && Self::is_valid_utf8_command(command) {
                self.commands.push(command.to_string());
            }
        }

        // Mantener solo los últimos max_size comandos
        if self.commands.len() > self.max_size {
            let start = self.commands.len() - self.max_size;
            self.commands = self.commands[start..].to_vec();
        }
    }

    /// Agrega un comando al historial
    pub fn add_command(&mut self, command: String) {
        // No agregar comandos vacíos o duplicados consecutivos
        if command.trim().is_empty() {
            return;
        }

        // No agregar si es igual al último comando
        if let Some(last) = self.commands.last() {
            if last == &command {
                return;
            }
        }

        self.commands.push(command.clone());

        // Limitar tamaño
        if self.commands.len() > self.max_size {
            self.commands.remove(0);
        }

        // Guardar en archivo inmediatamente
        self.save_to_file(&command);
    }

    /// Guarda un comando en el archivo de historial
    fn save_to_file(&self, command: &str) {
        // Detectar el shell actual
        let shell = std::env::var("SHELL").unwrap_or_default();
        let is_zsh = shell.contains("zsh");
        let is_bash = shell.contains("bash");

        if is_zsh {
            // Guardar en formato zsh
            let zsh_path = Self::get_home_path(".zsh_history");
            if let Some(path) = zsh_path {
                // Formato zsh con timestamp
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let line = format!(": {}:0;{}\n", timestamp, command);

                // Agregar al final del archivo
                if let Err(e) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .and_then(|mut file| {
                        use std::io::Write;
                        file.write_all(line.as_bytes())
                    })
                {
                    eprintln!("[HISTORIAL] Error guardando en archivo zsh: {}", e);
                } else {
                    eprintln!("[HISTORIAL] Comando guardado en .zsh_history: {}", command);
                }
            }
        } else if is_bash {
            // Guardar en formato bash (línea simple)
            let bash_path = Self::get_home_path(".bash_history");
            if let Some(path) = bash_path {
                let line = format!("{}\n", command);

                if let Err(e) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .and_then(|mut file| {
                        use std::io::Write;
                        file.write_all(line.as_bytes())
                    })
                {
                    eprintln!("[HISTORIAL] Error guardando en archivo bash: {}", e);
                } else {
                    eprintln!("[HISTORIAL] Comando guardado en .bash_history: {}", command);
                }
            }
        }
    }

    /// Busca el comando más reciente que empiece con el prefijo dado
    pub fn find_suggestion(&self, prefix: &str) -> Option<String> {
        if prefix.is_empty() {
            return None;
        }

        eprintln!(
            "[HISTORIAL] Buscando sugerencia para: '{}' (historial tiene {} comandos)",
            prefix,
            self.commands.len()
        );

        // Mostrar últimos 5 comandos que coinciden para debug
        let matching: Vec<_> = self
            .commands
            .iter()
            .rev()
            .filter(|cmd| cmd.starts_with(prefix) && cmd.len() > prefix.len())
            .take(5)
            .collect();
        eprintln!(
            "[HISTORIAL] Comandos que coinciden (más reciente primero): {:?}",
            matching
        );

        // Buscar desde el más reciente al más antiguo (primero en la búsqueda reversa)
        let result = self
            .commands
            .iter()
            .rev()
            .find(|cmd| cmd.starts_with(prefix) && cmd.len() > prefix.len())
            .map(|cmd| cmd[prefix.len()..].to_string());

        eprintln!("[HISTORIAL] Resultado seleccionado: {:?}", result);
        result
    }

    /// Obtiene todos los comandos del historial
    pub fn get_commands(&self) -> &[String] {
        &self.commands
    }

    /// Limpia el historial
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Obtiene el número de comandos en el historial
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Verifica si el historial está vacío
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_find() {
        let mut history = CommandHistory::new(100);

        history.add_command("git status".to_string());
        history.add_command("git commit -m 'test'".to_string());
        history.add_command("cargo build".to_string());

        // Buscar por prefijo
        assert_eq!(
            history.find_suggestion("git"),
            Some(" commit -m 'test'".to_string())
        );
        assert_eq!(history.find_suggestion("cargo"), Some(" build".to_string()));
        assert_eq!(history.find_suggestion("npm"), None);
    }

    #[test]
    fn test_no_duplicates() {
        let mut history = CommandHistory::new(100);

        history.add_command("ls -la".to_string());
        history.add_command("ls -la".to_string());

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_max_size() {
        let mut history = CommandHistory::new(3);

        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_command("cmd3".to_string());
        history.add_command("cmd4".to_string());

        assert_eq!(history.len(), 3);
        assert_eq!(history.get_commands()[0], "cmd2");
    }
}
