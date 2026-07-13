use slint::Weak;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
#[cfg(not(target_os = "android"))]
use sysinfo::Components;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

use crate::AppWindow;

pub mod login;
pub mod theme;

// 0 = Uptime, 1 = CpuTemp, 2 = Ram  (matches ComboBox order in Slint)
#[derive(Clone, Copy, PartialEq)]
pub enum DisplayMode {
  Uptime,
  CpuTemp,
  Ram,
}

#[derive(Clone, PartialEq)]
pub enum DataSource {
  Local,
  Remote(String),
}

#[derive(serde::Deserialize, Default, Clone)]
struct RemoteMonitor {
  station: Option<RemoteStation>,
}

#[derive(serde::Deserialize, Default, Clone)]
struct RemoteStation {
  uptime: Option<String>,
  cpu_temperature_c: Option<String>,
  ram_free: Option<String>,
}

pub struct AppState {
  mode: Arc<Mutex<DisplayMode>>,
  data_source: Arc<Mutex<DataSource>>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      mode: Arc::new(Mutex::new(DisplayMode::Uptime)),
      data_source: Arc::new(Mutex::new(DataSource::Local)),
    }
  }

  pub fn set_mode(&self, mode: DisplayMode) {
    *self.mode.lock().unwrap() = mode;
  }

  pub fn get_mode(&self) -> DisplayMode {
    *self.mode.lock().unwrap()
  }

  pub fn set_data_source(&self, ds: DataSource) {
    *self.data_source.lock().unwrap() = ds;
  }

  pub fn get_data_source(&self) -> DataSource {
    self.data_source.lock().unwrap().clone()
  }
}

