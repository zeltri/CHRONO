use anyhow::Result;
use log::info;
use std::io::Read;
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{CursorIcon, WindowBuilder},
};

use terminal_ansi::AnsiParser;
use terminal_core::Screen;
use terminal_pty::Pty;
use terminal_renderer::CpuRenderer;

mod config;
use config::Config;

/// Copia texto al portapapeles usando múltiples métodos como fallback
fn copy_to_clipboard(text: &str) -> Result<()> {
    // En Linux, preferir herramientas del sistema para evitar timeouts del clipboard manager
    #[cfg(target_os = "linux")]
    {
        // Intentar primero con wl-copy (Wayland)
        if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                if stdin.write_all(text.as_bytes()).is_ok() {
                    drop(stdin);
                    if child.wait().is_ok() {
                        log::info!("Text copied to clipboard using wl-copy");
                        return Ok(());
                    }
                }
            }
        }

        // Intentar con xclip (X11)
        if let Ok(mut child) = std::process::Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                if stdin.write_all(text.as_bytes()).is_ok() {
                    drop(stdin);
                    if child.wait().is_ok() {
                        log::info!("Text copied to clipboard using xclip");
                        return Ok(());
                    }
                }
            }
        }

        // Intentar con xsel (X11)
        if let Ok(mut child) = std::process::Command::new("xsel")
            .arg("-b")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                if stdin.write_all(text.as_bytes()).is_ok() {
                    drop(stdin);
                    if child.wait().is_ok() {
                        log::info!("Text copied to clipboard using xsel");
                        return Ok(());
                    }
                }
            }
        }

        // Si ninguna herramienta del sistema funciona, intentar arboard como último recurso
        log::debug!("System clipboard tools not available, trying arboard...");
    }

    // En otros sistemas o como fallback en Linux, usar arboard
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(text) {
            Ok(_) => {
                log::info!("Text copied to clipboard using arboard");
                Ok(())
            }
            Err(e) => {
                log::error!("Arboard failed: {}", e);
                Err(anyhow::anyhow!("Failed to copy: {}", e))
            }
        },
        Err(e) => {
            log::error!("Could not create clipboard: {}", e);
            Err(anyhow::anyhow!("Failed to create clipboard: {}", e))
        }
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Iniciando emulador de terminal");

    // Cargar configuración
    let config = Config::load().unwrap_or_else(|e| {
        log::warn!(
            "No se pudo cargar la configuración: {}. Usando valores por defecto.",
            e
        );
        Config::default()
    });

    info!(
        "Configuración cargada: tamaño de fuente = {}",
        config.font.size
    );

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Terminal Emulator - Rust")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 500))
        .build(&event_loop)?;

    let window = Arc::new(window);

    // Crear contexto de softbuffer
    let context =
        softbuffer::Context::new(window.clone()).expect("Failed to create softbuffer context");
    let mut surface = softbuffer::Surface::new(&context, window.clone())
        .expect("Failed to create softbuffer surface");

    // Inicializar renderer
    let window_size = window.inner_size();
    let mut renderer = CpuRenderer::new(window_size.width, window_size.height);
    let (rows, cols) = renderer.calculate_grid_size();

    info!("Tamaño de grid: {}x{}", rows, cols);

    // Inicializar screen
    let screen = Arc::new(Mutex::new(Screen::new(rows, cols)));

    // Inicializar PTY
    let mut pty = Pty::spawn_default_shell(rows as u16, cols as u16)?;
    info!("Shell iniciado");

    // Estado de modificadores
    let mut modifiers_state = ModifiersState::empty();

    // Estado de selección con mouse
    let mut is_dragging = false;
    let mut last_mouse_position = (0.0, 0.0);

    // Canal para notificar cuando el PTY se cierra
    let (tx, rx) = mpsc::channel();

    // Thread para leer del PTY
    let screen_clone = Arc::clone(&screen);
    let window_clone = Arc::clone(&window);
    let mut pty_reader = pty.take_reader();
    let pty_writer = pty.writer();
    thread::spawn(move || {
        let mut parser = AnsiParser::new();
        let mut buffer = [0u8; 4096];

        loop {
            // Read bloqueante - espera hasta que haya datos disponibles
            // Esto hace que la respuesta sea instantánea sin consumir CPU
            match pty_reader.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let mut screen = screen_clone.lock().unwrap();
                    let responses = parser.process(&buffer[..n], &mut screen);
                    let title = screen.take_title();
                    drop(screen); // Liberar lock antes de request_redraw

                    // Responder consultas del terminal (DA, DSR, CPR)
                    if !responses.is_empty() {
                        if let Err(e) = pty_writer.write(&responses) {
                            log::error!("Error writing terminal response: {}", e);
                        }
                    }

                    // Aplicar título de ventana solicitado por OSC 0/2
                    if let Some(title) = title {
                        window_clone.set_title(&title);
                    }

                    // Notificar al event loop que hay cambios
                    window_clone.request_redraw();
                }
                Ok(_) => {
                    info!("PTY closed");
                    // Notificar al event loop que el PTY se cerró
                    let _ = tx.send(());
                    break;
                }
                Err(e) => {
                    log::error!("Error reading from PTY: {}", e);
                    // Notificar al event loop que hubo un error
                    let _ = tx.send(());
                    break;
                }
            }
            // Sin sleep - el read() bloqueante ya espera por datos
        }
    });

    // Event loop
    event_loop.run(move |event, elwt| {
        // Verificar si el PTY se cerró
        if rx.try_recv().is_ok() {
            info!("Shell terminado, cerrando aplicación");
            elwt.exit();
        }

        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Cerrando aplicación");
                    elwt.exit();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // Guardar la posición del cursor
                    last_mouse_position = (position.x, position.y);

                    let screen_guard = screen.lock().unwrap();
                    renderer.check_file_hover(&screen_guard, position.x, position.y);

                    // Si estamos arrastrando, actualizar la selección
                    if is_dragging {
                        let (row, col) = renderer.screen_to_grid(position.x, position.y);
                        drop(screen_guard);
                        let mut screen_mut = screen.lock().unwrap();
                        screen_mut.update_selection(row, col);
                        window.request_redraw();
                        return;
                    }

                    // Cambiar cursor si está sobre un enlace
                    if renderer.hovered_file.is_some() {
                        window.set_cursor_icon(CursorIcon::Pointer);
                    } else {
                        window.set_cursor_icon(CursorIcon::Text);
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    // Primero verificar si hay un archivo clickeable
                    if let Some((row, path, line)) = &renderer.hovered_file {
                        log::info!("Click on file: {} at line {:?} (row {})", path, line, row);

                        // Intentar abrir el archivo en VS Code
                        let _ = if let Some(line_num) = line {
                            std::process::Command::new("code")
                                .arg("--goto")
                                .arg(format!("{}:{}", path, line_num))
                                .spawn()
                        } else {
                            std::process::Command::new("code").arg(path).spawn()
                        };
                        return;
                    }

                    // Si no hay archivo, iniciar selección
                    let (row, col) =
                        renderer.screen_to_grid(last_mouse_position.0, last_mouse_position.1);

                    let mut screen_mut = screen.lock().unwrap();
                    screen_mut.start_selection(row, col);
                    is_dragging = true;
                    window.request_redraw();
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Left,
                    ..
                } => {
                    is_dragging = false;
                }
                WindowEvent::Resized(size) => {
                    info!("Ventana redimensionada: {}x{}", size.width, size.height);

                    renderer.resize(size.width, size.height);
                    let (new_rows, new_cols) = renderer.calculate_grid_size();

                    {
                        let mut screen = screen.lock().unwrap();
                        screen.resize(new_rows, new_cols);
                    }

                    if let Err(e) = pty.resize(new_rows as u16, new_cols as u16) {
                        log::error!("Error resizing PTY: {}", e);
                    }

                    window.request_redraw();
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers_state = new_modifiers.state();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key_code),
                            state: winit::event::ElementState::Pressed,
                            ref logical_key,
                            ref text,
                            ..
                        },
                    ..
                } => {
                    // Tab para aceptar sugerencia
                    if key_code == KeyCode::Tab {
                        let mut screen_guard = screen.lock().unwrap();
                        if let Some(suggestion) = screen_guard.get_active_suggestion() {
                            let suggestion_text = suggestion.to_string();

                            // Actualizar current_command sin renderizar (el PTY hará eco)
                            screen_guard.accept_suggestion_for_pty();
                            drop(screen_guard);

                            // Enviar la sugerencia al PTY para que el shell la reciba
                            if let Err(e) = pty.write(suggestion_text.as_bytes()) {
                                log::error!("Error writing suggestion to PTY: {}", e);
                            }

                            window.request_redraw();
                            return;
                        }
                    }

                    // Enter para guardar comando
                    if key_code == KeyCode::Enter {
                        screen.lock().unwrap().reset_user_input();
                    }

                    // Backspace para borrar
                    if key_code == KeyCode::Backspace {
                        screen.lock().unwrap().remove_user_input();
                    }

                    // Flecha derecha - aceptar sugerencia si existe, sino enviar al PTY para navegación
                    if key_code == KeyCode::ArrowRight {
                        let mut screen_guard = screen.lock().unwrap();
                        if let Some(suggestion) = screen_guard.get_active_suggestion() {
                            let suggestion_text = suggestion.to_string();

                            // Actualizar current_command sin renderizar (el PTY hará eco)
                            screen_guard.accept_suggestion_for_pty();
                            drop(screen_guard);

                            // Enviar la sugerencia al PTY para que el shell la reciba
                            if let Err(e) = pty.write(suggestion_text.as_bytes()) {
                                log::error!("Error writing suggestion to PTY: {}", e);
                            }

                            window.request_redraw();
                            return;
                        }
                        // Si no hay sugerencia, continuar para enviar la secuencia de escape al PTY
                    }

                    // Ctrl+Shift+V para pegar
                    if key_code == KeyCode::KeyV
                        && modifiers_state.control_key()
                        && modifiers_state.shift_key()
                    {
                        if let Ok(clipboard_text) =
                            arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
                        {
                            // Bracketed paste: envolver el texto si la app lo pidió (modo 2004)
                            let bracketed = screen.lock().unwrap().bracketed_paste;
                            let payload = if bracketed {
                                let mut bytes = b"\x1b[200~".to_vec();
                                bytes.extend_from_slice(clipboard_text.as_bytes());
                                bytes.extend_from_slice(b"\x1b[201~");
                                bytes
                            } else {
                                clipboard_text.into_bytes()
                            };
                            if let Err(e) = pty.write(&payload) {
                                log::error!("Error pasting text: {}", e);
                            }
                        }
                        return;
                    }

                    // Ctrl+Shift+C para copiar
                    if key_code == KeyCode::KeyC
                        && modifiers_state.control_key()
                        && modifiers_state.shift_key()
                    {
                        let screen_guard = screen.lock().unwrap();
                        if let Some(selected_text) = screen_guard.get_selected_text() {
                            drop(screen_guard); // Liberar el lock antes de copiar

                            match copy_to_clipboard(&selected_text) {
                                Ok(_) => {
                                    log::info!("Selection copied to clipboard");
                                }
                                Err(e) => {
                                    log::error!("Failed to copy to clipboard: {}", e);
                                }
                            }
                        } else {
                            log::warn!("No text selected to copy");
                        }
                        return;
                    }

                    // Manejar combinaciones especiales primero
                    if let Some(bytes) =
                        handle_key_with_modifiers(key_code, logical_key, modifiers_state)
                    {
                        if let Err(e) = pty.write(&bytes) {
                            log::error!("Error writing to PTY: {}", e);
                        }
                        // El redibujado se solicitará automáticamente cuando el PTY responda
                    } else if let Some(text_str) = text {
                        // Si hay texto y no es una combinación especial, enviarlo
                        if !modifiers_state.control_key() && !modifiers_state.alt_key() {
                            // Agregar al comando actual ANTES de enviar al PTY
                            for ch in text_str.chars() {
                                screen.lock().unwrap().add_user_input(ch);
                            }

                            if let Err(e) = pty.write(text_str.as_bytes()) {
                                log::error!("Error writing to PTY: {}", e);
                            }
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    let window_size = window.inner_size();
                    let (width, height) = (window_size.width, window_size.height);

                    if width > 0 && height > 0 {
                        // Verificar si el screen está dirty antes de renderizar
                        let should_render = {
                            let screen_guard = screen.lock().unwrap();
                            screen_guard.is_dirty()
                        };

                        if !should_render {
                            // No hay cambios, skip rendering para ahorrar CPU
                            return;
                        }

                        surface
                            .resize(
                                NonZeroU32::new(width).unwrap(),
                                NonZeroU32::new(height).unwrap(),
                            )
                            .unwrap();

                        let mut buffer = surface.buffer_mut().unwrap();

                        // Renderizar
                        {
                            let mut screen = screen.lock().unwrap();
                            renderer.render(&mut screen, &mut buffer);
                            screen.mark_clean(); // Marcar como limpio después de renderizar
                        }

                        buffer.present().unwrap();
                    }
                }
                _ => {}
            },
            // Removido: Event::AboutToWait con request_redraw() continuo
            // Ahora solo redibujamos cuando hay eventos reales (PTY output, input, resize, etc.)
            _ => {}
        }
    })?;

    Ok(())
}

