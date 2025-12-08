use anyhow::{Context, Result};
use windows::{Win32::Media::Audio::Endpoints::*, Win32::Media::Audio::*, Win32::System::Com::*};

/// Simple audio controller for microphone volume management
pub struct AudioController;

impl AudioController {
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

        let enumerator = Self::get_device_enumerator()?;
        let device = Self::get_default_microphone(&enumerator)?;
        let volume = Self::get_volume_control(&device)?;

        unsafe {
            volume
                .SetMasterVolumeLevelScalar(target_volume, std::ptr::null())
                .context("Failed to set volume level")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        if result.is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Ok(volume) = AudioController::get_current_volume() {
                assert!((volume - 0.5).abs() < 0.02);
            }
        }
    }
}
