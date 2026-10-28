package com.glglekdy.quickmedia.download

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.SystemClock
import com.glglekdy.quickmedia.App
import com.glglekdy.quickmedia.MainActivity
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.Tools
import com.glglekdy.quickmedia.options.YtdlpArgs
import com.yausername.youtubedl_android.YoutubeDL
import com.yausername.youtubedl_android.YoutubeDLException
import com.yausername.youtubedl_android.YoutubeDLRequest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/**
 * 대기열의 작업을 하나씩 받는 포그라운드 서비스. 앱을 나가도 계속 받고 알림에 진행률을 보인다.
 *
 * 대기열 순서는 메인 스레드에서만 정한다. 그래서 "남은 작업 없음 → 서비스 종료" 사이에
 * 새 작업이 끼어 대기 중으로 남는 일이 없다. 다운로드와 저장만 IO 스레드에서 한다.
 */
class DownloadService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private var worker: kotlinx.coroutines.Job? = null
    private var lastStartId = 0
    private var lastNotify = 0L
    private lateinit var notifications: NotificationManager

    override fun onBind(intent: Intent?) = null

    override fun onCreate() {
        super.onCreate()
        notifications = getSystemService(NotificationManager::class.java)
        notifications.createNotificationChannel(
            NotificationChannel(CHANNEL, "다운로드", NotificationManager.IMPORTANCE_LOW))
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        lastStartId = startId
        startForeground(NOTIFICATION_ID, notification("준비 중…", -1f), ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        if (worker?.isActive != true) worker = scope.launch { runQueue() }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        // 시스템이 서비스를 멈춘 경우. 받던 작업은 끝낼 수 없다.
        Jobs.list.value.filter { it.state == JobState.RUNNING }.forEach { job ->
            Jobs.update(job.id) { it.copy(state = JobState.FAILED, status = "중단됐어요") }
            App.scope.launch(Dispatchers.IO) { YoutubeDL.getInstance().destroyProcessById(job.id.toString()) }
        }
        scope.cancel()
        super.onDestroy()
    }

    private suspend fun runQueue() {
        val tools = App.tools.first { it !is Tools.Loading }
        while (true) {
            val job = Jobs.nextQueued() ?: break
            if (tools is Tools.Failed) {
                Jobs.update(job.id) { it.copy(state = JobState.FAILED, status = tools.reason) }
                continue
            }
            run(job)
        }
        // 여기서부터 멈출 때까지 일시 중단 지점이 없다(위 주석)
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf(lastStartId)
    }

    private suspend fun run(job: Job) {
        Jobs.update(job.id) {
            if (it.state == JobState.QUEUED) it.copy(state = JobState.RUNNING, status = "영상 정보 불러오는 중…") else it
        }
        if (Jobs.get(job.id)?.state != JobState.RUNNING) return
        show(job.title, "영상 정보 불러오는 중…", -1f, force = true)

        val dir = File(cacheDir, "dl/${job.id}")
        val progress = JobProgress()
        try {
            withContext(Dispatchers.IO) { dir.deleteRecursively(); dir.mkdirs() }
            val request = YoutubeDLRequest(job.url).addCommands(YtdlpArgs.build(job.options, dir.absolutePath))
            // 재생목록은 일부 항목이 없어도(삭제·비공개) 나머지를 받고 오류로 끝난다. 받은 건 저장한다.
            val error = withContext(Dispatchers.IO) {
                try {
                    YoutubeDL.getInstance().execute(request, job.id.toString()) { pct, _, line ->
                        onOutput(job.id, progress, pct, line)
                    }
                    null
                } catch (e: YoutubeDLException) {
                    e
                }
            }
            if (cancelled(job.id)) return

            val files = withContext(Dispatchers.IO) { resultFiles(dir) }
            val reason = error?.let { friendlyError(it.message.orEmpty()) }
            if (files.isEmpty()) {
                fail(job.id, reason ?: "받은 파일이 없어요")
                return
            }
            Jobs.update(job.id) { it.copy(status = "파일 정리 중…") }
            val saver = MediaSaver(this)
            val saved = withContext(Dispatchers.IO) { files.map { saver.save(it).toString() } }
            Jobs.update(job.id) {
                if (it.state != JobState.RUNNING) it
                else it.copy(state = JobState.DONE, percent = 100f, saved = saved,
                    status = if (reason == null) "완료" else "완료 · 일부 오류: $reason")
            }
        } catch (e: YoutubeDL.CanceledException) {
            // 취소 버튼이 이미 "취소됨" 으로 바꿔 두었다
        } catch (e: Exception) {
            if (!cancelled(job.id)) fail(job.id, friendlyError(e.message ?: e.toString()))
        } finally {
            withContext(NonCancellable + Dispatchers.IO) { dir.deleteRecursively() }
        }
    }

    /** 라이브러리의 출력 읽기 스레드에서 불린다 */
    private fun onOutput(id: Long, progress: JobProgress, pct: Float, line: String) {
        val (percent, status) = progress.on(pct, line)
        var shown: Job? = null
        Jobs.update(id) { job ->
            if (job.state != JobState.RUNNING) job
            else job.copy(percent = percent, status = status, title = progress.title ?: job.title).also { shown = it }
        }
        shown?.let { show(it.title, it.status, it.percent) }
    }

    private fun cancelled(id: Long) = Jobs.get(id)?.state == JobState.CANCELLED

    private fun fail(id: Long, reason: String) = Jobs.update(id) {
        if (it.state == JobState.RUNNING) it.copy(state = JobState.FAILED, status = reason) else it
    }

    private fun show(title: String, status: String, percent: Float, force: Boolean = false) {
        val now = SystemClock.elapsedRealtime()
        if (!force && now - lastNotify < 500) return
        lastNotify = now
        notifications.notify(NOTIFICATION_ID, notification("$title · $status", percent))
    }

    private fun notification(text: String, percent: Float): Notification =
        Notification.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle("QuickMedia")
            .setContentText(text)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setContentIntent(PendingIntent.getActivity(this, 0,
                Intent(this, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
                PendingIntent.FLAG_IMMUTABLE))
            .apply { if (percent >= 0) setProgress(100, percent.toInt(), false) else setProgress(0, 0, true) }
            .build()

    companion object {
        private const val CHANNEL = "downloads"
        private const val NOTIFICATION_ID = 1

        /** 작업을 [Jobs] 에 넣은 뒤 부른다 */
        fun start(context: Context) {
            context.startForegroundService(Intent(context, DownloadService::class.java))
        }

        fun cancel(id: Long) {
            if (Jobs.cancel(id)) {
                // 대기 중이면 프로세스가 없어 아무 일도 하지 않는다
                App.scope.launch(Dispatchers.IO) { YoutubeDL.getInstance().destroyProcessById(id.toString()) }
            }
        }
    }
}
