package com.glglekdy.quickmedia

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import com.glglekdy.quickmedia.download.extractLink
import com.glglekdy.quickmedia.ui.MainScreen
import com.glglekdy.quickmedia.ui.theme.QuickMediaTheme
import com.glglekdy.quickmedia.ui.SettingsScreen
import com.glglekdy.quickmedia.ui.UpdateDialog
import com.glglekdy.quickmedia.update.Release
import com.glglekdy.quickmedia.update.UpdateException
import com.glglekdy.quickmedia.update.Updater
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    /** 공유로 받은 글에서 뽑은 링크. 링크가 없는 글이면 null. */
    private val shared = Channel<String?>(Channel.UNLIMITED)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        // 화면을 돌려 다시 만들어질 때는 같은 공유를 또 받지 않는다
        if (savedInstanceState == null) receiveShare(intent)
        val launched = savedInstanceState == null
        setContent {
            QuickMediaTheme {
                var settings by rememberSaveable { mutableStateOf(false) }
                var link by rememberSaveable { mutableStateOf("") }
                val snackbar = remember { SnackbarHostState() }
                var update by rememberSaveable { mutableStateOf<Release?>(null) }
                // 켤 때 한 번. 확인하지 못해도 아무것도 띄우지 않는다
                LaunchedEffect(Unit) {
                    if (launched) update = try { Updater.check() } catch (e: UpdateException) { null }
                }
                LaunchedEffect(Unit) {
                    for (found in shared) {
                        settings = false
                        if (found != null) link = found
                        else launch { snackbar.showSnackbar("링크를 찾지 못했어요") }
                    }
                }
                if (settings) {
                    BackHandler { settings = false }
                    SettingsScreen(onBack = { settings = false }, onNewVersion = { update = it })
                } else {
                    MainScreen(link, { link = it }, snackbar, onOpenSettings = { settings = true })
                }
                update?.let { UpdateDialog(it, onDismiss = { update = null }) }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        receiveShare(intent)
    }

    private fun receiveShare(intent: Intent?) {
        if (intent?.action == Intent.ACTION_SEND) shared.trySend(extractLink(intent.getStringExtra(Intent.EXTRA_TEXT)))
    }
}
