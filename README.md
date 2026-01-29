# L22i-40 BrightStay

The utility designed to fix a firmware issue on the L22i-40 monitor. When the monitor wakes from sleep mode while in a custom user color mode (custom RGB settings), it incorrectly resets its brightness level to 100%. This tool detects the monitor state and reapplies the desired brightness setting automatically, preserving your calibration and preventing unwanted brightness jumps.

## Cross-Platform Rust Implementation

BrightStay is now available as a cross-platform Rust application that works on both Windows and Linux with a single binary, no runtime dependencies required.

### Building from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux: Install required system dependencies
sudo apt-get install libudev-dev pkg-config

# Build the release binary
cargo build --release

# The binary will be at: target/release/brightstay
```

### CLI Usage

```bash
# Initialize configuration file
brightstay config init

# List detected monitors (find your monitor's identifier)
brightstay list

# Run brightness adjustment manually
brightstay run

# Override brightness level
brightstay run --brightness 10

# Check current status
brightstay status

# Toggle scheduled task on/off
brightstay toggle

# Get/set brightness directly on a specific monitor
brightstay brightness --monitor 0
brightstay brightness --monitor 0 --set 5
```

### Configuration

Configuration file locations:
- **Linux**: `~/.config/brightstay/config.toml`
- **Windows**: `%APPDATA%\BrightStay\config.toml`

Example configuration (see `config.toml.example`):

```toml
[monitor]
identifier = "26542"          # Product code (Linux) or instance name (Windows)
brightness_level = 5          # Target brightness (0-100)
vcp_code = 16                 # DDC/CI VCP code for brightness (0x10)
friendly_name = "Lenovo L22i-40"

[scheduler]
enabled = true
task_name = "BrightStay"
task_path = "\\NChalapinyo\\"  # Windows Task Scheduler path
trigger = "on_connect"

[logging]
level = "info"
```

### Features

- **Single Binary**: No runtime dependencies (replaces ddcutil and MonitorConfig PowerShell module)
- **Cross-Platform**: Same codebase for Windows and Linux
- **DDC/CI Direct**: Native DDC/CI protocol communication via `ddc-hi` crate
- **TOML Configuration**: Easy-to-edit configuration files
- **Scheduler Integration**: Windows Task Scheduler and Linux systemd support
- **CLI Interface**: Full command-line interface with subcommands

---

## Legacy Scripts

The original PowerShell (Windows) and Bash (Linux) scripts are still available for reference.

## How it works

- Detects when the display resumes from sleep or the workstation unlocks.
- Reapplies brightness level it to the L22i-40 using the MonitorConfig module.
- Run manually or via Task Scheduler so it can react reliably after wake/unlock events.

## Requirements

### Windows (Legacy PowerShell):
- PowerShell 5.1 or PowerShell 7+
- PowerShell module: MonitorConfig (from PowerShell Gallery)
- DDC/CI enabled on your monitor (Hold button for a few seconds to toggle)

### Linux (Legacy Bash):
- systemd
- ddcutil


If your system enforces script signing, ensure your execution policy allows running this tool use `-ExecutionPolicy Bypass` in task action.

## Install the MonitorConfig module

1) Open an elevated PowerShell prompt if required by your environment.
2) Install the module from the PowerShell Gallery:

    Install-Module MonitorConfig

Verify installation:

    Get-Command -Module MonitorConfig

Update later if needed:

    Update-Module MonitorConfig

Uninstall:

    Uninstall-Module MonitorConfig

## Usage

You can run the script after installing MonitorConfig. Typical options:

- Manual run (ad-hoc):

  - Open PowerShell.
  - Run the main script in this repository (adjust the path to match your checkout).

    Example pattern:
        `pwsh -File .\path\to\YourMainScript.ps1`
         or
        `powershell.exe -File .\path\to\YourMainScript.ps1`

- Schedule to run after wake/unlock (recommended):

  - Create a Scheduled Task that triggers:
    - On workstation unlock, and/or
    - On resume from sleep (Power-Troubleshooter event or built-in Resume triggers).
  - Set the Action to run PowerShell with script:
        `powershell.exe -NoProfile -ExecutionPolicy Bypass -File "C:\path\to\YourMainScript.ps1"`
  - Run only when user is logged on (require session 1).
  - Run with highest privileges.

Notes:
- If you have multiple monitors, ensure the script targets the correct display ID. The MonitorConfig module typically exposes commands to enumerate and select displays—verify the monitor selection logic in the script.
- If DDC/CI is disabled in the monitor OSD, brightness commands will not take effect.
- Some docking stations or GPU drivers can block or proxy DDC/CI. If brightness is not applied, try connecting directly to the GPU or testing with a different cable/port.

## Troubleshooting

- Module not found:

      Import-Module MonitorConfig
      # If it fails, (re)install:
      Install-Module MonitorConfig -Scope CurrentUser

- Permissions/ExecutionPolicy:

      Get-ExecutionPolicy -List
      # Consider using a Scheduled Task action with: -ExecutionPolicy Bypass

- Confirm DDC/CI control:

  Use MonitorConfig commands to enumerate displays and attempt a small brightness change to validate that the channel works.

## Security considerations

- Use `-ExecutionPolicy Bypass` only in trusted scenarios.
- If running under a service account, restrict its rights to the minimum required.

---

## Third-party license: [MonitorConfig (MIT)](https://github.com/MartinGC94/MonitorConfig)

The [MonitorConfig (MIT)](https://github.com/MartinGC94/MonitorConfig) PowerShell module is licensed under the [MIT License](https://github.com/MartinGC94/MonitorConfig/blob/main/LICENSE). The following MIT license text is provided for convenience. Refer to the module’s source/distribution for the authoritative license file and copyright holders.
