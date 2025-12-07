# Microphone Volume Control 🎤

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows)

A lightweight, efficient Windows application that automatically maintains your microphone volume at a configured level. Built with Rust for maximum performance and reliability.

## ✨ Features

- **Automatic Volume Control**: Continuously monitors and maintains microphone volume at your desired level
- **System Tray Integration**: Runs quietly in the background with a convenient system tray icon
- **Autostart Support**: Automatically starts with Windows via Task Scheduler
- **CLI Interface**: Full command-line interface for advanced users
- **TOML Configuration**: Easy-to-edit configuration file
- **Minimal Resource Usage**: Optimized for low CPU and memory footprint
- **Comprehensive Logging**: Detailed logs for troubleshooting

## 📦 Installation

### From Source

1. Install [Rust](https://www.rust-lang.org/tools/install) (1.75 or later)
2. Clone this repository:
   ```bash
   git clone https://github.com/plugfox/mic-volume-control.git
   cd mic-volume-control
   ```
3. Build the release version:
   ```bash
   cargo build --release
   ```
4. The executable will be in `target/release/mic-volume-control.exe`

### Pre-built Binaries

Download the latest release from the [Releases](https://github.com/plugfox/mic-volume-control/releases) page.

## 🚀 Quick Start

1. Run the application:
   ```bash
   mic-volume-control.exe
   ```

2. On first run, it will:
   - Create a default configuration file
   - Set microphone volume to 95%
   - Register autostart task
   - Show system tray icon

3. To customize settings, edit the configuration file at:
   ```
   %APPDATA%\mic-volume-control\config.toml
   ```

## 📖 Usage

### Basic Usage

Start with default settings (95% volume, 500ms check interval):
```bash
mic-volume-control.exe
```

### Command-Line Options

```bash
# Set custom volume (0-100)
mic-volume-control.exe --volume 90

# Set custom check interval in milliseconds
mic-volume-control.exe --interval 1000

# Disable system tray icon (console mode)
mic-volume-control.exe --no-tray

# Disable autostart
mic-volume-control.exe --no-autostart

# Combine options
mic-volume-control.exe -v 85 -i 500 --no-tray
```

### Subcommands

#### Install Autostart
```bash
mic-volume-control.exe install
```

#### Uninstall Autostart
```bash
mic-volume-control.exe uninstall
```

#### Show Configuration
```bash
mic-volume-control.exe config
```

#### Get Current Volume
```bash
mic-volume-control.exe get-volume
```

#### Set Volume (One-time)
```bash
mic-volume-control.exe set-volume 90
```

## ⚙️ Configuration

The configuration file is located at:
```
%APPDATA%\mic-volume-control\config.toml
```

Example configuration:
```toml
# Target volume level (0.0 to 1.0, where 1.0 = 100%)
target_volume = 0.95

# How often to check the volume in milliseconds
check_interval_ms = 500

# Enable system tray icon
enable_tray = true

# Enable autostart with Windows
enable_autostart = true
```

### Configuration Priority

Settings are applied in the following order (later overrides earlier):
1. Default values
2. Configuration file
3. Command-line arguments

## 🖥️ System Tray

When enabled, the system tray icon provides:
- **Status Display**: Shows target volume
- **Quick Actions**:
  - Open Config - Opens configuration directory
  - Pause Monitoring - (Future feature)
  - Quit - Exits the application

## 📝 Logs

Logs are stored at:
```
%LOCALAPPDATA%\mic-volume-control\logs\app.log
```

The log file contains:
- Application startup/shutdown events
- Volume changes and drift detection
- Errors and warnings
- Timestamps for all events

## 🔧 How It Works

1. **Initialization**:
   - Loads configuration from file
   - Initializes Windows Audio API (WASAPI)
   - Registers autostart task if enabled
   - Creates system tray icon

2. **Volume Monitoring**:
   - Checks microphone volume at configured intervals
   - Detects volume drift (>1% change)
   - Automatically corrects volume to target level

3. **Background Operation**:
   - Runs as a lightweight background process
   - Uses COM (Component Object Model) for audio control
   - Minimal CPU usage between checks (~0.1%)

## 🏗️ Architecture

```
┌─────────────────────┐
│   Main Application  │
└──────────┬──────────┘
           │
    ┌──────┴──────┬──────────────┬────────────┐
    │             │              │            │
┌───▼────┐  ┌────▼─────┐  ┌────▼────┐  ┌────▼─────┐
│ Audio  │  │ Config   │  │ Tray    │  │Scheduler │
│ Engine │  │ Manager  │  │ Icon    │  │(Autostart)│
└────┬───┘  └──────────┘  └─────────┘  └──────────┘
     │
     └──► Windows Audio API (WASAPI)
```

## 🛠️ Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run tests including ignored ones (requires microphone)
cargo test -- --ignored --test-threads=1
```

### Project Structure

```
mic-volume-control/
├── src/
│   ├── main.rs         # Application entry point and CLI handling
│   ├── audio.rs        # Windows Audio API wrapper with RAII COM guards
│   ├── config.rs       # Configuration management and CLI parser
│   ├── scheduler.rs    # Task Scheduler integration for autostart
│   └── tray.rs         # System tray implementation
├── Cargo.toml          # Dependencies and metadata
└── README.md           # This file
```

## 📋 Requirements

- **OS**: Windows 10 or later
- **Rust**: 1.75+ (for building from source)
- **Permissions**: Administrator rights for autostart installation

## 🔍 Troubleshooting

### Application won't start
- Check if microphone is connected and recognized by Windows
- Run as Administrator for first-time setup
- Check logs at `%LOCALAPPDATA%\mic-volume-control\logs\app.log`

### Volume keeps resetting
- Another application may be controlling microphone volume
- Check if exclusive mode is enabled in microphone properties
- Increase check interval if too aggressive

### Autostart not working
- Manually install: `mic-volume-control.exe install`
- Check Task Scheduler for "MicrophoneVolumeControl" task
- Ensure application path hasn't changed

### High CPU usage
- Increase `check_interval_ms` in config (e.g., 1000ms)
- Check for conflicting audio software

## 📊 Performance

- **Memory**: ~2-5 MB
- **CPU**: <0.1% (with 500ms interval)
- **Startup**: <100ms
- **Binary Size**: ~1.5 MB (release build, stripped)

## 🔒 Security

- ✅ No network connections
- ✅ No data collection
- ✅ Minimal Windows API usage
- ✅ Open source - audit the code yourself
- ✅ Uses RAII for COM resource management

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🙏 Acknowledgments

- Built with the [windows-rs](https://github.com/microsoft/windows-rs) crate
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- System tray via [tray-icon](https://github.com/tauri-apps/tray-icon)
- Configuration with [toml](https://github.com/toml-rs/toml)

## 💬 Support

If you find this project useful, please consider giving it a star ⭐

For issues and feature requests, please use the [GitHub Issues](https://github.com/plugfox/mic-volume-control/issues) page.

## 🗺️ Roadmap

- [ ] Multiple microphone support
- [ ] Profile-based configurations
- [ ] GUI configuration editor
- [ ] Volume scheduling (different levels at different times)
- [ ] Hotkey support
- [ ] Portable mode (no installation required)
- [ ] Update to windows 0.62+ when API stabilizes

---

**Made with ❤️ and Rust**
