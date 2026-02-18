use android_activity::AndroidApp;
use jni::{JavaVM, objects::JObject};

pub fn try_jni_call<F, R>(app: &AndroidApp, mut operation: F) -> Option<R>
where
  F: FnMut(&mut jni::JNIEnv, JObject) -> Result<R, jni::errors::Error>,
{
  let vm_ptr = app.vm_as_ptr();
  let activity_ptr = app.activity_as_ptr();

  if vm_ptr.is_null() || activity_ptr.is_null() {
    return None;
  }

  // Safety: Pointers provided by android_activity are assumed valid for the app lifetime
  let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _).ok()? };
  let mut env = vm.attach_current_thread().ok()?;
  let activity = unsafe { JObject::from_raw(activity_ptr as *mut _) };

  match operation(&mut env, activity) {
    Ok(result) => Some(result),
    Err(e) => {
      // CRITICAL: Print the Java exception stack trace to Logcat
      let _ = env.exception_describe();
      let _ = env.exception_clear();
      println!("❌ Rust: JNI Call Failed: {:?}", e);
      None
    }
  }
}
