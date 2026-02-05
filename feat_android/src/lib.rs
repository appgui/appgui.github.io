use feat_shared::AppWindow;
use android_activity::AndroidApp;
use slint::ComponentHandle;
use jni::JavaVM;
use jni::objects::{JObject, JValue};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Debug)]
struct InsetsState {
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
}

impl InsetsState {
  fn new() -> Self {
    Self { left: 0, top: 0, right: 0, bottom: 0 }
  }

  fn has_insets(&self) -> bool {
    self.left > 0 || self.top > 0 || self.right > 0 || self.bottom > 0
  }
}

struct AppState {
  actual_fullscreen: bool,
  last_insets: InsetsState,
  base_insets: Option<InsetsState>,
  initialized: bool,
}

impl AppState {
  fn new() -> Self {
    Self {
      actual_fullscreen: false,
      last_insets: InsetsState::new(),
      base_insets: None,
      initialized: false,
    }
  }
}

fn try_jni_call<F, R>(app: &AndroidApp, mut operation: F) -> Option<R>
where
  F: FnMut(&mut jni::JNIEnv, JObject) -> Result<R, jni::errors::Error>,
{
  let vm_ptr = app.vm_as_ptr();
  let activity_ptr = app.activity_as_ptr();

  if vm_ptr.is_null() || activity_ptr.is_null() {
    return None;
  }

  let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _).ok()? };
  let mut env = vm.attach_current_thread().ok()?;
  let activity = unsafe { JObject::from_raw(activity_ptr as *mut _) };

  match operation(&mut env, activity) {
    Ok(result) => Some(result),
    Err(_) => {
      let _ = env.exception_clear();
      None
    }
  }
}

fn toggle_system_bars(app: &AndroidApp, hide: bool) -> bool {
  try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let window = window.l()?;

    let controller = env.call_method(&window, "getInsetsController", "()Landroid/view/WindowInsetsController;", &[])?;
    let controller = controller.l()?;

    let type_class = env.find_class("android/view/WindowInsets$Type")?;
    let system_bars = env.call_static_method(type_class, "systemBars", "()I", &[])?;
    let system_bars_value = system_bars.i()?;

    if hide {
      env.call_method(&controller, "hide", "(I)V", &[JValue::Int(system_bars_value)])?;
      env.call_method(&controller, "setSystemBarsBehavior", "(I)V", &[JValue::Int(2)])?;
    } else {
      env.call_method(&controller, "show", "(I)V", &[JValue::Int(system_bars_value)])?;
    }

    Ok(())
  }).is_some()
}

fn is_portrait_orientation(app: &AndroidApp) -> bool {
  try_jni_call(app, |env, activity| {
    let resources = env.call_method(&activity, "getResources", "()Landroid/content/res/Resources;", &[])?;
    let resources_obj = resources.l()?;

    let config = env.call_method(&resources_obj, "getConfiguration", "()Landroid/content/res/Configuration;", &[])?;
    let config_obj = config.l()?;

    let orientation_field = env.get_field(&config_obj, "orientation", "I")?;
    let orientation = orientation_field.i()?;

    // 1 = PORTRAIT, 2 = LANDSCAPE
    Ok(orientation == 1)
  }).unwrap_or(true)
}

fn force_orientation_flip(app: &AndroidApp) -> bool {
  try_jni_call(app, |env, activity| {
    let resources = env.call_method(&activity, "getResources", "()Landroid/content/res/Resources;", &[])?;
    let resources_obj = resources.l()?;

    let config = env.call_method(&resources_obj, "getConfiguration", "()Landroid/content/res/Configuration;", &[])?;
    let config_obj = config.l()?;

    let orientation_field = env.get_field(&config_obj, "orientation", "I")?;
    let actual_orientation = orientation_field.i()?;

    let flip_to = if actual_orientation == 1 { 0 } else { 1 };

    env.call_method(&activity, "setRequestedOrientation", "(I)V", &[JValue::Int(flip_to)])?;

    Ok(())
  }).is_some()
}

fn restore_orientation(app: &AndroidApp) -> bool {
  try_jni_call(app, |env, activity| {
    env.call_method(&activity, "setRequestedOrientation", "(I)V", &[JValue::Int(-1)])?;
    Ok(())
  }).is_some()
}

fn force_view_refresh(app: &AndroidApp) -> bool {
  try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let window = window.l()?;

    let decor_view = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?;
    let decor_view = decor_view.l()?;

    env.call_method(&decor_view, "invalidate", "()V", &[])?;
    env.call_method(&decor_view, "requestLayout", "()V", &[])?;

    let content = env.call_method(&window, "findViewById", "(I)Landroid/view/View;", &[JValue::Int(0x01020002)])?;
    let content_view = content.l()?;
    if !content_view.is_null() {
      env.call_method(&content_view, "invalidate", "()V", &[])?;
      env.call_method(&content_view, "requestLayout", "()V", &[])?;
    }

    env.call_method(&decor_view, "requestApplyInsets", "()V", &[])?;

    Ok(())
  }).is_some()
}

