use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::{Win32::Media::Audio::Endpoints::*, Win32::Media::Audio::*, Win32::System::Com::*};

/// Manages Windows Audio API interactions for microphone volume control
pub struct AudioController {
    running: Arc<AtomicBool>,
}

/// RAII wrapper for COM initialization/cleanup
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

impl AudioController {
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    fn get_device_enumerator() -> Result<IMMDeviceEnumerator> {
        unsafe {
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .context("Failed to create device enumerator")
        }
    }

    fn get_default_microphone(enumerator: &IMMDeviceEnumerator) -> Result<IMMDevice> {
        unsafe {
            enumerator
                .GetDefaultAudioEndpoint(eCapture, eConsole)
                .context("Failed to get default microphone")
        }
    }

    fn get_volume_control(device: &IMMDevice) -> Result<IAudioEndpointVolume> {
        unsafe {
            device
                .Activate(CLSCTX_ALL, None)
                .context("Failed to activate audio endpoint volume")
        }
    }

    /// Get current microphone volume (0.0 - 1.0)
    pub fn get_current_volume() -> Result<f32> {
        let _guard = ComGuard::new()?;

        let enumerator = Self::get_device_enumerator()?;
        let device = Self::get_default_microphone(&enumerator)?;
        let volume = Self::get_volume_control(&device)?;

        unsafe {
            volume
                .GetMasterVolumeLevelScalar()
                .context("Failed to get volume level")
        }
    }

    /// Set microphone volume (0.0 - 1.0)
    pub fn set_volume(target_volume: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&target_volume) {
            anyhow::bail!("Volume must be between 0.0 and 1.0");
        }

        let _guard = ComGuard::new()?;

        let enumerator = Self::get_device_enumerator()?;
        let device = Self::get_default_microphone(&enumerator)?;
        let volume = Self::get_volume_control(&device)?;

        unsafe {
            volume
                .SetMasterVolumeLevelScalar(target_volume, std::ptr::null())
                .context("Failed to set volume level")?;
        }

        info!("Volume set to: {:.0}%", target_volume * 100.0);
        Ok(())
    }

    /// Continuously monitor and maintain target volume
    pub fn monitor_and_control(&self, target_volume: f32, check_interval_ms: u64) -> Result<()> {
        if !(0.0..=1.0).contains(&target_volume) {
            anyhow::bail!("Volume must be between 0.0 and 1.0");
        }

        info!("Starting microphone volume monitoring...");
        info!("Target volume: {:.0}%", target_volume * 100.0);
        info!("Check interval: {}ms", check_interval_ms);

        let _guard = ComGuard::new()?;
        let enumerator = Self::get_device_enumerator()?;
        let running = self.running.clone();

        while running.load(Ordering::Relaxed) {
            if let Err(e) = self.check_and_adjust_volume(&enumerator, target_volume) {
                error!("Error during volume check: {}", e);
            }

            std::thread::sleep(std::time::Duration::from_millis(check_interval_ms));
        }

        info!("Monitoring stopped");
        Ok(())
    }

    fn check_and_adjust_volume(
        &self,
        enumerator: &IMMDeviceEnumerator,
        target_volume: f32,
    ) -> Result<()> {
        let device = Self::get_default_microphone(enumerator)?;
        let volume = Self::get_volume_control(&device)?;

        unsafe {
            let current = volume
                .GetMasterVolumeLevelScalar()
                .context("Failed to get current volume")?;

            const TOLERANCE: f32 = 0.01; // 1%
            let diff = (current - target_volume).abs();

            if diff > TOLERANCE {
                warn!(
                    "Volume drift detected: {:.0}% -> {:.0}%",
                    current * 100.0,
                    target_volume * 100.0
                );

                volume
                    .SetMasterVolumeLevelScalar(target_volume, std::ptr::null())
                    .context("Failed to restore volume")?;

                info!("Volume restored to {:.0}%", target_volume * 100.0);
            } else {
                debug!("Volume stable at {:.0}%", current * 100.0);
            }
        }

        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&self) {
        info!("Stopping audio controller...");
        self.running.store(false, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Default for AudioController {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = AudioController::new();
        assert!(controller.is_ok());
    }

    #[test]
    fn test_controller_is_running() {
        let controller = AudioController::new().unwrap();
        assert!(controller.is_running());
    }

    #[test]
    fn test_controller_stop() {
        let controller = AudioController::new().unwrap();
        controller.stop();
        assert!(!controller.is_running());
    }

    #[test]
    fn test_invalid_volume_range() {
        assert!(AudioController::set_volume(1.5).is_err());
        assert!(AudioController::set_volume(-0.1).is_err());
    }

    // Note: The following tests require actual audio hardware and may fail in CI
    #[test]
    #[ignore]
    fn test_get_volume() {
        let result = AudioController::get_current_volume();
        if let Ok(volume) = result {
            assert!((0.0..=1.0).contains(&volume));
        }
    }

    #[test]
    #[ignore]
    fn test_set_volume() {
        let result = AudioController::set_volume(0.5);
        // May fail if no microphone is available
        if result.is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Ok(volume) = AudioController::get_current_volume() {
                assert!((volume - 0.5).abs() < 0.02);
            }
        }
    }
}
