package com.glglekdy.quickmedia.ui.components

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.glglekdy.quickmedia.ui.theme.Qm

/** 썸네일. 주소가 없거나 못 받으면 회색 칸에 아이콘이 남는다. */
@Composable
fun Thumbnail(url: String?, @DrawableRes fallbackIcon: Int, modifier: Modifier, shape: Shape) {
    val c = Qm.colors
    Box(modifier.clip(shape).background(c.field), contentAlignment = Alignment.Center) {
        Icon(painterResource(fallbackIcon), contentDescription = null, tint = c.textDim, modifier = Modifier.size(22.dp))
        if (url != null) {
            AsyncImage(url, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        }
    }
}
