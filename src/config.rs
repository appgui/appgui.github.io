use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Platform entry-points call this before `run_app` to provide a writable directory.
/// On Android this must be `AndroidApp::internal_data_path()`; on desktop it is
/// optional — the `dirs` crate provides a sensible default when not set.
pub fn set_config_dir(path: PathBuf) {
  CONFIG_DIR.set(path).ok();
}

fn settings_path() -> Option<PathBuf> {
  if let Some(dir) = CONFIG_DIR.get() {
    return Some(dir.join("settings.toml"));
  }
  #[cfg(not(target_os = "android"))]
  {
    dirs::config_dir().map(|p| p.join("xplatform").join("settings.toml"))
  }
  #[cfg(target_os = "android")]
  {
    None
  }
}

fn default_source() -> String {
  "local".to_string()
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Settings {
  #[serde(default = "default_source")]
  pub source: String, // "local" or "remote"
  #[serde(default)]
  pub remote_url: String,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      source: "local".to_string(),
      remote_url: String::new(),
    }
  }
}

pub fn load_settings() -> Settings {
  settings_path()
    .and_then(|p| std::fs::read_to_string(p).ok())
    .and_then(|s| toml::from_str(&s).ok())
    .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) {
  let Some(path) = settings_path() else {
    return;
  };
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if let Ok(content) = toml::to_string(settings) {
    let _ = std::fs::write(path, content);
  }
}
