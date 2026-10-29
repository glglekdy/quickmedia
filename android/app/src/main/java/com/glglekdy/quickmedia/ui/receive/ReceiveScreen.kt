package com.glglekdy.quickmedia.ui.receive

import android.Manifest
import android.content.ClipboardManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import androidx.core.content.edit
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.glglekdy.quickmedia.App
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.Tools
import com.glglekdy.quickmedia.download.DownloadService
import com.glglekdy.quickmedia.download.Jobs
import com.glglekdy.quickmedia.download.extractLink
import com.glglekdy.quickmedia.options.AUDIO_FORMATS
import com.glglekdy.quickmedia.options.DownloadOptions
import com.glglekdy.quickmedia.options.Kind
import com.glglekdy.quickmedia.options.OptionsStore
import com.glglekdy.quickmedia.options.QUALITIES
import com.glglekdy.quickmedia.options.YtdlpArgs
import com.glglekdy.quickmedia.options.optionsSummary
import com.glglekdy.quickmedia.options.qualityLabel
import com.glglekdy.quickmedia.preview.PreviewState
import com.glglekdy.quickmedia.preview.Previews
import com.glglekdy.quickmedia.ui.components.Chip
import com.glglekdy.quickmedia.ui.components.GradientButton
import com.glglekdy.quickmedia.ui.components.SegmentToggle
import com.glglekdy.quickmedia.ui.theme.Qm
import kotlinx.coroutines.delay

/**
 * 받기 탭: 링크 → 미리보기 → 영상/음성 → 화질 → 옵션 → 다운로드.
 * 링크 칸은 공유로도 채워져서 밖(AppShell)에서 갖고 있다. instant 는 붙여넣기·공유로 채워졌는지.
 */
@Composable
fun ReceiveScreen(
    link: String,
    instant: Boolean,
    onLinkChange: (text: String, instant: Boolean) -> Unit,
    say: (String) -> Unit,
    onQueued: () -> Unit,
    contentPadding: PaddingValues,
) {
    val c = Qm.colors
    val context = LocalContext.current
    val focus = LocalFocusManager.current
    val store = remember { OptionsStore(context) }
    // 고르는 즉시 저장한다. 화면을 돌려 다시 만들어져도 저장값에서 그대로 읽는다.
    var options by remember { mutableStateOf(store.load()) }
    var errors by remember { mutableStateOf(emptyMap<String, String>()) }
    var linkError by remember { mutableStateOf<String?>(null) }
    var sheet by rememberSaveable { mutableStateOf(false) }
    val tools by App.tools.collectAsStateWithLifecycle()
    val previews by Previews.states.collectAsStateWithLifecycle()
    val askNotifications = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}

    val url = extractLink(link)
    LaunchedEffect(url) {
        Previews.focus(url)
        if (url != null) {
            // 치는 중에는 멈출 때까지 기다린다
            if (!instant) delay(600)
            Previews.load(url)
        }
    }
    DisposableEffect(Unit) { onDispose { Previews.focus(null) } }
    // 공유로 채워질 때도 "먼저 링크를 넣어 주세요" 를 지운다
    LaunchedEffect(link) { if (link.isNotBlank()) linkError = null }

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
            onLinkChange(found, true)
            linkError = null
        }
    }

    fun download() {
        val target = url ?: link.trim()
        linkError = if (target.isEmpty()) "먼저 링크를 넣어 주세요" else null
        errors = YtdlpArgs.validate(options)
        if (errors.isNotEmpty()) sheet = true
        if (linkError != null || errors.isNotEmpty()) return
        askNotificationsOnce(context) { askNotifications.launch(Manifest.permission.POST_NOTIFICATIONS) }
        Jobs.add(target, options)
        DownloadService.start(context)
        // 링크 칸을 비워도 이 미리보기는 끝까지 받아 목록에 쓴다
        Previews.pin(target)
        onLinkChange("", false)
        focus.clearFocus()
        onQueued()
    }

    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 18.dp)
            .padding(top = contentPadding.calculateTopPadding(), bottom = contentPadding.calculateBottomPadding()),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(
            buildAnnotatedString {
                append("Quick")
                withStyle(SpanStyle(color = c.accent)) { append("Media") }
            },
            color = c.text,
            fontSize = 24.sp,
            fontWeight = FontWeight.ExtraBold,
            modifier = Modifier.padding(top = 8.dp),
        )

        (tools as? Tools.Failed)?.let { failed ->
            Text(
                failed.reason,
                color = c.bad,
                fontSize = 13.sp,
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(14.dp))
                    .background(c.bad.copy(alpha = 0.12f))
                    .padding(14.dp),
            )
        }

        QmField(
            value = link,
            onChange = {
                onLinkChange(it, false)
                linkError = null
            },
            placeholder = "영상·재생목록 링크를 붙여넣으세요",
            error = linkError,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Go),
            keyboardActions = KeyboardActions(onGo = { download() }),
            trailing = {
                if (link.isEmpty()) {
                    Text(
                        "붙여넣기",
                        color = c.accentText,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.clip(CircleShape).clickable(onClick = ::paste).padding(10.dp),
                    )
                } else {
                    Icon(
                        painterResource(R.drawable.ic_close),
                        contentDescription = "지우기",
                        tint = c.textDim,
                        modifier = Modifier
                            .clip(CircleShape)
                            .clickable { onLinkChange("", false) }
                            .padding(12.dp)
                            .size(16.dp),
                    )
                }
            },
        )

        if (url != null && tools !is Tools.Failed) {
            when (val state = previews[url]) {
                is PreviewState.Ready -> PreviewCard(state.preview)
                PreviewState.Failed -> PreviewFailed()
                PreviewState.Loading, null -> PreviewSkeleton()
            }
        }

        SegmentToggle(
            values = Kind.entries,
            selected = options.kind,
            label = { if (it == Kind.VIDEO) "영상" else "음성" },
            onSelect = { change(options.copy(kind = it)) },
        )
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            if (options.kind == Kind.VIDEO) {
                QUALITIES.forEach { q -> Chip(qualityLabel(q), q == options.quality, { change(options.copy(quality = q)) }) }
            } else {
                AUDIO_FORMATS.forEach { f ->
                    Chip(qualityLabel(f), f == options.audioFormat, { change(options.copy(audioFormat = f)) })
                }
            }
        }

        val summary = optionsSummary(options)
        Row(
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(c.field.copy(alpha = 0.6f))
                .clickable { sheet = true }
                .padding(horizontal = 14.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                summary ?: "재생목록 · 메타데이터 · 구간",
                color = if (summary != null) c.text else c.textDim,
                fontSize = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f).padding(end = 8.dp),
            )
            Text("옵션 ›", color = c.text, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
        }

        GradientButton("다운로드", ::download, Modifier.padding(top = 4.dp))
    }

    if (sheet) OptionsSheet(options, errors, ::change, onDismiss = { sheet = false })
}

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
