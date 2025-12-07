use anyhow::{Context, Result};
use log::info;
use std::sync::mpsc::{Receiver, Sender};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Quit,
    ShowConfig,
    ToggleMonitoring,
}

pub struct TrayApp {
    tx: Sender<TrayMessage>,
}

impl TrayApp {
    pub fn new() -> Result<(Self, Receiver<TrayMessage>)> {
        let (tx, rx) = std::sync::mpsc::channel();
        Ok((Self { tx }, rx))
    }

    pub fn run(&self, current_volume: f32) -> Result<()> {
        let event_loop = EventLoop::new()?;

        // Create tray menu
        let tray_menu = Menu::new();

        let status_item = MenuItem::new(
            format!("Target Volume: {:.0}%", current_volume * 100.0),
            false,
            None,
        );

        let toggle_item = MenuItem::new("Pause Monitoring", true, None);
        let config_item = MenuItem::new("Open Config", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        tray_menu.append(&status_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&toggle_item)?;
        tray_menu.append(&config_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&quit_item)?;

        // Create tray icon
        let icon = Self::create_icon()?;
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Microphone Volume Control")
            .with_icon(icon)
            .build()?;

        info!("System tray icon created");

        let tx = self.tx.clone();
        let menu_channel = MenuEvent::receiver();

        event_loop.run(move |_event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == quit_item.id() {
                    info!("Quit requested from tray");
                    let _ = tx.send(TrayMessage::Quit);
                    elwt.exit();
                } else if event.id == config_item.id() {
                    let _ = tx.send(TrayMessage::ShowConfig);
                } else if event.id == toggle_item.id() {
                    let _ = tx.send(TrayMessage::ToggleMonitoring);
                }
            }
        })?;

        Ok(())
    }

    fn create_icon() -> Result<tray_icon::Icon> {
        // Create a simple 32x32 RGBA icon
        let width = 32;
        let height = 32;
        let mut rgba = vec![0u8; (width * height * 4) as usize];

        // Draw a simple microphone icon (circle with stem)
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;

                // Center circle (microphone head)
                let dx = x as i32 - 16;
                let dy = y as i32 - 12;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 64 {
                    // Blue circle
                    rgba[idx] = 50;      // R
                    rgba[idx + 1] = 120; // G
                    rgba[idx + 2] = 200; // B
                    rgba[idx + 3] = 255; // A
                } else if x >= 14 && x <= 18 && y >= 16 && y <= 24 {
                    // Stem
                    rgba[idx] = 50;
                    rgba[idx + 1] = 120;
                    rgba[idx + 2] = 200;
                    rgba[idx + 3] = 255;
                } else if x >= 10 && x <= 22 && y >= 24 && y <= 26 {
                    // Base
                    rgba[idx] = 50;
                    rgba[idx + 1] = 120;
                    rgba[idx + 2] = 200;
                    rgba[idx + 3] = 255;
                }
            }
        }

        tray_icon::Icon::from_rgba(rgba, width, height)
            .map_err(|e| anyhow::anyhow!("Failed to create tray icon from RGBA data: {}", e))
    }
}
