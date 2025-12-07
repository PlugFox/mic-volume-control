use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APPLICATION: &str = "mic-volume-control";

#[derive(Debug, Parser)]
#[command(name = "mic-volume-control")]
#[command(about = "Automatically maintains microphone volume at a configured level", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Target volume level (0-100)
    #[arg(short = 'v', long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub volume: Option<u8>,

    /// Check interval in milliseconds
    #[arg(short = 'i', long, value_parser = clap::value_parser!(u64).range(100..))]
    pub interval: Option<u64>,

    /// Disable system tray icon
    #[arg(long)]
    pub no_tray: bool,

    /// Disable autostart
    #[arg(long)]
    pub no_autostart: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Install autostart task
    Install,

    /// Uninstall autostart task
    Uninstall,

    /// Show current configuration
    Config,

    /// Get current microphone volume
    GetVolume,

    /// Set microphone volume once (without monitoring)
    SetVolume {
        /// Volume level (0-100)
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        volume: u8,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Target volume level (0.0 to 1.0, where 1.0 = 100%)
    pub target_volume: f32,

    /// How often to check the volume in milliseconds
    pub check_interval_ms: u64,

    /// Enable system tray icon
    pub enable_tray: bool,

    /// Enable autostart with Windows
    pub enable_autostart: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_volume: 0.95, // 95%
            check_interval_ms: 300_000, // 5 minutes (300,000 ms)
            enable_tray: true,
            enable_autostart: true,
        }
    }
}

impl Config {
    /// Load config from file, merging with CLI arguments
    pub fn load(cli: &Cli) -> Result<Self> {
        let mut config = if let Ok(file_config) = Self::load_from_file() {
            file_config
        } else {
            Self::default()
        };

        // Override with CLI arguments
        if let Some(volume) = cli.volume {
            config.target_volume = volume as f32 / 100.0;
        }

        if let Some(interval) = cli.interval {
            config.check_interval_ms = interval;
        }

        if cli.no_tray {
            config.enable_tray = false;
        }

        if cli.no_autostart {
            config.enable_autostart = false;
        }

        // Validate
        config.validate()?;

        // Save the merged config back to file
        config.save()?;

        Ok(config)
    }

    pub fn load_from_file() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Err(anyhow::anyhow!("Config file not found"));
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if !(0.0..=1.0).contains(&self.target_volume) {
            anyhow::bail!("target_volume must be between 0.0 and 1.0");
        }

        if self.check_interval_ms < 100 {
            anyhow::bail!("check_interval_ms must be at least 100ms");
        }

        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let app_data = std::env::var("APPDATA")
            .context("APPDATA environment variable not found")?;

        // Create path: AppData/Roaming/mic-volume-control/config.toml
        let mut path = PathBuf::from(app_data);
        path.push(APPLICATION);
        path.push("config.toml");

        Ok(path)
    }

    pub fn get_config_dir() -> Result<PathBuf> {
        let config_path = Self::get_config_path()?;
        config_path.parent()
            .map(|p| p.to_path_buf())
            .context("Failed to get parent directory of config path")
    }

    pub fn get_log_path() -> Result<PathBuf> {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .context("LOCALAPPDATA environment variable not found")?;

        // Create path: AppData/Local/mic-volume-control/logs/app.log
        let mut path = PathBuf::from(local_app_data);
        path.push(APPLICATION);
        path.push("logs");
        fs::create_dir_all(&path)?;
        path.push("app.log");

        Ok(path)
    }

    /// Display current configuration
    pub fn display(&self) {
        println!("Current Configuration:");
        println!("  Target Volume: {:.0}%", self.target_volume * 100.0);
        println!("  Check Interval: {}ms", self.check_interval_ms);
        println!("  System Tray: {}", if self.enable_tray { "enabled" } else { "disabled" });
        println!("  Autostart: {}", if self.enable_autostart { "enabled" } else { "disabled" });

        if let Ok(path) = Self::get_config_path() {
            println!("\nConfig file: {}", path.display());
        }

        if let Ok(path) = Self::get_log_path() {
            println!("Log file: {}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_volume() {
        let mut config = Config::default();
        config.target_volume = 1.5;
        assert!(config.validate().is_err());

        config.target_volume = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_interval() {
        let mut config = Config::default();
        config.check_interval_ms = 50;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.target_volume, deserialized.target_volume);
        assert_eq!(config.check_interval_ms, deserialized.check_interval_ms);
    }
}
