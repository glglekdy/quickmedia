package com.glglekdy.quickmedia.ui.list

import com.glglekdy.quickmedia.download.Job
import com.glglekdy.quickmedia.download.JobState

/** 목록 탭 위쪽 칩. 받는 중에는 대기 중이, 실패에는 취소한 것이 들어간다. */
enum class JobFilter(val label: String) {
    ALL("전체"),
    ACTIVE("받는 중"),
    DONE("완료"),
    FAILED("실패");

    fun matches(job: Job): Boolean = when (this) {
        ALL -> true
        ACTIVE -> job.state == JobState.QUEUED || job.state == JobState.RUNNING
        DONE -> job.state == JobState.DONE
        FAILED -> job.state == JobState.FAILED || job.state == JobState.CANCELLED
    }
}

fun jobCounts(jobs: List<Job>): Map<JobFilter, Int> = JobFilter.entries.associateWith { f -> jobs.count(f::matches) }

/** 머리글(null 이면 없음)과 그 아래 작업들 */
data class JobGroup(val title: String?, val jobs: List<Job>)

/** 전체는 받는 중·끝남 두 묶음, 나머지는 머리글 없는 한 묶음. 빈 묶음은 뺀다. */
fun jobGroups(jobs: List<Job>, filter: JobFilter): List<JobGroup> =
    if (filter == JobFilter.ALL) {
        val (active, finished) = jobs.partition(JobFilter.ACTIVE::matches)
        listOf(JobGroup("받는 중", active), JobGroup("끝남", finished))
    } else {
        listOf(JobGroup(null, jobs.filter(filter::matches)))
    }.filter { it.jobs.isNotEmpty() }
