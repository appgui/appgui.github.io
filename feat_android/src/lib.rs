use feat_shared::AppWindow;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
fn android_main(app: android_activity::AndroidApp) {
  use slint::android;

  android::init(app).unwrap();

  let ui = AppWindow::new().unwrap();

  ui.set_is_mobile(true);

  ui.on_quit(move || {
    std::process::exit(0); // Force close (not ideal, but okay for now)
  });

  feat_shared::run_app(ui);
}
