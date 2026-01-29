//! Windows-specific implementations
//!
//! Provides Task Scheduler integration for Windows.

use anyhow::{Context, Result, bail};
use std::process::Command;
use tracing::{debug, info};

use crate::scheduler::TaskScheduler;

/// Windows Task Scheduler integration
pub struct WindowsTaskScheduler {
    task_name: String,
    task_path: String,
}

impl WindowsTaskScheduler {
    /// Create a new Windows Task Scheduler wrapper
    pub fn new(task_name: String, task_path: String) -> Result<Self> {
        Ok(Self {
            task_name,
            task_path,
        })
    }

    /// Get the full task path including name
    fn full_task_path(&self) -> String {
        format!("{}{}", self.task_path, self.task_name)
    }

    /// Run a schtasks command and return the output
    fn run_schtasks(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("schtasks")
            .args(args)
            .output()
            .context("Failed to execute schtasks command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("schtasks command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Query the task status
    fn query_task(&self) -> Result<String> {
        self.run_schtasks(&[
            "/Query",
            "/TN",
            &self.full_task_path(),
            "/V",
            "/FO",
            "LIST",
        ])
    }
}

impl TaskScheduler for WindowsTaskScheduler {
    fn is_enabled(&self) -> Result<bool> {
        let output = self.query_task()?;
        debug!("Task query output: {}", output);

        // Check for "Status: Disabled" or "Scheduled Task State: Disabled"
        let is_disabled = output.contains("Disabled");
        Ok(!is_disabled)
    }

    fn enable(&mut self) -> Result<()> {
        info!("Enabling task: {}", self.full_task_path());
        self.run_schtasks(&["/Change", "/TN", &self.full_task_path(), "/ENABLE"])?;
        info!("Task enabled successfully");
        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        info!("Disabling task: {}", self.full_task_path());
        self.run_schtasks(&["/Change", "/TN", &self.full_task_path(), "/DISABLE"])?;
        info!("Task disabled successfully");
        Ok(())
    }

    fn task_name(&self) -> &str {
        &self.task_name
    }
}

/// Show a Windows message box (requires windows crate features)
#[cfg(feature = "windows-gui")]
pub fn show_message_box(title: &str, message: &str) -> Result<()> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONINFORMATION};

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            None,
            PCWSTR(message_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }

    Ok(())
}

/// Show a message box using a simple fallback (console output)
#[cfg(not(feature = "windows-gui"))]
pub fn show_message_box(title: &str, message: &str) -> Result<()> {
    println!("[{}] {}", title, message);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_task_path() {
        let scheduler = WindowsTaskScheduler::new(
            "BrightStay".to_string(),
            "\\NChalapinyo\\".to_string(),
        )
        .unwrap();

        assert_eq!(scheduler.full_task_path(), "\\NChalapinyo\\BrightStay");
    }
}
