use crate::jni_utils::try_jni_call_labeled;
use android_activity::AndroidApp;
use feat_shared::AppWindow;
use jni::objects::{JClass, JValue};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InsetsState {
  pub left: i32,
  pub top: i32,
  pub right: i32,
  pub bottom: i32,
}

impl InsetsState {
  pub fn new() -> Self {
    Self {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    }
  }

  pub fn has_insets(&self) -> bool {
    self.left > 0 || self.top > 0 || self.right > 0 || self.bottom > 0
  }
}

pub struct AppState {
  pub actual_fullscreen: bool,
  pub last_insets: InsetsState,
  pub base_insets: Option<InsetsState>,
  pub initialized: bool,
  pub ready_ticks: u8,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      actual_fullscreen: false,
      last_insets: InsetsState::new(),
      base_insets: None,
      initialized: false,
      ready_ticks: 0,
    }
  }
}

pub fn toggle_system_bars(app: &AndroidApp, hide: bool) -> bool {
  // InsetsController.hide/show must run on the Java UI thread.
  // InsetsHelper dispatches via Activity.runOnUiThread.
  // env.find_class() on a native thread uses the bootstrap classloader and cannot
  // see app-defined classes — load via the activity's own classloader instead.
  try_jni_call_labeled(app, "toggle_system_bars", |env, activity| {
    let class_loader =
      env.call_method(&activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
    let class_name = env.new_string("ch/lehrax/appgui/InsetsHelper")?;
    let class_obj = env
      .call_method(
        &class_loader,
        "loadClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        &[JValue::Object(&class_name)],
      )?
      .l()?;
    let helper_class = JClass::from(class_obj);

    let method = if hide {
      "hideSystemBars"
    } else {
      "showSystemBars"
    };
    env.call_static_method(
      &helper_class,
      method,
      "(Landroid/app/Activity;)V",
      &[JValue::Object(&activity)],
    )?;
    Ok(())
  })
  .is_some()
}

pub fn is_portrait_orientation(app: &AndroidApp) -> bool {
  try_jni_call_labeled(app, "is_portrait_orientation", |env, activity| {
    let resources =
      env.call_method(&activity, "getResources", "()Landroid/content/res/Resources;", &[])?.l()?;
    let config = env
      .call_method(&resources, "getConfiguration", "()Landroid/content/res/Configuration;", &[])?
      .l()?;
    let orientation = env.get_field(&config, "orientation", "I")?.i()?;
    Ok(orientation == 1)
  })
  .unwrap_or(true)
}

pub fn simulate_statusbar_touch(app: &AndroidApp) -> bool {
  try_jni_call_labeled(app, "simulate_statusbar_touch", |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    let decor_view = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
    let system_clock = env.find_class("android/os/SystemClock")?;
    let event_time = env.call_static_method(&system_clock, "uptimeMillis", "()J", &[])?.j()?;
    let motion_event_class = env.find_class("android/view/MotionEvent")?;
    let (x, y) = (JValue::Float(100.0), JValue::Float(5.0));
    let down_time = JValue::Long(event_time);

    let down = env
      .call_static_method(
        &motion_event_class,
        "obtain",
        "(JJIFFI)Landroid/view/MotionEvent;",
        &[down_time, JValue::Long(event_time), JValue::Int(0), x, y, JValue::Int(0)],
      )?
      .l()?;
    env.call_method(
      &decor_view,
      "dispatchTouchEvent",
      "(Landroid/view/MotionEvent;)Z",
      &[JValue::Object(&down)],
    )?;

    let up = env
      .call_static_method(
        &motion_event_class,
        "obtain",
        "(JJIFFI)Landroid/view/MotionEvent;",
        &[down_time, JValue::Long(event_time + 50), JValue::Int(1), x, y, JValue::Int(0)],
      )?
      .l()?;
    env.call_method(
      &decor_view,
      "dispatchTouchEvent",
      "(Landroid/view/MotionEvent;)Z",
      &[JValue::Object(&up)],
    )?;

    env.call_method(&down, "recycle", "()V", &[])?;
    env.call_method(&up, "recycle", "()V", &[])?;
    Ok(())
  })
  .is_some()
}

