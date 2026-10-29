package com.glglekdy.quickmedia.ui

import android.content.Intent
import android.provider.Settings
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.net.toUri
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.withResumed
import com.glglekdy.quickmedia.BuildConfig
import com.glglekdy.quickmedia.update.Release
import com.glglekdy.quickmedia.update.UpdateException
import com.glglekdy.quickmedia.update.Updater
import com.glglekdy.quickmedia.update.noteLines
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/**
 * "새 버전 X" 안내. 흐름은 데스크톱 `update_dialog.py` 와 같다: 받는 동안 버튼을 막고 진행 막대,
 * 실패하면 대화상자 안에 문구와 "다시 시도". 다 받으면 안드로이드 설치 확인 화면으로 넘기고 닫는다.
 */
@Composable
fun UpdateDialog(release: Release, onDismiss: () -> Unit) {
    val context = LocalContext.current
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var percent by remember { mutableIntStateOf(-1) }
    var status by remember { mutableStateOf("") }
    var failed by remember { mutableStateOf(false) }
    // "출처를 알 수 없는 앱 설치" 를 켜러 간 사이 기다리는 APK. 그 사이 앱이 다시 만들어져도 이어지게 저장한다.
    var waiting by rememberSaveable { mutableStateOf<String?>(null) }

    fun start() {
        busy = true
        failed = false
        percent = -1
        status = "받는 중…"
        scope.launch {
            try {
                val apk = waiting?.let(::File)?.takeIf { it.isFile }
                    ?: Updater.download(context, release) { done, total ->
                        if (total > 0) {
                            percent = (done * 100 / total).toInt()
                            status = "받는 중… $percent%"
                        }
                    }
                // 다른 앱에 가 있으면 안드로이드가 설치 확인 화면을 띄우지 못한다
                lifecycle.withResumed {}
                if (!context.packageManager.canRequestPackageInstalls()) {
                    waiting = apk.path
                    status = "설치를 허용하고 돌아오면 이어서 설치해요"
                    context.startActivity(
                        Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, "package:${context.packageName}".toUri()),
                    )
                    return@launch
                }
                waiting = null
                status = "설치 화면을 여는 중…"
                withContext(Dispatchers.IO) { Updater.install(context, apk) }
                onDismiss()
            } catch (e: UpdateException) {
                failed = true
                status = e.message.orEmpty()
            } finally {
                busy = false
            }
        }
    }

    // 설치 허용 설정에서 돌아왔을 때. 처음 그려질 때도 불리지만 그때는 기다리는 APK 가 없다.
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
        if (waiting == null || busy) return@LifecycleEventEffect
        if (context.packageManager.canRequestPackageInstalls()) {
            start()
        } else {
            failed = true
            status = "설치를 허용해야 업데이트할 수 있어요"
        }
    }

    AlertDialog(
        onDismissRequest = { if (!busy) onDismiss() },
        title = { Text("새 버전 ${release.version}") },
        text = {
            Column {
                Text(
                    "지금 버전: ${BuildConfig.VERSION_NAME}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                val lines = remember(release.notes) { noteLines(release.notes) }
                if (lines.isNotEmpty()) {
                    Column(Modifier.padding(top = 12.dp).heightIn(max = 240.dp).verticalScroll(rememberScrollState())) {
                        lines.forEach {
                            Text(
                                it.text,
                                style = if (it.heading) MaterialTheme.typography.titleSmall else MaterialTheme.typography.bodyMedium,
                            )
                        }
                    }
                }
                if (busy) {
                    val bar = Modifier.fillMaxWidth().padding(top = 16.dp)
                    if (percent >= 0) LinearProgressIndicator(progress = { percent / 100f }, modifier = bar)
                    else LinearProgressIndicator(bar)
                }
                if (status.isNotEmpty()) {
                    Text(
                        status,
                        Modifier.padding(top = 8.dp),
                        style = MaterialTheme.typography.bodySmall,
                        color = if (failed) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        },
        confirmButton = {
            TextButton(onClick = { start() }, enabled = !busy) { Text(if (failed) "다시 시도" else "지금 업데이트") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !busy) { Text("나중에") }
        },
    )
}
