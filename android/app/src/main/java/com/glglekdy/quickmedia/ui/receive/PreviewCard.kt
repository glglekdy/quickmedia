package com.glglekdy.quickmedia.ui.receive

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.preview.Preview
import com.glglekdy.quickmedia.preview.formatDuration
import com.glglekdy.quickmedia.preview.formatViews
import com.glglekdy.quickmedia.ui.components.SectionCard
import com.glglekdy.quickmedia.ui.components.Skeleton
import com.glglekdy.quickmedia.ui.components.Thumbnail
import com.glglekdy.quickmedia.ui.theme.Qm

/** 다 불러온 미리보기: 큰 썸네일, 길이(재생목록은 개수), 제목, 채널·조회수 */
@Composable
fun PreviewCard(preview: Preview) {
    val c = Qm.colors
    SectionCard {
        Box {
            Thumbnail(
                preview.thumbnail,
                if (preview.isPlaylist) R.drawable.ic_list else R.drawable.ic_video,
                Modifier.fillMaxWidth().aspectRatio(16f / 9f),
                RectangleShape,
            )
            val tag = if (preview.isPlaylist) "재생목록 · ${preview.count ?: 0}개" else formatDuration(preview.duration)
            if (tag != null) {
                Text(
                    tag,
                    color = Color.White,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.SemiBold,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(8.dp)
                        .background(Color.Black.copy(alpha = 0.75f), RoundedCornerShape(5.dp))
                        .padding(horizontal = 6.dp, vertical = 2.dp),
                )
            }
        }
        Column(Modifier.padding(horizontal = 14.dp, vertical = 12.dp)) {
            Text(
                preview.title,
                color = c.text,
                fontSize = 15.sp,
                fontWeight = FontWeight.Bold,
                lineHeight = 21.sp,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            val sub = listOfNotNull(preview.channel, formatViews(preview.views)).joinToString(" · ")
            if (sub.isNotEmpty()) {
                Text(
                    sub,
                    color = c.textDim,
                    fontSize = 12.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 4.dp),
                )
            }
        }
    }
}

/** 불러오는 동안 같은 자리를 차지하는 뼈대 */
@Composable
fun PreviewSkeleton() {
    SectionCard {
        Skeleton(Modifier.fillMaxWidth().aspectRatio(16f / 9f), RectangleShape)
        Column(Modifier.padding(horizontal = 14.dp, vertical = 14.dp)) {
            Skeleton(Modifier.fillMaxWidth(0.85f).height(14.dp), RoundedCornerShape(6.dp))
            Skeleton(Modifier.padding(top = 8.dp).fillMaxWidth(0.45f).height(11.dp), RoundedCornerShape(6.dp))
        }
    }
}

@Composable
fun PreviewFailed() {
    SectionCard {
        Text(
            "영상 정보를 가져오지 못했어요",
            color = Qm.colors.textDim,
            fontSize = 13.sp,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 14.dp),
        )
    }
}
