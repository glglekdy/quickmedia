package com.glglekdy.quickmedia.ui

import android.Manifest
import android.app.DownloadManager
import android.content.ActivityNotFoundException
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.content.edit
import androidx.core.net.toUri
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.glglekdy.quickmedia.App
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.Tools
import com.glglekdy.quickmedia.download.DownloadService
import com.glglekdy.quickmedia.download.Job
import com.glglekdy.quickmedia.download.JobState
import com.glglekdy.quickmedia.download.Jobs
import com.glglekdy.quickmedia.download.extractLink
import com.glglekdy.quickmedia.options.AUDIO_FORMATS
import com.glglekdy.quickmedia.options.DownloadOptions
import com.glglekdy.quickmedia.options.Kind
import com.glglekdy.quickmedia.options.OptionsStore
import com.glglekdy.quickmedia.options.PlaylistMode
import com.glglekdy.quickmedia.options.QUALITIES
import com.glglekdy.quickmedia.options.YtdlpArgs
import com.glglekdy.quickmedia.options.changedSummary
import com.glglekdy.quickmedia.options.describe
import com.glglekdy.quickmedia.options.qualityLabel
import kotlinx.coroutines.launch

/** 링크 넣기 · 형식 고르기 · 옵션 더보기 · 다운로드 · 작업 목록. 링크 칸은 공유로도 채워져서 밖에서 갖고 있다. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    link: String,
    onLinkChange: (String) -> Unit,
    snackbar: SnackbarHostState,
    onOpenSettings: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val focus = LocalFocusManager.current
    val store = remember { OptionsStore(context) }
    // 고르는 즉시 저장한다. 화면을 돌려 다시 만들어져도 저장값에서 그대로 읽는다.
    var options by remember { mutableStateOf(store.load()) }
    var errors by remember { mutableStateOf(emptyMap<String, String>()) }
    var linkError by remember { mutableStateOf<String?>(null) }
    var expanded by rememberSaveable { mutableStateOf(false) }
    val tools by App.tools.collectAsStateWithLifecycle()
    val jobs by Jobs.list.collectAsStateWithLifecycle()
    val askNotifications = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}

    fun say(text: String) {
        scope.launch { snackbar.showSnackbar(text) }
    }

    fun change(o: DownloadOptions) {
        options = o
        store.save(o)
        // 이미 띄운 문구만 다시 맞춘다. 고치면 사라지고, 쓰는 중에 새 문구가 튀어나오지는 않는다.
        errors = YtdlpArgs.validate(o).filterKeys { it in errors }
    }

    fun paste() {
        val clip = context.getSystemService(ClipboardManager::class.java).primaryClip
        val text = clip?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.coerceToText(context)?.toString()
        val found = extractLink(text)
        if (found == null) say("복사한 글에 링크가 없어요")
        else {
            onLinkChange(found)
            linkError = null
        }
    }

    fun download() {
        val url = extractLink(link) ?: link.trim()
        linkError = if (url.isEmpty()) "먼저 링크를 넣어 주세요" else null
        errors = YtdlpArgs.validate(options)
        if (errors.isNotEmpty()) expanded = true
        if (linkError != null || errors.isNotEmpty()) return
        askNotificationsOnce(context) { askNotifications.launch(Manifest.permission.POST_NOTIFICATIONS) }
        Jobs.add(url, options)
        DownloadService.start(context)
        onLinkChange("")
        focus.clearFocus()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("QuickMedia") },
                actions = {
                    IconButton(onClick = onOpenSettings) {
                        Icon(painterResource(R.drawable.ic_settings), contentDescription = "설정")
                    }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbar) },
    ) { padding ->
        LazyColumn(
            Modifier.fillMaxSize().padding(padding).consumeWindowInsets(padding).imePadding(),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            (tools as? Tools.Failed)?.let { failed ->
                item("tools") { Banner(failed.reason) }
            }
            item("link") {
                OutlinedTextField(
                    value = link,
                    onValueChange = {
                        onLinkChange(it)
                        linkError = null
                    },
                    modifier = Modifier.fillMaxWidth(),
                    placeholder = { Text("영상·재생목록 링크를 붙여넣으세요") },
                    singleLine = true,
                    isError = linkError != null,
                    supportingText = linkError?.let { { Text(it) } },
                    trailingIcon = {
                        if (link.isEmpty()) {
                            TextButton(onClick = ::paste) { Text("붙여넣기") }
                        } else {
                            IconButton(onClick = { onLinkChange("") }) {
                                Icon(painterResource(R.drawable.ic_close), contentDescription = "지우기")
                            }
                        }
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Go),
                    keyboardActions = KeyboardActions(onGo = { download() }),
                )
            }
            item("kind") {
                Choices(
                    values = Kind.entries,
                    selected = options.kind,
                    label = { if (it == Kind.VIDEO) "영상" else "음성" },
                    onSelect = { change(options.copy(kind = it)) },
                )
            }
            item("quality") {
                if (options.kind == Kind.VIDEO) {
                    Choices(QUALITIES, options.quality, ::qualityLabel) { change(options.copy(quality = it)) }
                } else {
                    Choices(AUDIO_FORMATS, options.audioFormat, ::qualityLabel) { change(options.copy(audioFormat = it)) }
                }
            }
            item("more") {
                Column {
                    val changed = changedSummary(options)
                    TextButton(onClick = { expanded = !expanded }) {
                        Text(if (changed.isEmpty()) "옵션 더보기" else "옵션 더보기 · " + changed.joinToString(", "))
                        Icon(
                            painterResource(R.drawable.ic_expand),
                            contentDescription = null,
                            modifier = Modifier.padding(start = 4.dp).rotate(if (expanded) 180f else 0f),
                        )
                    }
                    AnimatedVisibility(expanded) {
                        MoreOptions(options, errors, ::change)
                    }
                }
            }
            item("download") {
                Button(onClick = ::download, modifier = Modifier.fillMaxWidth().height(52.dp)) {
                    Text("다운로드", style = MaterialTheme.typography.titleMedium)
                }
            }
            item("listHead") {
                Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text("다운로드 목록", style = MaterialTheme.typography.titleMedium, modifier = Modifier.weight(1f))
                    TextButton(onClick = Jobs::clearFinished, enabled = jobs.any { !it.active }) {
                        Text("끝난 항목 지우기")
                    }
                }
            }
            if (jobs.isEmpty()) {
                item("empty") {
                    Text(
                        "아직 받은 영상이 없어요",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(vertical = 8.dp),
                    )
                }
            }
            items(jobs, key = { it.id }) { job ->
                JobCard(
                    job,
                    onOpen = { open(context, job)?.let(::say) },
                    onCancel = { DownloadService.cancel(job.id) },
                )
            }
        }
    }
}

private val Job.active get() = state == JobState.QUEUED || state == JobState.RUNNING

/** 안드로이드 13+ 에서 처음 받을 때 한 번만 알림 권한을 묻는다. 거절해도 다운로드는 된다. */
private fun askNotificationsOnce(context: Context, ask: () -> Unit) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return
    if (ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) ==
        PackageManager.PERMISSION_GRANTED
    ) return
    val prefs = context.getSharedPreferences("ui", Context.MODE_PRIVATE)
    if (prefs.getBoolean("askedNotifications", false)) return
    prefs.edit { putBoolean("askedNotifications", true) }
    ask()
}

