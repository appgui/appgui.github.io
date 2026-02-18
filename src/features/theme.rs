use crate::AppWindow;
use slint::Weak;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColourScheme {
  Auto = 0,
  Light = 1,
  Dark = 2,
}

impl From<i32> for ColourScheme {
  fn from(value: i32) -> Self {
    match value {
      0 => ColourScheme::Auto,
      1 => ColourScheme::Light,
      2 => ColourScheme::Dark,
      _ => ColourScheme::Auto,
    }
  }
}

pub struct ThemeState {
  scheme: Arc<Mutex<ColourScheme>>,
}

impl ThemeState {
  pub fn new() -> Self {
    Self {
      scheme: Arc::new(Mutex::new(ColourScheme::Auto)),
    }
  }

  pub fn set_scheme(&self, new_scheme: ColourScheme) {
    let mut scheme = self.scheme.lock().unwrap();
    *scheme = new_scheme;
  }
}

pub fn setup_theme_callbacks(ui_weak: Weak<AppWindow>) {
  let state = Arc::new(ThemeState::new());
  let system_is_dark = detect_system_dark_mode();

  if let Some(ui) = ui_weak.upgrade() {
    ui.set_system_is_dark(system_is_dark);
    ui.set_theme_mode(ColourScheme::Auto as i32);
  }

  if let Some(ui) = ui_weak.upgrade() {
    ui.on_cycle_theme({
      let ui_weak = ui_weak.clone();
      let state = state.clone();
      move || {
        if let Some(ui) = ui_weak.upgrade() {
          let selection = ColourScheme::from(ui.get_theme_mode());
          state.set_scheme(selection);
        }
      }
    });
  }
}

pub fn detect_system_dark_mode() -> bool {
  #[cfg(target_os = "linux")]
  {
    // Check local colour preference on Linux
    detect_linux_dark_mode()
  }
  #[cfg(not(target_os = "linux"))]
  {
    true
  }
}

#[cfg(target_os = "linux")]
fn detect_linux_dark_mode() -> bool {
  use std::process::Command;
  let output = Command::new("gsettings")
    .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
    .output();

  if let Ok(out) = output {
    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.contains("dark") {
      return true;
    }
  }
  true
}
