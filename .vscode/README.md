# VS Code Configuration

This directory contains VS Code workspace configuration for the mic-volume-control project.

## Files

- **tasks.json** - Build, test, and CI tasks
- **settings.json** - Editor and Rust-specific settings
- **extensions.json** - Recommended VS Code extensions
- **launch.json** - Debug configurations

## Tasks

Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) and type "Tasks: Run Task" to see all available tasks.

### Check Tasks

- **Rust: Format Check** - Check code formatting with rustfmt
- **Rust: Lint (Clippy)** - Run clippy linter
- **Rust: Check All** - Check compilation without building
- **Rust: Test All** - Run all tests
- **TOML: Format Check** - Check TOML file formatting
- **TOML: Lint** - Lint TOML files

### CI Task

- **CI: Full Check** ⭐ - Run all checks in sequence (default test task)
  - Press `Ctrl+Shift+B` to run this task quickly

### Fix Tasks

- **Rust: Format Fix** - Auto-format Rust code
- **Rust: Lint Fix** - Auto-fix clippy warnings
- **TOML: Format Fix** - Auto-format TOML files
- **Fix: All** - Run all auto-fixes

### Build Tasks

- **Build: Release** ⭐ - Build optimized release binary (default build task)
- **Build: Debug** - Build debug binary

### Run Tasks

- **Run: Debug** - Run debug build
- **Run: Release** - Run release build

### Other Tasks

- **Clean: All** - Clean build artifacts

## Keyboard Shortcuts

- `Ctrl+Shift+B` - Run default build task (CI: Full Check)
- `F5` - Start debugging
- `Ctrl+Shift+P` → "Tasks: Run Task" - Show all tasks

## Recommended Extensions

Install the recommended extensions when prompted, or manually install:

- **rust-analyzer** - Rust language server (essential)
- **Even Better TOML** - TOML file support
- **CodeLLDB** - Native debugger for Rust
- **crates** - Helps manage Cargo.toml dependencies
- **dependi** - Visualize and manage dependencies

## Debug Configurations

Press `F5` or go to Run and Debug panel to use:

1. **Debug: mic-volume-control** - Standard debug run
2. **Debug: mic-volume-control (no tray)** - Debug without tray icon
3. **Debug: mic-volume-control (custom volume)** - Debug with custom settings
4. **Debug: Tests** - Debug tests

## Settings Highlights

- **Format on save** enabled for Rust and TOML
- **Clippy** used for real-time linting
- **100-character ruler** for code consistency
- **RUST_BACKTRACE=1** in terminal for better error messages
