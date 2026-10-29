package com.glglekdy.quickmedia.preview

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PreviewTest {
    @Test fun video() {
        val p = Preview.parse(
            """{"title":"Never Gonna","uploader":"Rick Astley","duration":213.0,"view_count":1500000000,"thumbnail":"https://i.ytimg.com/vi/x/maxres.jpg"}""",
        )!!
        assertEquals(Preview("Never Gonna", "Rick Astley", "https://i.ytimg.com/vi/x/maxres.jpg", 213, 1_500_000_000, false, null), p)
    }

    @Test fun channelWhenNoUploader() = assertEquals("Ch", Preview.parse("""{"title":"t","channel":"Ch"}""")!!.channel)

    @Test fun thumbnailsLastWhenNoThumbnail() =
        assertEquals("b", Preview.parse("""{"title":"t","thumbnails":[{"url":"a"},{"url":"b"}]}""")!!.thumbnail)

    @Test fun playlistCount() {
        val p = Preview.parse("""{"_type":"playlist","title":"여름","uploader":"u","playlist_count":12,"entries":[]}""")!!
        assertTrue(p.isPlaylist)
        assertEquals(12, p.count)
    }

    @Test fun playlistCountFromEntriesAndFirstEntryThumbnail() {
        val p = Preview.parse("""{"_type":"playlist","title":"여름","entries":[{"thumbnails":[{"url":"e1"},{"url":"e2"}]},{}]}""")!!
        assertEquals(2, p.count)
        assertEquals("e2", p.thumbnail)
    }

    @Test fun garbageAndNoTitle() {
        assertNull(Preview.parse("not json"))
        assertNull(Preview.parse("""{"uploader":"u"}"""))
        assertNull(Preview.parse("""{"title":""}"""))
    }
}
