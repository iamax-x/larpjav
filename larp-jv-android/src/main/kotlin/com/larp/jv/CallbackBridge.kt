package com.larp.jv

import android.app.Activity
import android.app.Application
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.os.Vibrator
import android.util.Log
import android.view.Surface
import android.widget.Toast
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

/**
 * JNI Bridge: Kotlin ↔ Rust
 * 
 * This class provides the Kotlin side of the JNI bridge.
 * The Rust side is in src/platform/callback.rs with JNI exports:
 * - Java_com_larp_jv_CallbackBridge_nativeInit
 * - Java_com_larp_jv_CallbackBridge_nativeGetClipboardText
 * - Java_com_larp_jv_CallbackBridge_nativeSetClipboardText
 * - Java_com_larp_jv_CallbackBridge_nativeVibrate
 * - Java_com_larp_jv_CallbackBridge_nativeRequestPermission
 * - Java_com_larp_jv_CallbackBridge_nativeCheckPermission
 * - Java_com_larp_jv_CallbackBridge_nativeOpenFilePicker
 * - Java_com_larp_jv_CallbackBridge_nativeOpenUrl
 * - Java_com_larp_jv_CallbackBridge_nativeShareText
 * - Java_com_larp_jv_CallbackBridge_nativeShowToast
 * - Java_com_larp_jv_CallbackBridge_nativeGetDeviceInfo
 * - Java_com_larp_jv_CallbackBridge_nativeGetMemoryInfo
 * - Java_com_larp_jv_CallbackBridge_nativeGetAssetPath
 * - Java_com_larp_jv_CallbackBridge_nativeGetFilesDir
 * - Java_com_larp_jv_CallbackBridge_nativeGetCacheDir
 * - Java_com_larp_jv_CallbackBridge_nativeGetExternalFilesDir
 * - Java_com_larp_jv_CallbackBridge_nativeRequestInstallPermission
 * - Java_com_larp_jv_CallbackBridge_nativeGetNativeWindow
 * - Java_com_larp_jv_CallbackBridge_nativeGetDisplayMetrics
 * - Java_com_larp_jv_CallbackBridge_nativeCreateNotificationChannel
 * - Java_com_larp_jv_CallbackBridge_nativeShowNotification
 * - Java_com_larp_jv_CallbackBridge_nativeCancelNotification
 */
class CallbackBridge private constructor(private val activity: Activity) {

    companion object {
        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeInit(bridgePtr: Long)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetClipboardText(bridgePtr: Long): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeSetClipboardText(bridgePtr: Long, text: String)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeVibrate(bridgePtr: Long, durationMs: Long)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeRequestPermission(bridgePtr: Long, permission: String): Boolean

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeCheckPermission(bridgePtr: Long, permission: String): Boolean

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeOpenFilePicker(bridgePtr: Long, mimeType: String, allowMultiple: Boolean): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeOpenUrl(bridgePtr: Long, url: String)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeShareText(bridgePtr: Long, text: String, subject: String)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeShowToast(bridgePtr: Long, message: String)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetDeviceInfo(bridgePtr: Long): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetMemoryInfo(bridgePtr: Long): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetAssetPath(bridgePtr: Long, assetName: String): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetFilesDir(bridgePtr: Long): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetCacheDir(bridgePtr: Long): String

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetExternalFilesDir(bridgePtr: Long): String?

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeRequestInstallPermission(bridgePtr: Long): Boolean

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetNativeWindow(bridgePtr: Long, surface: Surface): Long

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeGetDisplayMetrics(bridgePtr: Long): IntArray

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeCreateNotificationChannel(bridgePtr: Long, channelId: String, channelName: String, importance: Int)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeShowNotification(bridgePtr: Long, channelId: String, title: String, message: String, progress: Int)

        @Suppress("UNUSED_PARAMETER")
        @JvmStatic
        external fun nativeCancelNotification(bridgePtr: Long, notificationId: Int)
    }

    private val bridgePtr: Long

    init {
        bridgePtr = System.identityHashCode(this).toLong()
        nativeInit(bridgePtr)
    }

    // =========================================================================
    // Public API called from Rust
    // =========================================================================

    fun getClipboardText(): String = nativeGetClipboardText(bridgePtr)

    fun setClipboardText(text: String) = nativeSetClipboardText(bridgePtr, text)

