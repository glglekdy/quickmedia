package com.glglekdy.quickmedia.ui.list

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.glglekdy.quickmedia.R
import com.glglekdy.quickmedia.download.DownloadService
import com.glglekdy.quickmedia.download.Jobs
import com.glglekdy.quickmedia.preview.PreviewState
import com.glglekdy.quickmedia.preview.Previews
import com.glglekdy.quickmedia.ui.components.Chip
import com.glglekdy.quickmedia.ui.theme.Qm

/** 목록 탭. 작업 목록은 앱이 켜져 있는 동안만 남는다. */
@Composable
fun ListScreen(onGoReceive: () -> Unit, say: (String) -> Unit, contentPadding: PaddingValues) {
    val c = Qm.colors
    val context = LocalContext.current
    val jobs by Jobs.list.collectAsStateWithLifecycle()
    val previews by Previews.states.collectAsStateWithLifecycle()
    var filter by rememberSaveable { mutableStateOf(JobFilter.ALL) }
    val counts = jobCounts(jobs)
    val groups = jobGroups(jobs, filter)

    LazyColumn(
        Modifier.fillMaxSize(),
        contentPadding = PaddingValues(
            start = 18.dp,
            end = 18.dp,
            top = contentPadding.calculateTopPadding(),
            bottom = contentPadding.calculateBottomPadding(),
        ),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        item("head") {
            Row(Modifier.fillMaxWidth().padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("목록", color = c.text, fontSize = 24.sp, fontWeight = FontWeight.ExtraBold, modifier = Modifier.weight(1f))
                val finished = counts.getValue(JobFilter.ALL) > counts.getValue(JobFilter.ACTIVE)
                Text(
                    "끝난 항목 지우기",
                    color = if (finished) c.textDim else c.textDim.copy(alpha = 0.4f),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold,
                    modifier = Modifier
                        .clip(CircleShape)
                        .clickable(enabled = finished, onClick = Jobs::clearFinished)
                        .padding(horizontal = 8.dp, vertical = 6.dp),
                )
            }
        }
        item("filters") {
            // 좁은 폰에서 숫자가 커지면 넘치니 옆으로 밀리게
            Row(
                Modifier.horizontalScroll(rememberScrollState()).padding(top = 10.dp, bottom = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                JobFilter.entries.forEach { f ->
                    Chip("${f.label} ${counts.getValue(f)}", f == filter, { filter = f }, inverse = true)
                }
            }
        }
        if (groups.isEmpty()) {
            item("empty") { Empty(filter, onGoReceive, Modifier.animateItem()) }
        }
        groups.forEach { group ->
            group.title?.let { title ->
                item("group-$title") {
                    Text(
                        title,
                        color = c.textDim,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.animateItem().padding(top = 14.dp, bottom = 2.dp),
                    )
                }
            }
            items(group.jobs, key = { it.id }) { job ->
                // 새로 들어오고, 지워지고, 받는 중 → 끝남 으로 옮길 때 미끄러지며 자리를 잡는다
                JobRow(
                    job,
                    (previews[job.url] as? PreviewState.Ready)?.preview,
                    onCancel = { DownloadService.cancel(job.id) },
                    onOpen = { openJob(context, job)?.let(say) },
                    onRetry = {
                        if (Jobs.retry(job.id)) {
                            DownloadService.start(context)
                            Previews.load(job.url)
                        }
                    },
                    modifier = Modifier.animateItem(),
                )
            }
        }
    }
}

@Composable
private fun Empty(filter: JobFilter, onGoReceive: () -> Unit, modifier: Modifier) {
    val c = Qm.colors
    Column(
        modifier.fillMaxWidth().padding(top = 80.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Icon(painterResource(R.drawable.ic_list), contentDescription = null, tint = c.textDim, modifier = Modifier.size(40.dp))
        Text(
            if (filter == JobFilter.ALL) "아직 받은 게 없어요" else "해당하는 항목이 없어요",
            color = c.textDim,
            fontSize = 15.sp,
        )
        if (filter == JobFilter.ALL) {
            Text(
                "받기로 가기",
                color = c.text,
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier
                    .clip(CircleShape)
                    .border(1.dp, c.textDim.copy(alpha = 0.5f), CircleShape)
                    .clickable(onClick = onGoReceive)
                    .padding(horizontal = 18.dp, vertical = 10.dp),
            )
        }
    }
}
