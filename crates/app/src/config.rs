use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub font: FontConfig,

    #[serde(default)]
    pub colors: ColorsConfig,

    #[serde(default)]
    pub terminal: TerminalConfig,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            font: FontConfig::default(),
            colors: ColorsConfig::default(),
            terminal: TerminalConfig::default(),
        }
    }
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

fn default_font_size() -> f32 {
    16.0
}

fn default_font_family() -> String {
    "CascadiaCode".to_string()
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
