#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;

use slint::ComponentHandle;

use feat_shared::AppWindow;

fn main() -> Result<(), slint::PlatformError> {
  let ui = AppWindow::new()?;
  ui.set_is_mobile(false);

  let ui_weak = ui.as_weak();

  ui.on_toggle_fullscreen({
    let ui_weak = ui_weak.clone();
    move || {
      if let Some(ui) = ui_weak.upgrade() {
        let current_state = ui.get_is_fullscreen();
        ui.window().set_fullscreen(!current_state);
        ui.set_is_fullscreen(!current_state);
      }
    }
  });

  ui.on_quit(|| {
    std::process::exit(0);
  });

  println!("🚀 Starting app...");

  feat_shared::run_app_with_tray(ui, |ui_weak| {
    if let Err(e) = tray::setup_tray_callbacks(ui_weak) {
      eprintln!("⚠️  Failed to initialize tray icon: {}", e);
    }
  });

  Ok(())
}
