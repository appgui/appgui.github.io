slint::include_modules!();

mod logic;
use logic::start_background_updates;

pub fn run_app(ui: AppWindow) {
  start_background_updates(ui.as_weak());
  ui.run().unwrap();
}
