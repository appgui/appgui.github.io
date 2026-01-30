use feat_shared::AppWindow;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    feat_shared::run_app(ui);
    Ok(())
}
