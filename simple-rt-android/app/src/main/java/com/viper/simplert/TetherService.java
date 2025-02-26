/*
 * SimpleRT: Reverse tethering utility for Android
 * Copyright (C) 2016 Konstantin Menyaev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

package com.viper.simplert;

import android.annotation.TargetApi;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.graphics.Color;
import android.hardware.usb.UsbAccessory;
import android.hardware.usb.UsbManager;
import android.net.ConnectivityManager;
import android.net.LinkAddress;
import android.net.LinkProperties;
import android.net.Network;
import android.net.VpnService;
import android.os.Build;
import android.os.ParcelFileDescriptor;
import android.os.PowerManager;
import androidx.annotation.RequiresApi;
import androidx.core.app.NotificationChannelCompat;
import androidx.core.app.NotificationCompat;
import androidx.core.app.NotificationManagerCompat;
import android.util.Log;
import android.widget.Toast;

import java.util.List;

import static androidx.core.app.NotificationCompat.PRIORITY_MIN;

public class TetherService extends VpnService {
  private static final String TAG = "TetherService";
  private static final String ACTION_USB_PERMISSION = "com.viper.simplert.TetherService.action.USB_PERMISSION";
  private static final int FOREGROUND_NOTIFICATION_ID = 16;
  private static final String CHANNEL_ID = "simpleRT";
  private static final String CHANNEL_NAME = "SimpleRT Service";

    @Override
    public void onCreate() {
        super.onCreate();
        PowerManager powerManager = (PowerManager) getSystemService(POWER_SERVICE);
        PowerManager.WakeLock wakeLock = powerManager.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK,
            "MyApp::MyWakelockTag");
        wakeLock.acquire();
    }

    private final BroadcastReceiver mUsbReceiver = new BroadcastReceiver() {
    public void onReceive(Context context, Intent intent) {
      String action = intent.getAction();

      if (UsbManager.ACTION_USB_ACCESSORY_DETACHED.equals(action)) {
        Log.d(TAG,"Accessory detached");

        intent.getParcelableExtra(UsbManager.EXTRA_ACCESSORY);
        Native.stop();
        unregisterReceiver(mUsbReceiver);
      }
    }
  };

  @Override
  public int onStartCommand(final Intent intent, int flags, final int startId) {
    Log.w(TAG, "onStartCommand");

    if (intent == null) {
      Log.i(TAG, "Intent is null");
      return START_NOT_STICKY;
    }

    if (Native.isRunning()) {
      Log.e(TAG, "already running!");
      return START_NOT_STICKY;
    }

    final UsbAccessory accessory = intent.getParcelableExtra(UsbManager.EXTRA_ACCESSORY);

    if (accessory == null) {
      showErrorDialog(getString(R.string.accessory_error));
      stopSelf();
      return START_NOT_STICKY;
    }

    /* default values for compatibility with old simple_rt version */
    int prefixLength = 30;
    String ipAddr = "10.10.10.2";
    String dnsServer = "8.8.8.8";

    /* expected format: address,dns_server */
    String[] tokens = accessory.getSerial().split(",");
    if (tokens.length == 2) {
      ipAddr = tokens[0];
      dnsServer = tokens[1];
      prefixLength = 24;
    }

    Log.d(TAG, "Got accessory: " + accessory.getModel());

    IntentFilter filter = new IntentFilter(ACTION_USB_PERMISSION);
    filter.addAction(UsbManager.ACTION_USB_ACCESSORY_DETACHED);
    registerReceiver(mUsbReceiver, filter);

    Builder builder = new Builder();
    builder.setMtu(1500);
    if (Build.VERSION.SDK_INT >= 21) {
      builder.allowBypass();
    }
    builder.setSession(getString(R.string.app_name));
    builder.addAddress(ipAddr, prefixLength);
    builder.addRoute("0.0.0.0", 0);
    builder.addDnsServer(dnsServer);

    final ParcelFileDescriptor accessoryFd = ((UsbManager) getSystemService(Context.USB_SERVICE)).openAccessory(accessory);
    if (accessoryFd == null) {
      showErrorDialog(getString(R.string.accessory_error));
      stopSelf();
      return START_NOT_STICKY;
    }

    final ParcelFileDescriptor tunFd = builder.establish();
    if (tunFd == null) {
      showErrorDialog(getString(R.string.tun_error));
      stopSelf();
      return START_NOT_STICKY;
    }

    Toast.makeText(this, "SimpleRT Connected!", Toast.LENGTH_SHORT).show();
      Log.i("XXX", "start");
      Native.start(tunFd.detachFd(), accessoryFd.detachFd());
      Log.i("XXX", "end");

    setAsUnderlyingNetwork(ipAddr + "/" + prefixLength);

    startForeground(FOREGROUND_NOTIFICATION_ID, buildNotification());

    return START_NOT_STICKY;
  }

  private Notification buildNotification() {
    NotificationCompat.Builder notificationBuilder = new NotificationCompat.Builder(this, CHANNEL_ID);
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      NotificationChannelCompat channel = new NotificationChannelCompat.Builder(CHANNEL_ID, NotificationManagerCompat.IMPORTANCE_DEFAULT)
              .setName(CHANNEL_NAME)
              .build();
      NotificationManagerCompat manager = NotificationManagerCompat.from(this);
      manager.createNotificationChannel(channel);
      notificationBuilder = notificationBuilder.setPriority(NotificationManagerCompat.IMPORTANCE_DEFAULT);
    }
    return notificationBuilder.setOngoing(true)
            .setContentTitle(getString(R.string.app_name))
            .setContentText(getString(R.string.description_service_running))
            .setSmallIcon(android.R.drawable.ic_secure)
            .setCategory(Notification.CATEGORY_SERVICE)
            .build();
  }

  @Override
  public void onDestroy() {
    NotificationManagerCompat.from(this).cancel(FOREGROUND_NOTIFICATION_ID);
    super.onDestroy();
  }

  private void setAsUnderlyingNetwork(String Address) {
    if (Build.VERSION.SDK_INT >= 22) {
      Network vpnNetwork = findVpnNetwork(Address);
      if (vpnNetwork != null) {
        // so that applications knows that network is available
        setUnderlyingNetworks(new Network[]{vpnNetwork});
        Log.w(TAG, "VPN set as underlying network");
      }
    } else {
      Log.w(TAG, "Cannot set underlying network, API version " + Build.VERSION.SDK_INT + " < 22");
    }
  }

  @TargetApi(22)
  private Network findVpnNetwork(String Address) {
    ConnectivityManager cm = (ConnectivityManager) getSystemService(Context.CONNECTIVITY_SERVICE);
    Network[] networks = cm.getAllNetworks();
    for (Network network : networks) {
      LinkProperties linkProperties = cm.getLinkProperties(network);
      List<LinkAddress> addresses = linkProperties.getLinkAddresses();
      for (LinkAddress addr : addresses) {
        if (addr.toString().equals(Address)) {
          return network;
        }
      }
    }
    return null;
  }

  private void showErrorDialog(String err) {
    Intent activityIntent = new Intent(getApplicationContext(), InfoActivity.class);
    activityIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
    activityIntent.putExtra("text", err);
    startActivity(activityIntent);
  }
}
