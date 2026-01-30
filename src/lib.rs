slint::include_modules!();

mod logic;
use logic::start_background_updates;

// Embed icon at compile time - path relative to this file
const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

pub fn run_app(ui: AppWindow) {
    start_background_updates(ui.as_weak());
    ui.run().unwrap();
}

// Export for platforms to use
pub fn load_app_icon() -> Result<slint::Image, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(ICON_BYTES)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(slint::Image::from_rgba8(
        slint::SharedPixelBuffer::clone_from_slice(rgba.as_raw(), width, height)
    ))
}
