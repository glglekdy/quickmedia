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
}