fn simulate_statusbar_touch(app: &AndroidApp) -> bool {
  try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let window = window.l()?;

    let decor_view = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?;
    let decor_view = decor_view.l()?;

    let system_clock = env.find_class("android/os/SystemClock")?;
    let uptime = env.call_static_method(&system_clock, "uptimeMillis", "()J", &[])?;
    let event_time = uptime.j()?;

    let motion_event_class = env.find_class("android/view/MotionEvent")?;

    let x = JValue::Float(100.0);
    let y = JValue::Float(5.0);
    let down_time = JValue::Long(event_time);
    let meta_state = JValue::Int(0);

    let touch_down = env.call_static_method(
      &motion_event_class,
      "obtain",
      "(JJIFFI)Landroid/view/MotionEvent;",
      &[down_time, JValue::Long(event_time), JValue::Int(0), x, y, meta_state]
    )?;
    let touch_down_event = touch_down.l()?;
    env.call_method(&decor_view, "dispatchTouchEvent", "(Landroid/view/MotionEvent;)Z", &[JValue::Object(&touch_down_event)])?;

    let touch_up = env.call_static_method(
      &motion_event_class,
      "obtain",
      "(JJIFFI)Landroid/view/MotionEvent;",
      &[down_time, JValue::Long(event_time + 50), JValue::Int(1), x, y, meta_state]
    )?;
    let touch_up_event = touch_up.l()?;
    env.call_method(&decor_view, "dispatchTouchEvent", "(Landroid/view/MotionEvent;)Z", &[JValue::Object(&touch_up_event)])?;

    env.call_method(&touch_down_event, "recycle", "()V", &[])?;
    env.call_method(&touch_up_event, "recycle", "()V", &[])?;

    Ok(())
  }).is_some()
}

fn simulate_statusbar_swipe(app: &AndroidApp) -> bool {
  try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let window = window.l()?;

    let decor_view = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?;
    let decor_view = decor_view.l()?;

    let system_clock = env.find_class("android/os/SystemClock")?;
    let uptime = env.call_static_method(&system_clock, "uptimeMillis", "()J", &[])?;
    let event_time = uptime.j()?;

    let motion_event_class = env.find_class("android/view/MotionEvent")?;

    let x = JValue::Float(100.0);
    let y_start = JValue::Float(1.0);
    let down_time = JValue::Long(event_time);
    let meta_state = JValue::Int(0);

    let touch_down = env.call_static_method(
      &motion_event_class,
      "obtain",
      "(JJIFFI)Landroid/view/MotionEvent;",
      &[down_time, JValue::Long(event_time), JValue::Int(0), x, y_start, meta_state]
    )?;
    let touch_down_event = touch_down.l()?;
    env.call_method(&decor_view, "dispatchTouchEvent", "(Landroid/view/MotionEvent;)Z", &[JValue::Object(&touch_down_event)])?;

    for i in 1..=15 {
      let y_move = JValue::Float(1.0 + (i as f32 * 10.0));
      let move_time = event_time + (i * 8);

      let touch_move = env.call_static_method(
        &motion_event_class,
        "obtain",
        "(JJIFFI)Landroid/view/MotionEvent;",
        &[down_time, JValue::Long(move_time), JValue::Int(2), x, y_move, meta_state]
      )?;
      let touch_move_event = touch_move.l()?;
      env.call_method(&decor_view, "dispatchTouchEvent", "(Landroid/view/MotionEvent;)Z", &[JValue::Object(&touch_move_event)])?;
      env.call_method(&touch_move_event, "recycle", "()V", &[])?;
    }

    let y_end = JValue::Float(150.0);
    let up_time = event_time + 130;
    let touch_up = env.call_static_method(
      &motion_event_class,
      "obtain",
      "(JJIFFI)Landroid/view/MotionEvent;",
      &[down_time, JValue::Long(up_time), JValue::Int(1), x, y_end, meta_state]
    )?;
    let touch_up_event = touch_up.l()?;
    env.call_method(&decor_view, "dispatchTouchEvent", "(Landroid/view/MotionEvent;)Z", &[JValue::Object(&touch_up_event)])?;

    env.call_method(&touch_down_event, "recycle", "()V", &[])?;
    env.call_method(&touch_up_event, "recycle", "()V", &[])?;

    Ok(())
  }).is_some()
}

