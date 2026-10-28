package com.glglekdy.quickmedia.download

// 위에서부터 맞춰 본다. 데스크톱 ytdl/downloader.py 의 표에서 쿠키 줄을 뺐다(앱에 쿠키 기능이 없다).
private val ERRORS = listOf(
    listOf("unsupported url", "is not a valid url") to "지원하지 않는 링크예요",
    listOf("sign in", "login required", "private video", "members-only") to "로그인이 필요한 영상이라 받을 수 없어요",
    listOf("video unavailable", "video is unavailable", "this video is not available") to "삭제되었거나 볼 수 없는 영상이에요",
    listOf("requested format is not available") to "고른 화질·형식이 없는 영상이에요. 옵션을 바꿔 보세요",
    listOf("getaddrinfo", "timed out", "unable to connect", "no address associated", "network is unreachable")
        to "인터넷 연결을 확인해 주세요",
)

/** yt-dlp 오류를 화면 문구로. 모르는 오류는 원문 앞 160자. */
fun friendlyError(message: String): String {
    // stderr 에는 경고가 먼저 오고 오류는 마지막 ERROR: 줄에 있다
    val line = message.lines().lastOrNull { it.trimStart().startsWith("ERROR:") }?.trim() ?: message.trim()
    val low = line.lowercase()
    ERRORS.firstOrNull { (needles, _) -> needles.any { it in low } }?.let { return it.second }
    return line.removePrefix("ERROR:").trim().take(160)
}
