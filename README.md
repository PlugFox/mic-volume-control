# Microphone Volume Control 🎤

[![CI](https://github.com/plugfox/mic-volume-control/workflows/CI/badge.svg)](https://github.com/plugfox/mic-volume-control/actions/workflows/ci.yml)
[![Lint](https://github.com/plugfox/mic-volume-control/workflows/Lint/badge.svg)](https://github.com/plugfox/mic-volume-control/actions/workflows/lint.yml)
[![Release](https://github.com/plugfox/mic-volume-control/workflows/Release/badge.svg)](https://github.com/plugfox/mic-volume-control/actions/workflows/release.yml)
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

Start with default settings (95% volume, 5 minute check interval):
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

# How often to check the volume in milliseconds (300000 = 5 minutes)
check_interval_ms = 300000

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

### Recommended Check Intervals

- **Normal use**: 300000ms (5 minutes) - Default, minimal CPU usage
- **Responsive**: 60000ms (1 minute) - Good balance
- **Very responsive**: 10000ms (10 seconds) - For active monitoring
- **Real-time**: 1000ms (1 second) - Only if needed, higher CPU usage

The default 5-minute interval is sufficient for most use cases, as volume changes are typically rare.

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

### Log Rotation

Logs are automatically rotated when they exceed 5 MB:
- Current log: `app.log`
- Rotated logs: `app.log.1`, `app.log.2`, `app.log.3`
- Maximum rotated logs kept: 3
- Oldest logs are automatically deleted

This ensures logs don't grow indefinitely while preserving recent history.

## 🔧 How It Works

1. **Initialization**:
   - Loads configuration from file
   - Initializes Windows Audio API (WASAPI)
   - Registers autostart task if enabled
   - Creates system tray icon

2. **Volume Monitoring**:
   - Checks microphone volume at configured intervals (default: every 5 minutes)
   - Detects volume drift (>1% change)
   - Automatically corrects volume to target level

3. **Background Operation**:
   - Runs as a lightweight background process
   - Uses COM (Component Object Model) for audio control
   - Sleeps between checks - virtually zero CPU usage with default interval

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

### Cargo Aliases (npm scripts для Rust)

Проект использует встроенные cargo aliases для упрощения разработки (определены в [.cargo/config.toml](.cargo/config.toml)):

```bash
# Проверка форматирования
cargo fmt-check

# Автоформатирование кода
cargo fmt-fix

# Линтинг с clippy
cargo lint

# Автоисправление проблем clippy
cargo lint-fix

# Проверка компиляции
cargo check-all

# Запуск тестов
cargo test-all

# Сборка release версии
cargo build-release

# Очистка
cargo clean-all
```

Для полной CI-проверки локально запустите:
```bash
# Проверка Rust кода
cargo fmt-check && cargo lint && cargo check-all && cargo test-all

# Проверка TOML файлов (требует taplo-cli)
taplo format --check && taplo lint

# Или всё вместе
cargo fmt-check && cargo lint && taplo format --check && taplo lint && cargo test-all
```

Установка taplo (для проверки TOML):
```bash
cargo install taplo-cli --locked
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build-release  # или cargo build --release

# Run tests
cargo test-all       # или cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run all tests
cargo test-all       # или cargo test

# Run tests including ignored ones (requires microphone)
cargo test -- --ignored --test-threads=1
```

### Project Structure

```
mic-volume-control/
├── .cargo/
│   └── config.toml     # Cargo aliases (npm scripts аналог)
├── .github/
│   └── workflows/
│       ├── ci.yml      # Continuous integration
│       ├── lint.yml    # Code quality checks
│       └── release.yml # Release automation
├── .vscode/
│   ├── tasks.json      # VS Code tasks (build, test, CI)
│   ├── settings.json   # Editor and Rust settings
│   ├── launch.json     # Debug configurations
│   ├── extensions.json # Recommended extensions
│   └── README.md       # VS Code setup guide
├── src/
│   ├── main.rs         # Application entry point and CLI handling
│   ├── audio.rs        # Windows Audio API wrapper with RAII COM guards
│   ├── config.rs       # Configuration management and CLI parser
│   ├── scheduler.rs    # Task Scheduler integration for autostart
│   └── tray.rs         # System tray implementation
├── Cargo.toml          # Dependencies and metadata
├── rustfmt.toml        # Code formatting rules
├── clippy.toml         # Linter configuration
└── README.md           # This file
```

### CI/CD Pipeline

The project uses GitHub Actions for automated testing and releases:

#### Continuous Integration (CI)
Runs on every push/PR to main branches when Rust or TOML files change:
- **Format Check**: Ensures code follows rustfmt standards
- **Clippy Lint**: Static analysis for common mistakes
- **Tests**: Runs all unit tests
- **Build**: Compiles release binary
- **Security Audit**: Checks for vulnerable dependencies

#### Lint Workflow
Separate workflow for code quality:
- Rust formatting validation
- Clippy warnings as errors
- TOML syntax validation

#### Release Workflow
Manual trigger for creating releases:
- Updates version in Cargo.toml
- Builds optimized binary
- Generates checksums (SHA256)
- Creates GitHub release with:
  - Standalone EXE
  - ZIP archive with README
  - Release notes
  - Binary size information

**To create a release:**
1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter version (e.g., `v1.0.0`)
4. Optionally mark as pre-release
5. Run!

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
- Decrease check interval if you need faster response (default is 5 minutes)

### Autostart not working
- Manually install: `mic-volume-control.exe install`
- Check Task Scheduler for "MicrophoneVolumeControl" task
- Ensure application path hasn't changed

