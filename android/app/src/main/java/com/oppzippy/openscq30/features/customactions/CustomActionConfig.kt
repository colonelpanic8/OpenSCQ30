package com.oppzippy.openscq30.features.customactions

import java.net.URI

internal const val ASSISTANT_EVENT = "assistant-requested"

data class CustomActionConfig(val enabled: Boolean = false, val uri: String = "", val packageName: String = "") {
    fun isValid(): Boolean {
        val parsed = runCatching { URI(uri) }.getOrNull() ?: return false
        val scheme = parsed.scheme?.lowercase() ?: return false
        if (scheme in setOf("intent", "file", "content", "javascript", "data") ||
            parsed.rawSchemeSpecificPart.isNullOrBlank()
        ) {
            return false
        }
        return packageName.isEmpty() || packageName.matches(Regex("[A-Za-z][A-Za-z0-9_]*(\\.[A-Za-z][A-Za-z0-9_]*)+"))
    }
}

internal class ActionGate(private val clock: () -> Long) {
    private var lastLaunch: Long? = null

    fun accept(event: String, config: CustomActionConfig): Boolean {
        if (event != ASSISTANT_EVENT || !config.enabled || !config.isValid()) return false
        val now = clock()
        if (lastLaunch?.let { now - it < 1000 } == true) return false
        lastLaunch = now
        return true
    }
}
