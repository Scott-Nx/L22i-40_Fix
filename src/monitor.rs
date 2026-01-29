//! Monitor detection and brightness control via DDC/CI
//!
//! This module provides cross-platform monitor control using the DDC/CI protocol.

use anyhow::{Context, Result, bail};
use ddc_hi::{Ddc, Display};
use tracing::{debug, info, warn};

use crate::config::MonitorConfig;

/// VCP code for brightness control
pub const VCP_BRIGHTNESS: u8 = 0x10;

/// Information about a detected monitor
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Display index
    pub index: usize,

    /// Model name if available
    pub model: Option<String>,

    /// Manufacturer ID if available
    pub manufacturer_id: Option<String>,

    /// Serial number if available
    pub serial: Option<String>,

    /// Product code (from EDID)
    pub product_code: Option<u16>,

    /// Backend description
    pub backend: String,
}

impl std::fmt::Display for MonitorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Monitor {}", self.index)?;
        if let Some(ref model) = self.model {
            write!(f, " - {}", model)?;
        }
        if let Some(code) = self.product_code {
            write!(f, " (product: {})", code)?;
        }
        Ok(())
    }
}

/// Controller for monitor brightness operations
pub struct MonitorController {
    config: MonitorConfig,
}

impl MonitorController {
    /// Create a new monitor controller with the given configuration
    pub fn new(config: MonitorConfig) -> Self {
        Self { config }
    }

    /// List all available monitors
    pub fn list_monitors() -> Result<Vec<MonitorInfo>> {
        let mut monitors = Vec::new();

        for (index, display) in Display::enumerate().into_iter().enumerate() {
            let info = display.info;
            monitors.push(MonitorInfo {
                index,
                model: info.model_name.clone(),
                manufacturer_id: info.manufacturer_id.clone(),
                serial: info.serial_number.clone(),
                product_code: info.model_id,
                backend: info.backend.to_string(),
            });
        }

        Ok(monitors)
    }

    /// Detect the target monitor based on configuration
    pub fn detect_target_monitor(&self) -> Result<Option<usize>> {
        let identifier = &self.config.identifier;
        debug!("Searching for monitor with identifier: {}", identifier);

        for (index, display) in Display::enumerate().into_iter().enumerate() {
            let info = &display.info;

            // Check product code match (Linux style - numeric product code)
            if let Some(product_code) = info.model_id {
                if identifier == &product_code.to_string() {
                    info!(
                        "Found monitor by product code {} at index {}",
                        product_code, index
                    );
                    return Ok(Some(index));
                }
            }

            // Check model name match
            if let Some(ref model) = info.model_name {
                if model.contains(identifier) || identifier.contains(model) {
                    info!("Found monitor by model name '{}' at index {}", model, index);
                    return Ok(Some(index));
                }
            }

            // Check manufacturer + model ID combination (Windows style instance name partial match)
            if let Some(ref mfr) = info.manufacturer_id {
                let combined = format!("{}{:?}", mfr, info.model_id);
                if identifier.contains(mfr) {
                    info!("Found monitor by manufacturer '{}' at index {}", mfr, index);
                    return Ok(Some(index));
                }
                debug!("Monitor {} combined ID: {}", index, combined);
            }

            // Check serial number
            if let Some(ref serial) = info.serial_number {
                if identifier.contains(serial) || serial.contains(identifier) {
                    info!("Found monitor by serial '{}' at index {}", serial, index);
                    return Ok(Some(index));
                }
            }
        }

        warn!("Target monitor '{}' not found", identifier);
        Ok(None)
    }

    /// Set brightness on the target monitor
    pub fn set_brightness(&self) -> Result<bool> {
        let brightness = self.config.brightness_level;
        let vcp_code = self.config.vcp_code;

        info!(
            "Setting brightness to {}% using VCP code 0x{:02X}",
            brightness, vcp_code
        );

        let Some(index) = self.detect_target_monitor()? else {
            info!("Target monitor not connected - skipping brightness change");
            return Ok(false);
        };

        let displays: Vec<Display> = Display::enumerate();
        let Some(mut display) = displays.into_iter().nth(index) else {
            bail!("Failed to get display at index {}", index);
        };

        // Set brightness using DDC/CI
        display
            .handle
            .set_vcp_feature(vcp_code, brightness as u16)
            .with_context(|| {
                format!(
                    "Failed to set brightness to {} on monitor {}",
                    brightness, index
                )
            })?;

        info!(
            "Successfully set brightness to {}% on monitor {}",
            brightness, index
        );
        Ok(true)
    }

    /// Get current brightness from the target monitor
    pub fn get_brightness(&self) -> Result<Option<u8>> {
        let vcp_code = self.config.vcp_code;

        let Some(index) = self.detect_target_monitor()? else {
            return Ok(None);
        };

        let displays: Vec<Display> = Display::enumerate();
        let Some(mut display) = displays.into_iter().nth(index) else {
            bail!("Failed to get display at index {}", index);
        };

        let value = display
            .handle
            .get_vcp_feature(vcp_code)
            .with_context(|| format!("Failed to get brightness from monitor {}", index))?;

        Ok(Some(value.value() as u8))
    }

    /// Run the brightness adjustment (main entry point)
    pub fn run(&self) -> Result<()> {
        match self.set_brightness()? {
            true => info!("Brightness adjustment completed successfully"),
            false => info!("Monitor not connected - no action taken"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_info_display() {
        let info = MonitorInfo {
            index: 0,
            model: Some("Test Monitor".to_string()),
            manufacturer_id: Some("TST".to_string()),
            serial: None,
            product_code: Some(12345),
            backend: "test".to_string(),
        };

        let display = format!("{}", info);
        assert!(display.contains("Monitor 0"));
        assert!(display.contains("Test Monitor"));
        assert!(display.contains("12345"));
    }
}
