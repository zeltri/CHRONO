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

        self.commands.push(command);

        // Limitar tamaño
        if self.commands.len() > self.max_size {
            self.commands.remove(0);
        }
    }

    /// Busca el comando más reciente que empiece con el prefijo dado
    pub fn find_suggestion(&self, prefix: &str) -> Option<String> {
        if prefix.is_empty() {
            return None;
        }

        // Buscar desde el más reciente al más antiguo
        self.commands
            .iter()
            .rev()
            .find(|cmd| cmd.starts_with(prefix) && cmd.len() > prefix.len())
            .map(|cmd| cmd[prefix.len()..].to_string())
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
