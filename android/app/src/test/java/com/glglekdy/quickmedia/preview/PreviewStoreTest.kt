package com.glglekdy.quickmedia.preview

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.job
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class PreviewStoreTest {
    private val p = Preview("t", null, null, null, null, false, null)

    /** 링크마다 결과를 손으로 끝내는 가짜 fetch */
    private class FakeFetch {
        val calls = mutableListOf<String>()
        val pending = mutableMapOf<String, CompletableDeferred<Preview?>>()
        val jobs = mutableMapOf<String, Job>()
        var throwOn: String? = null

        suspend fun fetch(url: String): Preview? {
            calls += url
            jobs[url] = currentCoroutineContext().job
            if (url == throwOn) throw IllegalStateException("boom")
            return CompletableDeferred<Preview?>().also { pending[url] = it }.await()
        }
    }

    private fun TestScope.store(f: FakeFetch) = PreviewStore(backgroundScope, f::fetch)

    @Test fun loadsOnceAndCachesReady() = runTest {
        val f = FakeFetch()
        val s = store(f)
        s.load("a"); s.load("a"); runCurrent()
        assertEquals(PreviewState.Loading, s.states.value["a"])
        f.pending.getValue("a").complete(p); runCurrent()
        assertEquals(PreviewState.Ready(p), s.states.value["a"])
        s.load("a"); runCurrent()
        assertEquals(1, f.calls.size)
    }

    @Test fun failedNotCached() = runTest {
        val f = FakeFetch()
        val s = store(f)
        s.load("a"); runCurrent()
        f.pending.getValue("a").complete(null); runCurrent()
        assertEquals(PreviewState.Failed, s.states.value["a"])
        s.load("a"); runCurrent()
        assertEquals(2, f.calls.size)
    }

    @Test fun focusChangeCancelsUnpinned() = runTest {
        val f = FakeFetch()
        val s = store(f)
        s.focus("a"); s.load("a"); runCurrent()
        s.focus("b"); runCurrent()
        assertTrue(f.jobs.getValue("a").isCancelled)
        assertFalse("a" in s.states.value)
    }

    @Test fun pinnedSurvivesFocusChange() = runTest {
        val f = FakeFetch()
        val s = store(f)
        s.focus("a"); s.load("a"); runCurrent()
        s.pin("a"); s.focus(null); runCurrent()
        assertEquals(PreviewState.Loading, s.states.value["a"])
        f.pending.getValue("a").complete(p); runCurrent()
        assertEquals(PreviewState.Ready(p), s.states.value["a"])
    }

    @Test fun pinReloadsFailed() = runTest {
        val f = FakeFetch()
        val s = store(f)
        s.load("a"); runCurrent()
        f.pending.getValue("a").complete(null); runCurrent()
        s.pin("a"); runCurrent()
        assertEquals(2, f.calls.size)
        f.pending.getValue("a").complete(p); runCurrent()
        assertEquals(PreviewState.Ready(p), s.states.value["a"])
    }

    @Test fun fetchExceptionIsFailed() = runTest {
        val f = FakeFetch().apply { throwOn = "a" }
        val s = store(f)
        s.load("a"); runCurrent()
        assertEquals(PreviewState.Failed, s.states.value["a"])
    }
}