pub fn simulate_statusbar_swipe(app: &AndroidApp) -> bool {
  try_jni_call_labeled(app, "simulate_statusbar_swipe", |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    let decor_view = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
    let system_clock = env.find_class("android/os/SystemClock")?;
    let event_time = env.call_static_method(&system_clock, "uptimeMillis", "()J", &[])?.j()?;
    let motion_event_class = env.find_class("android/view/MotionEvent")?;
    let x = JValue::Float(100.0);
    let meta = JValue::Int(0);

    let down = env
      .call_static_method(
        &motion_event_class,
        "obtain",
        "(JJIFFI)Landroid/view/MotionEvent;",
        &[
          JValue::Long(event_time),
          JValue::Long(event_time),
          JValue::Int(0),
          x,
          JValue::Float(1.0),
          meta,
        ],
      )?
      .l()?;
    env.call_method(
      &decor_view,
      "dispatchTouchEvent",
      "(Landroid/view/MotionEvent;)Z",
      &[JValue::Object(&down)],
    )?;

    for i in 1..=15 {
      let move_evt = env
        .call_static_method(
          &motion_event_class,
          "obtain",
          "(JJIFFI)Landroid/view/MotionEvent;",
          &[
            JValue::Long(event_time),
            JValue::Long(event_time + (i * 8)),
            JValue::Int(2),
            x,
            JValue::Float(1.0 + (i as f32 * 10.0)),
            meta,
          ],
        )?
        .l()?;
      env.call_method(
        &decor_view,
        "dispatchTouchEvent",
        "(Landroid/view/MotionEvent;)Z",
        &[JValue::Object(&move_evt)],
      )?;
      env.call_method(&move_evt, "recycle", "()V", &[])?;
    }

    let up = env
      .call_static_method(
        &motion_event_class,
        "obtain",
        "(JJIFFI)Landroid/view/MotionEvent;",
        &[
          JValue::Long(event_time),
          JValue::Long(event_time + 130),
          JValue::Int(1),
          x,
          JValue::Float(150.0),
          meta,
        ],
      )?
      .l()?;
    env.call_method(
      &decor_view,
      "dispatchTouchEvent",
      "(Landroid/view/MotionEvent;)Z",
      &[JValue::Object(&up)],
    )?;

    env.call_method(&down, "recycle", "()V", &[])?;
    env.call_method(&up, "recycle", "()V", &[])?;
    Ok(())
  })
  .is_some()
}

pub fn get_insets(app: &AndroidApp) -> Option<InsetsState> {
  try_jni_call_labeled(app, "get_insets", |env, activity| {
    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    let decor = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
    let insets =
      env.call_method(&decor, "getRootWindowInsets", "()Landroid/view/WindowInsets;", &[])?.l()?;

    if insets.is_null() {
      return Ok(InsetsState::new());
    }

    let type_class = env.find_class("android/view/WindowInsets$Type")?;
    let system_bars = env.call_static_method(type_class, "systemBars", "()I", &[])?.i()?;
    let insets_obj = env
      .call_method(
        &insets,
        "getInsets",
        "(I)Landroid/graphics/Insets;",
        &[JValue::Int(system_bars)],
      )?
      .l()?;

    Ok(InsetsState {
      left: env.get_field(&insets_obj, "left", "I")?.i()?,
      top: env.get_field(&insets_obj, "top", "I")?.i()?,
      right: env.get_field(&insets_obj, "right", "I")?.i()?,
      bottom: env.get_field(&insets_obj, "bottom", "I")?.i()?,
    })
  })
}

pub fn apply_insets_to_ui(ui: &AppWindow, insets: InsetsState, fullscreen: bool, app: &AndroidApp) {
  ui.set_is_fullscreen(fullscreen);
  if fullscreen {
    ui.set_safe_inset_left(0.0);
    ui.set_safe_inset_top(0.0);
    ui.set_safe_inset_right(0.0);
    ui.set_safe_inset_bottom(0.0);
  } else {
    let is_p = is_portrait_orientation(app);
    ui.set_safe_inset_top(insets.top as f32);
    if is_p {
      ui.set_safe_inset_left(insets.left as f32);
      ui.set_safe_inset_right(insets.right as f32);
      ui.set_safe_inset_bottom((insets.bottom as f32) * 0.5);
    } else {
      ui.set_safe_inset_left((insets.left as f32) * 0.5);
      ui.set_safe_inset_right((insets.right as f32) * 0.5);
      ui.set_safe_inset_bottom(0.0);
    }
  }
}
