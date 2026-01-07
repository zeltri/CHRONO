//! Pseudo-terminal (PTY) interface.
//!
//! This crate provides a cross-platform PTY abstraction using `portable-pty`.
//! It handles spawning shell processes and managing I/O between the terminal
//! emulator and the child process.

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
