package com.glglekdy.quickmedia.preview

import com.glglekdy.quickmedia.App
import com.glglekdy.quickmedia.Tools
import com.yausername.youtubedl_android.YoutubeDL
import com.yausername.youtubedl_android.YoutubeDLRequest
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.withContext
import java.util.concurrent.atomic.AtomicLong

/** 앱 전체의 미리보기 하나. 받기 탭이 불러오고 목록 탭이 썸네일·제목을 가져다 쓴다. */
object Previews {
    private val nextId = AtomicLong(1)
    private val store = PreviewStore(App.scope, ::fetchWithYtdlp)

    val states get() = store.states

    fun load(url: String) = store.load(url)

    fun focus(url: String?) = store.focus(url)

    fun pin(url: String) = store.pin(url)

    /** 파일은 받지 않고 정보만. 재생목록은 항목을 펼치지 않아 빨리 끝난다. */
    private suspend fun fetchWithYtdlp(url: String): Preview? {
        if (App.tools.first { it !is Tools.Loading } != Tools.Ready) return null
        val id = "preview-${nextId.getAndIncrement()}"
        // 링크가 바뀌어 취소되면 돌고 있는 파이썬도 끝낸다
        currentCoroutineContext().job.invokeOnCompletion {
            if (it is CancellationException) YoutubeDL.getInstance().destroyProcessById(id)
        }
        return withContext(Dispatchers.IO) {
            val request = YoutubeDLRequest(url)
                .addOption("-J")
                .addOption("--flat-playlist")
                .addOption("--no-playlist")
                .addOption("--no-warnings")
            try {
                Preview.parse(YoutubeDL.getInstance().execute(request, id).out)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                null
            }
        }
    }
}
