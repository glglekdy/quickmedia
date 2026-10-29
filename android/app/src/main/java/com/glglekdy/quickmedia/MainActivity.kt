package com.glglekdy.quickmedia

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.glglekdy.quickmedia.download.extractLink
import com.glglekdy.quickmedia.ui.AppShell
import com.glglekdy.quickmedia.ui.theme.QuickMediaTheme
import kotlinx.coroutines.channels.Channel

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
            QuickMediaTheme { AppShell(shared, launched) }
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
