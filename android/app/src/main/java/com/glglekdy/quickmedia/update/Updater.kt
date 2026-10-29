package com.glglekdy.quickmedia.update

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import com.glglekdy.quickmedia.BuildConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.withContext
import org.json.JSONException
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

/** 화면에 그대로 보일 문구를 담는다. */
class UpdateException(message: String) : Exception(message)

/** 데스크톱 `ytdl/updater.py` 와 같은 순서: 새 릴리스 확인 → APK 받기(크기·SHA-256 확인) → 설치. */
object Updater {
    private const val TIMEOUT_MS = 20_000
    private const val CUT_OFF = "업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요."
    private val APK_FILE = Regex("""^QuickMedia-(.+)\.apk$""")

    /** 지금보다 새 APK 가 있는 릴리스. 없으면 null. */
    suspend fun check(url: String = LATEST_URL, current: String = BuildConfig.VERSION_NAME): Release? =
        withContext(Dispatchers.IO) {
            val conn = open(url, "application/vnd.github+json")
            try {
                when (val code = conn.responseCode) {
                    404 -> null // 릴리즈가 하나도 없음
                    in 200..299 -> parseRelease(conn.inputStream.bufferedReader().use { it.readText() }, current)
                    else -> throw UpdateException("릴리즈 정보를 가져오지 못했어요. (HTTP $code)")
                }
            } catch (e: IOException) {
                throw UpdateException("GitHub 에 연결하지 못했어요.")
            } catch (e: JSONException) {
                throw UpdateException("릴리즈 정보를 가져오지 못했어요.")
            } finally {
                conn.disconnect()
            }
        }

    /** 앱 캐시에 받는다. */
    suspend fun download(context: Context, r: Release, progress: (done: Long, total: Long) -> Unit): File =
        download(File(context.cacheDir, "update"), r, progress)

    /** [dir] 에 `QuickMedia-<버전>.apk` 로 받는다. 크기·SHA-256 이 맞을 때만 남는다. */
    suspend fun download(dir: File, r: Release, progress: (done: Long, total: Long) -> Unit): File =
        withContext(Dispatchers.IO) {
            // 지난 업데이트 때 받은 APK 는 더 쓸 일이 없다
            dir.listFiles()?.forEach { it.deleteRecursively() }
            dir.mkdirs()
            val target = File(dir, "QuickMedia-${r.version}.apk")
            val part = File(dir, "${target.name}.part")
            val sha = MessageDigest.getInstance("SHA-256")
            try {
                val conn = open(r.apkUrl, "application/octet-stream")
                try {
                    val total = conn.contentLengthLong.takeIf { it > 0 } ?: r.apkSize
                    var done = 0L
                    conn.inputStream.use { input ->
                        part.outputStream().use { out ->
                            val buf = ByteArray(256 * 1024)
                            while (true) {
                                ensureActive()
                                val n = input.read(buf)
                                if (n < 0) break
                                out.write(buf, 0, n)
                                sha.update(buf, 0, n)
                                done += n
                                progress(done, total)
                            }
                        }
                    }
                } finally {
                    conn.disconnect()
                }
                verify(part.length(), sha.digest().joinToString("") { "%02x".format(it) }, r)
                    ?.let { throw UpdateException(it) }
                if (!part.renameTo(target)) throw IOException("이름을 바꾸지 못함: $part")
                target
            } catch (e: IOException) {
                throw UpdateException(CUT_OFF)
            } finally {
                part.delete() // 다 받았으면 이미 이름이 바뀌어 없다
            }
        }

    /** 앱이 켜질 때: 이미 설치한 업데이트 APK 와 받다 만 조각을 지운다. 설치를 기다리는 새 버전 APK 만 남긴다. */
    fun removeLeftovers(dir: File, current: String = BuildConfig.VERSION_NAME) {
        dir.listFiles()?.forEach { file ->
            val version = APK_FILE.matchEntire(file.name)?.groupValues?.get(1)
            if (version == null || !isNewer(version, current)) file.deleteRecursively()
        }
    }

    /**
     * 받은 APK 를 안드로이드에 넘긴다. 설치 확인 화면은 안드로이드가 띄운다([InstallResultReceiver]).
     * "출처를 알 수 없는 앱 설치" 허용([Context.getPackageManager] 의 canRequestPackageInstalls)을 먼저 확인한다.
     * APK 를 옮겨 쓰므로 메인 스레드 밖에서 부른다.
     */
    fun install(context: Context, apk: File) {
        val installer = context.packageManager.packageInstaller
        val params = PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL).apply {
            setAppPackageName(context.packageName)
            setSize(apk.length())
        }
        val id = installer.createSession(params)
        installer.openSession(id).use { session ->
            try {
                session.openWrite("QuickMedia.apk", 0, apk.length()).use { out ->
                    apk.inputStream().use { it.copyTo(out) }
                    session.fsync(out)
                }
                // 안드로이드가 결과를 extra 로 채워 보내므로 MUTABLE
                val result = PendingIntent.getBroadcast(
                    context, id, Intent(context, InstallResultReceiver::class.java),
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE,
                )
                session.commit(result.intentSender)
            } catch (e: IOException) {
                session.abandon()
                throw UpdateException("업데이트를 설치하지 못했어요.")
            }
        }
    }

    private fun open(url: String, accept: String) = (URL(url).openConnection() as HttpURLConnection).apply {
        connectTimeout = TIMEOUT_MS
        readTimeout = TIMEOUT_MS
        setRequestProperty("User-Agent", "QuickMedia/${BuildConfig.VERSION_NAME}")
        setRequestProperty("Accept", accept)
    }
}
