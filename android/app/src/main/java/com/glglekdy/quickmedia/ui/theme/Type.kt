package com.glglekdy.quickmedia.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import com.glglekdy.quickmedia.R

// 가변 글꼴 하나에서 굵기마다 따로 꺼낸다
private fun pretendard(weight: Int) = Font(
    R.font.pretendard_variable,
    weight = FontWeight(weight),
    variationSettings = FontVariation.Settings(FontVariation.weight(weight)),
)

val Pretendard = FontFamily(listOf(400, 500, 600, 700, 800).map(::pretendard))

private fun TextStyle.p() = copy(fontFamily = Pretendard)

/** Material 부품(대화상자·시트·스낵바)도 Pretendard 로 */
internal val QmTypography = Typography().run {
    copy(
        displayLarge = displayLarge.p(), displayMedium = displayMedium.p(), displaySmall = displaySmall.p(),
        headlineLarge = headlineLarge.p(), headlineMedium = headlineMedium.p(), headlineSmall = headlineSmall.p(),
        titleLarge = titleLarge.p(), titleMedium = titleMedium.p(), titleSmall = titleSmall.p(),
        bodyLarge = bodyLarge.p(), bodyMedium = bodyMedium.p(), bodySmall = bodySmall.p(),
        labelLarge = labelLarge.p(), labelMedium = labelMedium.p(), labelSmall = labelSmall.p(),
    )
}
