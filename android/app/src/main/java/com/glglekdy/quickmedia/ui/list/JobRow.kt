package com.glglekdy.quickmedia.ui.list

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import android.app.DownloadManager
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.net.toUri
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.download.Job
import com.glglekdy.quickmedia.download.JobState
import com.glglekdy.quickmedia.options.Kind
import com.glglekdy.quickmedia.options.describe
import com.glglekdy.quickmedia.preview.Preview
import com.glglekdy.quickmedia.ui.components.Thumbnail
import com.glglekdy.quickmedia.ui.theme.AccentGradient
import com.glglekdy.quickmedia.ui.theme.Motion
import com.glglekdy.quickmedia.ui.theme.Qm

private enum class Action { CANCEL, OPEN, RETRY }

/** 목록 한 줄: 썸네일, 제목, 상태, 진행 막대, 오른쪽 버튼(취소·열기·다시 받기). */
@Composable
fun JobRow(
    job: Job,
    preview: Preview?,
    onCancel: () -> Unit,
    onOpen: () -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val c = Qm.colors
    val active = job.state == JobState.QUEUED || job.state == JobState.RUNNING
    Row(
        modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .clickable(enabled = job.state == JobState.DONE, onClick = onOpen)
            .padding(vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Thumbnail(
            preview?.thumbnail,
            if (job.options.kind == Kind.AUDIO) R.drawable.ic_audio else R.drawable.ic_video,
            Modifier.size(width = 96.dp, height = 56.dp),
            RoundedCornerShape(12.dp),
        )
        Column(Modifier.weight(1f)) {
            Text(
                preview?.title ?: job.title,
                color = c.text,
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                status(job),
                color = c.textDim,
                fontSize = 12.sp,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 2.dp),
            )
            if (job.state == JobState.RUNNING) ProgressBar(job.percent, Modifier.padding(top = 7.dp))
        }
        val button = when {
            active -> Action.CANCEL
            job.state == JobState.DONE -> Action.OPEN
            else -> Action.RETRY
        }
        // ✕ 가 ▶ 나 ↻ 로 바뀔 때 작아졌다 커지며 바뀐다
        AnimatedContent(
            button,
            transitionSpec = {
                (fadeIn(Motion.fade()) + scaleIn(Motion.slide(), initialScale = 0.6f)) togetherWith
                    (fadeOut(Motion.fade()) + scaleOut(Motion.fade(), targetScale = 0.6f))
            },
            label = "jobAction",
        ) { action ->
            when (action) {
                Action.CANCEL -> RoundButton(R.drawable.ic_close, "취소", onCancel)
                Action.OPEN -> RoundButton(R.drawable.ic_play, "열기", onOpen)
                Action.RETRY -> RoundButton(R.drawable.ic_retry, "다시 받기", onRetry)
            }
        }
    }
}

@Composable
private fun status(job: Job) = buildAnnotatedString {
    val c = Qm.colors
    when (job.state) {
        JobState.QUEUED, JobState.RUNNING -> append("${describe(job.options)} · ${job.status}")
        JobState.DONE -> {
            withStyle(SpanStyle(color = c.ok)) { append("● 완료") }
            append(" · ${describe(job.options)}")
        }
        JobState.FAILED -> {
            withStyle(SpanStyle(color = c.bad)) { append("● 실패") }
            append(" · ${job.status}")
        }
        JobState.CANCELLED -> withStyle(SpanStyle(color = c.bad)) { append("● 취소됨") }
    }
}

/** 퍼센트를 모르는 동안(영상 정보 불러오는 중)은 짧은 막대가 왔다 갔다 한다 */
@Composable
private fun ProgressBar(percent: Float, modifier: Modifier) {
    val c = Qm.colors
    BoxWithConstraints(modifier.fillMaxWidth().height(4.dp).clip(CircleShape).background(c.field)) {
        // 퍼센트가 띄엄띄엄 와도 막대는 이어서 찬다
        val shown by animateFloatAsState(percent / 100f, tween(400, easing = LinearOutSlowInEasing), label = "percent")
        if (percent > 0f) {
            Box(Modifier.fillMaxHeight().fillMaxWidth(shown).clip(CircleShape).background(AccentGradient))
        } else {
            val x by rememberInfiniteTransition(label = "progress").animateFloat(
                initialValue = 0f,
                targetValue = 0.7f,
                animationSpec = infiniteRepeatable(tween(900, easing = LinearEasing), RepeatMode.Reverse),
                label = "progress",
            )
            Box(
                Modifier.fillMaxHeight().width(maxWidth * 0.3f).offset(x = maxWidth * x)
                    .clip(CircleShape).background(AccentGradient),
            )
        }
    }
}

@Composable
private fun RoundButton(icon: Int, description: String, onClick: () -> Unit) {
    val c = Qm.colors
    Box(
        Modifier.size(34.dp).clip(CircleShape).background(c.field).clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Icon(painterResource(icon), contentDescription = description, tint = c.text, modifier = Modifier.size(16.dp))
    }
}

/** 받은 파일을 연다. 재생목록처럼 여럿이면 다운로드 화면을 연다. 못 열면 그 문구. */
fun openJob(context: Context, job: Job): String? {
    val intent = job.saved.singleOrNull()?.toUri()?.let { uri ->
        Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, context.contentResolver.getType(uri))
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    } ?: Intent(DownloadManager.ACTION_VIEW_DOWNLOADS)
    return try {
        context.startActivity(intent)
        null
    } catch (e: ActivityNotFoundException) {
        "이 파일을 열 수 있는 앱이 없어요"
    }
}
