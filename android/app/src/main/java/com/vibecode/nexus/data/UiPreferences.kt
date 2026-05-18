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

    // DANIEL-FUNKTIONAL DA-001: Filter-Persistenz für Hybrid-Pills im
    // Sparks-Tab. `null`/empty = „Alle" für die jeweilige Reihe.
    var sparkTypeFilter: String?
        get() = prefs.getString(KEY_SPARK_TYPE, null)?.takeIf { it.isNotBlank() }
        set(value) = prefs.edit().run {
            if (value.isNullOrBlank()) remove(KEY_SPARK_TYPE) else putString(KEY_SPARK_TYPE, value)
            apply()
        }

    var sparkLifeFilter: String?
        get() = prefs.getString(KEY_SPARK_LIFE, null)?.takeIf { it.isNotBlank() }
        set(value) = prefs.edit().run {
            if (value.isNullOrBlank()) remove(KEY_SPARK_LIFE) else putString(KEY_SPARK_LIFE, value)
            apply()
        }

    private companion object {
        const val PREFS_NAME = "nexus_ui"
        const val KEY_THEME = "theme_mode"
        const val KEY_SPARK_TYPE = "spark_type_filter"
        const val KEY_SPARK_LIFE = "spark_life_filter"
    }
}
