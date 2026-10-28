package com.glglekdy.quickmedia.options

import org.junit.Assert.assertEquals
import org.junit.Test

class OptionsStoreTest {
    private val custom = DownloadOptions(
        kind = Kind.AUDIO, quality = "720", audioFormat = "m4a", playlist = PlaylistMode.ITEMS,
        playlistItems = "1-3", embedMeta = true, sectionStart = "0:05", sectionEnd = "0:10",
    )

    @Test fun roundTrip() = assertEquals(custom, optionsFromPrefs(custom.toPrefs()))

    @Test fun keysAreFieldNames() = assertEquals(
        setOf("kind", "quality", "audioFormat", "playlist", "playlistItems", "embedMeta", "sectionStart", "sectionEnd"),
        custom.toPrefs().keys,
    )

    // 예전 버전이 남긴 값이나 목록 밖의 값은 기본값으로 둔다
    @Test fun unknownValuesFallBackToDefaults() = assertEquals(
        DownloadOptions(playlistItems = "2"),
        optionsFromPrefs(mapOf("kind" to "PODCAST", "quality" to "4320", "audioFormat" to 3,
            "playlist" to "x", "playlistItems" to "2", "embedMeta" to "yes")),
    )

    @Test fun emptyPrefsAreDefaults() = assertEquals(DownloadOptions(), optionsFromPrefs(emptyMap<String, Any>()))
}
