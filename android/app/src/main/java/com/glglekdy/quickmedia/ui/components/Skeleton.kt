package com.glglekdy.quickmedia.ui.components

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.dp
import com.glglekdy.quickmedia.ui.theme.Qm

/** 불러오는 중 자리. 은은하게 밝아졌다 어두워진다. */
@Composable
fun Skeleton(modifier: Modifier, shape: Shape = RoundedCornerShape(12.dp)) {
    val pulse by rememberInfiniteTransition(label = "skeleton").animateFloat(
        initialValue = 0.55f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(800), RepeatMode.Reverse),
        label = "skeleton",
    )
    Box(modifier.graphicsLayer { alpha = pulse }.clip(shape).background(Qm.colors.field))
}
