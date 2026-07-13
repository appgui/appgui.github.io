#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;

use i_slint_backend_winit::WinitWindowAccessor;
use slint::ComponentHandle;
use winit::dpi::LogicalSize;

use feat_shared::AppWindow;

fn main() -> Result<(), slint::PlatformError> {
  let ui = AppWindow::new()?;
  ui.set_is_mobile(false);

  ui.window().with_winit_window(|w| {
    w.set_min_inner_size(Some(LogicalSize::new(500.0_f64, 500.0_f64)));
  });

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

  println!("[INFO] Starting xPlatform...");

  feat_shared::run_app_with_tray(ui, |ui_weak| {
    if let Err(e) = tray::setup_tray_callbacks(ui_weak) {
      eprintln!("[FAIL] Failed to initialize tray icon: {}", e);
    }
  });

  Ok(())
}
