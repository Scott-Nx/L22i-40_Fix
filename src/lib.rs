//! BrightStay - Cross-platform monitor brightness manager
//!
//! This library provides monitor detection and brightness control via DDC/CI protocol,
//! with platform-specific scheduler integration for Windows and Linux.

pub mod config;
pub mod monitor;
pub mod platform;
pub mod scheduler;

pub use config::Config;
pub use monitor::MonitorController;
pub use scheduler::TaskScheduler;

/// Re-export common error types
pub use anyhow::{Context, Result};

/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
