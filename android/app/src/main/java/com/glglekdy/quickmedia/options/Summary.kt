package com.glglekdy.quickmedia.options

/** 화질·음성 형식 버튼 글자. `best` → `최고`, `1080` → `1080p`, `mp3` → `MP3`. */
fun qualityLabel(value: String): String = when (value) {
    "best" -> "최고"
    in AUDIO_FORMATS -> value.uppercase()
    else -> "${value}p"
}

/** 작업 카드에 쓸 짧은 요약. 예: `1080p · MP4`, `MP3`. 데스크톱 describe 와 같은 모양. */
fun describe(o: DownloadOptions): String = when (o.kind) {
    Kind.AUDIO -> o.audioFormat.uppercase()
    Kind.VIDEO -> (if (o.quality == "best") "최고 화질" else "${o.quality}p") + " · MP4"
}

/** 받기 탭 옵션 줄에 보일 바꾼 옵션. 예: `전체 재생목록 · 1:23부터`. 기본값 그대로면 null. */
fun optionsSummary(o: DownloadOptions): String? = buildList {
    when (o.playlist) {
        PlaylistMode.SINGLE -> {}
        PlaylistMode.ALL -> add("전체 재생목록")
        PlaylistMode.ITEMS -> add(o.playlistItems.trim().let { if (it.isEmpty()) "재생목록 일부" else "재생목록 $it" })
    }
    if (o.embedMeta) add("메타데이터")
    val start = o.sectionStart.trim()
    val end = o.sectionEnd.trim()
    when {
        start.isNotEmpty() && end.isNotEmpty() -> add("$start~$end")
        start.isNotEmpty() -> add("${start}부터")
        end.isNotEmpty() -> add("${end}까지")
    }
}.takeIf { it.isNotEmpty() }?.joinToString(" · ")
