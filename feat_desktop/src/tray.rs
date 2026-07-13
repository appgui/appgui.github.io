/// Platform-specific system tray implementation
use notify_rust::{Notification, Timeout};
use slint::{ComponentHandle, Weak, invoke_from_event_loop};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use feat_shared::AppWindow;

// Linux-specific imports
#[cfg(target_os = "linux")]
use feat_shared::{WINDOW_VISIBLE, is_window_visible};
#[cfg(target_os = "linux")]
use std::sync::atomic::Ordering;

#[cfg(target_os = "linux")]
mod linux_impl {
  use super::*;
  use ksni::blocking::TrayMethods;
  use ksni::{self, Tray, menu::StandardItem};

  pub struct SystemTray {
    ui_weak: Weak<AppWindow>,
    tooltip: Arc<Mutex<String>>,
  }

  impl Tray for SystemTray {
    fn id(&self) -> String {
      "com.mikko.xplatform".to_string()
    }

    fn title(&self) -> String {
      "System Monitor".to_string()
    }

    fn icon_name(&self) -> String {
      "utilities-system-monitor".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
      vec![]
    }

    fn tool_tip(&self) -> ksni::ToolTip {
      let tooltip_text = self.tooltip.lock().unwrap().clone();
      ksni::ToolTip {
        icon_name: "".to_string(),
        icon_pixmap: vec![],
        title: tooltip_text,
        description: "".to_string(),
      }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
      let ui_weak = self.ui_weak.clone();

      // Determine current visibility state
      let is_currently_visible =
        ui_weak.upgrade().map(|ui| ui.window().is_visible()).unwrap_or_else(|| is_window_visible());

      vec![
        StandardItem {
          label: if is_currently_visible {
            "Hide to Tray".to_string()
          } else {
            "Show Window".to_string()
          },
          activate: Box::new(move |_this: &mut Self| {
            let ui_weak = ui_weak.clone();
            let _ = invoke_from_event_loop(move || {
              if let Some(ui) = ui_weak.upgrade() {
                let window = ui.window();
                // Toggle based on actual current state
                if window.is_visible() {
                  window.hide().unwrap();
                  WINDOW_VISIBLE.store(false, Ordering::SeqCst);
                } else {
                  window.show().unwrap();
                  WINDOW_VISIBLE.store(true, Ordering::SeqCst);
                }
              }
            });
          }),
          ..Default::default()
        }
        .into(),
        ksni::MenuItem::Separator,
        StandardItem {
          label: "Quit".to_string(),
          activate: Box::new(|_| std::process::exit(0)),
          ..Default::default()
        }
        .into(),
      ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
      let ui_weak = self.ui_weak.clone();
      let _ = invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
          let window = ui.window();
          if window.is_visible() {
            window.hide().unwrap();
            WINDOW_VISIBLE.store(false, Ordering::SeqCst);
          } else {
            window.show().unwrap();
            WINDOW_VISIBLE.store(true, Ordering::SeqCst);
          }
        }
      });
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {}
  }

  pub struct TrayState {
    handle: Arc<Mutex<Option<ksni::blocking::Handle<SystemTray>>>>,
    tooltip: Arc<Mutex<String>>,
  }

  impl TrayState {
    pub fn new() -> Self {
      Self {
        handle: Arc::new(Mutex::new(None)),
        tooltip: Arc::new(Mutex::new("System Monitor".to_string())),
      }
    }

    pub fn init(&self, ui_weak: Weak<AppWindow>) -> Result<(), Box<dyn std::error::Error>> {
      let tray = SystemTray {
        ui_weak: ui_weak.clone(),
        tooltip: self.tooltip.clone(),
      };

      let handle = tray.spawn()?;
      *self.handle.lock().unwrap() = Some(handle.clone());

      self.start_updater(handle, ui_weak);

      println!("[INFO] Tray icon initialised (ksni/StatusNotifierItem)");
      Ok(())
    }

    fn start_updater(&self, handle: ksni::blocking::Handle<SystemTray>, ui_weak: Weak<AppWindow>) {
      let tooltip = self.tooltip.clone();

      thread::spawn(move || {
        loop {
          thread::sleep(Duration::from_millis(1000));

          let (tx, rx) = std::sync::mpsc::channel();
          let ui_weak_clone = ui_weak.clone();

          let _ = invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_clone.upgrade() {
              let _ = tx.send(ui.get_tray_text().to_string());
            }
          });

          if let Ok(tray_text) = rx.recv_timeout(Duration::from_millis(100)) {
            let old_tooltip = tooltip.lock().unwrap().clone();
            if old_tooltip != tray_text {
              *tooltip.lock().unwrap() = tray_text;
              let _ = handle.update(|_| {});
            }
          } else {
            break;
          }
        }
      });
    }
  }
}

#[cfg(not(target_os = "linux"))]
mod desktop_impl {
  use super::*;
  use std::time::Instant;
  use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
  use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

