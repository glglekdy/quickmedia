package com.glglekdy.quickmedia.options

import android.content.Context
import androidx.core.content.edit

/** 고른 옵션을 기기에 남긴다. 키는 [DownloadOptions] 의 필드 이름. */
class OptionsStore(context: Context) {
    private val prefs = context.getSharedPreferences("options", Context.MODE_PRIVATE)

    fun load(): DownloadOptions = optionsFromPrefs(prefs.all)

    fun save(o: DownloadOptions) = prefs.edit {
        o.toPrefs().forEach { (key, value) ->
            when (value) {
                is Boolean -> putBoolean(key, value)
                else -> putString(key, value.toString())
            }
        }
    }
}

fun DownloadOptions.toPrefs(): Map<String, Any> = mapOf(
    "kind" to kind.name,
    "quality" to quality,
    "audioFormat" to audioFormat,
    "playlist" to playlist.name,
    "playlistItems" to playlistItems,
    "embedMeta" to embedMeta,
    "sectionStart" to sectionStart,
    "sectionEnd" to sectionEnd,
)

/** 저장값을 되돌린다. 목록 밖의 값이나 형식이 다른 값은 기본값으로 둔다. */
fun optionsFromPrefs(values: Map<String, *>): DownloadOptions {
    val d = DownloadOptions()
    fun str(key: String) = values[key] as? String
    fun oneOf(key: String, allowed: List<String>, default: String) = str(key)?.takeIf { it in allowed } ?: default
    return DownloadOptions(
        kind = Kind.entries.firstOrNull { it.name == str("kind") } ?: d.kind,
        quality = oneOf("quality", QUALITIES, d.quality),
        audioFormat = oneOf("audioFormat", AUDIO_FORMATS, d.audioFormat),
        playlist = PlaylistMode.entries.firstOrNull { it.name == str("playlist") } ?: d.playlist,
        playlistItems = str("playlistItems") ?: d.playlistItems,
        embedMeta = values["embedMeta"] as? Boolean ?: d.embedMeta,
        sectionStart = str("sectionStart") ?: d.sectionStart,
        sectionEnd = str("sectionEnd") ?: d.sectionEnd,
    )
}