pub fn start_background_updates(ui_weak: Weak<AppWindow>) {
  let state = Arc::new(AppState::new());

  // Restore persisted settings and apply to state + UI immediately.
  let settings = crate::config::load_settings();
  state.set_data_source(if settings.source == "remote" {
    DataSource::Remote(settings.remote_url.clone())
  } else {
    DataSource::Local
  });
  if let Some(ui) = ui_weak.upgrade() {
    ui.set_data_source_mode(if settings.source == "remote" {
      1
    } else {
      0
    });
    ui.set_data_source_url(settings.remote_url.clone().into());
  }

  if let Some(ui) = ui_weak.upgrade() {
    let state_mode = state.clone();
    ui.on_set_display_mode(move |idx| {
      let mode = match idx {
        1 => DisplayMode::CpuTemp,
        2 => DisplayMode::Ram,
        _ => DisplayMode::Uptime,
      };
      state_mode.set_mode(mode);
    });

    let state_save = state.clone();
    ui.on_save_data_source(move |mode, url| {
      let ds = if mode == 0 {
        DataSource::Local
      } else {
        DataSource::Remote(url.to_string())
      };
      state_save.set_data_source(ds);
      crate::config::save_settings(&crate::config::Settings {
        source: if mode == 0 {
          "local".to_string()
        } else {
          "remote".to_string()
        },
        remote_url: url.to_string(),
      });
    });
  }

  // Shared slot: fetch thread writes, display thread reads.
  let cached_remote: Arc<Mutex<Option<RemoteMonitor>>> = Arc::new(Mutex::new(None));

  // ── Dedicated HTTP fetch thread ──────────────────────────────────────────
  // 2-second hard timeout prevents stalled connections from skewing the cadence.
  let cached_remote_fetch = cached_remote.clone();
  let state_fetch = state.clone();
  thread::spawn(move || {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(2)).build();
    let mut last_url = String::new();

    loop {
      let tick_start = Instant::now();

      let source = state_fetch.get_data_source();
      if let DataSource::Remote(ref url) = source {
        if *url != last_url {
          *cached_remote_fetch.lock().unwrap() = None;
          last_url = url.clone();
        }
        if let Ok(resp) = agent.get(url).call() {
          if let Ok(data) = resp.into_json::<RemoteMonitor>() {
            *cached_remote_fetch.lock().unwrap() = Some(data);
          }
        }
      } else if !last_url.is_empty() {
        *cached_remote_fetch.lock().unwrap() = None;
        last_url.clear();
      }

      let elapsed = tick_start.elapsed();
      if let Some(remaining) = Duration::from_secs(1).checked_sub(elapsed) {
        thread::sleep(remaining);
      }
    }
  });

  // ── Display / sysinfo thread (100 ms cadence) ────────────────────────────
  let state_bg = state.clone();
  thread::spawn(move || {
    let mut sys = System::new_with_specifics(
      RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );

    let mut last_value = String::new();
    let mut last_label = String::new();
    let mut last_ram_mb: Option<u64> = None;

    loop {
      let current_mode = state_bg.get_mode();
      let data_source = state_bg.get_data_source();
      let remote_snapshot = cached_remote.lock().unwrap().clone();

      // Uptime is always computed for tray/notification, independent of display mode.
      let tray_uptime = match &data_source {
        DataSource::Local => {
          let u = format_uptime(System::uptime());
          format!("Local Uptime: {}", u)
        }
        DataSource::Remote(_) => {
          let u = remote_snapshot
            .as_ref()
            .and_then(|d| d.station.as_ref())
            .and_then(|s| s.uptime.as_deref())
            .unwrap_or("--");
          format!("Remote Uptime: {}", u)
        }
      };

      let (value, label) = match data_source {
        DataSource::Local => match current_mode {
          DisplayMode::Uptime => (format_uptime(System::uptime()), "Uptime".to_string()),

          DisplayMode::CpuTemp => (local_cpu_temp(), "CPU Temperature".to_string()),

          DisplayMode::Ram => {
            sys.refresh_memory();
            let total_bytes = sys.available_memory();
            let gb = total_bytes / 1_073_741_824;
            let mb = (total_bytes % 1_073_741_824) / 1_048_576;
            let mb_total = total_bytes / 1_048_576;

            if let Some(prev_mb) = last_ram_mb {
              if prev_mb >= 500 && mb_total < 500 {
                let ui_weak_clone = ui_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                  if let Some(ui) = ui_weak_clone.upgrade() {
                    ui.invoke_show_notification(
                      "Low Memory Warning".into(),
                      format!("Available RAM: {} MiB", mb_total).into(),
                    );
                  }
                });
              }
            }
            last_ram_mb = Some(mb_total);

            let display = if gb > 0 {
              format!("{}GB {}MB", gb, mb)
            } else {
              format!("{}MB", mb)
            };
            (display, "Available RAM".to_string())
          }
        },

        DataSource::Remote(_) => {
          let station = remote_snapshot.as_ref().and_then(|d| d.station.as_ref());
          match current_mode {
            DisplayMode::Uptime => (
              station.and_then(|s| s.uptime.as_deref()).unwrap_or("--").to_string(),
              "Uptime".to_string(), // prepend "Remote " or "Local "
            ),
            DisplayMode::CpuTemp => {
              let raw = station.and_then(|s| s.cpu_temperature_c.as_deref()).unwrap_or("--");
              let display = if raw == "--" {
                "--".to_string()
              } else {
                format!("{}°C", raw)
              };
              (display, "CPU Temperature".to_string()) // prepend "Remote " or "Local "
            }
            DisplayMode::Ram => (
              station.and_then(|s| s.ram_free.as_deref()).unwrap_or("--").to_string(),
              "Free RAM".to_string(), // prepend "Remote " or "Local "
            ),
          }
        }
      };

      if value != last_value || label != last_label {
        let value_clone = value.clone();
        let label_clone = label.clone();
        let tray_clone = tray_uptime.clone();
        let ui_weak_clone = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
          if let Some(ui) = ui_weak_clone.upgrade() {
            ui.set_display_value(value_clone.into());
            ui.set_display_label(label_clone.into());
            ui.set_tray_text(tray_clone.into());
          }
        });
        last_value = value;
        last_label = label;
      } else {
        // Tray uptime still advances even when display value hasn't changed.
        let tray_clone = tray_uptime;
        let ui_weak_clone = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
          if let Some(ui) = ui_weak_clone.upgrade() {
            ui.set_tray_text(tray_clone.into());
          }
        });
      }

      thread::sleep(Duration::from_millis(100));
    }
  });
}

