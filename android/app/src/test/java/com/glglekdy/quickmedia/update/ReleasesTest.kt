package com.glglekdy.quickmedia.update

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ReleasesTest {
    @Test fun versions() {
        assertEquals(listOf(1, 1), parseVersion("v1.1.0"))
        assertEquals(listOf(1, 2), parseVersion("1.2.0-beta"))
        assertTrue(isNewer("1.1.0", "1.0.1"))
        assertFalse(isNewer("v1.1", "1.1.0"))
        assertFalse(isNewer("1.0.9", "1.1.0"))
        assertTrue(isNewer("1.1.0", "0.0.0-dev"))
    }

    private val json = """{"tag_name":"v1.1.0","body":"notes","assets":[
      {"name":"QuickMedia-Setup-1.1.0.exe","browser_download_url":"https://x/exe","size":1,"digest":"sha256:aa"},
      {"name":"QuickMedia-1.1.0.apk","browser_download_url":"https://x/apk","size":42,"digest":"sha256:ABCD"}]}"""

    @Test fun picksApk() = assertEquals(Release("1.1.0", "notes", "https://x/apk", 42, "abcd"), parseRelease(json, "1.0.1"))

    @Test fun notNewer() = assertNull(parseRelease(json, "1.1.0"))

    @Test fun noApk() = assertNull(parseRelease(json.replace("QuickMedia-1.1.0.apk", "other.zip"), "1.0.0"))

    @Test fun verifyChecksSizeAndHash() {
        val r = Release("1.1.0", "", "u", 3, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad") // "abc"
        assertNull(verify(3, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", r))
        assertEquals("받은 파일 크기가 맞지 않아요. 다시 시도해 주세요.", verify(4, "x", r))
        assertEquals("받은 파일이 손상되었어요. (SHA-256 불일치)", verify(3, "00", r))
    }

    // GitHub 가 크기·digest 를 주지 않은 자산이면 그 검사는 건너뛴다
    @Test fun verifySkipsMissingExpectations() = assertNull(verify(5, "ff", Release("1.1.0", "", "u", 0, "")))
}
