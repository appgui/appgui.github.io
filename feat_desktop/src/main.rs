#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use feat_shared::AppWindow;
use slint::ComponentHandle;
use i_slint_backend_winit::WinitWindowAccessor;

fn main() -> Result<(), slint::PlatformError> {
  let ui = AppWindow::new()?;

  // Ensure we are in desktop mode (default is false, but being explicit is good)
  ui.set_is_mobile(false);

  let ui_weak = ui.as_weak();

  // Handle Window Dragging
  ui.on_start_drag({
    let ui_weak = ui_weak.clone();
    move || {
      let ui = ui_weak.unwrap();
      ui.window().with_winit_window(|win| {
        let _ = win.drag_window();
      });
    }
  });

  // Handle Minimize
  ui.on_minimize({
    let ui_weak = ui_weak.clone();
    move || {
      let ui = ui_weak.unwrap();
      ui.window().with_winit_window(|win| {
        win.set_minimized(true);
      });
    }
  });

  // Handle Maximize / Restore
  ui.on_maximize({
    let ui_weak = ui_weak.clone();
    move || {
      let ui = ui_weak.unwrap();
      ui.window().with_winit_window(|win| {
        let currently_maximized = win.is_maximized();
        win.set_maximized(!currently_maximized);
        // Sync the UI property to change the icon
        ui.set_is_maximized(!currently_maximized);
      });
    }
  });

  ui.on_quit(|| {
    std::process::exit(0);
  });

  feat_shared::run_app(ui);
  Ok(())
}
