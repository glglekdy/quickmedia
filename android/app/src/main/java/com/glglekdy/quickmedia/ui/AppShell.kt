package com.glglekdy.quickmedia.ui

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.glglekdy.quickmedia.download.Jobs
import com.glglekdy.quickmedia.ui.components.Dock
import com.glglekdy.quickmedia.ui.list.JobFilter
import com.glglekdy.quickmedia.ui.list.ListScreen
import com.glglekdy.quickmedia.ui.list.jobCounts
import com.glglekdy.quickmedia.ui.receive.ReceiveScreen
import com.glglekdy.quickmedia.ui.settings.SettingsScreen
import com.glglekdy.quickmedia.ui.theme.Motion
import com.glglekdy.quickmedia.ui.theme.Qm
import com.glglekdy.quickmedia.update.Release
import com.glglekdy.quickmedia.update.UpdateException
import com.glglekdy.quickmedia.update.Updater
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.launch

// 떠 있는 메뉴 높이(약 58dp) + 화면 아래에서 띄운 18dp
private val DockSpace = 76.dp

/** 세 탭과 떠 있는 메뉴, 스낵바, 공유로 받은 링크, 켤 때 업데이트 확인. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun AppShell(shared: ReceiveChannel<String?>, launched: Boolean) {
    val c = Qm.colors
    val scope = rememberCoroutineScope()
    var tab by rememberSaveable { mutableStateOf(Tab.RECEIVE) }
    var link by rememberSaveable { mutableStateOf("") }
    var instant by rememberSaveable { mutableStateOf(false) }
    var update by rememberSaveable { mutableStateOf<Release?>(null) }
    val snackbar = remember { SnackbarHostState() }
    val hazeState = rememberHazeState()
    val jobs by Jobs.list.collectAsStateWithLifecycle()

    // 켤 때 한 번. 확인하지 못해도 아무것도 띄우지 않는다
    LaunchedEffect(Unit) {
        if (launched) update = try { Updater.check() } catch (e: UpdateException) { null }
    }
    LaunchedEffect(Unit) {
        for (found in shared) {
            tab = Tab.RECEIVE
            if (found != null) {
                link = found
                instant = true
            } else {
                launch { snackbar.showSnackbar("링크를 찾지 못했어요") }
            }
        }
    }
    BackHandler(enabled = tab != Tab.RECEIVE) { tab = Tab.RECEIVE }
    // 목록으로 직접 가면 "목록에 넣었어요 · 보기" 는 더 필요 없다
    LaunchedEffect(tab) {
        if (tab == Tab.LIST) snackbar.currentSnackbarData?.dismiss()
    }

    fun say(text: String) {
        scope.launch { snackbar.showSnackbar(text) }
    }

    fun queued() {
        scope.launch {
            snackbar.currentSnackbarData?.dismiss()
            // 동작 버튼이 있으면 기본값이 "닫을 때까지"라서 짧게로 정해 준다
            val result = snackbar.showSnackbar("목록에 넣었어요", actionLabel = "보기", duration = SnackbarDuration.Short)
            if (result == SnackbarResult.ActionPerformed) tab = Tab.LIST
        }
    }

    val keyboard = WindowInsets.isImeVisible
    val top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding() + 8.dp
    val bottom = if (keyboard) {
        WindowInsets.ime.asPaddingValues().calculateBottomPadding() + 16.dp
    } else {
        WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding() + DockSpace + 32.dp
    }
    val padding = PaddingValues(top = top, bottom = bottom)

    Box(Modifier.fillMaxSize().background(c.bg)) {
        Box(Modifier.fillMaxSize().hazeSource(hazeState)) {
            AnimatedContent(
                tab,
                transitionSpec = {
                    // 메뉴에서 오른쪽 탭으로 가면 새 화면이 오른쪽에서 살짝 밀려 들어온다
                    val dir = if (targetState.ordinal > initialState.ordinal) 1 else -1
                    val spec = tween<IntOffset>(Motion.TAB_MS, easing = FastOutSlowInEasing)
                    (fadeIn(tween(Motion.TAB_MS)) + slideInHorizontally(spec) { dir * it / 10 }) togetherWith
                        (fadeOut(tween(Motion.TAB_MS / 2)) + slideOutHorizontally(spec) { -dir * it / 10 })
                },
                label = "tab",
            ) { shown ->
                when (shown) {
                    Tab.RECEIVE -> ReceiveScreen(
                        link = link,
                        instant = instant,
                        onLinkChange = { text, now ->
                            link = text
                            instant = now
                        },
                        say = ::say,
                        onQueued = ::queued,
                        contentPadding = padding,
                    )
                    Tab.LIST -> ListScreen(onGoReceive = { tab = Tab.RECEIVE }, say = ::say, contentPadding = padding)
                    Tab.SETTINGS -> SettingsScreen(onNewVersion = { update = it }, say = ::say, contentPadding = padding)
                }
            }
        }

        AnimatedVisibility(!keyboard, Modifier.align(Alignment.BottomCenter), enter = fadeIn(), exit = fadeOut()) {
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.BottomCenter) {
                // 메뉴 뒤로 내용이 바탕색에 서서히 묻히게
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(110.dp + WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding())
                        .background(Brush.verticalGradient(listOf(Color.Transparent, c.bg.copy(alpha = 0.9f)))),
                )
                Dock(
                    selected = tab,
                    onSelect = { tab = it },
                    badge = jobCounts(jobs).getValue(JobFilter.ACTIVE),
                    hazeState = hazeState,
                    modifier = Modifier.navigationBarsPadding().padding(bottom = 18.dp),
                )
            }
        }

        Column(
            Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = bottom - 16.dp)
                .padding(horizontal = 16.dp),
        ) {
            SnackbarHost(snackbar) { data ->
                Snackbar(
                    data,
                    shape = RoundedCornerShape(16.dp),
                    containerColor = if (c.isDark) c.fieldOn else c.text,
                    contentColor = if (c.isDark) c.text else c.bg,
                    actionColor = c.accentText,
                )
            }
        }

        update?.let { UpdateDialog(it) { update = null } }
    }
}
