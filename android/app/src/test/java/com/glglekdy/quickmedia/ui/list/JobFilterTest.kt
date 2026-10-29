package com.glglekdy.quickmedia.ui.list

import com.glglekdy.quickmedia.download.Job
import com.glglekdy.quickmedia.download.JobState
import com.glglekdy.quickmedia.options.DownloadOptions
import org.junit.Assert.assertEquals
import org.junit.Test

class JobFilterTest {
    private fun job(id: Long, state: JobState) = Job(id, "u$id", DownloadOptions(), state = state)

    private val q = job(1, JobState.QUEUED)
    private val r = job(2, JobState.RUNNING)
    private val d = job(3, JobState.DONE)
    private val f = job(4, JobState.FAILED)
    private val c = job(5, JobState.CANCELLED)
    private val all = listOf(q, r, d, f, c)

    @Test fun countsByFilter() = assertEquals(
        mapOf(JobFilter.ALL to 5, JobFilter.ACTIVE to 2, JobFilter.DONE to 1, JobFilter.FAILED to 2),
        jobCounts(all),
    )

    @Test fun allSplitsIntoTwoGroups() = assertEquals(
        listOf(JobGroup("받는 중", listOf(q, r)), JobGroup("끝남", listOf(d, f, c))),
        jobGroups(all, JobFilter.ALL),
    )

    @Test fun emptyGroupHidden() = assertEquals(listOf(JobGroup("끝남", listOf(d))), jobGroups(listOf(d), JobFilter.ALL))

    @Test fun filteredSingleUntitledGroup() {
        assertEquals(listOf(JobGroup(null, listOf(f, c))), jobGroups(all, JobFilter.FAILED))
        assertEquals(emptyList<JobGroup>(), jobGroups(listOf(d), JobFilter.FAILED))
    }
}
