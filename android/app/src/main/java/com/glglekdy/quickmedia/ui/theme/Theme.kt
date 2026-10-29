package com.glglekdy.quickmedia.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.graphics.Color

object Qm {
    val colors: QmColors
        @Composable @ReadOnlyComposable get() = LocalQmColors.current
}

/** 휴대폰 설정을 따라 다크·라이트. Material 색도 같은 값으로 맞춰 대화상자·시트가 어울린다. */
@Composable
fun QuickMediaTheme(content: @Composable () -> Unit) {
    val c = if (isSystemInDarkTheme()) DarkColors else LightColors
    val base = if (c.isDark) darkColorScheme() else lightColorScheme()
    val scheme = base.copy(
        primary = c.accent,
        onPrimary = Color.White,
        primaryContainer = c.accent.copy(alpha = 0.18f),
        onPrimaryContainer = c.accentText,
        secondaryContainer = c.fieldOn,
        onSecondaryContainer = c.text,
        background = c.bg,
        onBackground = c.text,
        surface = c.surface,
        onSurface = c.text,
        surfaceVariant = c.field,
        onSurfaceVariant = c.textDim,
        surfaceContainerLowest = c.bg,
        surfaceContainerLow = c.surface,
        surfaceContainer = c.surface,
        surfaceContainerHigh = c.surface,
        surfaceContainerHighest = c.field,
        inverseSurface = c.text,
        inverseOnSurface = c.bg,
        outline = c.textDim,
        outlineVariant = c.field,
        error = c.bad,
    )
    CompositionLocalProvider(LocalQmColors provides c) {
        MaterialTheme(colorScheme = scheme, typography = QmTypography) {
            CompositionLocalProvider(LocalTextStyle provides QmTypography.bodyLarge.copy(color = c.text), content = content)
        }
    }
}
