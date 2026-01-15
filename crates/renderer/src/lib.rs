//! Terminal rendering engine.
//!
//! This crate provides CPU-based rendering using `fontdue` for glyph rasterization.
//! Features include:
//! - OneDark Pro color theme
//! - Semantic context-aware colorization
//! - File type detection and highlighting
//! - Animated cursor with pulse effect

pub mod cpu;
mod theme;

pub use cpu::CpuRenderer;
pub use theme::ModernTheme;

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_core::Screen;

    #[test]
    fn test_theme_creation() {
        let theme = ModernTheme::default();

        // Verificar colores principales
        let bg = theme.bg_primary_u32();
        let fg = theme.fg_primary_u32();

        assert_ne!(bg, 0, "Background should not be black");
        assert_ne!(fg, 0, "Foreground should not be black");
        assert_ne!(bg, fg, "Background and foreground should be different");
    }

    #[test]
    fn test_theme_ansi_colors() {
        let theme = ModernTheme::default();

        // Verificar que todos los colores ANSI estén definidos
        for i in 0..16 {
            let color = theme.get_ansi_color(i);
            assert_ne!(color, 0, "ANSI color {} should be defined", i);
        }
    }

    #[test]
    fn test_theme_accent_colors() {
        let theme = ModernTheme::default();

        // Verificar colores de acento
        let blue = theme.accent_blue_u32();
        let green = theme.accent_green_u32();
        let red = theme.accent_red_u32();
        let yellow = theme.accent_yellow_u32();

        assert_ne!(blue, 0);
        assert_ne!(green, 0);
        assert_ne!(red, 0);
        assert_ne!(yellow, 0);

        // Verificar que sean diferentes
        assert_ne!(blue, green);
        assert_ne!(blue, red);
        assert_ne!(green, red);
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = CpuRenderer::new(800, 600);
        assert!(
            renderer.hovered_file.is_none(),
            "Renderer should be created successfully"
        );
    }

    #[test]
    fn test_renderer_resize() {
        let mut renderer = CpuRenderer::new(800, 600);
        renderer.resize(1024, 768);
        // No panic = success
    }

    #[test]
    fn test_renderer_with_empty_screen() {
        let mut renderer = CpuRenderer::new(800, 600);
        let mut screen = Screen::new(24, 80);
        let mut buffer = vec![0u32; 800 * 600];

        renderer.render(&mut screen, &mut buffer);
        // No panic = success, rendering completed
    }

    #[test]
    fn test_renderer_with_text() {
        let mut renderer = CpuRenderer::new(800, 600);
        let mut screen = Screen::new(24, 80);
        let mut buffer = vec![0u32; 800 * 600];

        // Escribir algo en el screen usando write_char
        screen.write_char('H');
        screen.write_char('i');

        renderer.render(&mut screen, &mut buffer);
        // No panic = success
    }

    #[test]
    fn test_theme_rgb_conversion() {
        // Test rgb_to_u32 conversion
        let white = ModernTheme::rgb_to_u32(255, 255, 255);
        let black = ModernTheme::rgb_to_u32(0, 0, 0);

        assert_eq!(white, 0xFFFFFFFF);
        assert_eq!(black, 0xFF000000);
    }

    #[test]
    fn test_get_file_color() {
        use terminal_core::{FileEntry, FileType};

        let renderer = CpuRenderer::new(800, 600);

        // Verificar que cada tipo de archivo tenga un color asignado
        let types = vec![
            FileType::Directory,
            FileType::Executable,
            FileType::Archive,
            FileType::Image,
            FileType::Video,
            FileType::Audio,
            FileType::Document,
            FileType::Code,
            FileType::RegularFile,
        ];

        for file_type in types {
            // Crear un FileEntry temporal para testear
            let entry = FileEntry {
                name: "test".to_string(),
                file_type: file_type.clone(),
                is_executable: file_type == FileType::Executable,
                start_col: 0,
                end_col: 4,
            };

            let color = renderer.get_file_color(&entry);
            assert_ne!(color, 0, "FileType {:?} should have a color", file_type);
        }
    }
}