fn handle_key_with_modifiers(
    key_code: KeyCode,
    logical_key: &winit::keyboard::Key,
    modifiers: ModifiersState,
) -> Option<Vec<u8>> {
    use winit::keyboard::Key;

    // Ctrl combinaciones
    if modifiers.control_key() && !modifiers.shift_key() && !modifiers.alt_key() {
        return match key_code {
            // Ctrl+A a Ctrl+Z (0x01 a 0x1A)
            KeyCode::KeyA => Some(vec![0x01]),
            KeyCode::KeyB => Some(vec![0x02]),
            KeyCode::KeyC => Some(vec![0x03]),
            KeyCode::KeyD => Some(vec![0x04]),
            KeyCode::KeyE => Some(vec![0x05]),
            KeyCode::KeyF => Some(vec![0x06]),
            KeyCode::KeyG => Some(vec![0x07]),
            KeyCode::KeyH => Some(vec![0x08]),
            KeyCode::KeyI => Some(vec![0x09]),
            KeyCode::KeyJ => Some(vec![0x0A]),
            KeyCode::KeyK => Some(vec![0x0B]),
            KeyCode::KeyL => Some(vec![0x0C]),
            KeyCode::KeyM => Some(vec![0x0D]),
            KeyCode::KeyN => Some(vec![0x0E]),
            KeyCode::KeyO => Some(vec![0x0F]),
            KeyCode::KeyP => Some(vec![0x10]),
            KeyCode::KeyQ => Some(vec![0x11]),
            KeyCode::KeyR => Some(vec![0x12]),
            KeyCode::KeyS => Some(vec![0x13]),
            KeyCode::KeyT => Some(vec![0x14]),
            KeyCode::KeyU => Some(vec![0x15]),
            KeyCode::KeyV => Some(vec![0x16]),
            KeyCode::KeyW => Some(vec![0x17]),
            KeyCode::KeyX => Some(vec![0x18]),
            KeyCode::KeyY => Some(vec![0x19]),
            KeyCode::KeyZ => Some(vec![0x1A]),
            KeyCode::Space => Some(vec![0x00]),
            KeyCode::Backslash => Some(vec![0x1C]),
            KeyCode::BracketLeft => Some(vec![0x1B]),
            KeyCode::BracketRight => Some(vec![0x1D]),
            _ => None,
        };
    }

    // Alt combinaciones (ESC prefix)
    if modifiers.alt_key() && !modifiers.control_key() {
        if let Key::Character(c) = logical_key {
            let mut result = vec![0x1B]; // ESC
            result.extend_from_slice(c.as_bytes());
            return Some(result);
        }
    }

    // Teclas especiales sin modificadores
    match key_code {
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Backspace => Some(b"\x7f".to_vec()),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Escape => Some(b"\x1b".to_vec()),

        // Flechas
        KeyCode::ArrowUp => Some(b"\x1b[A".to_vec()),
        KeyCode::ArrowDown => Some(b"\x1b[B".to_vec()),
        KeyCode::ArrowRight => Some(b"\x1b[C".to_vec()),
        KeyCode::ArrowLeft => Some(b"\x1b[D".to_vec()),

        // Home/End
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),

        // Page Up/Down
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),

        // Insert/Delete
        KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),

        // Function keys
        KeyCode::F1 => Some(b"\x1bOP".to_vec()),
        KeyCode::F2 => Some(b"\x1bOQ".to_vec()),
        KeyCode::F3 => Some(b"\x1bOR".to_vec()),
        KeyCode::F4 => Some(b"\x1bOS".to_vec()),
        KeyCode::F5 => Some(b"\x1b[15~".to_vec()),
        KeyCode::F6 => Some(b"\x1b[17~".to_vec()),
        KeyCode::F7 => Some(b"\x1b[18~".to_vec()),
        KeyCode::F8 => Some(b"\x1b[19~".to_vec()),
        KeyCode::F9 => Some(b"\x1b[20~".to_vec()),
        KeyCode::F10 => Some(b"\x1b[21~".to_vec()),
        KeyCode::F11 => Some(b"\x1b[23~".to_vec()),
        KeyCode::F12 => Some(b"\x1b[24~".to_vec()),

        _ => None,
    }
}
