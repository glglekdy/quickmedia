package com.glglekdy.quickmedia.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.unit.dp
import com.glglekdy.quickmedia.ui.theme.Qm

/** 카드 바탕. 라이트에서는 아주 옅은 그림자로 바탕과 나눈다. */
@Composable
fun SectionCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val c = Qm.colors
    val shape = RoundedCornerShape(20.dp)
    Column(
        modifier
            .fillMaxWidth()
            .then(if (c.isDark) Modifier else Modifier.shadow(1.dp, shape))
            .clip(shape)
            .background(c.surface),
        content = content,
    )
}
