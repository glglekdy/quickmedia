package com.glglekdy.quickmedia.download

import org.junit.Assert.assertEquals
import org.junit.Test

class FriendlyErrorTest {
    @Test fun table() = mapOf(
        "ERROR: Unsupported URL: https://x" to "지원하지 않는 링크예요",
        "ERROR: Sign in to confirm your age" to "로그인이 필요한 영상이라 받을 수 없어요",
        "ERROR: Video unavailable" to "삭제되었거나 볼 수 없는 영상이에요",
        "ERROR: [youtube] abc: This video is unavailable" to "삭제되었거나 볼 수 없는 영상이에요",
        "ERROR: Requested format is not available" to "고른 화질·형식이 없는 영상이에요. 옵션을 바꿔 보세요",
        "ERROR: [Errno 7] No address associated with hostname" to "인터넷 연결을 확인해 주세요",
        "ERROR: something odd" to "something odd",
    ).forEach { (m, want) -> assertEquals(m, want, friendlyError(m)) }

    @Test fun longMessageCut() = assertEquals(160, friendlyError("ERROR: " + "a".repeat(300)).length)

    // yt-dlp 의 stderr 는 경고 여러 줄 뒤에 오류가 온다. 경고 속 단어에 끌려가지 않는다.
    @Test fun usesLastErrorLine() = assertEquals(
        "삭제되었거나 볼 수 없는 영상이에요",
        friendlyError("WARNING: [youtube] unable to connect to proxy, retrying\nERROR: [youtube] abc: Video unavailable\n"),
    )
}
