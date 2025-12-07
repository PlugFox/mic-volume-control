mod audio;
mod config;
mod scheduler;
mod tray;

use anyhow::{Context, Result};
use clap::Parser;
use config::{Cli, Commands, Config};
use log::{error, info, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    let audio_controller =
        Arc::new(audio::AudioController::new().context("Failed to initialize audio controller")?);

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
        })
        .context("Failed to set Ctrl+C handler")?;

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
            let scheduler =
                scheduler::TaskScheduler::new().context("Failed to create task scheduler")?;
            scheduler
                .register_autostart()
                .context("Failed to register autostart task")?;
            println!("Autostart task installed successfully!");
            Ok(())
        }

        Commands::Uninstall => {
            println!("Uninstalling autostart task...");
            let scheduler =
                scheduler::TaskScheduler::new().context("Failed to create task scheduler")?;
            scheduler
                .unregister_autostart()
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
            audio::AudioController::set_volume(volume_f32).context("Failed to set volume")?;
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

    // Rotate log if it's too large (>5 MB)
    rotate_log_if_needed(&log_path)?;

    let target = Box::new(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?,
    );

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

fn rotate_log_if_needed(log_path: &std::path::Path) -> Result<()> {
    const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024; // 5 MB
    const MAX_ROTATED_LOGS: usize = 3; // Keep 3 old logs

    // Check if log file exists and is too large
    if let Ok(metadata) = std::fs::metadata(log_path) {
        if metadata.len() > MAX_LOG_SIZE {
            // Rotate existing logs: app.log.2 -> app.log.3, app.log.1 -> app.log.2, etc.
            for i in (1..MAX_ROTATED_LOGS).rev() {
                let old_name = format!("{}.{}", log_path.display(), i);
                let new_name = format!("{}.{}", log_path.display(), i + 1);

                if std::path::Path::new(&old_name).exists() {
                    let _ = std::fs::rename(&old_name, &new_name);
                }
            }

            // Move current log to .1
            let rotated_name = format!("{}.1", log_path.display());
            std::fs::rename(log_path, &rotated_name).context("Failed to rotate log file")?;

            // Delete oldest log if it exists
            let oldest_log = format!("{}.{}", log_path.display(), MAX_ROTATED_LOGS + 1);
            let _ = std::fs::remove_file(&oldest_log);
        }
    }

    Ok(())
}

fn setup_autostart() -> Result<()> {
    let scheduler = scheduler::TaskScheduler::new().context("Failed to create task scheduler")?;

    if !scheduler.is_registered() {
        info!("Registering autostart task...");
        scheduler
            .register_autostart()
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
    use std::io::Write;

    #[test]
    fn test_audio_controller_creation() {
        let controller = audio::AudioController::new();
        assert!(controller.is_ok());
    }

    #[test]
    fn test_log_rotation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create a large log file (6 MB)
        {
            let mut file = std::fs::File::create(&log_path).unwrap();
            let data = vec![b'a'; 1024 * 1024]; // 1 MB chunk
            for _ in 0..6 {
                file.write_all(&data).unwrap();
            }
        }

        // Check file size before rotation
        let size_before = std::fs::metadata(&log_path).unwrap().len();
        assert!(size_before > 5 * 1024 * 1024);

        // Rotate log
        rotate_log_if_needed(&log_path).unwrap();

        // Original log should be rotated
        assert!(!log_path.exists() || std::fs::metadata(&log_path).unwrap().len() == 0);

        // Rotated log should exist
        let rotated = format!("{}.1", log_path.display());
        assert!(std::path::Path::new(&rotated).exists());
        assert_eq!(std::fs::metadata(&rotated).unwrap().len(), size_before);
    }

    #[test]
    fn test_log_rotation_multiple() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create and rotate multiple times
        for i in 1..=5 {
            let mut file = std::fs::File::create(&log_path).unwrap();
            let data = vec![b'0' + i as u8; 6 * 1024 * 1024];
            file.write_all(&data).unwrap();
            drop(file);

            rotate_log_if_needed(&log_path).unwrap();
        }

        // Check that we keep only MAX_ROTATED_LOGS (3) old logs
        assert!(std::path::Path::new(&format!("{}.1", log_path.display())).exists());
        assert!(std::path::Path::new(&format!("{}.2", log_path.display())).exists());
        assert!(std::path::Path::new(&format!("{}.3", log_path.display())).exists());
        assert!(!std::path::Path::new(&format!("{}.4", log_path.display())).exists());
    }

    #[test]
    fn test_log_no_rotation_when_small() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create a small log file (1 KB)
        {
            let mut file = std::fs::File::create(&log_path).unwrap();
            file.write_all(b"Small log content").unwrap();
        }

        let size_before = std::fs::metadata(&log_path).unwrap().len();

        // Try to rotate
        rotate_log_if_needed(&log_path).unwrap();

        // File should not be rotated
        assert!(log_path.exists());
        assert_eq!(std::fs::metadata(&log_path).unwrap().len(), size_before);

        // No rotated files should exist
        let rotated = format!("{}.1", log_path.display());
        assert!(!std::path::Path::new(&rotated).exists());
    }
}
