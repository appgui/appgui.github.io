use crate::jni_utils::try_jni_call;
use android_activity::AndroidApp;
use jni::objects::{JObject, JValue};
use std::sync::atomic::{AtomicBool, Ordering};

pub static TRAY_ACTIVE: AtomicBool = AtomicBool::new(false);
pub static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

fn make_service_intent<'a>(
  env: &mut jni::JNIEnv<'a>,
  activity: &JObject,
  extra_key: Option<&str>,
  extra_val: Option<&str>,
) -> Result<JObject<'a>, jni::errors::Error> {
  let class_loader =
    env.call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;

  let svc_class_name = env.new_string("com.mikko.xplatform.TrayService")?;
  let svc_class = env
    .call_method(
      &class_loader,
      "loadClass",
      "(Ljava/lang/String;)Ljava/lang/Class;",
      &[JValue::Object(&svc_class_name)],
    )?
    .l()?;

  let intent_class = env.find_class("android/content/Intent")?;
  let intent = env.new_object(
    &intent_class,
    "(Landroid/content/Context;Ljava/lang/Class;)V",
    &[JValue::Object(activity), JValue::Object(&svc_class)],
  )?;

  if let (Some(key), Some(val)) = (extra_key, extra_val) {
    let k = env.new_string(key)?;
    let v = env.new_string(val)?;
    env.call_method(
      &intent,
      "putExtra",
      "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
      &[JValue::Object(&k), JValue::Object(&v)],
    )?;
  }

  Ok(intent)
}

pub fn start_foreground_service(app: &AndroidApp, ram_text: &str) {
  if STOP_REQUESTED.load(Ordering::SeqCst) {
    return;
  }

  let ram_text = ram_text.to_string();
  let _ = try_jni_call(app, |env, activity| {
    let intent = make_service_intent(env, &activity, Some("ram_text"), Some(&ram_text))?;
    env.call_method(
      &activity,
      "startForegroundService",
      "(Landroid/content/Intent;)Landroid/content/ComponentName;",
      &[JValue::Object(&intent)],
    )?;
    Ok(())
  });
}

pub fn update_notification(app: &AndroidApp, ram_text: &str) {
  // Guard against reappearing notification during shutdown
  if STOP_REQUESTED.load(Ordering::SeqCst) || !TRAY_ACTIVE.load(Ordering::SeqCst) {
    return;
  }

  let ram_text = ram_text.to_string();
  let _ = try_jni_call(app, |env, activity| {
    let intent = make_service_intent(env, &activity, Some("ram_text"), Some(&ram_text))?;
    env.call_method(
      &activity,
      "startService",
      "(Landroid/content/Intent;)Landroid/content/ComponentName;",
      &[JValue::Object(&intent)],
    )?;
    Ok(())
  });
}

pub fn stop_foreground_service(app: &AndroidApp) {
  STOP_REQUESTED.store(true, Ordering::SeqCst);
  TRAY_ACTIVE.store(false, Ordering::SeqCst);

  let _ = try_jni_call(app, |env, activity| {
    let intent = make_service_intent(env, &activity, Some("action"), Some("stop"))?;
    env.call_method(
      &activity,
      "startService",
      "(Landroid/content/Intent;)Landroid/content/ComponentName;",
      &[JValue::Object(&intent)],
    )?;
    Ok(())
  });
}

pub fn request_notification_permission(app: &AndroidApp) {
  let _ = try_jni_call(app, |env, activity| {
    let build_class = env.find_class("android/os/Build$VERSION")?;
    let sdk_int = env.get_static_field(build_class, "SDK_INT", "I")?.i()?;

    if sdk_int >= 33 {
      let permission = env.new_string("android.permission.POST_NOTIFICATIONS")?;
      let string_class = env.find_class("java/lang/String")?;
      let arr = env.new_object_array(1, string_class, &permission)?;
      env.call_method(
        &activity,
        "requestPermissions",
        "([Ljava/lang/String;I)V",
        &[JValue::Object(&arr), JValue::Int(100)],
      )?;
    }
    Ok(())
  });
}

pub fn monitor_service_for_stop<F>(app: &AndroidApp, on_stop: F)
where
  F: Fn() + Send + 'static,
{
  let app_clone = app.clone();

  std::thread::spawn(move || {
    loop {
      std::thread::sleep(std::time::Duration::from_millis(400));

      if STOP_REQUESTED.load(Ordering::SeqCst) {
        break;
      }

      let service_alive = try_jni_call(&app_clone, |env, activity| {
        let manager_name = env.new_string("activity")?;
        let manager = env
          .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&manager_name)],
          )?
          .l()?;

        let services = env
          .call_method(&manager, "getRunningServices", "(I)Ljava/util/List;", &[JValue::Int(50)])?
          .l()?;

        let size = env.call_method(&services, "size", "()I", &[])?.i()?;
        let target = env.new_string("com.mikko.xplatform.TrayService")?;

        for i in 0..size {
          let info =
            env.call_method(&services, "get", "(I)Ljava/lang/Object;", &[JValue::Int(i)])?.l()?;

          let service_field = env.get_field(&info, "service", "Landroid/content/ComponentName;")?;
          let component = service_field.l()?;
          let class_name =
            env.call_method(&component, "getClassName", "()Ljava/lang/String;", &[])?.l()?;

          let matches = env
            .call_method(
              &target,
              "equals",
              "(Ljava/lang/Object;)Z",
              &[JValue::Object(&class_name)],
            )?
            .z()?;

          if matches {
            return Ok(true);
          }
        }
        Ok(false)
      })
      .unwrap_or(false);

      if !service_alive && !STOP_REQUESTED.load(Ordering::SeqCst) {
        println!("[DONE] Service stopped via Stop button, triggering app quit");
        STOP_REQUESTED.store(true, Ordering::SeqCst);
        TRAY_ACTIVE.store(false, Ordering::SeqCst);
        on_stop();
        break;
      }
    }
  });
}
