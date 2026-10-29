package com.glglekdy.quickmedia.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.ui.theme.Qm

/** 알약 칩. 고르면 빨강, inverse 면 글자색 바탕(목록 필터). */
@Composable
fun Chip(text: String, selected: Boolean, onClick: () -> Unit, inverse: Boolean = false, modifier: Modifier = Modifier) {
    val c = Qm.colors
    val (bg, fg) = when {
        !selected -> c.field to c.textDim
        inverse -> c.text to c.bg
        else -> c.accent to Color.White
    }
    Text(
        text,
        color = fg,
        fontSize = 13.sp,
        fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
        maxLines = 1,
        modifier = modifier
            .clip(CircleShape)
            .background(bg)
            .clickable(role = Role.RadioButton, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 8.dp),
    )
}
