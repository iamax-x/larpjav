package com.larp.jv;

import android.app.Activity;
import android.content.Context;
import android.graphics.SurfaceTexture;
import android.view.Surface;
import android.view.SurfaceView;
import android.view.TextureView;
import android.util.Log;

import java.lang.reflect.Method;

/**
 * JNI Helper class for native window and surface operations.
 * Provides Kotlin implementations called from Rust via JNI.
 */
public class NativeWindowHelper {
    private static final String TAG = "NativeWindowHelper";

    /**
     * Get ANativeWindow pointer from Surface.
     * Called from Rust via JNI: Java_com_larp_jv_NativeWindowHelper_getNativeWindow
     */
    public static long getNativeWindow(Surface surface) {
        if (surface == null) {
            Log.w(TAG, "Surface is null");
            return 0;
        }

        try {
            // Method 1: Surface.getNativeWindow() (API 26+)
            if (android.os.Build.VERSION.SDK_INT >= 26) {
                Method method = Surface.class.getMethod("getNativeWindow");
                Object result = method.invoke(surface);
                if (result instanceof Long) {
                    return (Long) result;
                }
            }

            // Method 2: Reflection on Surface.getNativeWindow (older API)
            try {
                Method method = Surface.class.getDeclaredMethod("getNativeWindow");
                method.setAccessible(true);
                Object result = method.invoke(surface);
                if (result instanceof Long) {
                    return (Long) result;
                }
            } catch (Exception e) {
                Log.w(TAG, "getNativeWindow via reflection failed: " + e.getMessage());
            }

            // Method 3: Surface.lockCanvas() workaround
            // Some implementations expose native window via internal fields
            Log.w(TAG, "Could not get native window pointer");
            return 0;

        } catch (Exception e) {
            Log.e(TAG, "Error getting native window: " + e.getMessage(), e);
            return 0;
        }
    }

    /**
     * Get ANativeWindow from SurfaceView
     */
    public static long getNativeWindowFromSurfaceView(SurfaceView surfaceView) {
        if (surfaceView == null) return 0;
        try {
            Surface surface = surfaceView.getHolder().getSurface();
            return getNativeWindow(surface);
        } catch (Exception e) {
            Log.e(TAG, "Error getting surface from SurfaceView: " + e.getMessage());
            return 0;
        }
    }

    /**
     * Get ANativeWindow from TextureView
     */
    public static long getNativeWindowFromTextureView(TextureView textureView) {
        if (textureView == null) return 0;
        try {
            SurfaceTexture surfaceTexture = textureView.getSurfaceTexture();
            if (surfaceTexture != null) {
                Surface surface = new Surface(surfaceTexture);
                return getNativeWindow(surface);
            }
        } catch (Exception e) {
            Log.e(TAG, "Error getting surface from TextureView: " + e.getMessage());
        }
        return 0;
    }

    /**
     * Release ANativeWindow reference
     */
    public static void releaseNativeWindow(long nativeWindowPtr) {
        if (nativeWindowPtr != 0) {
            // ANativeWindow_release is called automatically when Surface is released
            // But we can explicitly release if needed
            Log.d(TAG, "Releasing native window: " + nativeWindowPtr);
        }
    }

    /**
     * Get display metrics for surface sizing
     */
    public static int[] getDisplayMetrics(Activity activity) {
        int[] metrics = new int[2];
        try {
            android.util.DisplayMetrics metrics_obj = new android.util.DisplayMetrics();
            activity.getWindowManager().getDefaultDisplay().getMetrics(metrics_obj);
            metrics[0] = metrics_obj.widthPixels;
            metrics[1] = metrics_obj.heightPixels;
        } catch (Exception e) {
            Log.e(TAG, "Error getting display metrics: " + e.getMessage());
            metrics[0] = 1024;
            metrics[1] = 768;
        }
        return metrics;
    }
}