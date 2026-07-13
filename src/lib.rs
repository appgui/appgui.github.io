slint::include_modules!();

mod config;
mod features;
use features::{setup_login_callbacks, start_background_updates, theme};
use lazy_static::lazy_static;
use std::sync::Arc;
pub use std::sync::atomic::{AtomicBool, Ordering};

pub use config::set_config_dir;

lazy_static! {
  pub static ref WINDOW_VISIBLE: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
}

slint::slint! {
  export component Keeper inherits Window {
    width: 1px;
    height: 1px;
    title: "";
    no-frame: true;
    background: transparent;
    Rectangle {
      background: transparent;
    }
  }
}

pub fn run_app(ui: AppWindow) {
  theme::setup_theme_callbacks(ui.as_weak());
  start_background_updates(ui.as_weak());
  setup_login_callbacks(ui.as_weak());
  ui.run().unwrap();
}

#[cfg(not(target_os = "android"))]
pub fn run_app_with_tray<F>(ui: AppWindow, tray_init: F)
where
  F: FnOnce(slint::Weak<AppWindow>) + Send + 'static,
{
  let rt = tokio::runtime::Runtime::new().unwrap();
  let _guard = rt.enter();

  let ui_weak = ui.as_weak();
  theme::setup_theme_callbacks(ui_weak.clone());
  start_background_updates(ui_weak.clone());
  setup_login_callbacks(ui.as_weak());

  // 1. Initial State Sync
  WINDOW_VISIBLE.store(true, Ordering::SeqCst);
  ui.show().unwrap();

  // 2. THE CLOSE BUTTON FIX:
  // Intercept the titlebar close button to update our atomic state
  let ui_weak_for_close = ui.as_weak();
  ui.window().on_close_requested(move || {
    if let Some(ui) = ui_weak_for_close.upgrade() {
      ui.window().hide().unwrap();
      // Update the state so the tray knows we are hidden!
      WINDOW_VISIBLE.store(false, Ordering::SeqCst);
    }
    // Tell Slint we handled the hide manually
    slint::CloseRequestResponse::HideWindow
  });

  // 3. Init Tray
  tray_init(ui_weak.clone());

  // 4. Run background loop
  slint::run_event_loop_until_quit().expect("[FAIL] Failed to run Slint event loop");
}

pub fn is_window_visible() -> bool {
  WINDOW_VISIBLE.load(Ordering::SeqCst)
}