  pub struct TrayState {
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    tooltip: Arc<Mutex<String>>,
  }

  impl TrayState {
    pub fn new() -> Self {
      Self {
        tray_icon: Arc::new(Mutex::new(None)),
        tooltip: Arc::new(Mutex::new("System Monitor".to_string())),
      }
    }

    pub fn init(&self, ui_weak: Weak<AppWindow>) -> Result<(), Box<dyn std::error::Error>> {
      let menu = Menu::new();
      let show_item = MenuItem::new("Toggle Window", true, None);
      let separator = PredefinedMenuItem::separator();
      let quit_item = MenuItem::new("Quit", true, None);

      menu.append(&show_item)?;
      menu.append(&separator)?;
      menu.append(&quit_item)?;

      let show_item_id = show_item.id().clone();
      let quit_item_id = quit_item.id().clone();

      let icon = create_default_icon();

      let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu.clone()))
        .with_menu_on_left_click(false)
        .with_tooltip(self.tooltip.lock().unwrap().clone())
        .with_icon(icon)
        .build()?;

      *self.tray_icon.lock().unwrap() = Some(tray);

      // Menu event handler
      let ui_weak_clone = ui_weak.clone();
      thread::spawn(move || {
        let menu_channel = MenuEvent::receiver();
        while let Ok(event) = menu_channel.recv() {
          if event.id == show_item_id {
            let ui_weak = ui_weak_clone.clone();
            let _ = invoke_from_event_loop(move || {
              if let Some(ui) = ui_weak.upgrade() {
                let window = ui.window();
                if window.is_visible() {
                  let _ = window.hide();
                } else {
                  let _ = window.show();
                }
              }
            });
          } else if event.id == quit_item_id {
            std::process::exit(0);
          }
        }
      });

      // Tray icon event handler with debouncing
      let ui_weak_clone = ui_weak.clone();
      thread::spawn(move || {
        let tray_channel = TrayIconEvent::receiver();
        let mut last_click = Instant::now();

        while let Ok(event) = tray_channel.recv() {
          match event {
            // Left click - toggle window, but debounce to prevent double-fire
            TrayIconEvent::Click {
              button,
              ..
            } if button == tray_icon::MouseButton::Left => {
              let now = Instant::now();
              // Only process if at least 300ms since last click
              if now.duration_since(last_click).as_millis() > 300 {
                last_click = now;
                let ui_weak = ui_weak_clone.clone();
                let _ = invoke_from_event_loop(move || {
                  if let Some(ui) = ui_weak.upgrade() {
                    let window = ui.window();
                    if window.is_visible() {
                      let _ = window.hide();
                    } else {
                      let _ = window.show();
                    }
                  }
                });
              }
            }
            // Explicitly ignore right-clicks - let default menu behavior handle it
            TrayIconEvent::Click {
              button,
              ..
            } if button == tray_icon::MouseButton::Right => {
              // Do nothing - default behavior shows/hides menu
            }
            // Ignore all other events
            _ => {}
          }
        }
      });

      self.start_updater(ui_weak);
      println!("[INFO] Tray icon initialized (tray-icon)");
      Ok(())
    }

    fn start_updater(&self, ui_weak: Weak<AppWindow>) {
      let tooltip = self.tooltip.clone();

      thread::spawn(move || {
        loop {
          thread::sleep(Duration::from_millis(1000));

          let (tx, rx) = std::sync::mpsc::channel();
          let ui_weak_clone = ui_weak.clone();

          let _ = invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_clone.upgrade() {
              let _ = tx.send(ui.get_tray_text().to_string());
            }
          });

          if let Ok(tray_text) = rx.recv_timeout(Duration::from_millis(100)) {
            *tooltip.lock().unwrap() = tray_text;
          } else {
            break;
          }
        }
      });
    }
  }

  fn create_default_icon() -> tray_icon::Icon {
    let size: u32 = 32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..(size * size) {
      rgba.extend_from_slice(&[100, 150, 200, 255]);
    }
    tray_icon::Icon::from_rgba(rgba, size, size).expect("[FAIL] Failed to create default icon")
  }
}

#[cfg(not(target_os = "linux"))]
use desktop_impl::TrayState;
#[cfg(target_os = "linux")]
use linux_impl::TrayState;

pub fn setup_tray_callbacks(ui_weak: Weak<AppWindow>) -> Result<(), Box<dyn std::error::Error>> {
  let state = Arc::new(TrayState::new());
  state.init(ui_weak.clone())?;

  if let Some(ui) = ui_weak.upgrade() {
    ui.on_show_notification(move |title, message| {
      let _ = Notification::new()
        .summary(&title)
        .body(&message)
        .timeout(Timeout::Milliseconds(5000))
        .show();
    });

    // Tooltip is maintained by start_updater; no separate on_update_tray handler needed.
  }

  Ok(())
}
