package com.larp.jv

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Vibrator
import android.util.Log
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
    }

    private val bridgePtr: Long

    init {
        bridgePtr = System.identityHashCode(this).toLong()
        nativeInit(bridgePtr)
    }

    /**
     * Get clipboard text
     */
    fun getClipboardText(): String {
        return nativeGetClipboardText(bridgePtr)
    }

    /**
     * Set clipboard text
     */
    fun setClipboardText(text: String) {
        nativeSetClipboardText(bridgePtr, text)
    }

    /**
     * Vibrate device
     */
    fun vibrate(durationMs: Long) {
        nativeVibrate(bridgePtr, durationMs)
    }

    /**
     * Request runtime permission
     */
    fun requestPermission(permission: String): Boolean {
        return nativeRequestPermission(bridgePtr, permission)
    }

    /**
     * Check if permission is granted
     */
    fun checkPermission(permission: String): Boolean {
        return nativeCheckPermission(bridgePtr, permission)
    }

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
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
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
    fun getNativeWindow(surface: android.view.Surface): Long {
        // Get ANativeWindow pointer from Surface
        // This requires API 26+ (Android 8.0)
        if (android.os.Build.VERSION.SDK_INT >= 26) {
            return android.view.Surface.nativeGetFromSurface(surface)
        }
        return 0
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
        return android.os.Environment.getExternalStorageState() == android.os.Environment.MEDIA_MOUNTED
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun showToast(message: String) {
        activity.runOnUiThread {
            android.widget.Toast.makeText(activity, message, android.widget.Toast.LENGTH_SHORT).show()
        }
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun openUrl(url: String) {
        val intent = android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(url))
        intent.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
        activity.startActivity(intent)
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun shareText(text: String, subject: String) {
        val intent = android.content.Intent(android.content.Intent.ACTION_SEND)
        intent.type = "text/plain"
        intent.putExtra(android.content.Intent.EXTRA_SUBJECT, subject)
        intent.putExtra(android.content.Intent.EXTRA_TEXT, text)
        intent.flags = android.content.Intent.FLAG_ACTIVITY_NEW_TASK
        activity.startActivity(android.content.Intent.createChooser(intent, "Share via"))
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun requestInstallPermission(): Boolean {
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
            val canInstall = activity.packageManager.canRequestPackageInstalls()
            if (!canInstall) {
                val intent = android.content.Intent(android.provider.Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES)
                intent.data = android.net.Uri.parse("package:${activity.packageName}")
                intent.flags = android.content.Intent.FLAG_ACTIVITY_NEW_TASK
                activity.startActivity(intent)
            }
            return canInstall
        }
        return true
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getDeviceInfo(): String {
        return """
            Model: ${android.os.Build.MODEL}
            Manufacturer: ${android.os.Build.MANUFACTURER}
            Android Version: ${android.os.Build.VERSION.RELEASE}
            SDK: ${android.os.Build.VERSION.SDK_INT}
            Device: ${android.os.Build.DEVICE}
            Product: ${android.os.Build.PRODUCT}
        """.trimIndent()
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getAvailableMemoryMB(): Long {
        val memInfo = android.app.ActivityManager.MemoryInfo()
        val am = activity.getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager
        am.getMemoryInfo(memInfo)
        return memInfo.availMem / (1024 * 1024)
    }

    @Suppress("UNUSED_PARAMETER")
    @JvmStatic
    fun getTotalMemoryMB(): Long {
        val memInfo = android.app.ActivityManager.MemoryInfo()
        val am = activity.getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager
        am.getMemoryInfo(memInfo)
        return memInfo.totalMem / (1024 * 1024)
    }
}