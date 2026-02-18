package com.mikko.xplatform;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.os.IBinder;
import android.util.Log;

public class TrayService extends Service {
  private static final String TAG = "TrayService";
  private static final int NOTIFICATION_ID = 1337;
  private static final String CHANNEL_ID = "xplatform_tray_v6";

  // The Hard Lock: Rust will check this before updating
  public static volatile boolean stopRequested = false;

  @Override
  public void onCreate() {
    super.onCreate();
    NotificationManager manager = getSystemService(NotificationManager.class);
    if (manager != null) {
      NotificationChannel channel = new NotificationChannel(CHANNEL_ID, "xPlatform", NotificationManager.IMPORTANCE_LOW);
      channel.setSound(null, null);
      manager.createNotificationChannel(channel);
    }
  }

  @Override
  public int onStartCommand(Intent intent, int flags, int startId) {
    if (intent != null && "stop".equals(intent.getStringExtra("action"))) {
      stopRequested = true; // Set the lock!
      stopForeground(true);
      stopSelf();
      return START_NOT_STICKY;
    }

    stopRequested = false; // Reset if started normally
    String ramText = (intent != null) ? intent.getStringExtra("ram_text") : "...";
    startForeground(NOTIFICATION_ID, buildNotification(ramText));
    return START_STICKY;
  }

  @Override
  public IBinder onBind(Intent intent) { return null; }

  private Notification buildNotification(String ramText) {
    Intent openIntent = getPackageManager().getLaunchIntentForPackage(getPackageName());
    PendingIntent openPending = PendingIntent.getActivity(this, 0, openIntent, PendingIntent.FLAG_IMMUTABLE);

    Intent stopIntent = new Intent(this, TrayService.class);
    stopIntent.putExtra("action", "stop");
    PendingIntent stopPending = PendingIntent.getService(this, 1, stopIntent, PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);

    return new Notification.Builder(this, CHANNEL_ID)
      .setContentTitle("xPlatform")
      .setContentText(ramText)
      .setSmallIcon(android.R.drawable.ic_dialog_info)
      .setOngoing(true)
      .setContentIntent(openPending)
      .addAction(new Notification.Action.Builder(null, "Stop Service", stopPending).build())
      .build();
  }
}
