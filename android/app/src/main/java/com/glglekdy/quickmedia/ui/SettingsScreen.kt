package com.glglekdy.quickmedia.ui

import android.content.Context
import android.util.Log
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.glglekdy.quickmedia.App
import com.glglekdy.quickmedia.BuildConfig
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.Tools
import com.glglekdy.quickmedia.download.JobState
import com.glglekdy.quickmedia.download.Jobs
import com.glglekdy.quickmedia.update.Release
import com.glglekdy.quickmedia.update.UpdateException
import com.glglekdy.quickmedia.update.Updater
import com.yausername.youtubedl_android.YoutubeDL
import com.yausername.youtubedl_android.YoutubeDLRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** 앱 버전과 업데이트, yt-dlp 버전과 갱신, 저장 위치 안내. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(onBack: () -> Unit, onNewVersion: (Release) -> Unit) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }
    val tools by App.tools.collectAsStateWithLifecycle()
    val jobs by Jobs.list.collectAsStateWithLifecycle()
    var ytdlp by remember { mutableStateOf<String?>(null) }
    var checking by remember { mutableStateOf(false) }
    var updating by remember { mutableStateOf(false) }
    LaunchedEffect(tools) {
        if (tools == Tools.Ready) ytdlp = withContext(Dispatchers.IO) { ytdlpVersion(context) }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("설정") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(painterResource(R.drawable.ic_back), contentDescription = "뒤로")
                    }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbar) },
    ) { padding ->
        Column(
            Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState()).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Item("앱 버전", BuildConfig.VERSION_NAME) {
                OutlinedButton(
                    onClick = {
                        checking = true
                        scope.launch {
                            val message = try {
                                when (val release = Updater.check()) {
                                    null -> "최신 버전이에요"
                                    else -> {
                                        onNewVersion(release)
                                        null
                                    }
                                }
                            } catch (e: UpdateException) {
                                e.message
                            } finally {
                                checking = false
                            }
                            message?.let { snackbar.showSnackbar(it) }
                        }
                    },
                    enabled = !checking,
                ) { Text(if (checking) "확인 중…" else "업데이트 확인") }
            }
            Item(
                "yt-dlp 버전",
                ytdlp ?: when (val t = tools) {
                    is Tools.Failed -> t.reason
                    else -> "확인 중…"
                },
            ) {
                OutlinedButton(
                    onClick = {
                        // 받는 중에 yt-dlp 를 바꿔 끼우면 그 작업이 깨질 수 있다
                        if (jobs.any { it.state == JobState.QUEUED || it.state == JobState.RUNNING }) {
                            scope.launch { snackbar.showSnackbar("받는 중인 작업이 끝난 뒤에 갱신해 주세요") }
                        } else {
                            updating = true
                            scope.launch {
                                val message = withContext(Dispatchers.IO) { updateYtdlp(context) }
                                ytdlp = withContext(Dispatchers.IO) { ytdlpVersion(context) }
                                updating = false
                                snackbar.showSnackbar(message)
                            }
                        }
                    },
                    enabled = tools == Tools.Ready && !updating,
                ) { Text(if (updating) "갱신 중…" else "yt-dlp 갱신") }
            }
            Item("저장 위치", "다운로드/QuickMedia")
        }
    }
}

@Composable
private fun Item(title: String, value: String, action: (@Composable () -> Unit)? = null) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow),
    ) {
        Row(Modifier.padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f).padding(end = 12.dp)) {
                Text(title, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text(value, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.padding(top = 2.dp))
            }
            action?.invoke()
        }
    }
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
