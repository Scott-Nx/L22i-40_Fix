//! Configuration management for BrightStay
//!
//! Handles loading and saving TOML configuration files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Monitor configuration
    pub monitor: MonitorConfig,

    /// Scheduler configuration
    #[serde(default)]
    pub scheduler: SchedulerConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Monitor identification and brightness settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Monitor identifier
    /// - Linux: Product code (e.g., "26542")
    /// - Windows: Instance name (e.g., "DISPLAY\\LEN67AE\\5&16e23401&0&UID277")
    pub identifier: String,

    /// Target brightness level (0-100)
    #[serde(default = "default_brightness")]
    pub brightness_level: u8,

    /// VCP code for brightness control (default: 0x10)
    #[serde(default = "default_vcp_code")]
    pub vcp_code: u8,

    /// Optional friendly name for the monitor
    pub friendly_name: Option<String>,
}

/// Scheduler configuration for automatic brightness adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Whether the scheduler is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Task name for the scheduler
    #[serde(default = "default_task_name")]
    pub task_name: String,

    /// Task path (Windows only)
    #[serde(default = "default_task_path")]
    pub task_path: String,

    /// Trigger type
    #[serde(default)]
    pub trigger: TriggerType,
}

/// Trigger types for the scheduler
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Trigger on monitor connect
    #[default]
    OnConnect,
    /// Trigger on system boot
    OnBoot,
    /// Trigger periodically
    Periodic,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log to file
    #[serde(default)]
    pub file: Option<PathBuf>,
}

// Default value functions
fn default_brightness() -> u8 {
    5
}

fn default_vcp_code() -> u8 {
    0x10
}

fn default_true() -> bool {
    true
}

fn default_task_name() -> String {
    "BrightStay".to_string()
}

fn default_task_path() -> String {
    "\\NChalapinyo\\".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            task_name: default_task_name(),
            task_path: default_task_path(),
            trigger: TriggerType::default(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
        }
    }
}

impl Config {
    /// Load configuration from a file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    /// Save configuration to a file path
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Get the default configuration file path
    pub fn default_path() -> PathBuf {
        // Platform-specific config paths
        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(app_data)
                .join("BrightStay")
                .join("config.toml")
        }

        #[cfg(target_os = "linux")]
        {
            let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config", home)
            });
            PathBuf::from(config_home)
                .join("brightstay")
                .join("config.toml")
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            PathBuf::from("config.toml")
        }
    }

    /// Create a default configuration for the L22i-40 monitor
    pub fn default_l22i_40() -> Self {
        Self {
            monitor: MonitorConfig {
                identifier: "26542".to_string(),
                brightness_level: 5,
                vcp_code: 0x10,
                friendly_name: Some("Lenovo L22i-40".to_string()),
            },
            scheduler: SchedulerConfig::default(),
            logging: LoggingConfig::default(),
        }
    }

    /// Create a default configuration for Windows with instance name
    #[allow(dead_code)]
    pub fn default_l22i_40_windows() -> Self {
        Self {
            monitor: MonitorConfig {
                identifier: "DISPLAY\\LEN67AE\\5&16e23401&0&UID277".to_string(),
                brightness_level: 5,
                vcp_code: 0x10,
                friendly_name: Some("Lenovo L22i-40".to_string()),
            },
            scheduler: SchedulerConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_l22i_40();
        assert_eq!(config.monitor.identifier, "26542");
        assert_eq!(config.monitor.brightness_level, 5);
        assert_eq!(config.monitor.vcp_code, 0x10);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default_l22i_40();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.monitor.identifier, config.monitor.identifier);
    }
}