/** 받은 파일을 연다. 재생목록처럼 여럿이면 다운로드 폴더를 연다. 못 열면 그 문구. */
private fun open(context: Context, job: Job): String? {
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

@Composable
private fun <T> Choices(values: List<T>, selected: T, label: (T) -> String, onSelect: (T) -> Unit) {
    SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
        values.forEachIndexed { i, value ->
            SegmentedButton(
                selected = value == selected,
                onClick = { onSelect(value) },
                shape = SegmentedButtonDefaults.itemShape(i, values.size),
                // 체크 표시를 빼야 좁은 폰에서도 1080p 가 한 줄에 들어간다
                icon = {},
            ) { Text(label(value), maxLines = 1) }
        }
    }
}

@Composable
private fun MoreOptions(o: DownloadOptions, errors: Map<String, String>, onChange: (DownloadOptions) -> Unit) {
    Column(Modifier.padding(top = 4.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        SectionLabel("재생목록")
        Choices(
            values = PlaylistMode.entries,
            selected = o.playlist,
            label = {
                when (it) {
                    PlaylistMode.SINGLE -> "이 영상만"
                    PlaylistMode.ALL -> "전체"
                    PlaylistMode.ITEMS -> "일부 항목"
                }
            },
            onSelect = { onChange(o.copy(playlist = it)) },
        )
        if (o.playlist == PlaylistMode.ITEMS) {
            Field(o.playlistItems, "받을 항목", "1-5,8", errors["playlistItems"], Modifier.fillMaxWidth()) {
                onChange(o.copy(playlistItems = it))
            }
        }

        Row(
            Modifier.fillMaxWidth().clickable { onChange(o.copy(embedMeta = !o.embedMeta)) }.padding(vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("썸네일·메타데이터 넣기", Modifier.weight(1f))
            Switch(checked = o.embedMeta, onCheckedChange = { onChange(o.copy(embedMeta = it)) })
        }

        SectionLabel("구간 자르기")
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Field(o.sectionStart, "시작", "처음", errors["sectionStart"], Modifier.weight(1f)) {
                onChange(o.copy(sectionStart = it))
            }
            Field(o.sectionEnd, "끝", "끝까지", errors["sectionEnd"], Modifier.weight(1f)) {
                onChange(o.copy(sectionEnd = it))
            }
        }
        Text(
            "1:23, 1:02:03, 83(초) 처럼 적어요. 하나만 적어도 돼요.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun SectionLabel(text: String) = Text(
    text,
    style = MaterialTheme.typography.labelLarge,
    color = MaterialTheme.colorScheme.onSurfaceVariant,
    modifier = Modifier.padding(top = 8.dp),
)

@Composable
private fun Field(
    value: String,
    label: String,
    placeholder: String,
    error: String?,
    modifier: Modifier,
    onChange: (String) -> Unit,
) = OutlinedTextField(
    value = value,
    onValueChange = onChange,
    modifier = modifier,
    label = { Text(label) },
    placeholder = { Text(placeholder) },
    singleLine = true,
    isError = error != null,
    supportingText = error?.let { { Text(it) } },
    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii, imeAction = ImeAction.Done),
)

@Composable
private fun Banner(text: String) = Surface(
    color = MaterialTheme.colorScheme.errorContainer,
    contentColor = MaterialTheme.colorScheme.onErrorContainer,
    shape = MaterialTheme.shapes.medium,
    modifier = Modifier.fillMaxWidth(),
) {
    Text(text, Modifier.padding(16.dp))
}

@Composable
private fun JobCard(job: Job, onOpen: () -> Unit, onCancel: () -> Unit) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow),
    ) {
        Row(
            Modifier
                .clickable(enabled = job.state == JobState.DONE, onClick = onOpen)
                .padding(start = 16.dp, end = 4.dp, top = 12.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f).padding(end = 12.dp)) {
                Text(job.title, style = MaterialTheme.typography.bodyLarge, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Text(
                    "${describe(job.options)} · ${job.status}",
                    style = MaterialTheme.typography.bodySmall,
                    color = if (job.state == JobState.FAILED) MaterialTheme.colorScheme.error
                    else MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 2.dp),
                )
                if (job.state == JobState.RUNNING) {
                    Spacer(Modifier.height(8.dp))
                    // 영상 정보를 불러오는 동안은 얼마나 남았는지 모른다
                    if (job.percent > 0f) {
                        LinearProgressIndicator(progress = { job.percent / 100f }, modifier = Modifier.fillMaxWidth())
                    } else {
                        LinearProgressIndicator(Modifier.fillMaxWidth())
                    }
                }
            }
            if (job.active) {
                IconButton(onClick = onCancel) {
                    Icon(painterResource(R.drawable.ic_close), contentDescription = "취소")
                }
            }
        }
    }
}