fn format_uptime(seconds: u64) -> String {
  let days = seconds / 86_400;
  let hours = (seconds % 86_400) / 3_600;
  let minutes = (seconds % 3_600) / 60;
  let secs = seconds % 60;
  format!("{:02}:{:02}:{:02}:{:02}", days, hours, minutes, secs)
}

// On Android, sysinfo Components does not enumerate hwmon sensors.
// Read directly from the kernel thermal subsystem instead.
#[cfg(target_os = "android")]
fn local_cpu_temp() -> String {
  for i in 0u8..=30 {
    let type_path = format!("/sys/class/thermal/thermal_zone{i}/type");
    let temp_path = format!("/sys/class/thermal/thermal_zone{i}/temp");
    let Ok(zone_type) = std::fs::read_to_string(&type_path) else {
      continue;
    };
    let z = zone_type.trim().to_lowercase();
    if !z.contains("cpu") && !z.contains("soc") && z != "skin" {
      continue;
    }
    let Ok(raw) = std::fs::read_to_string(&temp_path) else {
      continue;
    };
    let Ok(millideg) = raw.trim().parse::<i32>() else {
      continue;
    };
    let celsius = millideg as f32 / 1000.0;
    if (1.0..150.0).contains(&celsius) {
      return format!("{:.1}°C", celsius);
    }
  }
  "--".to_string()
}

#[cfg(not(target_os = "android"))]
fn local_cpu_temp() -> String {
  let comps = Components::new_with_refreshed_list();
  comps
    .iter()
    .find(|c| {
      let l = c.label().to_lowercase();
      l.contains("tctl")
        || l.contains("package id 0")
        || l.starts_with("core 0")
        || l.contains("cpu temp")
    })
    .and_then(|c| c.temperature())
    .map(|t| format!("{:.1}°C", t))
    .unwrap_or_else(|| "--".to_string())
}

// ─────────────────────────────────────────────────────────────
// Login callbacks
// ─────────────────────────────────────────────────────────────
use slint::SharedString;

pub fn setup_login_callbacks(ui_weak: Weak<AppWindow>) {
  // Anonymous login handler
  if let Some(ui) = ui_weak.upgrade() {
    ui.on_connect_anonymous({
      let weak = ui_weak.clone(); // clone for the whole callback
      move |server_url: SharedString| {
        let url = server_url.to_string();

        // Clone again for the async task (this is the key fix)
        let weak_async = weak.clone();

        tokio::spawn(async move {
          match login::perform_anonymous_login(url).await {
            Ok(session) => {
              let msg =
                format!("Session UUID: {}\nUser UUID: {}", session.session_uuid, session.user_uuid);

              let weak_invoke = weak_async.clone();
              let _ = slint::invoke_from_event_loop(move || {
                if let Some(u) = weak_invoke.upgrade() {
                  u.invoke_show_notification("[INFO] Login successful".into(), msg.into());
                  u.set_app_mode(0); // return to normal view
                }
              });
            }
            Err(e) => {
              let weak_invoke = weak_async.clone();
              let _ = slint::invoke_from_event_loop(move || {
                if let Some(u) = weak_invoke.upgrade() {
                  u.invoke_show_notification("[INFO] Login failed".into(), e.into());
                }
              });
            }
          }
        });
      }
    });
  }

  // Login success callback (called from Rust after successful login)
  if let Some(ui) = ui_weak.upgrade() {
    ui.on_login_success({
      let weak = ui_weak.clone();
      move || {
        let weak_invoke = weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
          if let Some(u) = weak_invoke.upgrade() {
            u.set_app_mode(0);
          }
        });
      }
    });
  }
}
