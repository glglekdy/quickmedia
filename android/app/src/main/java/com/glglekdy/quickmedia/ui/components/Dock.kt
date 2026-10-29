package com.glglekdy.quickmedia.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.ui.Tab
import com.glglekdy.quickmedia.ui.theme.Qm
import dev.chrisbanes.haze.HazeInput
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.blur.HazeBlurStyle
import dev.chrisbanes.haze.blur.HazeColorEffect
import dev.chrisbanes.haze.blur.hazeBlur

/** 화면 아래 가운데 떠 있는 알약 메뉴. 뒤 내용이 흐리게 비친다(안드로이드 12+, 그 아래는 반투명만). */
@Composable
fun Dock(selected: Tab, onSelect: (Tab) -> Unit, badge: Int, hazeState: HazeState, modifier: Modifier = Modifier) {
    val c = Qm.colors
    val style = remember(c) {
        HazeBlurStyle {
            blurRadius(18.dp)
            backgroundColor(c.bg)
            colorEffects(listOf(HazeColorEffect.tint(c.dock)))
            // 흐림을 못 쓰는 기기에서는 바탕을 더 짙게 해 뒤 글자가 메뉴 글자와 겹쳐 보이지 않게
            fallbackColorEffect(HazeColorEffect.tint(c.dock.copy(alpha = 0.92f)))
        }
    }
    Row(
        modifier
            .shadow(24.dp, CircleShape, ambientColor = Color.Black, spotColor = Color.Black)
            .clip(CircleShape)
            .hazeBlur(HazeInput.Sources(hazeState), style)
            .border(1.dp, if (c.isDark) Color.White.copy(alpha = 0.10f) else Color.Black.copy(alpha = 0.06f), CircleShape)
            .padding(5.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Tab.entries.forEach { tab ->
            val on = tab == selected
            Box {
                Column(
                    Modifier
                        .clip(CircleShape)
                        .background(if (on) c.accent.copy(alpha = if (c.isDark) 0.18f else 0.12f) else Color.Transparent)
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                            role = Role.Tab,
                        ) { onSelect(tab) }
                        .padding(horizontal = 18.dp, vertical = 7.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    val color = if (on) c.accentText else c.textDim
                    Icon(painterResource(tab.icon), contentDescription = null, tint = color, modifier = Modifier.size(22.dp))
                    Text(
                        tab.label,
                        color = color,
                        fontSize = 11.sp,
                        fontWeight = if (on) FontWeight.Bold else FontWeight.Medium,
                        modifier = Modifier.padding(top = 2.dp),
                    )
                }
                if (tab == Tab.LIST && badge > 0) {
                    Text(
                        badge.toString(),
                        color = Color.White,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.ExtraBold,
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .offset(x = (-8).dp, y = 3.dp)
                            .clip(CircleShape)
                            .background(c.accent)
                            .padding(horizontal = 5.dp),
                    )
                }
            }
        }
    }
}
