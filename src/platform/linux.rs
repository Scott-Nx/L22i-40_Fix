//! Linux-specific implementations
//!
//! Provides systemd service integration for Linux.

use anyhow::{Context, Result, bail};
use std::process::Command;
use tracing::{debug, info};

use crate::scheduler::TaskScheduler;

/// systemd service scheduler for Linux
pub struct SystemdScheduler {
    service_name: String,
}

impl SystemdScheduler {
    /// Create a new systemd scheduler wrapper
    pub fn new(service_name: String) -> Result<Self> {
        // Ensure the service name ends with .service
        let service_name = if service_name.ends_with(".service") {
            service_name
        } else {
            format!("{}.service", service_name.to_lowercase())
        };

        Ok(Self { service_name })
    }

    /// Run a systemctl command and return the output
    fn run_systemctl(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("systemctl")
            .args(args)
            .output()
            .context("Failed to execute systemctl command")?;

        // systemctl returns non-zero for some queries (like is-enabled when disabled)
        // so we don't fail on non-zero exit codes
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Run a systemctl command that should succeed
    fn run_systemctl_checked(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("systemctl")
            .args(args)
            .output()
            .context("Failed to execute systemctl command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("systemctl command failed: {}", stderr);
        }

        Ok(())
    }

    /// Check if the service exists
    pub fn service_exists(&self) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(["list-unit-files", &self.service_name])
            .output()
            .context("Failed to check service existence")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&self.service_name))
    }

    /// Get the service status
    pub fn status(&self) -> Result<String> {
        let output = self.run_systemctl(&["status", &self.service_name])?;
        Ok(output)
    }

    /// Check if the service is active (running)
    pub fn is_active(&self) -> Result<bool> {
        let output = self.run_systemctl(&["is-active", &self.service_name])?;
        Ok(output.trim() == "active")
    }

    /// Start the service
    pub fn start(&self) -> Result<()> {
        info!("Starting service: {}", self.service_name);
        self.run_systemctl_checked(&["start", &self.service_name])?;
        info!("Service started successfully");
        Ok(())
    }

    /// Stop the service
    pub fn stop(&self) -> Result<()> {
        info!("Stopping service: {}", self.service_name);
        self.run_systemctl_checked(&["stop", &self.service_name])?;
        info!("Service stopped successfully");
        Ok(())
    }

    /// Restart the service
    pub fn restart(&self) -> Result<()> {
        info!("Restarting service: {}", self.service_name);
        self.run_systemctl_checked(&["restart", &self.service_name])?;
        info!("Service restarted successfully");
        Ok(())
    }

    /// Reload systemd daemon (after unit file changes)
    pub fn daemon_reload() -> Result<()> {
        info!("Reloading systemd daemon");
        let output = Command::new("systemctl")
            .args(["daemon-reload"])
            .output()
            .context("Failed to reload systemd daemon")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to reload daemon: {}", stderr);
        }

        Ok(())
    }
}

impl TaskScheduler for SystemdScheduler {
    fn is_enabled(&self) -> Result<bool> {
        let output = self.run_systemctl(&["is-enabled", &self.service_name])?;
        let status = output.trim();
        debug!("Service {} is-enabled: {}", self.service_name, status);
        Ok(status == "enabled")
    }

    fn enable(&mut self) -> Result<()> {
        info!("Enabling service: {}", self.service_name);
        self.run_systemctl_checked(&["enable", &self.service_name])?;
        info!("Service enabled successfully");
        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        info!("Disabling service: {}", self.service_name);
        self.run_systemctl_checked(&["disable", &self.service_name])?;
        info!("Service disabled successfully");
        Ok(())
    }

    fn task_name(&self) -> &str {
        &self.service_name
    }
}

/// Install the systemd service unit file
pub fn install_service_unit(service_content: &str, service_name: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let service_path = Path::new("/etc/systemd/system").join(service_name);

    info!("Installing service unit to: {}", service_path.display());
    fs::write(&service_path, service_content)
        .with_context(|| format!("Failed to write service unit to {}", service_path.display()))?;

    SystemdScheduler::daemon_reload()?;
    info!("Service unit installed successfully");

    Ok(())
}

/// Generate a systemd service unit for BrightStay
pub fn generate_service_unit(binary_path: &str, config_path: &str) -> String {
    format!(
        r#"[Unit]
Description=BrightStay Monitor Brightness Manager
After=graphical-session.target

[Service]
Type=oneshot
ExecStart={binary_path} run --config {config_path}
RemainAfterExit=no

[Install]
WantedBy=graphical-session.target
"#,
        binary_path = binary_path,
        config_path = config_path
    )
}

/// udev rule for monitor hotplug events
pub fn generate_udev_rule(binary_path: &str) -> String {
    format!(
        r#"# BrightStay - Adjust brightness on monitor connect
ACTION=="change", SUBSYSTEM=="drm", RUN+="{binary_path} run"
"#,
        binary_path = binary_path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_normalization() {
        let scheduler = SystemdScheduler::new("BrightStay".to_string()).unwrap();
        assert_eq!(scheduler.service_name, "brightstay.service");

        let scheduler2 = SystemdScheduler::new("brightstay.service".to_string()).unwrap();
        assert_eq!(scheduler2.service_name, "brightstay.service");
    }

    #[test]
    fn test_generate_service_unit() {
        let unit = generate_service_unit("/usr/bin/brightstay", "/etc/brightstay/config.toml");
        assert!(unit.contains("ExecStart=/usr/bin/brightstay"));
        assert!(unit.contains("Type=oneshot"));
    }
}
