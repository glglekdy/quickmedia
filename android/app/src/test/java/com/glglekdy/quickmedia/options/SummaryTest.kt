package com.glglekdy.quickmedia.options

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
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

    @Test fun summaryNullByDefault() {
        assertNull(optionsSummary(DownloadOptions()))
        // 일부 항목을 고른 것 자체가 바꾼 것이다. 번호를 안 적었으면 받을 때 검사에 걸린다
        assertEquals("재생목록 일부", optionsSummary(DownloadOptions(playlist = PlaylistMode.ITEMS)))
    }

    @Test fun summaryEach() {
        assertEquals("전체 재생목록", optionsSummary(DownloadOptions(playlist = PlaylistMode.ALL)))
        assertEquals("재생목록 1-5,8", optionsSummary(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = " 1-5,8 ")))
        assertEquals("메타데이터", optionsSummary(DownloadOptions(embedMeta = true)))
        assertEquals("1:23부터", optionsSummary(DownloadOptions(sectionStart = "1:23")))
        assertEquals("1:23까지", optionsSummary(DownloadOptions(sectionEnd = "1:23")))
        assertEquals("1:23~2:00", optionsSummary(DownloadOptions(sectionStart = "1:23", sectionEnd = "2:00")))
    }

    @Test fun summaryJoined() = assertEquals(
        "전체 재생목록 · 메타데이터 · 1:23부터",
        optionsSummary(DownloadOptions(playlist = PlaylistMode.ALL, embedMeta = true, sectionStart = "1:23")),
    )
}
