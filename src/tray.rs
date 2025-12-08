use anyhow::Result;
use crossbeam_channel::Receiver as CrossbeamReceiver;
use log::info;
use std::sync::mpsc::{Receiver, Sender};
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Quit,
    ShowConfig,
    ToggleMonitoring,
}

struct TrayEventHandler {
    menu_channel: CrossbeamReceiver<MenuEvent>,
    tx: Sender<TrayMessage>,
    quit_item: MenuItem,
    config_item: MenuItem,
    toggle_item: MenuItem,
}

impl ApplicationHandler for TrayEventHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Set control flow to wait for events
        event_loop.set_control_flow(ControlFlow::Wait);

        // Start polling for tray menu events
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
        // This is called for window events, but we don't have windows in a tray app
        self.check_menu_events(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // This is called before the event loop waits for new events
        // Perfect place to check for tray menu events
        self.check_menu_events(event_loop);
    }
}

impl TrayEventHandler {
    fn check_menu_events(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(event) = self.menu_channel.try_recv() {
            if event.id == self.quit_item.id() {
                info!("Quit requested from tray");
                let _ = self.tx.send(TrayMessage::Quit);
                event_loop.exit();
            } else if event.id == self.config_item.id() {
                let _ = self.tx.send(TrayMessage::ShowConfig);
            } else if event.id == self.toggle_item.id() {
                let _ = self.tx.send(TrayMessage::ToggleMonitoring);
            }
        }
    }
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
        let menu_channel = MenuEvent::receiver().clone();

        event_loop.run_app(&mut TrayEventHandler {
            menu_channel,
            tx,
            quit_item,
            config_item,
            toggle_item,
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
                    rgba[idx] = 50; // R
                    rgba[idx + 1] = 120; // G
                    rgba[idx + 2] = 200; // B
                    rgba[idx + 3] = 255; // A
                } else if (14..=18).contains(&x) && (16..=24).contains(&y) {
                    // Stem
                    rgba[idx] = 50;
                    rgba[idx + 1] = 120;
                    rgba[idx + 2] = 200;
                    rgba[idx + 3] = 255;
                } else if (10..=22).contains(&x) && (24..=26).contains(&y) {
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
