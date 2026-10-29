package com.glglekdy.quickmedia.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.ui.theme.Qm

/** 칸 몇 개짜리 전환 버튼. 고른 칸만 밝아진다. */
@Composable
fun <T> SegmentToggle(
    values: List<T>,
    selected: T,
    label: (T) -> String,
    onSelect: (T) -> Unit,
    modifier: Modifier = Modifier,
) {
    val c = Qm.colors
    Row(modifier.fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(c.field).padding(4.dp)) {
        values.forEach { value ->
            val on = value == selected
            Box(
                Modifier
                    .weight(1f)
                    .then(if (on && !c.isDark) Modifier.shadow(2.dp, RoundedCornerShape(10.dp)) else Modifier)
                    .clip(RoundedCornerShape(10.dp))
                    .background(if (on) c.fieldOn else Color.Transparent)
                    .clickable(role = Role.RadioButton) { onSelect(value) }
                    .padding(vertical = 10.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    label(value),
                    color = if (on) c.text else c.textDim,
                    fontSize = 14.sp,
                    fontWeight = if (on) FontWeight.Bold else FontWeight.Medium,
                    maxLines = 1,
                )
            }
        }
    }
}
