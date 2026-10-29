package com.glglekdy.quickmedia.ui.settings

import android.app.DownloadManager
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.net.toUri
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.glglekdy.quickmedia.App
import com.glglekdy.quickmedia.BuildConfig
import com.glglekdy.quickmedia.Tools
import com.glglekdy.quickmedia.download.JobState
import com.glglekdy.quickmedia.download.Jobs
import com.glglekdy.quickmedia.ui.components.SectionCard
import com.glglekdy.quickmedia.ui.theme.Qm
import com.glglekdy.quickmedia.update.Release
import com.glglekdy.quickmedia.update.UpdateException
import com.glglekdy.quickmedia.update.Updater
import com.yausername.youtubedl_android.YoutubeDL
import com.yausername.youtubedl_android.YoutubeDLRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private enum class UpdateStatus { IDLE, CHECKING, LATEST, NEW, FAILED }

private const val REPO_URL = "https://github.com/glglekdy/quickmedia"

/** 설정 탭: 앱 버전·업데이트, yt-dlp 갱신, 저장 위치, GitHub. */
@Composable
fun SettingsScreen(onNewVersion: (Release) -> Unit, say: (String) -> Unit, contentPadding: PaddingValues) {
    val c = Qm.colors
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val tools by App.tools.collectAsStateWithLifecycle()
    val jobs by Jobs.list.collectAsStateWithLifecycle()
    var status by rememberSaveable { mutableStateOf(UpdateStatus.IDLE) }
    var release by rememberSaveable { mutableStateOf<Release?>(null) }
    var failure by rememberSaveable { mutableStateOf<String?>(null) }
    var ytdlp by remember { mutableStateOf<String?>(null) }
    var updating by remember { mutableStateOf(false) }
    LaunchedEffect(tools) {
        if (tools == Tools.Ready) ytdlp = withContext(Dispatchers.IO) { ytdlpVersion(context) }
    }

    fun check() {
        status = UpdateStatus.CHECKING
        scope.launch {
            status = try {
                when (val found = Updater.check()) {
                    null -> UpdateStatus.LATEST
                    else -> {
                        release = found
                        onNewVersion(found)
                        UpdateStatus.NEW
                    }
                }
            } catch (e: UpdateException) {
                failure = e.message
                UpdateStatus.FAILED
            }
        }
    }

    fun updateTools() {
        // 받는 중에 yt-dlp 를 바꿔 끼우면 그 작업이 깨질 수 있다
        if (jobs.any { it.state == JobState.QUEUED || it.state == JobState.RUNNING }) {
            say("받는 중인 작업이 끝난 뒤에 갱신해 주세요")
            return
        }
        updating = true
        scope.launch {
            val message = withContext(Dispatchers.IO) { updateYtdlp(context) }
            ytdlp = withContext(Dispatchers.IO) { ytdlpVersion(context) }
            updating = false
            say(message)
        }
    }

    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(start = 18.dp, end = 18.dp)
            .padding(top = contentPadding.calculateTopPadding(), bottom = contentPadding.calculateBottomPadding()),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(
            "설정",
            color = c.text,
            fontSize = 24.sp,
            fontWeight = FontWeight.ExtraBold,
            modifier = Modifier.padding(top = 8.dp, bottom = 4.dp),
        )

        // 앱 버전 카드
        val glow = if (c.isDark) Color(0xFF2A0F10) else Color(0xFFFFE4E2)
        Column(
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(20.dp))
                .background(Brush.linearGradient(listOf(glow, c.surface), Offset.Zero, Offset.Infinite))
                .padding(18.dp),
        ) {
            Text(
                buildAnnotatedString {
                    append("Quick")
                    withStyle(SpanStyle(color = c.accentText)) { append("Media") }
                    append(" ${BuildConfig.VERSION_NAME}")
                },
                color = c.text,
                fontSize = 21.sp,
                fontWeight = FontWeight.ExtraBold,
            )
            val newOne = release
            Text(
                when (status) {
                    UpdateStatus.IDLE -> "업데이트를 확인해 보세요"
                    UpdateStatus.CHECKING -> "확인하는 중…"
                    UpdateStatus.LATEST -> "최신 버전이에요"
                    UpdateStatus.NEW -> "새 버전 ${newOne?.version} 이 있어요"
                    UpdateStatus.FAILED -> failure.orEmpty()
                },
                color = if (status == UpdateStatus.NEW) c.accentText else c.textDim,
                fontSize = 13.sp,
                fontWeight = if (status == UpdateStatus.NEW) FontWeight.Bold else FontWeight.Normal,
                modifier = Modifier
                    .padding(top = 6.dp)
                    .clickable(enabled = status == UpdateStatus.NEW && newOne != null) { newOne?.let(onNewVersion) },
            )
            Pill(
                if (status == UpdateStatus.CHECKING) "확인 중…" else "업데이트 확인",
                accent = true,
                enabled = status != UpdateStatus.CHECKING,
                onClick = ::check,
                modifier = Modifier.padding(top = 14.dp),
            )
        }

        SectionCard {
            Item(
                "yt-dlp",
                ytdlp ?: when (val t = tools) {
                    is Tools.Failed -> t.reason
                    else -> "확인 중…"
                },
            ) {
                Pill(if (updating) "갱신 중…" else "갱신", enabled = tools == Tools.Ready && !updating, onClick = ::updateTools)
            }
            Divider()
            Item("저장 위치", "다운로드/QuickMedia") {
                Pill("열기", onClick = {
                    if (!start(context, Intent(DownloadManager.ACTION_VIEW_DOWNLOADS))) say("이 파일을 열 수 있는 앱이 없어요")
                })
            }
        }

        SectionCard {
            Row(
                Modifier
                    .fillMaxWidth()
                    .clickable { start(context, Intent(Intent.ACTION_VIEW, REPO_URL.toUri())) }
                    .padding(horizontal = 16.dp, vertical = 16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("GitHub에서 보기", color = c.text, fontSize = 15.sp, modifier = Modifier.weight(1f))
                Text("›", color = c.textDim, fontSize = 18.sp)
            }
        }
    }
}

