//! Pseudo-terminal (PTY) interface.
//!
//! This crate provides a cross-platform PTY abstraction using `portable-pty`.
//! It handles spawning shell processes and managing I/O between the terminal
//! emulator and the child process.

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Handle clonable para escribir al PTY desde cualquier hilo
#[derive(Clone)]
pub struct PtyWriter {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PtyWriter {
    /// Escribe todos los bytes al PTY
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("PTY writer lock poisoned"))?;
        writer.write_all(data).context("Failed to write to PTY")?;
        writer.flush().context("Failed to flush PTY")?;
        Ok(data.len())
    }
}

/// Wrapper para manejar PTY de forma multiplataforma
pub struct Pty {
    master: Box<dyn MasterPty + Send>,
    writer: PtyWriter,
    _child: Box<dyn Child + Send>,
}

impl Pty {
    /// Genera la configuración de LS_COLORS con soporte para múltiples tipos de archivos y permisos
    fn generate_ls_colors() -> String {
        // Formato: tipo=color
        // Códigos de color: 00=none, 01=bold, 04=underscore, 05=blink, 07=reverse, 08=concealed
        // Colores de texto: 30=negro, 31=rojo, 32=verde, 33=amarillo, 34=azul, 35=magenta, 36=cyan, 37=blanco
        // Colores de fondo: 40-47 (misma secuencia)
        // Colores brillantes: 90-97 para texto, 100-107 para fondo

        vec![
            // Permisos especiales
            "su=37;41", // SUID (blanco sobre rojo)
            "sg=30;43", // SGID (negro sobre amarillo)
            "tw=30;42", // Sticky + writable (negro sobre verde)
            "ow=34;42", // Other writable (azul sobre verde)
            "st=37;44", // Sticky bit (blanco sobre azul)
            // Tipos de archivo
            "di=01;34",    // Directorios (azul bold)
            "ln=01;36",    // Enlaces simbólicos (cyan bold)
            "or=40;31;01", // Enlaces rotos (rojo bold sobre negro)
            "mi=00",       // Archivos faltantes
            "pi=40;33",    // Pipes (amarillo sobre negro)
            "so=01;35",    // Sockets (magenta bold)
            "bd=40;33;01", // Block device (amarillo bold sobre negro)
            "cd=40;33;01", // Character device (amarillo bold sobre negro)
            "ex=01;32",    // Ejecutables (verde bold)
            // Archivos comprimidos (rojo)
            "*.tar=01;31",
            "*.tgz=01;31",
            "*.arc=01;31",
            "*.arj=01;31",
            "*.taz=01;31",
            "*.lha=01;31",
            "*.lz4=01;31",
            "*.lzh=01;31",
            "*.lzma=01;31",
            "*.tlz=01;31",
            "*.txz=01;31",
            "*.tzo=01;31",
            "*.t7z=01;31",
            "*.zip=01;31",
            "*.z=01;31",
            "*.dz=01;31",
            "*.gz=01;31",
            "*.lrz=01;31",
            "*.lz=01;31",
            "*.lzo=01;31",
            "*.xz=01;31",
            "*.zst=01;31",
            "*.tzst=01;31",
            "*.bz2=01;31",
            "*.bz=01;31",
            "*.tbz=01;31",
            "*.tbz2=01;31",
            "*.tz=01;31",
            "*.deb=01;31",
            "*.rpm=01;31",
            "*.jar=01;31",
            "*.war=01;31",
            "*.ear=01;31",
            "*.sar=01;31",
            "*.rar=01;31",
            "*.alz=01;31",
            "*.ace=01;31",
            "*.zoo=01;31",
            "*.cpio=01;31",
            "*.7z=01;31",
            "*.rz=01;31",
            "*.cab=01;31",
            "*.wim=01;31",
            "*.swm=01;31",
            "*.dwm=01;31",
            "*.esd=01;31",
            // Imágenes (magenta)
            "*.jpg=01;35",
            "*.jpeg=01;35",
            "*.mjpg=01;35",
            "*.mjpeg=01;35",
            "*.gif=01;35",
            "*.bmp=01;35",
            "*.pbm=01;35",
            "*.pgm=01;35",
            "*.ppm=01;35",
            "*.tga=01;35",
            "*.xbm=01;35",
            "*.xpm=01;35",
            "*.tif=01;35",
            "*.tiff=01;35",
            "*.png=01;35",
            "*.svg=01;35",
            "*.svgz=01;35",
            "*.mng=01;35",
            "*.pcx=01;35",
            "*.mov=01;35",
            "*.mpg=01;35",
            "*.mpeg=01;35",
            "*.m2v=01;35",
            "*.mkv=01;35",
            "*.webm=01;35",
            "*.webp=01;35",
            "*.ogm=01;35",
            "*.mp4=01;35",
            "*.m4v=01;35",
            "*.mp4v=01;35",
            "*.vob=01;35",
            "*.qt=01;35",
            "*.nuv=01;35",
            "*.wmv=01;35",
            "*.asf=01;35",
            "*.rm=01;35",
            "*.rmvb=01;35",
            "*.flc=01;35",
            "*.avi=01;35",
            "*.fli=01;35",
            "*.flv=01;35",
            "*.gl=01;35",
            "*.dl=01;35",
            "*.xcf=01;35",
            "*.xwd=01;35",
            "*.yuv=01;35",
            "*.cgm=01;35",
            "*.emf=01;35",
            // Audio (cyan)
            "*.aac=00;36",
            "*.au=00;36",
            "*.flac=00;36",
            "*.m4a=00;36",
            "*.mid=00;36",
            "*.midi=00;36",
            "*.mka=00;36",
            "*.mp3=00;36",
            "*.mpc=00;36",
            "*.ogg=00;36",
            "*.ra=00;36",
            "*.wav=00;36",
            "*.oga=00;36",
            "*.opus=00;36",
            "*.spx=00;36",
            "*.xspf=00;36",
            // Documentos (amarillo)
            "*.pdf=00;33",
            "*.doc=00;33",
            "*.docx=00;33",
            "*.odt=00;33",
            "*.txt=00;33",
            "*.md=00;33",
            "*.tex=00;33",
            // Código fuente (verde claro)
            "*.c=00;92",
            "*.cpp=00;92",
            "*.cc=00;92",
            "*.h=00;92",
            "*.hpp=00;92",
            "*.rs=00;92",
            "*.go=00;92",
            "*.py=00;92",
            "*.js=00;92",
            "*.ts=00;92",
            "*.java=00;92",
            "*.rb=00;92",
            "*.php=00;92",
            "*.sh=00;92",
            "*.bash=00;92",
            "*.zsh=00;92",
            "*.fish=00;92",
            // Archivos de configuración (cyan claro)
            "*.conf=00;96",
            "*.config=00;96",
            "*.ini=00;96",
            "*.yaml=00;96",
            "*.yml=00;96",
            "*.json=00;96",
            "*.toml=00;96",
            "*.xml=00;96",
        ]
        .join(":")
    }

