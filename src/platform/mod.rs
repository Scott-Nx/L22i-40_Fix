//! Platform-specific implementations
//!
//! This module contains platform-specific code for Windows and Linux.

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

/// Re-export the current platform's scheduler
#[cfg(target_os = "windows")]
pub use windows::WindowsTaskScheduler as PlatformScheduler;

#[cfg(target_os = "linux")]
pub use linux::SystemdScheduler as PlatformScheduler;
