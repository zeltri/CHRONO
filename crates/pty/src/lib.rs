use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::Read;

#[cfg(unix)]
use std::os::unix::io::FromRawFd;

/// Wrapper para manejar PTY de forma multiplataforma
pub struct Pty {
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send>,
}

impl Pty {
    /// Crea un nuevo PTY y ejecuta el shell por defecto
    pub fn spawn_default_shell(rows: u16, cols: u16) -> Result<Self> {
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(pty_size).context("Failed to open PTY")?;

        // Obtener el shell por defecto
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd.exe".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });

        log::info!("Spawning shell: {}", shell);

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");

        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell")?;

        Ok(Self {
            master: pair.master,
            _child: child,
        })
    }

    /// Lee datos del PTY (no bloqueante)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut reader = self.master.try_clone_reader()?;
        reader.read(buf).context("Failed to read from PTY")
    }

    /// Escribe datos al PTY
    #[cfg(unix)]
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        use std::io::Write;

        // En Unix, obtenemos el file descriptor y escribimos directamente
        if let Some(fd) = self.master.as_raw_fd() {
            let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
            let result = file.write(data);
            std::mem::forget(file); // No cerrar el fd porque no es nuestro

            result.context("Failed to write to PTY")
        } else {
            anyhow::bail!("PTY file descriptor not available")
        }
    }

    #[cfg(not(unix))]
    pub fn write(&mut self, _data: &[u8]) -> Result<usize> {
        anyhow::bail!("Write not implemented for non-Unix platforms")
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
}
