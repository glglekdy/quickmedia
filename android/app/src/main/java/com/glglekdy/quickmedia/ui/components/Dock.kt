package com.glglekdy.quickmedia.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
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
import com.glglekdy.quickmedia.ui.theme.Motion
import com.glglekdy.quickmedia.ui.theme.Qm
import com.glglekdy.quickmedia.ui.theme.pressScale
import dev.chrisbanes.haze.HazeInput
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.blur.HazeBlurStyle
import dev.chrisbanes.haze.blur.HazeColorEffect
import dev.chrisbanes.haze.blur.hazeBlur

// 아이콘 한 칸 크기와 칸 사이. 선택 표시가 이 간격으로 미끄러진다.
private val ItemWidth = 64.dp
private val ItemHeight = 48.dp
private val Gap = 4.dp

/** 화면 아래 가운데 떠 있는 알약 메뉴. 아이콘만 있다. 뒤 내용이 흐리게 비친다(안드로이드 12+, 그 아래는 반투명만). */
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
    val indicator by animateDpAsState((ItemWidth + Gap) * selected.ordinal, Motion.slide(), label = "dock")
    Box(
        modifier
            .shadow(24.dp, CircleShape, ambientColor = Color.Black, spotColor = Color.Black)
            .clip(CircleShape)
            .hazeBlur(HazeInput.Sources(hazeState), style)
            .border(1.dp, if (c.isDark) Color.White.copy(alpha = 0.10f) else Color.Black.copy(alpha = 0.06f), CircleShape)
            .padding(5.dp),
    ) {
        // 고른 탭 뒤의 동그란 표시. 아이콘마다 따로 그리지 않고 하나를 옮긴다.
        Box(
            Modifier
                .offset(x = indicator)
                .size(ItemWidth, ItemHeight)
                .clip(CircleShape)
                .background(c.accent.copy(alpha = if (c.isDark) 0.18f else 0.12f)),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(Gap)) {
            Tab.entries.forEach { tab ->
                DockItem(tab, on = tab == selected, badge = if (tab == Tab.LIST) badge else 0) { onSelect(tab) }
            }
        }
    }
}

@Composable
private fun DockItem(tab: Tab, on: Boolean, badge: Int, onClick: () -> Unit) {
    val c = Qm.colors
    val source = remember { MutableInteractionSource() }
    val tint by animateColorAsState(if (on) c.accentText else c.textDim, Motion.fade(), label = "dockTint")
    Box {
        Box(
            Modifier
                .size(ItemWidth, ItemHeight)
                .pressScale(source, pressed = 0.86f)
                .clip(CircleShape)
                .clickable(interactionSource = source, indication = null, role = Role.Tab, onClick = onClick),
            contentAlignment = Alignment.Center,
        ) {
            // 글자 없이 아이콘만. 이름은 화면 읽기용으로 둔다
            Icon(painterResource(tab.icon), contentDescription = tab.label, tint = tint, modifier = Modifier.size(24.dp))
        }
        if (badge > 0) {
            Text(
                badge.toString(),
                color = Color.White,
                fontSize = 10.sp,
                fontWeight = FontWeight.ExtraBold,
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .offset(x = (-10).dp, y = 6.dp)
                    .clip(CircleShape)
                    .background(c.accent)
                    .padding(horizontal = 5.dp),
            )
        }
    }
}
