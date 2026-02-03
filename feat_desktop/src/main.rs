#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use feat_shared::AppWindow;
use slint::ComponentHandle;
use i_slint_backend_winit::WinitWindowAccessor;
use winit::window::{Window, ResizeDirection, Fullscreen};

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.set_is_mobile(false);

    let ui_weak = ui.as_weak();

    // ── Drag ────────────────────────────────────────────────────────────────
    ui.on_start_drag({
        let ui_weak = ui_weak.clone();
        move || {
            let ui = ui_weak.unwrap();
            ui.window().with_winit_window(|win: &Window| {
                let _ = win.drag_window();
            });
        }
    });

    // ── Resize ──────────────────────────────────────────────────────────────
    ui.on_start_resize_drag({
        let ui_weak = ui_weak.clone();
        move |direction| {
            let ui = ui_weak.unwrap();
            ui.window().with_winit_window(|win: &Window| {
                let dir = match direction.as_str() {
                    "n"  => ResizeDirection::North,
                    "s"  => ResizeDirection::South,
                    "w"  => ResizeDirection::West,
                    "e"  => ResizeDirection::East,
                    "nw" => ResizeDirection::NorthWest,
                    "ne" => ResizeDirection::NorthEast,
                    "sw" => ResizeDirection::SouthWest,
                    "se" => ResizeDirection::SouthEast,
                    _    => ResizeDirection::SouthEast,
                };
                let _ = win.drag_resize_window(dir);
            });
        }
    });

    // ── Minimize ────────────────────────────────────────────────────────────
    ui.on_minimize({
        let ui_weak = ui_weak.clone();
        move || {
            let ui = ui_weak.unwrap();
            ui.window().with_winit_window(|win: &Window| {
                win.set_minimized(true);
            });
        }
    });

    // ── Maximize / Restore ──────────────────────────────────────────────────
    // On Wayland, set_maximized() is an async request to the compositor.
    // Reading win.is_maximized() immediately after setting it returns the OLD
    // value – the toggle flips back and the window never actually changes size.
    //
    // Fix: the Slint property is the source of truth.  Toggle it first, then
    // send the request to winit unconditionally based on the new value.  Never
    // read state back from winit.
    ui.on_maximize({
        let ui_weak = ui_weak.clone();
        move || {
            let ui = ui_weak.unwrap();
            let new_state = !ui.get_is_maximized();
            ui.set_is_maximized(new_state);
            ui.window().with_winit_window(|win: &Window| {
                win.set_maximized(new_state);
            });
        }
    });

    // ── Fullscreen toggle ───────────────────────────────────────────────────
    // Same async-read-back race as maximize.  Toggle property first, request
    // unconditionally.
    ui.on_toggle_fullscreen({
        let ui_weak = ui_weak.clone();
        move || {
            let ui = ui_weak.unwrap();
            let new_state = !ui.get_is_fullscreen();
            ui.set_is_fullscreen(new_state);
            ui.window().with_winit_window(|win: &Window| {
                if new_state {
                    win.set_fullscreen(Some(Fullscreen::Borderless(None)));
                } else {
                    win.set_fullscreen(None);
                }
            });
        }
    });

    // ── Quit ────────────────────────────────────────────────────────────────
    ui.on_quit(|| {
        std::process::exit(0);
    });

    feat_shared::run_app(ui);
    Ok(())
}
