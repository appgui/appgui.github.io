/// Cross-platform theme detection and management
use slint::Weak;
use std::sync::{Arc, Mutex};

use crate::AppWindow;

/// Theme mode enum
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ThemeMode {
    Auto = 0,
    Light = 1,
    Dark = 2,
}

impl From<i32> for ThemeMode {
    fn from(value: i32) -> Self {
        match value {
            0 => ThemeMode::Auto,
            1 => ThemeMode::Light,
            2 => ThemeMode::Dark,
            _ => ThemeMode::Auto,
        }
    }
}

/// Theme state manager
pub struct ThemeState {
    mode: Arc<Mutex<ThemeMode>>,
}

impl ThemeState {
    pub fn new() -> Self {
        Self {
            mode: Arc::new(Mutex::new(ThemeMode::Auto)),
        }
    }

    pub fn cycle(&self) -> ThemeMode {
        let mut mode = self.mode.lock().unwrap();
        *mode = match *mode {
            ThemeMode::Auto => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Auto,
        };
        *mode
    }
}

/// Setup theme callbacks for the UI
pub fn setup_theme_callbacks(ui_weak: Weak<AppWindow>) {
    let state = Arc::new(ThemeState::new());

    // Detect system theme and initialize
    let system_is_dark = detect_system_dark_mode();

    if let Some(ui) = ui_weak.upgrade() {
        ui.set_system_is_dark(system_is_dark);
        ui.set_theme_mode(ThemeMode::Auto as i32);

        println!("🎨 Theme initialized: system is {}",
            if system_is_dark { "dark" } else { "light" });
    }

    // Set up theme cycling callback
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_cycle_theme(move || {
            let new_mode = state.cycle();

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_theme_mode(new_mode as i32);

                println!("🎨 Theme changed to: {:?}", new_mode);
            }
        });
    }
}

/// Detects if the system is currently using dark mode
pub fn detect_system_dark_mode() -> bool {
    #[cfg(target_os = "windows")]
    {
        detect_windows_dark_mode()
    }

    #[cfg(target_os = "macos")]
    {
        detect_macos_dark_mode()
    }

    #[cfg(target_os = "linux")]
    {
        detect_linux_dark_mode()
    }

    #[cfg(target_os = "android")]
    {
        // Android detection would require JNI calls
        // For now, default to dark mode
        true
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    {
        true // Default to dark mode for other platforms
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_dark_mode() -> bool {
    use std::process::Command;

    // Query Windows registry for AppsUseLightTheme
    let output = Command::new("reg")
        .args(&[
            "query",
            "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            "/v",
            "AppsUseLightTheme"
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // If AppsUseLightTheme is 0x0, dark mode is enabled
        if stdout.contains("0x0") {
            return true;
        }
    }

    false // Default to light mode if detection fails
}

#[cfg(target_os = "macos")]
fn detect_macos_dark_mode() -> bool {
    use std::process::Command;

    // Query macOS for dark mode setting
    let output = Command::new("defaults")
        .args(&["read", "-g", "AppleInterfaceStyle"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.trim() == "Dark";
    }

    false // Default to light mode if detection fails
}

#[cfg(target_os = "linux")]
fn detect_linux_dark_mode() -> bool {
    use std::process::Command;

    // Try GNOME/GTK settings first
    let output = Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dark") {
            return true;
        }
        if stdout.contains("light") {
            return false;
        }
    }

    // Try alternative GTK theme check
    let output = Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if stdout.contains("dark") {
            return true;
        }
    }

    // Check KDE Plasma settings
    let output = Command::new("kreadconfig5")
        .args(&["--group", "General", "--key", "ColorScheme"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if stdout.contains("dark") {
            return true;
        }
    }

    true // Default to dark mode on Linux if detection fails
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_conversion() {
        assert_eq!(ThemeMode::from(0), ThemeMode::Auto);
        assert_eq!(ThemeMode::from(1), ThemeMode::Light);
        assert_eq!(ThemeMode::from(2), ThemeMode::Dark);
        assert_eq!(ThemeMode::from(99), ThemeMode::Auto);
    }
}
