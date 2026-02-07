use slint::Weak;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

use crate::AppWindow;

pub mod theme;

#[derive(Clone, Copy, PartialEq)]
pub enum DisplayMode {
    Ram,
    Uptime,
}

pub struct AppState {
    mode: Arc<Mutex<DisplayMode>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Arc::new(Mutex::new(DisplayMode::Ram)),
        }
    }

    pub fn toggle_mode(&self) {
        let mut mode = self.mode.lock().unwrap();
        *mode = if *mode == DisplayMode::Ram {
            DisplayMode::Uptime
        } else {
            DisplayMode::Ram
        };
    }

    pub fn get_mode(&self) -> DisplayMode {
        *self.mode.lock().unwrap()
    }
}

pub fn start_background_updates(ui_weak: Weak<AppWindow>) {
    let state = Arc::new(AppState::new());

    // Clone for the background thread
    let state_bg = state.clone();

    // Set up the toggle callback with the state
    if let Some(ui) = ui_weak.upgrade() {
        let state_toggle = state.clone();
        ui.on_toggle_mode(move || {
            state_toggle.toggle_mode();
        });
    }

    // Start background update loop
    thread::spawn(move || {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(MemoryRefreshKind::everything())
        );

        let mut last_value = String::new();
        let mut last_label = String::new();
        let mut last_button = String::new();

        loop {
            sys.refresh_memory();

            let current_mode = state_bg.get_mode();

            let (value, label, button_text) = match current_mode {
                DisplayMode::Ram => {
                    let mb = sys.available_memory() / 1_048_576;
                    (
                        format!("{} MiB", mb),
                        "Available RAM".to_string(),
                        "Show Uptime".to_string(),
                    )
                }
                DisplayMode::Uptime => (
                    format_uptime(System::uptime()),
                    "Uptime since last boot".to_string(),
                    "Show RAM".to_string(),
                ),
            };

            // Only update UI if values actually changed
            if value != last_value || label != last_label || button_text != last_button {
                let value_clone = value.clone();
                let label_clone = label.clone();
                let button_clone = button_text.clone();

                let ui_weak_clone = ui_weak.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_clone.upgrade() {
                        ui.set_display_value(value_clone.into());
                        ui.set_display_label(label_clone.into());
                        ui.set_button_text(button_clone.into());
                    }
                }).ok();

                last_value = value;
                last_label = label;
                last_button = button_text;
            }

            thread::sleep(Duration::from_millis(100));  // Check more frequently for mode changes
        }
    });
}

fn format_uptime(seconds: u64) -> String {
    let days    = seconds / 86_400;
    let hours   = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600)   / 60;
    let secs    = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
