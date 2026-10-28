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

/** 옵션 더보기에서 기본값과 달라진 항목. 접혀 있어도 켜 둔 걸 알 수 있게 버튼 옆에 보인다. */
fun changedSummary(o: DownloadOptions): List<String> = buildList {
    if (o.playlist == PlaylistMode.ALL || (o.playlist == PlaylistMode.ITEMS && o.playlistItems.isNotBlank())) add("재생목록")
    if (o.embedMeta) add("썸네일·메타데이터")
    if (o.sectionStart.isNotBlank() || o.sectionEnd.isNotBlank()) add("구간")
}
