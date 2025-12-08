# Microphone Volume Control

[![CI](https://github.com/plugfox/mic-volume-control/workflows/CI/badge.svg)](https://github.com/plugfox/mic-volume-control/actions/workflows/ci.yml)
[![Lint](https://github.com/plugfox/mic-volume-control/workflows/Lint/badge.svg)](https://github.com/plugfox/mic-volume-control/actions/workflows/lint.yml)
[![Release](https://github.com/plugfox/mic-volume-control/workflows/Release/badge.svg)](https://github.com/plugfox/mic-volume-control/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows)

A simple Windows utility to automatically maintain your microphone volume at a configured level using Windows Task Scheduler. Unlike traditional background applications, this tool runs only when scheduled, making it extremely lightweight.

## Features

- **Automatic Volume Control**: Set your desired microphone volume level and let the app maintain it
- **Windows Task Scheduler Integration**: Runs periodically in the background without manual intervention
- **Lightweight**: Simple CLI tool with minimal resource usage
- **No Background Process**: Runs when scheduled and exits immediately
- **Easy Installation**: Simple commands to set up and configure

## Installation

### Option 1: Download Release Binary

1. Download the latest `mic-volume-control.exe` from the [Releases](https://github.com/plugfox/mic-volume-control/releases) page
2. Place it in a permanent location (e.g., `C:\Program Files\MicVolumeControl\`)
3. Run the install command (see Usage below)

### Option 2: Build from Source

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

## Usage

> [!CAUTION]
> Install and Uninstall commands require Administrator privileges to create/remove Task Scheduler tasks.
> Run your terminal as Administrator or use `sudo` equivalent.

### Install Automatic Task

Install a Windows Task Scheduler task that will automatically set your microphone volume:

```bash
# Install with default settings (95% volume, run every 5 minutes)
mic-volume-control.exe install

# Install with custom settings
mic-volume-control.exe install --volume 80 --interval 10
```

Options:
- `--volume <0-100>`: Target volume percentage (default: 95)
- `--interval <minutes>`: How often to run (default: 5 minutes)

The task will:
- Run at Windows login (after 1 minute delay)
- Repeat at the specified interval
- Set your microphone volume to the target level

### Manual Volume Control

```bash
# Get current microphone volume
mic-volume-control.exe volume

# Set microphone volume once (without installing task)
mic-volume-control.exe volume 95

# Set volume in quiet mode (no output)
mic-volume-control.exe --quiet volume 95
```

### View Configuration

```bash
# Show current configuration and task status
mic-volume-control.exe config
```

### Uninstall Task

```bash
# Remove the scheduled task
mic-volume-control.exe uninstall
```

## How It Works

1. **Install**: When you run `install`, the app:
   - Saves your configuration to `%APPDATA%\mic-volume-control\config.toml`
   - Creates a Windows Task Scheduler task that runs `mic-volume-control.exe volume <target>`

2. **Automatic Execution**: Windows Task Scheduler runs the app:
   - At login (after 1 minute delay)
   - Every N minutes (configurable)
   - The app sets the microphone volume and exits immediately

3. **No Background Process**: Unlike traditional background apps, this utility doesn't run continuously. It only executes when scheduled, making it extremely lightweight.

## Configuration

Configuration is stored in: `%APPDATA%\mic-volume-control\config.toml`

Example:
```toml
target_volume = 0.95  # 95%
run_interval_minutes = 5
```

You can edit this file manually, but it's recommended to use the `install` command to update settings.

## Managing the Task

You can view and manage the scheduled task in Windows Task Scheduler:

1. Press `Win + R` and type `taskschd.msc`
2. Look for "MicrophoneVolumeControl" in the task list
3. You can run it manually, disable it, or view execution history

## Requirements

- Windows 10 or later
- Administrator privileges (for Task Scheduler operations)

## Why This Approach?

Many microphone volume control apps run continuously in the system tray, consuming memory and resources. This app takes a different approach:

- **No Background Process**: Only runs when scheduled
- **Native Scheduling**: Uses Windows Task Scheduler instead of custom scheduling
- **Minimal Resource Usage**: No memory footprint between runs
- **System Integration**: Works with Windows power management and task scheduling

## Development

### Cargo Aliases

Project uses cargo aliases for development (defined in [.cargo/config.toml](.cargo/config.toml)):

```bash
# Check formatting
cargo fmt-check

# Auto-format code
cargo fmt-fix

# Lint with clippy
cargo lint

# Auto-fix clippy issues
cargo lint-fix

# Check compilation
cargo check-all

# Run tests
cargo test-all

# Build release version
cargo build-release

# Clean
cargo clean-all
```

For complete local CI check:
```bash
# Check Rust code
cargo fmt-check && cargo lint && cargo check-all && cargo test-all

# Check TOML files (requires taplo-cli)
taplo format --check && taplo lint

# Or all together
cargo fmt-check && cargo lint && taplo format --check && taplo lint && cargo test-all
```

Install taplo (for TOML validation):
```bash
cargo install taplo-cli --locked
```

### Project Structure

```
mic-volume-control/
├── .cargo/
│   └── config.toml     # Cargo aliases
├── .github/
│   └── workflows/      # CI/CD pipelines
├── .vscode/            # VS Code configuration
├── src/
│   ├── main.rs         # Application entry point and CLI handling
│   ├── audio.rs        # Windows Audio API wrapper
│   ├── config.rs       # Configuration management and CLI parser
│   └── scheduler.rs    # Task Scheduler integration
├── Cargo.toml          # Dependencies and metadata
├── rustfmt.toml        # Code formatting rules
├── clippy.toml         # Linter configuration
└── README.md           # This file
```

### CI/CD Pipeline

The project uses GitHub Actions for automated testing and releases:

#### Continuous Integration (CI)
Runs on every push/PR:
- **Format Check**: Ensures code follows rustfmt standards
- **Clippy Lint**: Static analysis for common mistakes
- **Tests**: Runs all unit tests
- **Build**: Compiles release binary
- **Security Audit**: Checks for vulnerable dependencies

#### Release Workflow
Manual trigger for creating releases:
- Updates version in Cargo.toml
- Builds optimized binary
- Generates checksums (SHA256)
- Creates GitHub release with artifacts

## Troubleshooting

### Application won't start
- Check if microphone is connected and recognized by Windows
- Run as Administrator for Task Scheduler operations

### Volume keeps resetting
- Another application may be controlling microphone volume
- Check if exclusive mode is enabled in microphone properties
- Adjust the interval if needed

### Task not running
- Check Task Scheduler: Press `Win + R`, type `taskschd.msc`
- Look for "MicrophoneVolumeControl" task
- Verify the task is enabled and the executable path is correct

## License

MIT License - see [LICENSE](LICENSE) file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Plague Fox <plugfox@gmail.com>
