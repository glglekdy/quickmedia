package com.glglekdy.quickmedia.download

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SharedLinkTest {
    @Test fun links() {
        assertEquals("https://youtu.be/abc?si=x1", extractLink("이 영상 봐 https://youtu.be/abc?si=x1 재밌음"))
        assertEquals("https://a.com/1", extractLink("(https://a.com/1) and https://b.com/2"))
        assertNull(extractLink("링크 없음"))
        assertNull(extractLink(null))
    }
}
