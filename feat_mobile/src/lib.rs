mod fullscreen;
mod jni_utils;
mod tray;

use android_activity::AndroidApp;
use feat_shared::AppWindow;
use slint::ComponentHandle;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::fullscreen::{
  AppState, InsetsState, apply_insets_to_ui, force_orientation_flip, force_view_refresh,
  get_insets, restore_orientation, simulate_statusbar_swipe, simulate_statusbar_touch,
  toggle_system_bars,
};
use crate::jni_utils::try_jni_call;
use crate::tray::{
  STOP_REQUESTED, TRAY_ACTIVE, monitor_service_for_stop, request_notification_permission,
  start_foreground_service, stop_foreground_service, update_notification,
};

fn warmup_jni_silent(app: &AndroidApp) {
  for _ in 0..5 {
    let _ = get_insets(app);
    std::thread::sleep(std::time::Duration::from_millis(50));
  }
  let _ = try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    let _ = env.call_method(
      &window,
      "getInsetsController",
      "()Landroid/view/WindowInsetsController;",
      &[],
    )?;
    Ok(())
  });
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
fn android_main(app: android_activity::AndroidApp) {
  slint::android::init(app.clone()).unwrap();

  let ui = AppWindow::new().unwrap();
  ui.set_is_mobile(true);

  let state = Arc::new(Mutex::new(AppState::new()));

  // Warmup JNI
  let app_warmup = app.clone();
  std::thread::spawn(move || warmup_jni_silent(&app_warmup));

  // Permission and Service Start
  request_notification_permission(&app);
  STOP_REQUESTED.store(false, Ordering::SeqCst);
  TRAY_ACTIVE.store(true, Ordering::SeqCst);
  start_foreground_service(&app, "Available RAM: …");

  // Monitor service status (detects Stop button press and triggers quit)
  let ui_weak_for_stop = ui.as_weak();
  monitor_service_for_stop(&app, move || {
    let ui_weak = ui_weak_for_stop.clone();
    let _ = slint::invoke_from_event_loop(move || {
      if let Some(ui) = ui_weak.upgrade() {
        ui.invoke_quit();
      }
    });
  });

  // ── Insets Monitor ────────────────────────────────────────────────────────
  let app_monitor = app.clone();
  let ui_weak_monitor = ui.as_weak();
  let state_monitor = state.clone();

  let timer = slint::Timer::default();
  timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(100), move || {
    if let Some(current_insets) = get_insets(&app_monitor) {
      let mut app_state = state_monitor.lock().unwrap();

      if !app_state.initialized && current_insets.has_insets() {
        app_state.base_insets = Some(current_insets);
        app_state.last_insets = current_insets;
        app_state.actual_fullscreen = false;
        app_state.initialized = true;
        if let Some(ui) = ui_weak_monitor.upgrade() {
          apply_insets_to_ui(&ui, current_insets, false, &app_monitor);
        }
        return;
      }

      if !app_state.initialized {
        return;
      }

      let detected_fullscreen = !current_insets.has_insets();
      if current_insets != app_state.last_insets
        || detected_fullscreen != app_state.actual_fullscreen
      {
        app_state.last_insets = current_insets;
        app_state.actual_fullscreen = detected_fullscreen;

        if let Some(ui) = ui_weak_monitor.upgrade() {
          let padding = if detected_fullscreen {
            InsetsState::new()
          } else if current_insets.has_insets() {
            current_insets
          } else {
            app_state.base_insets.unwrap_or(InsetsState::new())
          };
          apply_insets_to_ui(&ui, padding, detected_fullscreen, &app_monitor);
        }
      }
    }
  });

  // ── Notification Updater ──────────────────────────────────────────────────
  let app_notif = app.clone();
  let ui_weak_notif = ui.as_weak();
  let notif_timer = slint::Timer::default();
  notif_timer.start(
    slint::TimerMode::Repeated,
    std::time::Duration::from_millis(1000),
    move || {
      if STOP_REQUESTED.load(Ordering::SeqCst) || !TRAY_ACTIVE.load(Ordering::SeqCst) {
        return;
      }
      if let Some(ui) = ui_weak_notif.upgrade() {
        let ram_text = format!("Available RAM: {}", ui.get_display_value());
        update_notification(&app_notif, &ram_text);
      }
    },
  );

  // ── Quit Handler ──────────────────────────────────────────────────────────
  let app_quit = app.clone();
  ui.on_quit(move || {
    println!("🔔 Quit handler called");
    stop_foreground_service(&app_quit);
    std::process::exit(0);
  });

  // ── Fullscreen Toggle ─────────────────────────────────────────────────────
  let app_fs = app.clone();
  let state_fs = state.clone();
  ui.on_toggle_fullscreen({
    let ui_weak = ui.as_weak();
    move || {
      if let Some(ui) = ui_weak.upgrade() {
        let mut app_state = state_fs.lock().unwrap();
        if !app_state.initialized {
          return;
        }

        let target_fullscreen = !app_state.actual_fullscreen;
        let base_insets = app_state.base_insets;
        app_state.actual_fullscreen = target_fullscreen;
        drop(app_state);

        let app_toggle = app_fs.clone();
        let ui_weak_toggle = ui.as_weak();

        std::thread::spawn(move || {
          simulate_statusbar_touch(&app_toggle);
          std::thread::sleep(std::time::Duration::from_millis(100));

          let mut success = false;
          for _ in 0..5 {
            if toggle_system_bars(&app_toggle, target_fullscreen) {
              success = true;
              break;
            }
            std::thread::sleep(std::time::Duration::from_millis(30));
          }

          if success {
            force_orientation_flip(&app_toggle);
            std::thread::sleep(std::time::Duration::from_millis(150));
            restore_orientation(&app_toggle);
            std::thread::sleep(std::time::Duration::from_millis(50));
            force_view_refresh(&app_toggle);

            if !target_fullscreen {
              std::thread::sleep(std::time::Duration::from_millis(100));
              simulate_statusbar_swipe(&app_toggle);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            force_view_refresh(&app_toggle);

            for _ in 0..10 {
              if let Some(new_insets) = get_insets(&app_toggle) {
                let expected_fs = !new_insets.has_insets();
                let padding = if expected_fs {
                  InsetsState::new()
                } else if new_insets.has_insets() {
                  new_insets
                } else {
                  base_insets.unwrap_or(InsetsState::new())
                };

                let ui_weak_up = ui_weak_toggle.clone();
                let app_up = app_toggle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                  if let Some(ui) = ui_weak_up.upgrade() {
                    apply_insets_to_ui(&ui, padding, expected_fs, &app_up);
                  }
                });

                if expected_fs == target_fullscreen {
                  break;
                }
              }
              std::thread::sleep(std::time::Duration::from_millis(50));
            }
          }
        });
      }
    }
  });

  feat_shared::run_app(ui);

  // Final Cleanup
  stop_foreground_service(&app);
}
