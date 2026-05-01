package com.vibecode.nexus.data

import android.content.Context
import android.content.SharedPreferences
import com.vibecode.nexus.ui.theme.ThemeMode

class UiPreferences(context: Context) {

    private val prefs: SharedPreferences =
        context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    var themeMode: ThemeMode
        get() = prefs.getString(KEY_THEME, null)
            ?.let { runCatching { ThemeMode.valueOf(it) }.getOrNull() }
            ?: ThemeMode.SYSTEM
        set(value) = prefs.edit().putString(KEY_THEME, value.name).apply()

    private companion object {
        const val PREFS_NAME = "nexus_ui"
        const val KEY_THEME = "theme_mode"
    }
}
