use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APPLICATION: &str = "mic-volume-control";

#[derive(Debug, Parser)]
#[command(name = "mic-volume-control")]
#[command(about = "Simple microphone volume control utility", long_about = None)]
#[command(version)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Suppress all output (silent mode)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Get or set microphone volume
    Volume {
        /// Volume level to set (0-100). If not specified, shows current volume
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        level: Option<u8>,
    },

    /// Install Windows Task Scheduler task for automatic volume control
    Install {
        /// Target volume level (0-100)
        #[arg(short, long, default_value = "95")]
        volume: u8,

        /// Run interval in minutes
        #[arg(short, long, default_value = "5")]
        interval: u32,
    },

    /// Uninstall Windows Task Scheduler task
    Uninstall,

    /// Show current configuration
    Config,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Target volume level (0.0 to 1.0, where 1.0 = 100%)
    #[serde(default = "default_volume")]
    pub target_volume: f32,

    /// Task run interval in minutes
    #[serde(default = "default_interval")]
    pub run_interval_minutes: u32,
}

fn default_volume() -> f32 {
    0.95
}

fn default_interval() -> u32 {
    5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_volume: default_volume(),
            run_interval_minutes: default_interval(),
        }
    }
}

impl Config {
    pub fn load_from_file() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let app_data =
            std::env::var("APPDATA").context("APPDATA environment variable not found")?;

        let mut path = PathBuf::from(app_data);
        path.push(APPLICATION);
        path.push("config.toml");

        Ok(path)
    }

    /// Display current configuration
    pub fn display(&self) {
        println!("Current Configuration:");
        println!("  Target Volume: {:.0}%", self.target_volume * 100.0);
        println!("  Run Interval: {} minutes", self.run_interval_minutes);

        if let Ok(path) = Self::get_config_path() {
            println!("\nConfig file: {}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.target_volume, 0.95);
        assert_eq!(config.run_interval_minutes, 5);
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.target_volume, deserialized.target_volume);
        assert_eq!(
            config.run_interval_minutes,
            deserialized.run_interval_minutes
        );
    }

    #[test]
    fn test_partial_config() {
        // Config with only target_volume (old format)
        let partial = "target_volume = 0.8";
        let config: Config = toml::from_str(partial).unwrap();
        assert_eq!(config.target_volume, 0.8);
        assert_eq!(config.run_interval_minutes, 5); // default value

        // Empty config
        let empty = "";
        let config: Config = toml::from_str(empty).unwrap();
        assert_eq!(config.target_volume, 0.95); // default value
        assert_eq!(config.run_interval_minutes, 5); // default value
    }
}
