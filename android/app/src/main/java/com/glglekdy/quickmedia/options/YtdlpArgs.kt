package com.glglekdy.quickmedia.options

/** 고른 옵션을 yt-dlp 명령줄 인자로. 데스크톱 `ytdl/options.py` 의 `build_args` 와 같은 값을 쓴다. */
object YtdlpArgs {
    private const val TEMPLATE = "%(title)s.%(ext)s"

    // 한글은 3바이트라 255바이트 파일 이름 한도에 금방 닿는다
    private const val MAX_NAME = "80"

    private val ITEMS = Regex("""^\d+(-\d+)?(,\d+(-\d+)?)*$""")

    fun build(o: DownloadOptions, outDir: String): List<String> = buildList {
        addAll(listOf("-P", outDir, "-o", TEMPLATE, "--no-mtime", "--trim-filenames", MAX_NAME))

        when (o.playlist) {
            PlaylistMode.SINGLE -> add("--no-playlist")
            PlaylistMode.ALL -> add("--yes-playlist")
            PlaylistMode.ITEMS -> addAll(listOf("--yes-playlist", "-I", o.playlistItems.filterNot(Char::isWhitespace)))
        }

        if (o.kind == Kind.AUDIO) {
            addAll(listOf("-x", "--audio-format", o.audioFormat, "--audio-quality", "0"))
        } else {
            val sort = if (o.quality == "best") "ext:mp4:m4a" else "res:${o.quality},ext:mp4:m4a"
            addAll(listOf("-S", sort, "--merge-output-format", "mp4"))
        }

        if (o.embedMeta) addAll(listOf("--embed-thumbnail", "--embed-metadata"))

        val start = o.sectionStart.trim()
        val end = o.sectionEnd.trim()
        if (start.isNotEmpty() || end.isNotEmpty()) {
            addAll(listOf("--download-sections", "*${start.ifEmpty { "0" }}-${end.ifEmpty { "inf" }}",
                "--force-keyframes-at-cuts"))
        }
    }

    /** 칸 이름 → 화면 문구. 비어 있으면 받을 수 있다. */
    fun validate(o: DownloadOptions): Map<String, String> = buildMap {
        if (o.playlist == PlaylistMode.ITEMS && !ITEMS.matches(o.playlistItems.filterNot(Char::isWhitespace))) {
            put("playlistItems", "1-5,8 처럼 번호와 범위로 적어 주세요")
        }
        val start = o.sectionStart.takeIf { it.isNotBlank() }?.let(::parseTime)
        if (o.sectionStart.isNotBlank() && start == null) put("sectionStart", "1:23 이나 83 처럼 적어 주세요")
        val end = o.sectionEnd.takeIf { it.isNotBlank() }?.let(::parseTime)
        if (o.sectionEnd.isNotBlank() && end == null) put("sectionEnd", "1:23 이나 83 처럼 적어 주세요")
        if (start != null && end != null && end <= start) put("sectionEnd", "끝이 시작보다 뒤여야 해요")
    }

    /** `83`, `1:23`, `1:02:03` → 초. 형식이 틀리면 null. 앞 칸 뒤의 분·초는 0~59. */
    fun parseTime(text: String): Double? {
        val parts = text.trim().split(":")
        if (parts.size !in 1..3) return null
        val numbers = parts.mapIndexed { i, part ->
            val last = i == parts.lastIndex
            val ok = if (last) part.matches(Regex("""\d+(\.\d+)?""")) else part.matches(Regex("""\d+"""))
            if (!ok) return null
            part.toDouble().also { if (i > 0 && it >= 60) return null }
        }
        return numbers.fold(0.0) { acc, n -> acc * 60 + n }
    }
}
