package com.mikko.xplatform;

import android.app.ActivityManager;
import android.app.NativeActivity;
import android.content.Context;
import android.content.Intent;
import java.util.List;

public class MainActivity extends NativeActivity {
  // Set true in onStop, cleared in onResume (not onStart).
  // For a singleTask activity restored from background the lifecycle order is:
  //   onStart → onNewIntent → onResume
  // so this flag is still true when onNewIntent fires in the show-from-background case,
  // whereas wasStopped cleared in onStart would already be false at that point.
  private boolean comingFromBackground = false;

  // Set in onNewIntent hide path; consumed in onStop to exclude the task from recents
  // after the task is fully backgrounded (setExcludeFromRecents must be called after
  // moveTaskToBack, not before).
  private boolean pendingHide = false;

  @Override
  protected void onStop() {
    super.onStop();
    comingFromBackground = true;
    if (pendingHide) {
      pendingHide = false;
      setTaskExcludeFromRecents(true);
    }
  }

  @Override
  protected void onResume() {
    super.onResume();
    comingFromBackground = false;
    setTaskExcludeFromRecents(false);
  }

  @Override
  protected void onNewIntent(Intent intent) {
    super.onNewIntent(intent);
    if (!"com.mikko.xplatform.TOGGLE".equals(intent.getAction())) return;

    if (!comingFromBackground) {
      // Activity is still in foreground or only paused (not stopped) → hide
      pendingHide = true;
      moveTaskToBack(true);
    }
    // comingFromBackground == true: was stopped, Android is bringing to front; onResume re-includes in recents.
  }

  private void setTaskExcludeFromRecents(boolean exclude) {
    ActivityManager am = (ActivityManager) getSystemService(Context.ACTIVITY_SERVICE);
    List<ActivityManager.AppTask> tasks = am.getAppTasks();
    if (tasks != null) {
      for (ActivityManager.AppTask task : tasks) {
        task.setExcludeFromRecents(exclude);
      }
    }
  }
}
