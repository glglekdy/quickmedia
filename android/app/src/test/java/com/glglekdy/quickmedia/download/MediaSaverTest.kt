package com.glglekdy.quickmedia.download

import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class MediaSaverTest {
    @get:Rule val tmp = TemporaryFolder()

    @Test fun numberedNameSkipsTakenNumbers() =
        assertEquals("Me (2).mp4", numberedName("Me.mp4") { it in setOf("Me.mp4", "Me (1).mp4") })

    @Test fun numberedNameKeepsFreeName() = assertEquals("Me.mp4", numberedName("Me.mp4") { false })

    @Test fun numberedNameUsesLastDot() = assertEquals("a.b (1).mp3", numberedName("a.b.mp3") { it == "a.b.mp3" })

    @Test fun numberedNameWithoutExtension() = assertEquals("noext (1)", numberedName("noext") { it == "noext" })

    @Test fun mime() {
        assertEquals("audio/mpeg", mimeOf("x.MP3"))
        assertEquals("video/mp4", mimeOf("a b.mp4"))
        assertEquals("audio/mp4", mimeOf("x.m4a"))
        assertEquals("application/octet-stream", mimeOf("x.bin"))
    }

    @Test fun resultFilesSkipsPartialFiles() {
        listOf("a.mp4", "a.f137.mp4.part", "b.mp3", "c.ytdl", "d.mp4.part-Frag3", "e.temp").forEach { tmp.newFile(it) }
        tmp.newFolder("sub")
        assertEquals(listOf("a.mp4", "b.mp3"), resultFiles(tmp.root).map { it.name })
    }

    // 영상·음성을 합치기 전에 yt-dlp 가 오류로 끝나면 `.f137.mp4` 같은 따로 받은 조각이 남는다. 저장하지 않는다.
    @Test fun resultFilesAfterErrorSkipsUnmergedStreams() {
        listOf("a.f395.mp4", "a.f140.m4a", "b.mp4", "c.fhls-720p.mp4").forEach { tmp.newFile(it) }
        assertEquals(listOf("b.mp4"), resultFiles(tmp.root, failed = true).map { it.name })
    }

    // 받다가 막히면(HTTP 403) 파일에 넣으려고 먼저 받은 썸네일만 남는다. 영상·음성 파일만 저장한다.
    @Test fun resultFilesAfterErrorKeepsOnlyMedia() {
        listOf("a.mp4", "a.webp", "b.description", "c.mp3", "d.jpg").forEach { tmp.newFile(it) }
        assertEquals(listOf("a.mp4", "c.mp3"), resultFiles(tmp.root, failed = true).map { it.name })
    }

    // 오류 없이 끝났으면 남은 건 모두 결과다. 제목이 `.fr` 로 끝나도 그대로 둔다.
    @Test fun resultFilesKeepsDotFNamesWhenSucceeded() {
        tmp.newFile("lemonde.fr.mp4")
        assertEquals(listOf("lemonde.fr.mp4"), resultFiles(tmp.root).map { it.name })
    }
}
