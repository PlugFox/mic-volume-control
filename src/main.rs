mod audio;
mod config;
mod scheduler;

use anyhow::{Context, Result};
use clap::Parser;
use config::{Cli, Commands, Config};
use windows::Win32::System::Com::*;

/// RAII guard for COM initialization/uninitialization
struct ComGuard;

impl ComGuard {
    fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .context("Failed to initialize COM")?;
        }
        Ok(ComGuard)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let _com = ComGuard::new()?;
    let cli = Cli::parse();
    let quiet = cli.quiet;

    // If no command provided, clap will show help due to arg_required_else_help
    let Some(command) = cli.command else {
        return Ok(());
    };

    match command {
        Commands::Volume { level } => {
            if let Some(volume) = level {
                // Set volume
                let volume_f32 = volume as f32 / 100.0;
                audio::AudioController::set_volume(volume_f32).context("Failed to set volume")?;
                if !quiet {
                    println!("Microphone volume set to: {}%", volume);
                }
            } else {
                // Get volume
                let volume = audio::AudioController::get_current_volume()
                    .context("Failed to get current volume")?;
                if !quiet {
                    println!("Current microphone volume: {:.0}%", volume * 100.0);
                }
            }
        }

        Commands::Install { volume, interval } => {
            println!("Installing Windows Task Scheduler task...");
            println!("  Target volume: {}%", volume);
            println!("  Run interval: {} minutes", interval);

            let volume_f32 = volume as f32 / 100.0;

            // Save config
            let config = Config {
                target_volume: volume_f32,
                run_interval_minutes: interval,
            };
            config.save().context("Failed to save configuration")?;

            // Register task
            let scheduler =
                scheduler::TaskScheduler::new().context("Failed to create task scheduler")?;
            scheduler
                .register_task(volume_f32, interval)
                .context("Failed to register task")?;

            println!("\nTask installed successfully!");
            println!("The task will:");
            println!("  - Run at login (after 1 minute delay)");
            println!("  - Repeat every {} minutes", interval);
            println!("  - Set microphone volume to {}%", volume);
            println!("\nYou can manage the task in Windows Task Scheduler.");
        }

        Commands::Uninstall => {
            println!("Uninstalling Windows Task Scheduler task...");

            let scheduler =
                scheduler::TaskScheduler::new().context("Failed to create task scheduler")?;

            if !scheduler.is_registered() {
                println!("Task is not installed.");
                return Ok(());
            }

            scheduler
                .unregister_task()
                .context("Failed to unregister task")?;

            println!("Task uninstalled successfully!");
        }

        Commands::Config => {
            let config = Config::load_from_file()?;
            config.display();

            let scheduler =
                scheduler::TaskScheduler::new().context("Failed to create task scheduler")?;

            println!(
                "\nTask Status: {}",
                if scheduler.is_registered() {
                    "Installed"
                } else {
                    "Not installed"
                }
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_com_guard() {
        let guard = ComGuard::new();
        assert!(guard.is_ok());
    }
}
