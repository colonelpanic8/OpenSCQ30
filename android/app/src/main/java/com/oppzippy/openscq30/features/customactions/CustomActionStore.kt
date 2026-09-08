package com.oppzippy.openscq30.features.customactions

import android.content.Context
import androidx.core.content.edit
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

@Singleton
class CustomActionStore @Inject constructor(@ApplicationContext context: Context) {
    private val preferences = context.getSharedPreferences("custom_actions", Context.MODE_PRIVATE)
    private val mutableConfig = MutableStateFlow(
        CustomActionConfig(
            preferences.getBoolean("enabled", false),
            preferences.getString("uri", "") ?: "",
            preferences.getString("package", "") ?: "",
        ),
    )
    val config = mutableConfig.asStateFlow()

    fun save(config: CustomActionConfig) {
        require(!config.enabled || config.isValid())
        preferences.edit {
            putBoolean("enabled", config.enabled)
            putString("uri", config.uri)
            putString("package", config.packageName)
        }
        mutableConfig.value = config
    }
}