    fun vibrate(durationMs: Long) = nativeVibrate(bridgePtr, durationMs)

    fun requestPermission(permission: String): Boolean = nativeRequestPermission(bridgePtr, permission)

    fun checkPermission(permission: String): Boolean = nativeCheckPermission(bridgePtr, permission)

    fun openFilePicker(mimeType: String, allowMultiple: Boolean): String = nativeOpenFilePicker(bridgePtr, mimeType, allowMultiple)

    fun openUrl(url: String) = nativeOpenUrl(bridgePtr, url)

    fun shareText(text: String, subject: String) = nativeShareText(bridgePtr, text, subject)

    fun showToast(message: String) = nativeShowToast(bridgePtr, message)

    fun getDeviceInfo(): String = nativeGetDeviceInfo(bridgePtr)

    fun getMemoryInfo(): String = nativeGetMemoryInfo(bridgePtr)

    fun getAssetPath(assetName: String): String = nativeGetAssetPath(bridgePtr, assetName)

    fun getFilesDir(): String = nativeGetFilesDir(bridgePtr)

    fun getCacheDir(): String = nativeGetCacheDir(bridgePtr)

    fun getExternalFilesDir(): String? = nativeGetExternalFilesDir(bridgePtr)

    fun requestInstallPermission(): Boolean = nativeRequestInstallPermission(bridgePtr)

    fun getNativeWindow(surface: Surface): Long = nativeGetNativeWindow(bridgePtr, surface)

    fun getDisplayMetrics(): IntArray = nativeGetDisplayMetrics(bridgePtr)

    fun createNotificationChannel(channelId: String, channelName: String, importance: Int) = nativeCreateNotificationChannel(bridgePtr, channelId, channelName, importance)

    fun showNotification(channelId: String, title: String, message: String, progress: Int) = nativeShowNotification(bridgePtr, channelId, title, message, progress)

    fun cancelNotification(notificationId: Int) = nativeCancelNotification(bridgePtr, notificationId)

