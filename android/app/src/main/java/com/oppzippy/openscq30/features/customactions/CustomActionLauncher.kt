package com.oppzippy.openscq30.features.customactions

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.os.SystemClock
import android.provider.Settings
import android.util.Log
import androidx.core.net.toUri
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CustomActionLauncher @Inject constructor(
    @ApplicationContext private val context: Context,
    private val store: CustomActionStore,
) {
    private val gate = ActionGate(SystemClock::elapsedRealtime)

    fun onDeviceEvent(event: String) {
        val config = store.config.value
        Log.i(
            "CustomActions",
            "Device event: $event; selected trigger: ${config.trigger.event}; enabled: ${config.enabled}",
        )
        if (!gate.accept(event, config)) return
        if (!Settings.canDrawOverlays(context)) {
            Log.w("CustomActions", "Custom action requires background launch permission")
            return
        }
        launch(config)
    }

    fun launch(config: CustomActionConfig): Boolean {
        if (!config.isValid()) return false
        val intent = Intent(Intent.ACTION_VIEW, config.uri.toUri()).apply {
            addCategory(Intent.CATEGORY_BROWSABLE)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            if (config.packageName.isNotEmpty()) setPackage(config.packageName)
        }
        return try {
            context.startActivity(intent)
            Log.i("CustomActions", "Requested custom action launch")
            true
        } catch (_: ActivityNotFoundException) {
            Log.w("CustomActions", "No app handles the configured action")
            false
        } catch (_: SecurityException) {
            Log.w("CustomActions", "Android denied the configured action")
            false
        }
    }
}
