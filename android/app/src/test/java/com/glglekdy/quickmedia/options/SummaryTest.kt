package com.glglekdy.quickmedia.options

import org.junit.Assert.assertEquals
import org.junit.Test

class SummaryTest {
    @Test fun describeVideo() {
        assertEquals("최고 화질 · MP4", describe(DownloadOptions()))
        assertEquals("1080p · MP4", describe(DownloadOptions(quality = "1080")))
    }

    @Test fun describeAudio() = assertEquals("M4A", describe(DownloadOptions(kind = Kind.AUDIO, audioFormat = "m4a")))

    @Test fun qualityLabels() = assertEquals(
        listOf("최고", "1080p", "720p", "480p", "MP3", "M4A"),
        QUALITIES.map(::qualityLabel) + AUDIO_FORMATS.map(::qualityLabel),
    )

    @Test fun nothingChangedByDefault() = assertEquals(emptyList<String>(), changedSummary(DownloadOptions()))

    // 접힌 "옵션 더보기" 옆에 보인다. 구간 자르기가 켜진 채 잊고 받는 일이 없게.
    @Test fun changedOptionsAreListed() = assertEquals(
        listOf("재생목록", "썸네일·메타데이터", "구간"),
        changedSummary(DownloadOptions(playlist = PlaylistMode.ALL, embedMeta = true, sectionEnd = "0:10")),
    )

    // 일부 항목을 골랐어도 번호를 안 적었으면 바뀐 게 없다
    @Test fun emptyItemsAreNotAChange() =
        assertEquals(emptyList<String>(), changedSummary(DownloadOptions(playlist = PlaylistMode.ITEMS)))
}
