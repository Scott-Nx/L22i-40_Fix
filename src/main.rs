//! BrightStay CLI - Cross-platform monitor brightness manager
//!
//! A command-line tool for controlling monitor brightness via DDC/CI.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use brightstay::config::Config;
use brightstay::monitor::MonitorController;
use brightstay::scheduler::{Scheduler, TaskScheduler};

/// BrightStay - Cross-platform monitor brightness manager
#[derive(Parser)]
#[command(name = "brightstay")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<std::path::PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run brightness adjustment on target monitor
    Run {
        /// Override brightness level (0-100)
        #[arg(short, long)]
        brightness: Option<u8>,
    },

    /// Toggle scheduled task on/off
    Toggle,

    /// Show current status
    Status,

    /// List all detected monitors
    List,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Get or set brightness directly
    Brightness {
        /// Monitor index (from 'list' command)
        #[arg(short, long)]
        monitor: Option<usize>,

        /// Set brightness level (0-100), or get if not specified
        #[arg(short, long)]
        set: Option<u8>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Create default configuration file
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,

        /// Use Windows-style identifiers
        #[arg(long)]
        windows: bool,
    },

    /// Show configuration file path
    Path,
}

fn setup_logging(verbose: bool, debug: bool) {
    let level = if debug {
        Level::DEBUG
    } else if verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    let filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).compact())
        .with(filter)
        .init();
}

fn load_config(path: Option<&std::path::Path>) -> Result<Config> {
    let config_path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(Config::default_path);

    if config_path.exists() {
        Config::load(&config_path)
    } else {
        info!(
            "Config file not found at {}, using defaults",
            config_path.display()
        );
        Ok(Config::default_l22i_40())
    }
}

fn cmd_run(config: &Config, brightness_override: Option<u8>) -> Result<()> {
    let mut monitor_config = config.monitor.clone();

    if let Some(brightness) = brightness_override {
        monitor_config.brightness_level = brightness;
    }

    let controller = MonitorController::new(monitor_config);
    controller.run()
}

fn cmd_toggle(config: &Config) -> Result<()> {
    let mut scheduler = Scheduler::new(config.scheduler.clone())?;
    let new_state = scheduler.toggle()?;

    let status_msg = if new_state {
        "Task now enabled"
    } else {
        "Task now disabled"
    };

    println!("{}", status_msg);
    info!("{}", status_msg);

    // On Windows, we might want to show a message box
    #[cfg(target_os = "windows")]
    {
        use brightstay::platform::windows::show_message_box;
        show_message_box("BrightStay", status_msg)?;
    }

    Ok(())
}

