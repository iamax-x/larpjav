# Keep JNI native methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep larp.jv classes
-keep class com.larp.jv.** { *; }

# Keep CallbackBridge for JNI
-keep class com.larp.jv.CallbackBridge { *; }

# Keep NativeWindowHelper for JNI
-keep class com.larp.jv.NativeWindowHelper { *; }

# Keep LarpJvActivity
-keep class com.larp.jv.LarpJvActivity { *; }

# Keep LarpJvApplication
-keep class com.larp.jv.LarpJvApplication { *; }

# Keep services
-keep class com.larp.jv.DownloadService { *; }
-keep class com.larp.jv.GameService { *; }

# Keepkotlin metadata
-keep class kotlin.Metadata { *; }

# Keep OkHttp/Retrofit if used
-keep class okhttp3.** { *; }
-keep interface okhttp3.** { *; }
-dontwarn okhttp3.**

# Keep kotlinx.coroutines
-keep class kotlinx.coroutines.** { *; }

# Keep kotlinx.serialization
-keep class kotlinx.serialization.** { *; }

# Keep Room if used
-keep class androidx.room.** { *; }

# Keep WorkManager
-keep class androidx.work.** { *; }

# Keep coroutines
-keepclassmembers class * {
    @kotlinx.coroutines.* *;
}

# Suppress warnings for unused code
-dontwarn java.lang.invoke.*
-dontwarn java.lang.module.*