@Composable
private fun Item(title: String, value: String, action: @Composable () -> Unit) {
    val c = Qm.colors
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(1f).padding(end = 12.dp)) {
            Text(title, color = c.text, fontSize = 15.sp)
            Text(value, color = c.textDim, fontSize = 13.sp, modifier = Modifier.padding(top = 2.dp))
        }
        action()
    }
}

@Composable
private fun Divider() =
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp).height(1.dp).background(Qm.colors.field)) {}

@Composable
private fun Pill(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    accent: Boolean = false,
    enabled: Boolean = true,
) {
    val c = Qm.colors
    Text(
        text,
        color = if (accent) Color.White else c.text,
        fontSize = 13.sp,
        fontWeight = FontWeight.SemiBold,
        modifier = modifier
            .clip(CircleShape)
            .background(if (accent) c.accent else c.field)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 8.dp),
    )
}

private fun start(context: Context, intent: Intent): Boolean = try {
    context.startActivity(intent)
    true
} catch (e: ActivityNotFoundException) {
    false
}

/** 안정 채널의 최신 yt-dlp 로 바꾼다. 결과를 화면 문구로 돌려준다. */
private fun updateYtdlp(context: Context): String = try {
    val dl = YoutubeDL.getInstance()
    if (dl.updateYoutubeDL(context, YoutubeDL.UpdateChannel.STABLE) == YoutubeDL.UpdateStatus.ALREADY_UP_TO_DATE) {
        "최신 버전이에요"
    } else {
        "${dl.version(context)} 으로 갱신했어요"
    }
} catch (e: Exception) {
    Log.w(App.TAG, "yt-dlp 갱신 실패", e)
    "yt-dlp 를 갱신하지 못했어요"
}

/** 갱신한 적이 있으면 라이브러리가 기억한 버전, 아니면(앱에 들어 있던 그대로) yt-dlp 에 직접 묻는다. */
private fun ytdlpVersion(context: Context): String? {
    val dl = YoutubeDL.getInstance()
    dl.version(context)?.let { return it }
    return try {
        dl.execute(YoutubeDLRequest(emptyList<String>()).addOption("--version")).out.trim().ifEmpty { null }
    } catch (e: Exception) {
        Log.w(App.TAG, "yt-dlp 버전 확인 실패", e)
        null
    }
}
