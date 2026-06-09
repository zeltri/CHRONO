use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    #[serde(default)]
    pub font: FontConfig,

    #[serde(default)]
    pub colors: ColorsConfig,

    #[serde(default)]
    pub terminal: TerminalConfig,

    #[serde(default)]
    pub rendering: RenderingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    #[serde(default = "default_font_size")]
    pub size: f32,

    #[serde(default = "default_font_family")]
    pub family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsConfig {
    #[serde(default = "default_foreground")]
    pub foreground: [u8; 3],

    #[serde(default = "default_background")]
    pub background: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: usize,

    pub shell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Habilita detección inteligente de contenido (logs, JSON, tablas, etc.)
    #[serde(default = "default_smart_detection")]
    pub smart_detection: bool,

    /// Habilita colores para JSON
    #[serde(default = "default_json_colors")]
    pub json_colors: bool,

    /// Habilita colores para logs
    #[serde(default = "default_log_colors")]
    pub log_colors: bool,

    /// Habilita detección de tablas
    #[serde(default = "default_table_detection")]
    pub table_detection: bool,
}


impl Default for FontConfig {
    fn default() -> Self {
        Self {
            size: default_font_size(),
            family: default_font_family(),
        }
    }
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            foreground: default_foreground(),
            background: default_background(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback(),
            shell: None,
        }
    }
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            smart_detection: default_smart_detection(),
            json_colors: default_json_colors(),
            log_colors: default_log_colors(),
            table_detection: default_table_detection(),
        }
    }
}

fn default_font_size() -> f32 {
    16.0
}

fn default_font_family() -> String {
    "CascadiaCode".to_string()
}

fn default_smart_detection() -> bool {
    true // Habilitado por defecto
}

fn default_json_colors() -> bool {
    true
}

fn default_log_colors() -> bool {
    true
}

fn default_table_detection() -> bool {
    true
}

fn default_foreground() -> [u8; 3] {
    [229, 229, 229]
}

fn default_background() -> [u8; 3] {
    [0, 0, 0]
}

fn default_scrollback() -> usize {
    10_000
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Crear config por defecto
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Crear directorio si no existe
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        Ok(config_dir.join("terminal-emulator").join("config.toml"))
    }
}