fn cmd_status(config: &Config) -> Result<()> {
    println!("BrightStay Status");
    println!("=================");
    println!();

    // Monitor configuration
    println!("Monitor Configuration:");
    println!(
        "  Identifier: {}",
        config.monitor.identifier
    );
    println!(
        "  Brightness: {}%",
        config.monitor.brightness_level
    );
    println!(
        "  VCP Code: 0x{:02X}",
        config.monitor.vcp_code
    );
    if let Some(ref name) = config.monitor.friendly_name {
        println!("  Friendly Name: {}", name);
    }
    println!();

    // Try to detect monitor
    let controller = MonitorController::new(config.monitor.clone());
    match controller.detect_target_monitor()? {
        Some(index) => {
            println!("Target Monitor: Connected (index {})", index);

            // Try to get current brightness
            if let Ok(Some(current)) = controller.get_brightness() {
                println!("Current Brightness: {}%", current);
            }
        }
        None => {
            println!("Target Monitor: Not connected");
        }
    }
    println!();

    // Scheduler status
    println!("Scheduler:");
    println!("  Task Name: {}", config.scheduler.task_name);

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        match Scheduler::new(config.scheduler.clone()) {
            Ok(scheduler) => match scheduler.is_enabled() {
                Ok(enabled) => {
                    println!(
                        "  Status: {}",
                        if enabled { "Enabled" } else { "Disabled" }
                    );
                }
                Err(e) => {
                    println!("  Status: Unknown ({})", e);
                }
            },
            Err(e) => {
                println!("  Status: Error ({})", e);
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        println!("  Status: Not supported on this platform");
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    let monitors = MonitorController::list_monitors()?;

    if monitors.is_empty() {
        println!("No monitors detected.");
        println!();
        println!("Note: DDC/CI must be enabled in your monitor's OSD settings.");
        return Ok(());
    }

    println!("Detected Monitors:");
    println!("==================");
    println!();

    for monitor in &monitors {
        println!("Monitor {}:", monitor.index);
        if let Some(ref model) = monitor.model {
            println!("  Model: {}", model);
        }
        if let Some(ref mfr) = monitor.manufacturer_id {
            println!("  Manufacturer: {}", mfr);
        }
        if let Some(code) = monitor.product_code {
            println!("  Product Code: {}", code);
        }
        if let Some(ref serial) = monitor.serial {
            println!("  Serial: {}", serial);
        }
        println!("  Backend: {}", monitor.backend);
        println!();
    }

    Ok(())
}

fn cmd_config_show(config: &Config) -> Result<()> {
    let toml_str = toml::to_string_pretty(config).context("Failed to serialize config")?;
    println!("{}", toml_str);
    Ok(())
}

fn cmd_config_init(force: bool, windows_style: bool) -> Result<()> {
    let config_path = Config::default_path();

    if config_path.exists() && !force {
        anyhow::bail!(
            "Config file already exists at {}. Use --force to overwrite.",
            config_path.display()
        );
    }

    let config = if windows_style {
        Config::default_l22i_40_windows()
    } else {
        Config::default_l22i_40()
    };

    config.save(&config_path)?;
    println!("Configuration file created at: {}", config_path.display());
    println!();
    println!("Edit this file to configure your monitor settings.");

    Ok(())
}

fn cmd_config_path() -> Result<()> {
    let path = Config::default_path();
    println!("{}", path.display());
    Ok(())
}

fn cmd_brightness(config: &Config, monitor_idx: Option<usize>, set_value: Option<u8>) -> Result<()> {
    use ddc_hi::{Ddc, Display};
    use brightstay::monitor::VCP_BRIGHTNESS;

    let displays: Vec<Display> = Display::enumerate();

    if displays.is_empty() {
        println!("No monitors detected.");
        return Ok(());
    }

    let index = monitor_idx.unwrap_or_else(|| {
        // Try to find target monitor from config
        let controller = MonitorController::new(config.monitor.clone());
        controller
            .detect_target_monitor()
            .ok()
            .flatten()
            .unwrap_or(0)
    });

    let Some(mut display) = displays.into_iter().nth(index) else {
        anyhow::bail!("Monitor index {} not found", index);
    };

    if let Some(value) = set_value {
        display
            .handle
            .set_vcp_feature(VCP_BRIGHTNESS, value as u16)
            .context("Failed to set brightness")?;
        println!("Brightness set to {}% on monitor {}", value, index);
    } else {
        let value = display
            .handle
            .get_vcp_feature(VCP_BRIGHTNESS)
            .context("Failed to get brightness")?;
        println!(
            "Monitor {}: Brightness = {}% (max: {})",
            index,
            value.value(),
            value.maximum()
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose, cli.debug);

    let config = load_config(cli.config.as_deref())?;

    match cli.command {
        Commands::Run { brightness } => cmd_run(&config, brightness),
        Commands::Toggle => cmd_toggle(&config),
        Commands::Status => cmd_status(&config),
        Commands::List => cmd_list(),
        Commands::Config { action } => match action {
            ConfigAction::Show => cmd_config_show(&config),
            ConfigAction::Init { force, windows } => cmd_config_init(force, windows),
            ConfigAction::Path => cmd_config_path(),
        },
        Commands::Brightness { monitor, set } => cmd_brightness(&config, monitor, set),
    }
}
