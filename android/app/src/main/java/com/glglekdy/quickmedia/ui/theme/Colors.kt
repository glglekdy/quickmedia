package com.glglekdy.quickmedia.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color

/** 앱 전용 색. 값은 설계 문서의 "색" 표와 같다. */
@Immutable
data class QmColors(
    val bg: Color,
    val surface: Color,
    val field: Color,
    val fieldOn: Color,
    val text: Color,
    val textDim: Color,
    val accent: Color,
    val accentText: Color,
    val ok: Color,
    val bad: Color,
    val dock: Color,
    val isDark: Boolean,
)

val DarkColors = QmColors(
    bg = Color(0xFF0C0C0E),
    surface = Color(0xFF17171A),
    field = Color(0xFF1B1B1F),
    fieldOn = Color(0xFF2A2A30),
    text = Color(0xFFF2F2F2),
    textDim = Color(0xFF8B8B93),
    accent = Color(0xFFFF3B3B),
    accentText = Color(0xFFFF5A5A),
    ok = Color(0xFF4ADE80),
    bad = Color(0xFFFF6B6B),
    dock = Color(0xFF28282E).copy(alpha = 0.62f),
    isDark = true,
)

val LightColors = QmColors(
    bg = Color(0xFFF6F6F8),
    surface = Color(0xFFFFFFFF),
    field = Color(0xFFEBEBEF),
    fieldOn = Color(0xFFFFFFFF),
    text = Color(0xFF141416),
    textDim = Color(0xFF7A7A82),
    accent = Color(0xFFFF3B3B),
    accentText = Color(0xFFE5262B),
    ok = Color(0xFF16A34A),
    bad = Color(0xFFDC2626),
    dock = Color(0xFFFFFFFF).copy(alpha = 0.6f),
    isDark = false,
)

/** 다운로드 버튼과 진행 막대 */
val AccentGradient = Brush.horizontalGradient(listOf(Color(0xFFFF3B3B), Color(0xFFFF6A3B)))

internal val LocalQmColors = staticCompositionLocalOf { DarkColors }
