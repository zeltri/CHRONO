use anyhow::Result;
use log::info;
use std::io::Read;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{CursorIcon, Window, WindowBuilder},
};

use terminal_ansi::AnsiParser;
use terminal_core::{MouseMode, Screen};
use terminal_pty::Pty;
use terminal_renderer::CpuRenderer;

mod config;
use config::Config;

/// Una sesión de terminal: su pantalla, su PTY y su hilo lector
struct Session {
    screen: Arc<Mutex<Screen>>,
    pty: Pty,
    title: Arc<Mutex<String>>,
    alive: Arc<AtomicBool>,
}

impl Session {
    fn spawn(
        rows: usize,
        cols: usize,
        shell: Option<&str>,
        scrollback: usize,
        window: Arc<Window>,
    ) -> Result<Self> {
        let mut screen = Screen::new(rows, cols);
        screen.set_max_scrollback(scrollback);
        let screen = Arc::new(Mutex::new(screen));

        let mut pty = Pty::spawn_shell(shell, rows as u16, cols as u16)?;

        let default_title = shell
            .map(|s| s.rsplit('/').next().unwrap_or(s).to_string())
            .or_else(|| {
                std::env::var("SHELL")
                    .ok()
                    .and_then(|s| s.rsplit('/').next().map(str::to_string))
            })
            .unwrap_or_else(|| "shell".to_string());
        let title = Arc::new(Mutex::new(default_title));
        let alive = Arc::new(AtomicBool::new(true));

        let mut reader = pty.take_reader();
        let writer = pty.writer();
        let screen_c = Arc::clone(&screen);
        let title_c = Arc::clone(&title);
        let alive_c = Arc::clone(&alive);

        thread::spawn(move || {
            let mut parser = AnsiParser::new();
            let mut buffer = [0u8; 4096];

            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let mut screen = screen_c.lock().unwrap();
                        let responses = parser.process(&buffer[..n], &mut screen);
                        let new_title = screen.take_title();
                        drop(screen);

                        // Responder consultas del terminal (DA, DSR, CPR)
                        if !responses.is_empty() {
                            if let Err(e) = writer.write(&responses) {
                                log::error!("Error writing terminal response: {}", e);
                            }
                        }

                        if let Some(t) = new_title {
                            *title_c.lock().unwrap() = t;
                        }

                        window.request_redraw();
                    }
                    _ => {
                        info!("PTY closed");
                        alive_c.store(false, Ordering::SeqCst);
                        window.request_redraw();
                        break;
                    }
                }
            }
        });

        Ok(Self {
            screen,
            pty,
            title,
            alive,
        })
    }
}

/// Copia texto al portapapeles usando múltiples métodos como fallback
fn copy_to_clipboard(text: &str) -> Result<()> {
    // En Linux, preferir herramientas del sistema para evitar timeouts del clipboard manager
    #[cfg(target_os = "linux")]
    {
        for (cmd, args) in [
            ("wl-copy", vec![]),
            ("xclip", vec!["-selection", "clipboard"]),
            ("xsel", vec!["-b"]),
        ] {
            if let Ok(mut child) = std::process::Command::new(cmd)
                .args(&args)
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Text copied to clipboard using {}", cmd);
                            return Ok(());
                        }
                    }
                }
            }
        }
        log::debug!("System clipboard tools not available, trying arboard...");
    }

    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| anyhow::anyhow!("Failed to create clipboard: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("Failed to copy: {}", e))?;
    Ok(())
}

/// Abre un hyperlink OSC 8 con el handler del sistema
fn open_hyperlink(uri: &str) {
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(target_os = "macos"))]
    let opener = "xdg-open";

    let _ = std::process::Command::new(opener).arg(uri).spawn();
}

/// Codifica un evento de mouse en formato SGR (modo 1006)
fn mouse_sgr(button: u8, col: usize, row: usize, press: bool) -> Vec<u8> {
    format!(
        "\x1b[<{};{};{}{}",
        button,
        col + 1,
        row + 1,
        if press { 'M' } else { 'm' }
    )
    .into_bytes()
}

