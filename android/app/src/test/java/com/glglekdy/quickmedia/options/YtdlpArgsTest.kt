package com.glglekdy.quickmedia.options

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class YtdlpArgsTest {
    private fun args(o: DownloadOptions) = YtdlpArgs.build(o, "/c/dl")
    private fun List<String>.after(flag: String) = this[indexOf(flag) + 1]
    private fun List<String>.windowAfter(flag: String, n: Int) = subList(indexOf(flag), indexOf(flag) + n)

    @Test fun alwaysArgs() {
        val a = args(DownloadOptions())
        assertTrue(a.containsAll(listOf("--no-mtime", "--trim-filenames", "80")))
        assertEquals(listOf("-P", "/c/dl", "-o", "%(title)s.%(ext)s"), a.windowAfter("-P", 4))
    }

    @Test fun videoBest() {
        val a = args(DownloadOptions())
        assertEquals("ext:mp4:m4a", a.after("-S"))
        assertEquals("mp4", a.after("--merge-output-format"))
    }

    @Test fun video720() = assertEquals("res:720,ext:mp4:m4a", args(DownloadOptions(quality = "720")).after("-S"))

    @Test fun audio() {
        val a = args(DownloadOptions(kind = Kind.AUDIO, audioFormat = "m4a"))
        assertTrue(a.containsAll(listOf("-x")))
        assertEquals("m4a", a.after("--audio-format"))
        assertEquals("0", a.after("--audio-quality"))
        assertFalse("-S" in a)
    }

    @Test fun playlistModes() {
        assertTrue("--no-playlist" in args(DownloadOptions()))
        assertTrue("--yes-playlist" in args(DownloadOptions(playlist = PlaylistMode.ALL)))
        val items = args(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "1-5, 8"))
        assertTrue("--yes-playlist" in items)
        assertEquals("1-5,8", items.after("-I"))
    }

    @Test fun embedMeta() =
        assertTrue(args(DownloadOptions(embedMeta = true)).containsAll(listOf("--embed-thumbnail", "--embed-metadata")))

    @Test fun section() {
        assertEquals("*1:23-inf", args(DownloadOptions(sectionStart = "1:23")).after("--download-sections"))
        assertEquals("*0-90", args(DownloadOptions(sectionEnd = "90")).after("--download-sections"))
        assertTrue("--force-keyframes-at-cuts" in args(DownloadOptions(sectionEnd = "90")))
        assertFalse("--download-sections" in args(DownloadOptions()))
    }

    @Test fun parseTime() {
        assertEquals(83.0, YtdlpArgs.parseTime("1:23"))
        assertEquals(3723.0, YtdlpArgs.parseTime("1:02:03"))
        assertEquals(83.0, YtdlpArgs.parseTime(" 83 "))
        assertNull(YtdlpArgs.parseTime("1:2:3:4"))
        assertNull(YtdlpArgs.parseTime("abc"))
    }

    // yt-dlp 는 구간의 분·초가 60 이상이면 받지 않는다
    @Test fun parseTimeRejectsMinutesOrSecondsOver59() {
        assertNull(YtdlpArgs.parseTime("1:75"))
        assertNull(YtdlpArgs.parseTime("1:60:00"))
    }

    @Test fun validate() {
        assertEquals(emptyMap<String, String>(), YtdlpArgs.validate(DownloadOptions()))
        assertTrue("playlistItems" in YtdlpArgs.validate(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "a-b")))
        assertTrue("playlistItems" in YtdlpArgs.validate(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "")))
        assertEquals(emptyMap<String, String>(),
            YtdlpArgs.validate(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "1-5, 8")))
        assertTrue("sectionStart" in YtdlpArgs.validate(DownloadOptions(sectionStart = "x")))
        assertEquals("끝이 시작보다 뒤여야 해요",
            YtdlpArgs.validate(DownloadOptions(sectionStart = "2:00", sectionEnd = "1:00"))["sectionEnd"])
    }
}