    // =========================================================================
    // Kotlin implementations called from Rust via JNI
    // =========================================================================

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun clipboardGetText(): String {
        val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val primaryClip = clipboard.primaryClip
        return primaryClip?.getItemAt(0)?.text?.toString() ?: ""
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun clipboardSetText(text: String) {
        val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clip = ClipData.newPlainText("larp.jv clipboard", text)
        clipboard.primaryClip = clip
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun vibrate(durationMs: Long) {
        val vibrator = activity.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(android.os.VibrationEffect.createOneShot(durationMs, android.os.VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(durationMs)
        }
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun requestPermission(permission: String): Boolean {
        if (ContextCompat.checkSelfPermission(activity, permission) == PackageManager.PERMISSION_GRANTED) {
            return true
        }
        
        // In a real implementation, you'd use ActivityCompat.requestPermissions
        // and handle the result in onRequestPermissionsResult
        // For now, we just return the current state
        return ContextCompat.checkSelfPermission(activity, permission) == PackageManager.PERMISSION_GRANTED
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun checkPermission(permission: String): Boolean {
        return ContextCompat.checkSelfPermission(activity, permission) == PackageManager.PERMISSION_GRANTED
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun openFilePicker(mimeType: String, allowMultiple: Boolean): String {
        // This would launch a file picker intent and return the result
        // For now, return empty string - real impl needs ActivityResultContracts
        return ""
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun openUrl(url: String) {
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        activity.startActivity(intent)
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun shareText(text: String, subject: String) {
        val intent = Intent(Intent.ACTION_SEND)
        intent.type = "text/plain"
        intent.putExtra(Intent.EXTRA_SUBJECT, subject)
        intent.putExtra(Intent.EXTRA_TEXT, text)
        intent.flags = Intent.FLAG_ACTIVITY_NEW_TASK
        activity.startActivity(Intent.createChooser(intent, "Share via"))
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun showToast(message: String) {
        activity.runOnUiThread {
            Toast.makeText(activity, message, Toast.LENGTH_SHORT).show()
        }
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getDeviceInfo(): String {
        return """
            Model: ${Build.MODEL}
            Manufacturer: ${Build.MANUFACTURER}
            Android Version: ${Build.VERSION.RELEASE}
            SDK: ${Build.VERSION.SDK_INT}
            Device: ${Build.DEVICE}
            Product: ${Build.PRODUCT}
            Brand: ${Build.BRAND}
            Hardware: ${Build.HARDWARE}
            ABI: ${Build.SUPPORTED_ABIS.joinToString(",")}
        """.trimIndent()
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getMemoryInfo(): String {
        val memInfo = android.app.ActivityManager.MemoryInfo()
        val am = activity.getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager
        am.getMemoryInfo(memInfo)
        return """
            Available: ${memInfo.availMem / (1024 * 1024)} MB
            Total: ${memInfo.totalMem / (1024 * 1024)} MB
            Low Memory: ${memInfo.lowMemory}
            Threshold: ${memInfo.threshold / (1024 * 1024)} MB
        """.trimIndent()
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getAssetPath(assetName: String): String {
        return "file:///android_asset/$assetName"
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getFilesDir(): String {
        return activity.filesDir.absolutePath
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getCacheDir(): String {
        return activity.cacheDir.absolutePath
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getExternalFilesDir(): String? {
        return activity.getExternalFilesDir(null)?.absolutePath
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun isExternalStorageAvailable(): Boolean {
        return Environment.getExternalStorageState() == Environment.MEDIA_MOUNTED
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun requestInstallPermission(): Boolean {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val canInstall = activity.packageManager.canRequestPackageInstalls()
            if (!canInstall) {
                val intent = Intent(android.provider.Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES)
                intent.data = Uri.parse("package:${activity.packageName}")
                intent.flags = Intent.FLAG_ACTIVITY_NEW_TASK
                activity.startActivity(intent)
            }
            return canInstall
        }
        return true
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getNativeWindow(surface: Surface): Long {
        // Get ANativeWindow pointer from Surface
        // This requires API 26+ (Android 8.0)
        if (Build.VERSION.SDK_INT >= 26) {
            return Surface.nativeGetFromSurface(surface)
        }
        return 0
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getDisplayMetrics(): IntArray {
        val metrics = android.util.DisplayMetrics()
        activity.windowManager.defaultDisplay.getMetrics(metrics)
        return intArrayOf(metrics.widthPixels, metrics.heightPixels, metrics.densityDpi)
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun createNotificationChannel(channelId: String, channelName: String, importance: Int) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = android.app.NotificationChannel(channelId, channelName, importance)
            val manager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as android.app.NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun showNotification(channelId: String, title: String, message: String, progress: Int) {
        val builder = android.app.NotificationCompat.Builder(activity, channelId)
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentTitle(title)
            .setContentText(message)
            .setPriority(android.app.NotificationCompat.PRIORITY_DEFAULT)
        
        if (progress >= 0) {
            builder.setProgress(100, progress, false)
        }
        
        val manager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as android.app.NotificationManager
        manager.notify(channelId.hashCode(), builder.build())
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun cancelNotification(notificationId: Int) {
        val manager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as android.app.NotificationManager
        manager.cancel(notificationId)
    }
}

/**
 * Main Application Class
 */
class LarpJvApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // Initialize native libraries
        System.loadLibrary("larp_jv_android")
    }
}

/**
 * Main Activity Class
 */
class LarpJvActivity : android.app.NativeActivity() {
    companion object {
        init {
            System.loadLibrary("larp_jv_android")
        }
    }

    private lateinit var callbackBridge: CallbackBridge

    override fun onCreate(savedInstanceState: android.os.Bundle?) {
        super.onCreate(savedInstanceState)
        callbackBridge = CallbackBridge(this)
        
        // Request permissions
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            requestPermissions(
                arrayOf(
                    android.Manifest.permission.POST_NOTIFICATIONS,
                    android.Manifest.permission.READ_MEDIA_IMAGES,
                    android.Manifest.permission.READ_MEDIA_VIDEO,
                    android.Manifest.permission.READ_MEDIA_AUDIO
                ),
                100
            )
        } else {
            requestPermissions(
                arrayOf(
                    android.Manifest.permission.WRITE_EXTERNAL_STORAGE,
                    android.Manifest.permission.READ_EXTERNAL_STORAGE
                ),
                100
            )
        }
    }

    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == 100) {
            for ((permission, result) in permissions.zip(grantResults)) {
                Log.d("LarpJv", "Permission $permission: ${if (result == PackageManager.PERMISSION_GRANTED) "granted" else "denied"}")
            }
        }
    }
}