fn get_insets(app: &AndroidApp) -> Option<InsetsState> {
  try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let window = window.l()?;

    let decor_view = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?;
    let decor_view = decor_view.l()?;

    let insets = env.call_method(&decor_view, "getRootWindowInsets", "()Landroid/view/WindowInsets;", &[])?;
    let insets = insets.l()?;

    if insets.is_null() {
      return Ok(InsetsState::new());
    }

    let type_class = env.find_class("android/view/WindowInsets$Type")?;
    let system_bars = env.call_static_method(type_class, "systemBars", "()I", &[])?;
    let system_bars_value = system_bars.i()?;

    let insets_obj = env.call_method(&insets, "getInsets", "(I)Landroid/graphics/Insets;", &[JValue::Int(system_bars_value)])?;
    let insets_obj = insets_obj.l()?;

    let left = env.get_field(&insets_obj, "left", "I")?.i()?;
    let top = env.get_field(&insets_obj, "top", "I")?.i()?;
    let right = env.get_field(&insets_obj, "right", "I")?.i()?;
    let bottom = env.get_field(&insets_obj, "bottom", "I")?.i()?;

    Ok(InsetsState { left, top, right, bottom })
  })
}

fn apply_insets_to_ui(ui: &AppWindow, insets: InsetsState, fullscreen: bool, app: &AndroidApp) {
  ui.set_is_fullscreen(fullscreen);

  if fullscreen {
    ui.set_safe_inset_left(0.0);
    ui.set_safe_inset_top(0.0);
    ui.set_safe_inset_right(0.0);
    ui.set_safe_inset_bottom(0.0);
  } else {
    let is_portrait = is_portrait_orientation(app);

    ui.set_safe_inset_top(insets.top as f32);

    if is_portrait {
      // Portrait mode: 50% bottom padding, normal left/right
      ui.set_safe_inset_left(insets.left as f32);
      ui.set_safe_inset_right(insets.right as f32);
      ui.set_safe_inset_bottom((insets.bottom as f32) * 0.5);
    } else {
      // Landscape mode: 50% on both left and right sides
      ui.set_safe_inset_left((insets.left as f32) * 0.5);
      ui.set_safe_inset_right((insets.right as f32) * 0.5);
      ui.set_safe_inset_bottom(0.0);
    }
  }
}

fn warmup_jni_silent(app: &AndroidApp) {
  for _ in 0..5 {
    let _ = get_insets(app);
    std::thread::sleep(std::time::Duration::from_millis(50));
  }

  let _ = try_jni_call(app, |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let window = window.l()?;
    let _controller = env.call_method(&window, "getInsetsController", "()Landroid/view/WindowInsetsController;", &[])?;
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

  let app_warmup = app.clone();
  std::thread::spawn(move || {
    warmup_jni_silent(&app_warmup);
  });

  let app_monitor = app.clone();
  let ui_weak_monitor = ui.as_weak();
  let state_monitor = state.clone();

  let timer = slint::Timer::default();
  timer.start(
    slint::TimerMode::Repeated,
    std::time::Duration::from_millis(100),
    move || {
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

        if current_insets != app_state.last_insets || detected_fullscreen != app_state.actual_fullscreen {
          app_state.last_insets = current_insets;
          app_state.actual_fullscreen = detected_fullscreen;

          if let Some(ui) = ui_weak_monitor.upgrade() {
            let padding_insets = if detected_fullscreen {
              InsetsState::new()
            } else {
              if current_insets.has_insets() {
                current_insets
              } else {
                app_state.base_insets.unwrap_or(InsetsState::new())
              }
            };

            apply_insets_to_ui(&ui, padding_insets, detected_fullscreen, &app_monitor);
          }
        }
      }
    },
  );

  ui.on_quit(move || {
    std::process::exit(0);
  });

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

        let app_toggle = app_fs.clone();
        let ui_weak_toggle = ui.as_weak();
        let base_insets = app_state.base_insets;

        app_state.actual_fullscreen = target_fullscreen;
        drop(app_state);

        std::thread::spawn(move || {
          // Brief statusbar touch to dismiss navbar gracefully before toggling
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

            // If exiting fullscreen, swipe statusbar to trigger navbar
            if !target_fullscreen {
              std::thread::sleep(std::time::Duration::from_millis(100));
              simulate_statusbar_swipe(&app_toggle);
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
            force_view_refresh(&app_toggle);

            for _ in 0..10 {
              if let Some(new_insets) = get_insets(&app_toggle) {
                let expected_fullscreen = !new_insets.has_insets();

                let ui_weak_update = ui_weak_toggle.clone();
                let app_for_ui = app_toggle.clone();
                let padding_insets = if expected_fullscreen {
                  InsetsState::new()
                } else {
                  if new_insets.has_insets() {
                    new_insets
                  } else {
                    base_insets.unwrap_or(InsetsState::new())
                  }
                };

                let _ = slint::invoke_from_event_loop(move || {
                  if let Some(ui) = ui_weak_update.upgrade() {
                    apply_insets_to_ui(&ui, padding_insets, expected_fullscreen, &app_for_ui);
                  }
                });

                if expected_fullscreen == target_fullscreen {
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
}
