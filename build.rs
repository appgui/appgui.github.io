fn main() {
  #[cfg(target_os = "windows")]
  if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().unwrap();
  }

  eprintln!("DEBUG: ANDROID_HOME={:?}", std::env::var("ANDROID_HOME"));
  eprintln!(
    "DEBUG: ANDROID_SDK_ROOT={:?}",
    std::env::var("ANDROID_SDK_ROOT")
  );

  let path = std::env::var("ANDROID_HOME").unwrap() + "/platforms";
  eprintln!("DEBUG: contents of {:?}", path);
  if let Ok(entries) = std::fs::read_dir(&path) {
    for entry in entries.flatten() {
      eprintln!("  found: {:?}", entry.path());
    }
  } else {
    eprintln!("  (directory missing)");
  }

  slint_build::compile("ui/app_window.slint").unwrap();
}
