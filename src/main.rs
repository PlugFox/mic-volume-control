mod audio;
mod config;
mod scheduler;
mod tray;

use anyhow::{Context, Result};
use clap::Parser;
use config::{Cli, Commands, Config};
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    if let Err(e) = run() {
        error!("Fatal error: {:#}", e);
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    setup_logging()?;

    // Handle subcommands
    if let Some(command) = &cli.command {
        return handle_command(command);
    }

    info!("=== Microphone Volume Control Starting ===");

    // Load configuration
    let config = Config::load(&cli).context("Failed to load configuration")?;
    info!("Target volume: {:.0}%", config.target_volume * 100.0);
    info!("Check interval: {}ms", config.check_interval_ms);

    // Setup autostart if enabled
    if config.enable_autostart {
        match setup_autostart() {
            Ok(_) => info!("Autostart configuration verified"),
            Err(e) => warn!("Failed to setup autostart: {}", e),
        }
    }

    // Create audio controller
    let audio_controller = Arc::new(
        audio::AudioController::new().context("Failed to initialize audio controller")?
    );

    // Set initial volume
    info!("Setting initial volume to target...");
    audio::AudioController::set_volume(config.target_volume)
        .context("Failed to set initial volume")?;

    // Start monitoring thread
    let controller_clone = Arc::clone(&audio_controller);
    let target_volume = config.target_volume;
    let check_interval = config.check_interval_ms;

    let monitoring_thread = thread::spawn(move || {
        if let Err(e) = controller_clone.monitor_and_control(target_volume, check_interval) {
            error!("Monitoring error: {}", e);
        }
    });

    // Run system tray or console mode
    if config.enable_tray {
        info!("Starting system tray interface...");
        match run_tray(Arc::clone(&audio_controller), config.target_volume) {
            Ok(_) => info!("Tray interface closed"),
            Err(e) => warn!("Tray interface error: {}", e),
        }
    } else {
        info!("System tray disabled, running in console mode");
        info!("Press Ctrl+C to exit");

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            info!("Received shutdown signal");
            r.store(false, Ordering::Relaxed);
        }).context("Failed to set Ctrl+C handler")?;

        while running.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    // Cleanup
    info!("Shutting down...");
    audio_controller.stop();
    monitoring_thread.join().ok();
    info!("=== Microphone Volume Control Stopped ===");

    Ok(())
}

fn handle_command(command: &Commands) -> Result<()> {
    match command {
        Commands::Install => {
            println!("Installing autostart task...");
            let scheduler = scheduler::TaskScheduler::new()
                .context("Failed to create task scheduler")?;
            scheduler.register_autostart()
                .context("Failed to register autostart task")?;
            println!("Autostart task installed successfully!");
            Ok(())
        }

        Commands::Uninstall => {
            println!("Uninstalling autostart task...");
            let scheduler = scheduler::TaskScheduler::new()
                .context("Failed to create task scheduler")?;
            scheduler.unregister_autostart()
                .context("Failed to unregister autostart task")?;
            println!("Autostart task uninstalled successfully!");
            Ok(())
        }

        Commands::Config => {
            let config = Config::load_from_file().unwrap_or_default();
            config.display();
            Ok(())
        }

        Commands::GetVolume => {
            let volume = audio::AudioController::get_current_volume()
                .context("Failed to get current volume")?;
            println!("Current microphone volume: {:.0}%", volume * 100.0);
            Ok(())
        }

        Commands::SetVolume { volume } => {
            let volume_f32 = *volume as f32 / 100.0;
            audio::AudioController::set_volume(volume_f32)
                .context("Failed to set volume")?;
            println!("Microphone volume set to: {}%", volume);
            Ok(())
        }
    }
}

fn run_tray(audio_controller: Arc<audio::AudioController>, volume: f32) -> Result<()> {
    let (tray_app, receiver) = tray::TrayApp::new()?;

    let tray_thread = thread::spawn(move || {
        if let Err(e) = tray_app.run(volume) {
            error!("Tray error: {}", e);
        }
    });

    // Handle tray messages
    loop {
        match receiver.recv() {
            Ok(tray::TrayMessage::Quit) => {
                info!("Quit command received");
                audio_controller.stop();
                break;
            }
            Ok(tray::TrayMessage::ShowConfig) => {
                info!("Opening configuration...");
                if let Err(e) = open_config() {
                    error!("Failed to open config: {}", e);
                }
            }
            Ok(tray::TrayMessage::ToggleMonitoring) => {
                info!("Toggle monitoring requested");
                // Note: Could implement pause/resume functionality here
            }
            Err(_) => break,
        }
    }

    tray_thread.join().ok();
    Ok(())
}

fn setup_logging() -> Result<()> {
    let log_path = Config::get_log_path()?;

    let target = Box::new(std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?);

    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(target))
        .filter_level(log::LevelFilter::Info)
        .init();

    Ok(())
}

fn setup_autostart() -> Result<()> {
    let scheduler = scheduler::TaskScheduler::new()
        .context("Failed to create task scheduler")?;

    if !scheduler.is_registered() {
        info!("Registering autostart task...");
        scheduler.register_autostart()
            .context("Failed to register autostart task")?;
    } else {
        info!("Autostart task already registered");
    }

    Ok(())
}

fn open_config() -> Result<()> {
    let config_dir = Config::get_config_dir()?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(config_dir)
            .spawn()
            .context("Failed to open config directory")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_controller_creation() {
        let controller = audio::AudioController::new();
        assert!(controller.is_ok());
    }
}
