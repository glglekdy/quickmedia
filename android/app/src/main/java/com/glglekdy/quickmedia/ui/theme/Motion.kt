package com.glglekdy.quickmedia.ui.theme

import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.interaction.InteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer

/** 앱 전체 움직임. 모두 0.3초 안에 끝나게 짧게 둔다. 휴대폰에서 애니메이션을 끄면 안드로이드가 알아서 건너뛴다. */
object Motion {
    /** 색·투명도처럼 그냥 바뀌는 것 */
    fun <T> fade(): FiniteAnimationSpec<T> = tween(200)

    /** 자리를 옮기는 것(메뉴 선택 표시, 영상/음성 칸). 살짝 튕기며 멈춘다. */
    fun <T> slide(): FiniteAnimationSpec<T> = spring(dampingRatio = 0.8f, stiffness = Spring.StiffnessMediumLow)

    /** 탭이 바뀔 때 */
    const val TAB_MS = 240
}

/** 누르는 동안 살짝 작아진다. clickable 에 같은 interactionSource 를 넘겨야 한다. */
@Composable
fun Modifier.pressScale(source: InteractionSource, pressed: Float = 0.96f): Modifier {
    val down by source.collectIsPressedAsState()
    val scale by animateFloatAsState(if (down) pressed else 1f, Motion.slide(), label = "press")
    return graphicsLayer {
        scaleX = scale
        scaleY = scale
    }
}
