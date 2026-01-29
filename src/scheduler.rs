//! Task scheduler abstraction for cross-platform scheduling
//!
//! Provides a trait-based abstraction for managing scheduled tasks
//! with platform-specific implementations.

use anyhow::Result;

use crate::config::SchedulerConfig;

/// Trait for task scheduler operations
pub trait TaskScheduler {
    /// Check if the scheduled task is currently enabled
    fn is_enabled(&self) -> Result<bool>;

    /// Enable the scheduled task
    fn enable(&mut self) -> Result<()>;

    /// Disable the scheduled task
    fn disable(&mut self) -> Result<()>;

    /// Toggle the scheduled task state
    fn toggle(&mut self) -> Result<bool> {
        if self.is_enabled()? {
            self.disable()?;
            Ok(false)
        } else {
            self.enable()?;
            Ok(true)
        }
    }

    /// Get the task name
    fn task_name(&self) -> &str;

    /// Get the current status as a string
    fn status(&self) -> Result<String> {
        let enabled = self.is_enabled()?;
        Ok(if enabled {
            format!("Task '{}' is enabled", self.task_name())
        } else {
            format!("Task '{}' is disabled", self.task_name())
        })
    }
}

/// Platform-agnostic scheduler that delegates to platform-specific implementations
pub struct Scheduler {
    config: SchedulerConfig,
    #[cfg(target_os = "windows")]
    inner: crate::platform::windows::WindowsTaskScheduler,
    #[cfg(target_os = "linux")]
    inner: crate::platform::linux::SystemdScheduler,
}

impl Scheduler {
    /// Create a new scheduler with the given configuration
    #[cfg(target_os = "windows")]
    pub fn new(config: SchedulerConfig) -> Result<Self> {
        let inner = crate::platform::windows::WindowsTaskScheduler::new(
            config.task_name.clone(),
            config.task_path.clone(),
        )?;
        Ok(Self { config, inner })
    }

    #[cfg(target_os = "linux")]
    pub fn new(config: SchedulerConfig) -> Result<Self> {
        let inner = crate::platform::linux::SystemdScheduler::new(config.task_name.clone())?;
        Ok(Self { config, inner })
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    pub fn new(config: SchedulerConfig) -> Result<Self> {
        anyhow::bail!("Scheduler not supported on this platform")
    }

    /// Get the scheduler configuration
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
impl TaskScheduler for Scheduler {
    fn is_enabled(&self) -> Result<bool> {
        self.inner.is_enabled()
    }

    fn enable(&mut self) -> Result<()> {
        self.inner.enable()
    }

    fn disable(&mut self) -> Result<()> {
        self.inner.disable()
    }

    fn task_name(&self) -> &str {
        self.inner.task_name()
    }
}

/// Stub implementation for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
impl TaskScheduler for Scheduler {
    fn is_enabled(&self) -> Result<bool> {
        anyhow::bail!("Scheduler not supported on this platform")
    }

    fn enable(&mut self) -> Result<()> {
        anyhow::bail!("Scheduler not supported on this platform")
    }

    fn disable(&mut self) -> Result<()> {
        anyhow::bail!("Scheduler not supported on this platform")
    }

    fn task_name(&self) -> &str {
        &self.config.task_name
    }
}
