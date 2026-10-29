package com.glglekdy.quickmedia.preview

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.job
import kotlinx.coroutines.launch

sealed interface PreviewState {
    data object Loading : PreviewState
    data class Ready(val preview: Preview) : PreviewState
    data object Failed : PreviewState
}

/**
 * 링크별 미리보기. 받은 정보는 프로세스가 사는 동안 기억하고, 실패는 다음에 다시 시도한다.
 * 받기 칸이 다른 링크로 넘어가면 보던 링크의 불러오기를 버린다. 다운로드에 쓰인 링크(pin)는 목록에 쓰려고 끝까지 받는다.
 */
class PreviewStore(private val scope: CoroutineScope, private val fetch: suspend (url: String) -> Preview?) {
    private val _states = MutableStateFlow<Map<String, PreviewState>>(emptyMap())
    val states: StateFlow<Map<String, PreviewState>> = _states.asStateFlow()

    private val lock = Any()
    private val jobs = mutableMapOf<String, Job>()
    private val pinned = mutableSetOf<String>()
    private var focused: String? = null

    fun load(url: String) {
        synchronized(lock) { start(url) }
    }

    private fun start(url: String) {
        val now = _states.value[url]
        if (now is PreviewState.Ready || now == PreviewState.Loading) return
        _states.update { it + (url to PreviewState.Loading) }
        // 시작 전에 jobs 에 넣어야 끝날 때 자기 것인지 알 수 있다
        val job = scope.launch(start = CoroutineStart.LAZY) {
            val result = try {
                fetch(url)?.let { PreviewState.Ready(it) } ?: PreviewState.Failed
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                PreviewState.Failed
            }
            finish(url, coroutineContext.job, result)
        }
        jobs[url] = job
        job.start()
    }

    fun focus(url: String?) {
        synchronized(lock) { refocus(url) }
    }

    private fun refocus(url: String?) {
        val before = focused
        focused = url
        if (before == null || before == url || before in pinned) return
        if (_states.value[before] != PreviewState.Loading) return
        jobs.remove(before)?.cancel()
        _states.update { it - before }
    }

    fun pin(url: String) {
        synchronized(lock) { pinned += url }
        load(url)
    }

    private fun finish(url: String, self: Job, result: PreviewState) = synchronized(lock) {
        // 취소된 뒤 다시 불러온 경우 옛 결과로 덮지 않는다
        if (jobs[url] === self) {
            jobs.remove(url)
            _states.update { it + (url to result) }
        }
    }
}