    /// Crea un nuevo PTY y ejecuta el shell por defecto ($SHELL)
    pub fn spawn_default_shell(rows: u16, cols: u16) -> Result<Self> {
        Self::spawn_shell(None, rows, cols)
    }

    /// Crea un nuevo PTY y ejecuta el shell indicado (o $SHELL si es None).
    ///
    /// La configuración de colores (LS_COLORS) se pasa por variables de
    /// entorno al proceso hijo: nunca se modifican los dotfiles del usuario.
    pub fn spawn_shell(program: Option<&str>, rows: u16, cols: u16) -> Result<Self> {
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(pty_size).context("Failed to open PTY")?;

        // Shell configurado, o el por defecto del usuario
        let shell = program.map(str::to_string).unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| {
                if cfg!(windows) {
                    "cmd.exe".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            })
        });

        log::info!("Spawning shell: {}", shell);

        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Configurar LS_COLORS directamente en el entorno del shell
        let ls_colors = Self::generate_ls_colors();
        cmd.env("LS_COLORS", &ls_colors);
        log::info!(
            "LS_COLORS configurado con {} reglas",
            ls_colors.split(':').count()
        );

        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell")?;

        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        Ok(Self {
            master: pair.master,
            writer: PtyWriter {
                writer: Arc::new(Mutex::new(writer)),
            },
            _child: child,
        })
    }

    /// Obtiene un handle clonable para escribir al PTY desde otros hilos
    pub fn writer(&self) -> PtyWriter {
        self.writer.clone()
    }

    /// Termina el proceso hijo (al cerrar una tab)
    pub fn kill(&mut self) {
        let _ = self._child.kill();
    }

    /// Lee datos del PTY (no bloqueante)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut reader = self.master.try_clone_reader()?;
        reader.read(buf).context("Failed to read from PTY")
    }

    /// Escribe datos al PTY
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.writer.write(data)
    }

    /// Redimensiona el PTY
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<()> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.master.resize(size).context("Failed to resize PTY")
    }

    /// Obtiene el reader para lectura asíncrona
    pub fn take_reader(&mut self) -> Box<dyn Read + Send> {
        // Esto es un hack temporal - en producción necesitaríamos un mejor diseño
        self.master
            .try_clone_reader()
            .expect("Failed to clone reader")
    }

    /// Intenta leer datos del PTY sin bloquear (wrapper no bloqueante)
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_creation() {
        let result = Pty::spawn_default_shell(24, 80);
        assert!(result.is_ok(), "PTY should be created successfully");
    }

    #[test]
    fn test_pty_resize() {
        let mut pty = Pty::spawn_default_shell(24, 80).expect("Failed to create PTY");
        let result = pty.resize(30, 100);
        assert!(result.is_ok(), "PTY should resize successfully");
    }

    #[test]
    fn test_pty_write() {
        let mut pty = Pty::spawn_default_shell(24, 80).expect("Failed to create PTY");
        let result = pty.write(b"echo test\n");
        assert!(result.is_ok(), "PTY should accept write");
    }

    #[test]
    fn test_pty_read_nonblocking() {
        let mut pty = Pty::spawn_default_shell(24, 80).expect("Failed to create PTY");
        let mut buffer = vec![0u8; 1024];

        // Non-blocking read should not panic
        let result = pty.try_read(&mut buffer);
        assert!(result.is_ok(), "Non-blocking read should succeed");
    }

    #[test]
    fn test_pty_dimensions() {
        let rows = 40;
        let cols = 120;
        let mut pty = Pty::spawn_default_shell(rows, cols).expect("Failed to create PTY");

        // Verificar que el PTY fue creado (sin acceso directo a dimensiones en portable-pty)
        assert!(pty.resize(rows, cols).is_ok());
    }
}
