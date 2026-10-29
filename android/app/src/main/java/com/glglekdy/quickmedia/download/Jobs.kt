package com.glglekdy.quickmedia.download

import com.glglekdy.quickmedia.options.DownloadOptions
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import java.util.concurrent.atomic.AtomicLong

enum class JobState { QUEUED, RUNNING, DONE, FAILED, CANCELLED }

data class Job(
    val id: Long,
    val url: String,
    val options: DownloadOptions,
    val title: String = url,
    val state: JobState = JobState.QUEUED,
    val percent: Float = 0f,
    val status: String = "대기 중",
    /** 다운로드 폴더에 저장한 파일의 content:// 주소 */
    val saved: List<String> = emptyList(),
)

/** 다운로드 작업 목록. 화면과 다운로드 서비스가 같이 본다. 앱 프로세스가 살아 있는 동안만 남는다. */
object Jobs {
    private val nextId = AtomicLong(1)
    private val _list = MutableStateFlow<List<Job>>(emptyList())

    /** 새 작업이 맨 위 */
    val list: StateFlow<List<Job>> = _list.asStateFlow()

    fun add(url: String, o: DownloadOptions): Job =
        Job(nextId.getAndIncrement(), url, o).also { job -> _list.update { listOf(job) + it } }

    fun update(id: Long, f: (Job) -> Job) = _list.update { jobs -> jobs.map { if (it.id == id) f(it) else it } }

    fun get(id: Long): Job? = _list.value.firstOrNull { it.id == id }

    /** 먼저 넣은 것부터 */
    fun nextQueued(): Job? = _list.value.lastOrNull { it.state == JobState.QUEUED }

    /** 대기 중이거나 받는 중일 때만 취소된다 */
    fun cancel(id: Long): Boolean {
        var cancelled = false
        update(id) {
            if (it.state == JobState.QUEUED || it.state == JobState.RUNNING) {
                cancelled = true
                it.copy(state = JobState.CANCELLED, status = "취소됨")
            } else it
        }
        return cancelled
    }

    /** 실패·취소한 작업을 같은 링크·옵션으로 대기열에 다시 넣는다. 목록 자리는 그대로. */
    fun retry(id: Long): Boolean {
        var queued = false
        update(id) {
            if (it.state == JobState.FAILED || it.state == JobState.CANCELLED) {
                queued = true
                it.copy(state = JobState.QUEUED, percent = 0f, status = "대기 중", saved = emptyList())
            } else it
        }
        return queued
    }

    fun clearFinished() = _list.update { jobs -> jobs.filter { it.state == JobState.QUEUED || it.state == JobState.RUNNING } }

    /** 테스트용 */
    fun reset() {
        _list.value = emptyList()
        nextId.set(1)
    }
}

private val DESTINATION = Regex("""^\[download] Destination: (.+)$""")
private val ITEM = Regex("""^\[download] Downloading item (\d+) of (\d+)""")

// 라이브러리는 모든 줄에 직전 진행률을 붙여 넘긴다. 이 줄일 때만 값이 새것이다.
private val PROGRESS_LINE = Regex("""^\[download]\s+\d+(\.\d+)?%""")

// 받은 뒤 ffmpeg 로 합치거나 바꾸는 단계
private val POST_PROCESSING =
    Regex("""^\[(Merger|ExtractAudio|EmbedThumbnail|Metadata|Fixup\w*|VideoConvertor|VideoRemuxer|ThumbnailsConvertor|ModifyChapters)]""")

/** `[download] Destination: …/제목.f137.mp4` → `제목`. 다른 줄이면 null. */
fun titleFromLine(line: String): String? {
    val path = DESTINATION.find(line.trim())?.groupValues?.get(1) ?: return null
    return path.substringAfterLast('/').substringBeforeLast('.').replace(Regex("""\.f[\w-]+$"""), "")
}

/** yt-dlp 출력 줄과 진행률로 화면에 보일 (퍼센트, 문구). 영상·음성을 따로 받아도 막대가 뒤로 가지 않는다. */
class JobProgress {
    private var item = ""
    private var best = 0f

    var title: String? = null
        private set

    fun on(progress: Float, line: String): Pair<Float, String> {
        titleFromLine(line)?.let { title = it }
        ITEM.find(line)?.let { m ->
            item = "(${m.groupValues[1]}/${m.groupValues[2]}) "
            best = 0f
            return best to "받는 중 $item${best.toInt()}%"
        }
        if (POST_PROCESSING.containsMatchIn(line)) return best to "파일 정리 중…"
        if (PROGRESS_LINE.containsMatchIn(line) && progress > best) best = progress
        return best to "받는 중 $item${best.toInt()}%"
    }
}
