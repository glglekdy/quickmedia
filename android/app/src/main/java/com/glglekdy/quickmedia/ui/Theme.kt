package com.glglekdy.quickmedia.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// 데스크톱 ytdl/theme.py 의 빨강과 회색. Material 기본값(보라 기운)은 쓰지 않는다.
private val Light = lightColorScheme(
    primary = Color(0xFFDC2626),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFFEE2E2),
    onPrimaryContainer = Color(0xFF7F1D1D),
    secondaryContainer = Color(0xFFFEE2E2),
    onSecondaryContainer = Color(0xFF7F1D1D),
    background = Color.White,
    surface = Color.White,
    onSurface = Color(0xFF1F1F1F),
    surfaceVariant = Color(0xFFF0F0F0),
    onSurfaceVariant = Color(0xFF737373),
    surfaceContainerLowest = Color.White,
    surfaceContainerLow = Color(0xFFF7F7F7),
    surfaceContainer = Color(0xFFF3F3F3),
    surfaceContainerHigh = Color(0xFFEDEDED),
    surfaceContainerHighest = Color(0xFFE8E8E8),
    outline = Color(0xFFD4D4D4),
    outlineVariant = Color(0xFFE5E5E5),
    error = Color(0xFFB91C1C),
)

private val Dark = darkColorScheme(
    primary = Color(0xFFF87171),
    onPrimary = Color(0xFF450A0A),
    primaryContainer = Color(0xFF7F1D1D),
    onPrimaryContainer = Color(0xFFFECACA),
    secondaryContainer = Color(0xFF7F1D1D),
    onSecondaryContainer = Color(0xFFFECACA),
    background = Color(0xFF141414),
    surface = Color(0xFF141414),
    onSurface = Color(0xFFEDEDED),
    surfaceVariant = Color(0xFF262626),
    onSurfaceVariant = Color(0xFFA3A3A3),
    surfaceContainerLowest = Color(0xFF0F0F0F),
    surfaceContainerLow = Color(0xFF1A1A1A),
    surfaceContainer = Color(0xFF1F1F1F),
    surfaceContainerHigh = Color(0xFF262626),
    surfaceContainerHighest = Color(0xFF2E2E2E),
    outline = Color(0xFF525252),
    outlineVariant = Color(0xFF3A3A3A),
    error = Color(0xFFF87171),
)

@Composable
fun QuickMediaTheme(content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = if (isSystemInDarkTheme()) Dark else Light, content = content)
}
