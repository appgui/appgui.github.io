#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use feat_shared::AppWindow;
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
  let ui = AppWindow::new()?;
  ui.set_is_mobile(false);

  let ui_weak = ui.as_weak();

  // ── Fullscreen ──────────────────────────────────────────────────────────
  ui.on_toggle_fullscreen({
    let ui_weak = ui_weak.clone();
    move || {
      println!("🔵 toggle_fullscreen callback triggered!");

      if let Some(ui) = ui_weak.upgrade() {
        let current_state = ui.get_is_fullscreen();
        println!("🟡 Current fullscreen state: {}", current_state);

        let window = ui.window();
        let new_state = !current_state;

        println!("🔷 Setting fullscreen to: {}", new_state);
        window.set_fullscreen(new_state);

        println!("🔷 Updating UI state");
        ui.set_is_fullscreen(new_state);

        println!("✅ Fullscreen toggle complete");
      }
    }
  });

  // ── Quit ────────────────────────────────────────────────────────────────
  ui.on_quit(|| {
    println!("👋 Quit callback triggered");
    std::process::exit(0);
  });

  println!("🚀 Starting app...");
  feat_shared::run_app(ui);
  Ok(())
}
