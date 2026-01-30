use feat_shared::AppWindow;

#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
  use slint::android;
  android::init(app).unwrap();

  let ui = AppWindow::new().unwrap();
  feat_shared::run_app(ui);
}
