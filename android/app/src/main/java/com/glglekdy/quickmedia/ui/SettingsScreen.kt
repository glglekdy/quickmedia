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
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import com.yausername.youtubedl_android.YoutubeDL
import com.yausername.youtubedl_android.YoutubeDLRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** 앱 버전과 업데이트, yt-dlp 버전과 갱신, 저장 위치 안내. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(onBack: () -> Unit) {
    val context = LocalContext.current
    val tools by App.tools.collectAsStateWithLifecycle()
    var ytdlp by remember { mutableStateOf<String?>(null) }
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
    ) { padding ->
        Column(
            Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState()).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Item("앱 버전", BuildConfig.VERSION_NAME) {
                OutlinedButton(onClick = {}) { Text("업데이트 확인") }
            }
            Item(
                "yt-dlp 버전",
                ytdlp ?: when (val t = tools) {
                    is Tools.Failed -> t.reason
                    else -> "확인 중…"
                },
            ) {
                OutlinedButton(onClick = {}, enabled = tools == Tools.Ready) { Text("yt-dlp 갱신") }
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
