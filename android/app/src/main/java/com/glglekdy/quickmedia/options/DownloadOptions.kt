package com.glglekdy.quickmedia.options

enum class Kind { VIDEO, AUDIO }

/** 이 영상만 / 재생목록 전체 / 재생목록 일부 항목 */
enum class PlaylistMode { SINGLE, ALL, ITEMS }

/** 메인 화면에서 고르는 값. yt-dlp 인자로는 [YtdlpArgs.build] 가 바꾼다. */
data class DownloadOptions(
    val kind: Kind = Kind.VIDEO,
    /** best · 1080 · 720 · 480 */
    val quality: String = "best",
    /** mp3 · m4a */
    val audioFormat: String = "mp3",
    val playlist: PlaylistMode = PlaylistMode.SINGLE,
    /** `1-5,8` 같은 항목 목록. [PlaylistMode.ITEMS] 일 때만 쓴다. */
    val playlistItems: String = "",
    /** 썸네일과 메타데이터를 파일에 넣는다 */
    val embedMeta: Boolean = false,
    /** 구간 자르기 시작·끝. `1:23`, `1:02:03`, `83`. 비우면 처음·끝까지. */
    val sectionStart: String = "",
    val sectionEnd: String = "",
)

val QUALITIES = listOf("best", "1080", "720", "480")
val AUDIO_FORMATS = listOf("mp3", "m4a")
