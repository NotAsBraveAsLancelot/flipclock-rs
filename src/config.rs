use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("Failed to deserialize TOML: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("Invalid hex color: {0}")]
    InvalidHexColor(String),

    #[error("Failed to find home directory")]
    HomeDirNotFound,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for RgbColor {
    fn default() -> Self {
        RgbColor {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

impl FromStr for RgbColor {
    type Err = ConfigError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(ConfigError::InvalidHexColor(hex.to_string()));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| ConfigError::InvalidHexColor(hex.to_string()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| ConfigError::InvalidHexColor(hex.to_string()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| ConfigError::InvalidHexColor(hex.to_string()))?;

        Ok(RgbColor { r, g, b })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "ThemeConfig::default_background_color")]
    pub background_color: String,
    #[serde(default = "ThemeConfig::default_background_opacity")]
    pub background_opacity: f32,
    #[serde(default = "ThemeConfig::default_card_color")]
    pub card_color: String,
    #[serde(default = "ThemeConfig::default_card_opacity")]
    pub card_opacity: f32,
    #[serde(default = "ThemeConfig::default_card_border_color")]
    pub card_border_color: String,
    #[serde(default = "ThemeConfig::default_card_border_size")]
    pub card_border_size: u32,
    #[serde(default = "ThemeConfig::default_card_rounded_corners")]
    pub card_rounded_corners: bool,
    #[serde(default = "ThemeConfig::default_card_gap")]
    pub card_gap: i32,
    #[serde(default = "ThemeConfig::default_number_color")]
    pub number_color: String,
    #[serde(default = "ThemeConfig::default_font_path")]
    pub font_path: String,
}

impl ThemeConfig {
    fn default_background_color() -> String {
        "#0F0F0F".to_string()
    }
    fn default_background_opacity() -> f32 {
        0.8
    }
    fn default_card_color() -> String {
        "#000000".to_string()
    }
    fn default_card_opacity() -> f32 {
        0.4
    }
    fn default_card_border_color() -> String {
        "#FFFFFF".to_string()
    }
    fn default_card_border_size() -> u32 {
        2
    }
    fn default_card_rounded_corners() -> bool {
        true
    }
    fn default_card_gap() -> i32 {
        5
    }
    fn default_number_color() -> String {
        "#FFFFFF".to_string()
    }
    fn default_font_path() -> String {
        "/usr/share/fonts/TTF/DejaVuSans.ttf".to_string()
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            background_color: Self::default_background_color(),
            background_opacity: Self::default_background_opacity(),
            card_color: Self::default_card_color(),
            card_opacity: Self::default_card_opacity(),
            card_border_color: Self::default_card_border_color(),
            card_border_size: Self::default_card_border_size(),
            card_rounded_corners: Self::default_card_rounded_corners(),
            card_gap: Self::default_card_gap(),
            number_color: Self::default_number_color(),
            font_path: Self::default_font_path(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub show_seconds: bool,
    #[serde(default)]
    pub show_ampm: bool,
    #[serde(default)]
    pub show_leading_zero: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            show_seconds: false,
            show_ampm: false,
            show_leading_zero: false,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub animation: AnimationConfig,
    #[serde(default)]
    pub window: WindowConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationConfig {
    #[serde(default = "AnimationConfig::default_enabled")]
    pub enabled: bool,
    #[serde(default = "AnimationConfig::default_duration_ms")]
    pub duration_ms: u32,
}

impl AnimationConfig {
    fn default_enabled() -> bool {
        true
    }
    fn default_duration_ms() -> u32 {
        500
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            enabled: Self::default_enabled(),
            duration_ms: Self::default_duration_ms(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "WindowConfig::default_width")]
    pub width: u32,
    #[serde(default = "WindowConfig::default_height")]
    pub height: u32,
    #[serde(default = "WindowConfig::default_fs")]
    pub fullscreen: bool,
    #[serde(default = "WindowConfig::default_any_close")]
    pub close_on_any_input: bool,
}

impl WindowConfig {
    fn default_width() -> u32 {
        1280
    }
    fn default_height() -> u32 {
        720
    }
    fn default_fs() -> bool {
        true
    }
    fn default_any_close() -> bool {
        false
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            width: Self::default_width(),
            height: Self::default_height(),
            fullscreen: true,
            close_on_any_input: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: ThemeConfig::default(),
            display: DisplayConfig::default(),
            animation: AnimationConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::default_config_path()?;
        Self::load_from_path(&config_path)
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self, ConfigError> {
        match fs::read_to_string(path) {
            Ok(content) => {
                let config: Config = toml::from_str(&content).map_err(ConfigError::TomlDeError)?;
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(ConfigError::IoError(e)),
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::default_config_path()?;
        Self::save_to_path(self, &config_path)
    }

    pub fn save_to_path(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(&self).map_err(ConfigError::TomlSerError)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn default_config_path() -> Result<PathBuf, ConfigError> {
        let mut path = dirs::home_dir().ok_or(ConfigError::HomeDirNotFound)?;
        path.push(".config/flipclock/config.toml");
        Ok(path)
    }

    pub fn to_clock_settings(&self) -> ClockSettings {
        ClockSettings {
            background_color: self.theme.background_color.parse().unwrap_or_default(),
            background_opacity: self.theme.background_opacity,
            font_color: self.theme.number_color.parse().unwrap_or_default(),
            show_seconds: self.display.show_seconds,
            show_leading_zero: self.display.show_leading_zero,
            use_24hour: !self.display.show_ampm,
            width: self.window.width,
            height: self.window.height,
            fullscreen: self.window.fullscreen,
            close_on_any_input: self.window.close_on_any_input,
            animate_flip: self.animation.enabled,
            animation_duration_ms: self.animation.duration_ms,
            font_path: self.theme.font_path.clone(),
            card_color: self.theme.card_color.parse().unwrap_or_default(),
            card_border_color: self.theme.card_border_color.parse().unwrap_or_default(),
            card_border_size: self.theme.card_border_size,
            card_gap: self.theme.card_gap,
            card_rounded_corners: self.theme.card_rounded_corners,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClockSettings {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub use_24hour: bool,
    pub show_leading_zero: bool,
    pub background_color: RgbColor,
    pub background_opacity: f32,
    pub font_color: RgbColor,
    pub animate_flip: bool,
    pub animation_duration_ms: u32,
    pub close_on_any_input: bool,
    pub show_seconds: bool,
    pub font_path: String,
    pub card_color: RgbColor,
    pub card_border_color: RgbColor,
    pub card_border_size: u32,
    pub card_gap: i32,
    pub card_rounded_corners: bool,
}

impl Default for ClockSettings {
    fn default() -> Self {
        Config::default().to_clock_settings()
    }
}
