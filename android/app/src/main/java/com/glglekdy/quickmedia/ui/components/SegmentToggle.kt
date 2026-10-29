package com.glglekdy.quickmedia.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.ui.theme.Motion
import com.glglekdy.quickmedia.ui.theme.Qm

private val Thumb = RoundedCornerShape(10.dp)

/** 칸 몇 개짜리 전환 버튼. 고른 칸 뒤의 밝은 바탕이 옆으로 미끄러진다. */
@Composable
fun <T> SegmentToggle(
    values: List<T>,
    selected: T,
    label: (T) -> String,
    onSelect: (T) -> Unit,
    modifier: Modifier = Modifier,
) {
    val c = Qm.colors
    BoxWithConstraints(modifier.fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(c.field).padding(4.dp)) {
        val cell = maxWidth / values.size
        val x by animateDpAsState(cell * values.indexOf(selected).coerceAtLeast(0), Motion.slide(), label = "segment")
        // Row 높이에 맞추려고 matchParentSize 로 깐다
        Box(Modifier.matchParentSize()) {
            Box(
                Modifier
                    .offset(x = x)
                    .width(cell)
                    .fillMaxHeight()
                    .then(if (!c.isDark) Modifier.shadow(2.dp, Thumb) else Modifier)
                    .clip(Thumb)
                    .background(c.fieldOn),
            )
        }
        Row {
            values.forEach { value ->
                val on = value == selected
                val color by animateColorAsState(if (on) c.text else c.textDim, Motion.fade(), label = "segmentText")
                Box(
                    Modifier
                        .weight(1f)
                        .clip(Thumb)
                        .clickable(role = Role.RadioButton) { onSelect(value) }
                        .padding(vertical = 10.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        label(value),
                        color = color,
                        fontSize = 14.sp,
                        fontWeight = if (on) FontWeight.Bold else FontWeight.Medium,
                        maxLines = 1,
                    )
                }
            }
        }
    }
}
