package com.glglekdy.quickmedia.update

import com.glglekdy.quickmedia.BuildConfig
import com.sun.net.httpserver.HttpServer
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.fail
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.security.MessageDigest
import java.util.concurrent.ConcurrentHashMap

class UpdaterTest {
    @get:Rule val tmp = TemporaryFolder()

    private val server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0).apply { start() }
    private val base = "http://127.0.0.1:${server.address.port}"
    private val apk = ByteArray(300_000) { (it % 251).toByte() }

    @After fun stop() = server.stop(0)

    /** [path] 에 [status] 와 [body] 로 답한다. 받은 요청 헤더(소문자 이름)를 돌려준다. */
    private fun serve(path: String, status: Int, body: ByteArray): Map<String, String> {
        val headers = ConcurrentHashMap<String, String>()
        server.createContext(path) { ex ->
            ex.requestHeaders.forEach { (name, values) -> headers[name.lowercase()] = values.first() }
            ex.sendResponseHeaders(status, if (body.isEmpty()) -1 else body.size.toLong())
            ex.responseBody.use { it.write(body) }
        }
        return headers
    }

    private fun sha256(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }

    private fun release(size: Long = apk.size.toLong(), sha256: String = sha256(apk), url: String = "$base/QuickMedia-9.9.9.apk") =
        Release("9.9.9", "", url, size, sha256)

    // 아무것도 듣고 있지 않은 포트
    private fun deadUrl() = "http://127.0.0.1:${ServerSocket(0).use { it.localPort }}/x"

    private fun assertFails(message: String, block: suspend () -> Unit) {
        try {
            runBlocking { block() }
            fail("UpdateException 이 나야 한다")
        } catch (e: UpdateException) {
            assertEquals(message, e.message)
        }
    }

    private fun leftFiles() = tmp.root.list().orEmpty().sorted()

    @Test fun checkFindsNewerApk() {
        val headers = serve("/latest", 200, """{"tag_name":"v9.9.9","body":"","assets":[
            {"name":"QuickMedia-9.9.9.apk","browser_download_url":"$base/a.apk","size":3,"digest":null}]}""".toByteArray())
        assertEquals("9.9.9", runBlocking { Updater.check("$base/latest", current = "1.1.0") }?.version)
        assertEquals("QuickMedia/${BuildConfig.VERSION_NAME}", headers["user-agent"])
        assertEquals("application/vnd.github+json", headers["accept"])
    }

    // 릴리즈가 하나도 없으면 GitHub 는 404 를 준다
    @Test fun checkWithoutReleasesIsNull() {
        serve("/latest", 404, """{"message":"Not Found"}""".toByteArray())
        assertNull(runBlocking { Updater.check("$base/latest", current = "1.1.0") })
    }

    @Test fun checkServerErrorShowsCode() {
        serve("/latest", 503, ByteArray(0))
        assertFails("릴리즈 정보를 가져오지 못했어요. (HTTP 503)") { Updater.check("$base/latest", current = "1.1.0") }
    }

    @Test fun checkBrokenAnswer() {
        serve("/latest", 200, "<html>".toByteArray())
        assertFails("릴리즈 정보를 가져오지 못했어요.") { Updater.check("$base/latest", current = "1.1.0") }
    }

    @Test fun checkOffline() = assertFails("GitHub 에 연결하지 못했어요.") { Updater.check(deadUrl(), current = "1.1.0") }

    // 지난 업데이트 때 받은 APK 는 지우고 확인을 거친 새 APK 하나만 남긴다
    @Test fun downloadKeepsOnlyCheckedApk() {
        serve("/QuickMedia-9.9.9.apk", 200, apk)
        tmp.newFile("QuickMedia-9.9.8.apk")
        var last = 0L to 0L
        val file = runBlocking { Updater.download(tmp.root, release()) { done, total -> last = done to total } }
        assertEquals(listOf("QuickMedia-9.9.9.apk"), leftFiles())
        assertArrayEquals(apk, file.readBytes())
        assertEquals(apk.size.toLong() to apk.size.toLong(), last)
    }

    @Test fun downloadWrongSizeLeavesNothing() {
        serve("/QuickMedia-9.9.9.apk", 200, apk)
        assertFails("받은 파일 크기가 맞지 않아요. 다시 시도해 주세요.") {
            Updater.download(tmp.root, release(size = apk.size + 1L)) { _, _ -> }
        }
        assertEquals(emptyList<String>(), leftFiles())
    }

    @Test fun downloadWrongHashLeavesNothing() {
        serve("/QuickMedia-9.9.9.apk", 200, apk)
        assertFails("받은 파일이 손상되었어요. (SHA-256 불일치)") {
            Updater.download(tmp.root, release(sha256 = "0".repeat(64))) { _, _ -> }
        }
        assertEquals(emptyList<String>(), leftFiles())
    }

    // 업데이트하고 다시 켜지면 설치한 APK 는 쓸모없다. 설치 허용을 기다리는 새 버전만 남긴다.
    @Test fun leftoversKeepOnlyNewerApk() {
        listOf("QuickMedia-1.1.0.apk", "QuickMedia-1.2.0.apk", "QuickMedia-1.2.0.apk.part", "x.tmp").forEach { tmp.newFile(it) }
        Updater.removeLeftovers(tmp.root, current = "1.1.0")
        assertEquals(listOf("QuickMedia-1.2.0.apk"), leftFiles())
    }

    @Test fun downloadCutOffLeavesNothing() {
        assertFails("업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요.") {
            Updater.download(tmp.root, release(url = deadUrl())) { _, _ -> }
        }
        assertEquals(emptyList<String>(), leftFiles())
    }
}
