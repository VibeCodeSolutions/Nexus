package com.vibecode.nexus.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

enum class ThemeMode { LIGHT, DARK, SYSTEM }

/**
 * NEXUS-spezifische Akzentfarben außerhalb des Material3-ColorSchemes,
 * gemäß docs/UI_SPEC.md §2 (--warning, --success, Priority-Codes).
 */
object NexusAccent {
    val Warning = Color(0xFFF59E0B)
    val WarningDim = Color(0xFFD97706)
    val Success = Color(0xFF4CAF50)
    val PriorityHigh = Color(0xFFEF5350)    // --danger
    val PriorityMedium = Color(0xFFF59E0B)  // --warning
    val PriorityLow = Color(0xFF4CAF50)     // --success
}

private val NexusLightColorScheme = lightColorScheme(
    primary = Color(0xFF3D5AFE),
    onPrimary = Color(0xFFFFFFFF),
    primaryContainer = Color(0xFFE0E5FF),
    onPrimaryContainer = Color(0xFF001257),
    secondary = Color(0xFF00897B),
    onSecondary = Color(0xFFFFFFFF),
    tertiary = Color(0xFFEF6C00),
    background = Color(0xFFF7F8FB),
    onBackground = Color(0xFF0E1116),
    surface = Color(0xFFFFFFFF),
    onSurface = Color(0xFF0E1116),
    surfaceVariant = Color(0xFFF0F2F7),
    onSurfaceVariant = Color(0xFF5A6172),
    outline = Color(0xFFE1E5EE),
    error = Color(0xFFC62828),
)

// Dark-Werte gemäß docs/UI_SPEC.md §2 (Source of Truth).
private val NexusDarkColorScheme = darkColorScheme(
    primary = Color(0xFF8C9EFF),          // --primary
    onPrimary = Color(0xFF001A66),
    primaryContainer = Color(0xFF263080),
    onPrimaryContainer = Color(0xFFE0E5FF),
    secondary = Color(0xFF4DB6AC),        // --secondary
    onSecondary = Color(0xFF003733),
    tertiary = Color(0xFFFFB74D),
    background = Color(0xFF09090F),       // --bg (near-black)
    onBackground = Color(0xFFE8EAF0),     // --text
    surface = Color(0xFF111318),          // --bg-card
    onSurface = Color(0xFFE8EAF0),        // --text
    surfaceVariant = Color(0xFF181C25),   // --bg-surface (inputs, pill bg)
    onSurfaceVariant = Color(0xFF6B7280), // --text-dim
    outline = Color(0xFF1C2030),          // --border
    error = Color(0xFFEF5350),            // --danger
)

@Composable
fun NexusTheme(
    themeMode: ThemeMode = ThemeMode.SYSTEM,
    content: @Composable () -> Unit
) {
    val isDark = when (themeMode) {
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
        ThemeMode.SYSTEM -> isSystemInDarkTheme()
    }
    val colorScheme = if (isDark) NexusDarkColorScheme else NexusLightColorScheme

    MaterialTheme(
        colorScheme = colorScheme,
        content = content
    )
}
