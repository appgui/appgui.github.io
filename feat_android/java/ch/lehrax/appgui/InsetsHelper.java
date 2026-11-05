package ch.lehrax.appgui;

import android.app.Activity;
import android.view.Window;
import android.view.WindowInsetsController;
import android.view.WindowInsets;

public class InsetsHelper {

  public static void hideSystemBars(Activity activity) {
    activity.runOnUiThread(() -> {
      Window window = activity.getWindow();
      WindowInsetsController controller = window.getInsetsController();
      if (controller == null) return;
      int systemBars = WindowInsets.Type.systemBars();
      controller.hide(systemBars);
      controller.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
    });
  }

  public static void showSystemBars(Activity activity) {
    activity.runOnUiThread(() -> {
      Window window = activity.getWindow();
      WindowInsetsController controller = window.getInsetsController();
      if (controller == null) return;
      controller.show(WindowInsets.Type.systemBars());
    });
  }
}