/// Codifica un evento de mouse en formato legacy X10
fn mouse_legacy(button: u8, col: usize, row: usize) -> Vec<u8> {
    let cx = (col + 1).min(222) as u8 + 32;
    let cy = (row + 1).min(222) as u8 + 32;
    vec![0x1b, b'[', b'M', 32 + button, cx, cy]
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Iniciando CHRONO");

    let config = Config::load().unwrap_or_else(|e| {
        log::warn!(
            "No se pudo cargar la configuración: {}. Usando defaults.",
            e
        );
        Config::default()
    });

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("CHRONO")
        .with_inner_size(winit::dpi::LogicalSize::new(900, 560))
        .build(&event_loop)?;
    let window = Arc::new(window);

    let context =
        softbuffer::Context::new(window.clone()).expect("Failed to create softbuffer context");
    let mut surface = softbuffer::Surface::new(&context, window.clone())
        .expect("Failed to create softbuffer surface");

    let window_size = window.inner_size();
    let mut renderer = CpuRenderer::new(window_size.width, window_size.height, config.font.size);
    let (rows, cols) = renderer.calculate_grid_size();
    info!("Tamaño de grid: {}x{}", rows, cols);

    let shell = config.terminal.shell.clone();
    let scrollback = config.terminal.scrollback_lines;

    let mut sessions = vec![Session::spawn(
        rows,
        cols,
        shell.as_deref(),
        scrollback,
        Arc::clone(&window),
    )?];
    let mut active: usize = 0;

    // Estado de input
    let mut modifiers_state = ModifiersState::empty();
    let mut is_dragging = false;
    let mut mouse_button_down = false;
    let mut last_mouse_position = (0.0_f64, 0.0_f64);
    let mut last_mouse_cell = (usize::MAX, usize::MAX);
    let mut last_click: Option<(Instant, usize, usize)> = None;
    let mut click_count: u8 = 0;
    let mut window_title = String::new();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        // Eliminar sesiones cuyo shell terminó
        if sessions.iter().any(|s| !s.alive.load(Ordering::SeqCst)) {
            sessions.retain(|s| s.alive.load(Ordering::SeqCst));
            if sessions.is_empty() {
                info!("Última sesión terminada, cerrando");
                elwt.exit();
                return;
            }
            active = active.min(sessions.len() - 1);
            window.request_redraw();
        }

        let Event::WindowEvent { event, .. } = event else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Cerrando aplicación");
                elwt.exit();
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                modifiers_state = new_modifiers.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                last_mouse_position = (position.x, position.y);
                let (row, col) = renderer.screen_to_grid(position.x, position.y);

                let session = &mut sessions[active];

                // Mouse tracking: reportar movimiento si la app lo pidió
                let (mouse_mode, mouse_sgr_enc) = {
                    let s = session.screen.lock().unwrap();
                    (s.mouse_mode, s.mouse_sgr)
                };
                let report_motion = !modifiers_state.shift_key()
                    && (mouse_mode == MouseMode::AnyEvent
                        || (mouse_mode == MouseMode::ButtonEvent && mouse_button_down));
                if report_motion && (row, col) != last_mouse_cell {
                    last_mouse_cell = (row, col);
                    // 32 = botón izquierdo + flag de motion; 35 = sin botón + motion
                    let button = if mouse_button_down { 32 } else { 35 };
                    let seq = if mouse_sgr_enc {
                        mouse_sgr(button, col, row, true)
                    } else {
                        mouse_legacy(button, col, row)
                    };
                    let _ = session.pty.write(&seq);
                    return;
                }

                {
                    let screen_guard = session.screen.lock().unwrap();
                    renderer.check_file_hover(&screen_guard, position.x, position.y);
                }

                if is_dragging {
                    let mut screen_mut = session.screen.lock().unwrap();
                    screen_mut.update_selection(row, col);
                    drop(screen_mut);
                    window.request_redraw();
                    return;
                }

                if renderer.hovered_file.is_some() || renderer.hovered_link.is_some() {
                    window.set_cursor_icon(CursorIcon::Pointer);
                } else {
                    window.set_cursor_icon(CursorIcon::Text);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines: i32 = match delta {
                    MouseScrollDelta::LineDelta(_, y) => (y * 3.0) as i32,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 16.0) as i32,
                };
                if lines == 0 {
                    return;
                }

                let session = &mut sessions[active];
                let (mouse_mode, mouse_sgr_enc, alt_screen) = {
                    let s = session.screen.lock().unwrap();
                    (s.mouse_mode, s.mouse_sgr, s.is_alt_screen())
                };

                if mouse_mode != MouseMode::None && !modifiers_state.shift_key() {
                    // La app quiere los eventos de rueda (modo mouse activo)
                    let (row, col) =
                        renderer.screen_to_grid(last_mouse_position.0, last_mouse_position.1);
                    let button = if lines > 0 { 64 } else { 65 };
                    for _ in 0..lines.unsigned_abs() {
                        let seq = if mouse_sgr_enc {
                            mouse_sgr(button, col, row, true)
                        } else {
                            mouse_legacy(button, col, row)
                        };
                        let _ = session.pty.write(&seq);
                    }
                } else if alt_screen {
                    // Convención: rueda = flechas en apps a pantalla completa
                    let seq: &[u8] = if lines > 0 { b"\x1b[A" } else { b"\x1b[B" };
                    for _ in 0..lines.unsigned_abs() {
                        let _ = session.pty.write(seq);
                    }
                } else {
                    // Scroll por el scrollback
                    let mut screen = session.screen.lock().unwrap();
                    if lines > 0 {
                        screen.scroll_view_up(lines as usize);
                    } else {
                        screen.scroll_view_down(lines.unsigned_abs() as usize);
                    }
                    drop(screen);
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                // Click en la barra de tabs
                if let Some(tab) =
                    renderer.tab_hit(last_mouse_position.0, last_mouse_position.1, sessions.len())
                {
                    if tab != active {
                        active = tab;
                        window.request_redraw();
                    }
                    return;
                }

                // Hyperlink OSC 8
                if let Some(uri) = renderer.hovered_link.clone() {
                    log::info!("Opening hyperlink: {}", uri);
                    open_hyperlink(&uri);
                    return;
                }

                // Referencia de archivo (stack trace)
                if let Some((row, path, line)) = &renderer.hovered_file {
                    log::info!("Click on file: {} at line {:?} (row {})", path, line, row);
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

                let (row, col) =
                    renderer.screen_to_grid(last_mouse_position.0, last_mouse_position.1);
                let session = &mut sessions[active];

                // Mouse tracking: reportar el click si la app lo pidió (Shift lo anula)
                let (mouse_mode, mouse_sgr_enc) = {
                    let s = session.screen.lock().unwrap();
                    (s.mouse_mode, s.mouse_sgr)
                };
                if mouse_mode != MouseMode::None && !modifiers_state.shift_key() {
                    mouse_button_down = true;
                    let seq = if mouse_sgr_enc {
                        mouse_sgr(0, col, row, true)
                    } else {
                        mouse_legacy(0, col, row)
                    };
                    let _ = session.pty.write(&seq);
                    return;
                }

                // Conteo de clicks: doble = palabra, triple = línea
                let now = Instant::now();
                click_count = match last_click {
                    Some((t, r, c))
                        if now.duration_since(t) < Duration::from_millis(400)
                            && r == row
                            && c == col =>
                    {
                        (click_count % 3) + 1
                    }
                    _ => 1,
                };
                last_click = Some((now, row, col));

                let mut screen_mut = session.screen.lock().unwrap();
                match click_count {
                    2 => screen_mut.select_word(row, col),
                    3 => screen_mut.select_line(row),
                    _ => {
                        screen_mut.start_selection(row, col);
                        is_dragging = true;
                    }
                }
                drop(screen_mut);
                window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                is_dragging = false;
                if mouse_button_down {
                    mouse_button_down = false;
                    let session = &mut sessions[active];
                    let (mouse_mode, mouse_sgr_enc) = {
                        let s = session.screen.lock().unwrap();
                        (s.mouse_mode, s.mouse_sgr)
                    };
                    if mouse_mode != MouseMode::None && mouse_mode != MouseMode::X10 {
                        let (row, col) =
                            renderer.screen_to_grid(last_mouse_position.0, last_mouse_position.1);
                        let seq = if mouse_sgr_enc {
                            mouse_sgr(0, col, row, false)
                        } else {
                            mouse_legacy(3, col, row)
                        };
                        let _ = session.pty.write(&seq);
                    }
                }
            }
            WindowEvent::Resized(size) => {
                info!("Ventana redimensionada: {}x{}", size.width, size.height);
                renderer.resize(size.width, size.height);
                let (new_rows, new_cols) = renderer.calculate_grid_size();

                for session in &mut sessions {
                    session.screen.lock().unwrap().resize(new_rows, new_cols);
                    if let Err(e) = session.pty.resize(new_rows as u16, new_cols as u16) {
                        log::error!("Error resizing PTY: {}", e);
                    }
                }
                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ref logical_key,
                        ref text,
                        ..
                    },
                ..
            } => {
                let ctrl_shift = modifiers_state.control_key() && modifiers_state.shift_key();

                // ===== Gestión de tabs =====
                if ctrl_shift && key_code == KeyCode::KeyT {
                    let (rows, cols) = renderer.calculate_grid_size();
                    match Session::spawn(
                        rows,
                        cols,
                        shell.as_deref(),
                        scrollback,
                        Arc::clone(&window),
                    ) {
                        Ok(session) => {
                            sessions.push(session);
                            active = sessions.len() - 1;
                            window.request_redraw();
                        }
                        Err(e) => log::error!("No se pudo crear la tab: {}", e),
                    }
                    return;
                }
                if ctrl_shift && key_code == KeyCode::KeyW {
                    let mut session = sessions.remove(active);
                    session.pty.kill();
                    if sessions.is_empty() {
                        elwt.exit();
                        return;
                    }
                    active = active.min(sessions.len() - 1);
                    window.request_redraw();
                    return;
                }
                if ctrl_shift && key_code == KeyCode::ArrowRight {
                    active = (active + 1) % sessions.len();
                    window.request_redraw();
                    return;
                }
                if ctrl_shift && key_code == KeyCode::ArrowLeft {
                    active = (active + sessions.len() - 1) % sessions.len();
                    window.request_redraw();
                    return;
                }

                // ===== Navegación por prompts (OSC 133) =====
                if ctrl_shift && key_code == KeyCode::ArrowUp {
                    sessions[active]
                        .screen
                        .lock()
                        .unwrap()
                        .view_to_prev_prompt();
                    window.request_redraw();
                    return;
                }
                if ctrl_shift && key_code == KeyCode::ArrowDown {
                    sessions[active]
                        .screen
                        .lock()
                        .unwrap()
                        .view_to_next_prompt();
                    window.request_redraw();
                    return;
                }

                // ===== Scroll por el scrollback =====
                if modifiers_state.shift_key() && key_code == KeyCode::PageUp {
                    let mut screen = sessions[active].screen.lock().unwrap();
                    let page = screen.rows.saturating_sub(1).max(1);
                    screen.scroll_view_up(page);
                    drop(screen);
                    window.request_redraw();
                    return;
                }
                if modifiers_state.shift_key() && key_code == KeyCode::PageDown {
                    let mut screen = sessions[active].screen.lock().unwrap();
                    let page = screen.rows.saturating_sub(1).max(1);
                    screen.scroll_view_down(page);
                    drop(screen);
                    window.request_redraw();
                    return;
                }

                // ===== Clipboard =====
                if ctrl_shift && key_code == KeyCode::KeyV {
                    if let Ok(clipboard_text) =
                        arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
                    {
                        let session = &mut sessions[active];
                        let bracketed = session.screen.lock().unwrap().bracketed_paste;
                        let payload = if bracketed {
                            let mut bytes = b"\x1b[200~".to_vec();
                            bytes.extend_from_slice(clipboard_text.as_bytes());
                            bytes.extend_from_slice(b"\x1b[201~");
                            bytes
                        } else {
                            clipboard_text.into_bytes()
                        };
                        if let Err(e) = session.pty.write(&payload) {
                            log::error!("Error pasting text: {}", e);
                        }
                    }
                    return;
                }
                if ctrl_shift && key_code == KeyCode::KeyC {
                    let screen_guard = sessions[active].screen.lock().unwrap();
                    if let Some(selected_text) = screen_guard.get_selected_text() {
                        drop(screen_guard);
                        match copy_to_clipboard(&selected_text) {
                            Ok(_) => log::info!("Selection copied to clipboard"),
                            Err(e) => log::error!("Failed to copy: {}", e),
                        }
                    } else {
                        log::warn!("No text selected to copy");
                    }
                    return;
                }

                // ===== Sugerencias =====
                if key_code == KeyCode::Tab || key_code == KeyCode::ArrowRight {
                    let session = &mut sessions[active];
                    let mut screen_guard = session.screen.lock().unwrap();
                    if let Some(suggestion) = screen_guard.get_active_suggestion() {
                        let suggestion_text = suggestion.to_string();
                        screen_guard.accept_suggestion_for_pty();
                        drop(screen_guard);

                        if let Err(e) = session.pty.write(suggestion_text.as_bytes()) {
                            log::error!("Error writing suggestion to PTY: {}", e);
                        }
                        window.request_redraw();
                        return;
                    }
                }

                if key_code == KeyCode::Enter {
                    sessions[active].screen.lock().unwrap().reset_user_input();
                }
                if key_code == KeyCode::Backspace {
                    sessions[active].screen.lock().unwrap().remove_user_input();
                }

                // Cualquier tecla normal vuelve la vista al presente
                sessions[active].screen.lock().unwrap().reset_view();

                // ===== Envío al PTY =====
                let session = &mut sessions[active];
                if let Some(bytes) =
                    handle_key_with_modifiers(key_code, logical_key, modifiers_state)
                {
                    if let Err(e) = session.pty.write(&bytes) {
                        log::error!("Error writing to PTY: {}", e);
                    }
                } else if let Some(text_str) = text {
                    if !modifiers_state.control_key() && !modifiers_state.alt_key() {
                        for ch in text_str.chars() {
                            session.screen.lock().unwrap().add_user_input(ch);
                        }
                        if let Err(e) = session.pty.write(text_str.as_bytes()) {
                            log::error!("Error writing to PTY: {}", e);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let window_size = window.inner_size();
                let (width, height) = (window_size.width, window_size.height);
                if width == 0 || height == 0 {
                    return;
                }

                // Actualizar título de ventana desde la sesión activa
                let active_title = sessions[active].title.lock().unwrap().clone();
                let new_title = if active_title.is_empty() {
                    "CHRONO".to_string()
                } else {
                    format!("{} — CHRONO", active_title)
                };
                if new_title != window_title {
                    window_title = new_title;
                    window.set_title(&window_title);
                }

                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();
                let mut buffer = surface.buffer_mut().unwrap();

                let tab_titles: Vec<String> = sessions
                    .iter()
                    .map(|s| s.title.lock().unwrap().clone())
                    .collect();

                {
                    let mut screen = sessions[active].screen.lock().unwrap();
                    renderer.render(&mut screen, &mut buffer, &tab_titles, active);
                    screen.mark_clean();
                }

                buffer.present().unwrap();
            }
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
