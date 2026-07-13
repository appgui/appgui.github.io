package com.mikko.xplatform;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.os.IBinder;

public class TrayService extends Service {
  private static final int NOTIFICATION_ID = 1337;
  private static final String CHANNEL_ID = "xplatform_tray_v6";

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
      stopForeground(STOP_FOREGROUND_REMOVE);
      stopSelf();
      android.os.Process.killProcess(android.os.Process.myPid());
      return START_NOT_STICKY;
    }

    stopRequested = false;
    String text = (intent != null) ? intent.getStringExtra("ram_text") : "...";
    startForeground(NOTIFICATION_ID, buildNotification(text));
    return START_STICKY;
  }

  @Override
  public IBinder onBind(Intent intent) { return null; }

  private Notification buildNotification(String text) {
    Intent toggleIntent = new Intent("com.mikko.xplatform.TOGGLE");
    toggleIntent.setClassName(getPackageName(), getPackageName() + ".MainActivity");
    toggleIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_SINGLE_TOP);
    PendingIntent togglePending = PendingIntent.getActivity(this, 0, toggleIntent,
        PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);

    Intent stopIntent = new Intent(this, TrayService.class);
    stopIntent.putExtra("action", "stop");
    PendingIntent stopPending = PendingIntent.getService(this, 1, stopIntent,
        PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);

    return new Notification.Builder(this, CHANNEL_ID)
      .setContentTitle("xPlatform")
      .setContentText(text)
      .setSmallIcon(android.R.drawable.ic_dialog_info)
      .setOngoing(true)
      .setContentIntent(togglePending)
      .addAction(new Notification.Action.Builder(null, "Quit", stopPending).build())
      .build();
  }
}


