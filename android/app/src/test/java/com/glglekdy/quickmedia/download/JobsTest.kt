package com.glglekdy.quickmedia.download

import com.glglekdy.quickmedia.options.DownloadOptions
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class JobsTest {
    @Before fun setUp() = Jobs.reset()

    @Test fun newestFirstAndQueueOrder() {
        val a = Jobs.add("u1", DownloadOptions())
        val b = Jobs.add("u2", DownloadOptions())
        assertEquals(listOf(b.id, a.id), Jobs.list.value.map { it.id })
        assertEquals(a.id, Jobs.nextQueued()?.id) // 먼저 넣은 것부터 받는다
    }

    @Test fun cancelOnlyUnfinished() {
        val a = Jobs.add("u", DownloadOptions())
        Jobs.update(a.id) { it.copy(state = JobState.DONE) }
        assertFalse(Jobs.cancel(a.id))
        val b = Jobs.add("u", DownloadOptions())
        assertTrue(Jobs.cancel(b.id))
        assertNull(Jobs.nextQueued())
    }

    @Test fun clearFinishedKeepsActive() {
        val a = Jobs.add("u", DownloadOptions())
        val b = Jobs.add("u", DownloadOptions())
        Jobs.update(a.id) { it.copy(state = JobState.FAILED) }
        Jobs.clearFinished()
        assertEquals(listOf(b.id), Jobs.list.value.map { it.id })
    }

    @Test fun retryFailedOrCancelled() {
        val a = Jobs.add("u1", DownloadOptions())
        val b = Jobs.add("u2", DownloadOptions())
        Jobs.update(a.id) { it.copy(state = JobState.FAILED, percent = 40f, status = "오류", saved = listOf("content://x")) }
        assertTrue(Jobs.retry(a.id))
        val again = Jobs.get(a.id)!!
        assertEquals(JobState.QUEUED, again.state)
        assertEquals(0f, again.percent)
        assertEquals("대기 중", again.status)
        assertEquals(emptyList<String>(), again.saved)
        assertEquals(listOf(b.id, a.id), Jobs.list.value.map { it.id }) // 자리는 그대로
        Jobs.cancel(b.id)
        assertTrue(Jobs.retry(b.id))
        assertEquals(JobState.QUEUED, Jobs.get(b.id)!!.state)
    }

    @Test fun retryIgnoresActiveAndDone() {
        val a = Jobs.add("u1", DownloadOptions())
        Jobs.update(a.id) { it.copy(state = JobState.RUNNING, percent = 30f) }
        assertFalse(Jobs.retry(a.id))
        assertEquals(30f, Jobs.get(a.id)!!.percent)
        Jobs.update(a.id) { it.copy(state = JobState.DONE) }
        assertFalse(Jobs.retry(a.id))
        assertEquals(JobState.DONE, Jobs.get(a.id)!!.state)
    }

    @Test fun titleFromDestination() = assertEquals(
        "Me at the zoo",
        titleFromLine("[download] Destination: /data/user/0/x/cache/dl/3/Me at the zoo.f137.mp4"),
    )

    @Test fun titleFromSingleFormatAndOtherLines() {
        assertEquals("a.b c", titleFromLine("[download] Destination: /x/a.b c.webm"))
        assertNull(titleFromLine("[youtube] abc: Downloading webpage"))
    }

    // 영상·음성을 따로 받으면 yt-dlp 진행률이 0 으로 돌아간다. 막대는 뒤로 가지 않는다.
    @Test fun progressNeverGoesBack() {
        val p = JobProgress()
        assertEquals(60f to "받는 중 60%", p.on(60f, "[download]  60.0% of 10MiB at 1MiB/s ETA 00:04"))
        p.on(60f, "[download] Destination: /x/a.f140.m4a")
        assertEquals(60f to "받는 중 60%", p.on(5f, "[download]   5.0% of 1MiB at 1MiB/s ETA 00:01"))
    }

    @Test fun progressRestartsForEachPlaylistItem() {
        val p = JobProgress()
        p.on(90f, "[download]  90.0% of 1MiB at 1MiB/s ETA 00:00")
        assertEquals(0f to "받는 중 (2/5) 0%", p.on(90f, "[download] Downloading item 2 of 5"))
        assertEquals(12.5f to "받는 중 (2/5) 12%", p.on(12.5f, "[download]  12.5% of 1MiB at 1MiB/s ETA 00:01"))
    }

    // 라이브러리는 진행률 줄이 아니어도 직전 진행률을 그대로 넘긴다. 진행률 줄의 값만 믿는다.
    @Test fun progressOnlyFromProgressLines() {
        val p = JobProgress()
        p.on(90f, "[download]  90.0% of 1MiB at 1MiB/s ETA 00:00")
        p.on(90f, "[download] Downloading item 2 of 5")
        assertEquals(0f to "받는 중 (2/5) 0%", p.on(90f, "[youtube] abc: Downloading webpage"))
    }

    @Test fun postProcessingStatus() {
        val p = JobProgress()
        p.on(100f, "[download] 100% of 10MiB")
        assertEquals(100f to "파일 정리 중…", p.on(100f, "[Merger] Merging formats into \"/x/a.mp4\""))
        assertEquals(100f to "파일 정리 중…", p.on(100f, "[ExtractAudio] Destination: /x/a.mp3"))
    }

    @Test fun titleFollowsDestination() {
        val p = JobProgress()
        p.on(-1f, "[download] Destination: /x/Me at the zoo.f137.mp4")
        assertEquals("Me at the zoo", p.title)
    }
}
