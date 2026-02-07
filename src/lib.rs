slint::include_modules!();

mod features;
use features::{start_background_updates, theme};

pub fn run_app(ui: AppWindow) {
  // Setup theme system
  theme::setup_theme_callbacks(ui.as_weak());

  // Setup background updates for RAM/Uptime
  start_background_updates(ui.as_weak());

  ui.run().unwrap();
}
