package com.glglekdy.quickmedia.download

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.provider.MediaStore
import java.io.File
import java.io.IOException

const val SAVE_DIR = "Download/QuickMedia"

private val MIME = mapOf(
    "mp4" to "video/mp4",
    "mkv" to "video/x-matroska",
    "webm" to "video/webm",
    "mp3" to "audio/mpeg",
    "m4a" to "audio/mp4",
    "opus" to "audio/ogg",
    "jpg" to "image/jpeg",
)

// yt-dlp 가 받는 동안 쓰는 조각·임시 파일
private val PARTIAL = Regex("""\.(part|ytdl|temp)$|\.part-Frag\d*$""")

/** 같은 이름이 있으면 데스크톱처럼 `이름 (1).확장자`, `(2)`… */
fun numberedName(name: String, taken: (String) -> Boolean): String {
    if (!taken(name)) return name
    val dot = name.lastIndexOf('.')
    val (stem, ext) = if (dot > 0) name.substring(0, dot) to name.substring(dot) else name to ""
    return generateSequence(1) { it + 1 }.map { "$stem ($it)$ext" }.first { !taken(it) }
}

fun mimeOf(name: String): String = MIME[name.substringAfterLast('.', "").lowercase()] ?: "application/octet-stream"

// 영상·음성을 따로 받아 합치기 전 이름: `제목.f137.mp4`. 합쳐지면 지워진다.
private val UNMERGED = Regex("""\.f[\w-]+\.\w+$""")

/**
 * 작업 폴더에서 다 받은 파일만, 이름순.
 * [failed] 면 yt-dlp 가 오류로 끝난 것이라 영상·음성 파일만 두고, 합치지 못한 조각과 파일에 넣으려던 썸네일은 뺀다.
 * 성공했을 때는 조각이 남지 않으니 `lemonde.fr.mp4` 같은 제목을 조각으로 잘못 보지 않게 빼지 않는다.
 */
fun resultFiles(dir: File, failed: Boolean = false): List<File> =
    dir.listFiles().orEmpty()
        .filter { it.isFile && !PARTIAL.containsMatchIn(it.name) }
        .filter { !failed || (isMedia(it.name) && !UNMERGED.containsMatchIn(it.name)) }
        .sortedBy { it.name }

private fun isMedia(name: String) = mimeOf(name).let { it.startsWith("video/") || it.startsWith("audio/") }

/** 앱 캐시에 받은 파일을 `다운로드/QuickMedia` 로 옮긴다. 안드로이드 10+ 는 저장소 권한이 필요 없다. */
class MediaSaver(context: Context) {
    private val resolver = context.contentResolver
    private val collection = MediaStore.Downloads.EXTERNAL_CONTENT_URI

    fun save(file: File): Uri {
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, numberedName(file.name, ::taken))
            put(MediaStore.MediaColumns.MIME_TYPE, mimeOf(file.name))
            put(MediaStore.MediaColumns.RELATIVE_PATH, "$SAVE_DIR/")
            put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val uri = resolver.insert(collection, values) ?: throw IOException("다운로드 폴더에 파일을 만들지 못했어요")
        try {
            resolver.openOutputStream(uri)?.use { out -> file.inputStream().use { it.copyTo(out) } }
                ?: throw IOException("다운로드 폴더에 파일을 쓰지 못했어요")
            resolver.update(uri, ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) }, null, null)
        } catch (e: Exception) {
            resolver.delete(uri, null, null)
            throw e
        }
        return uri
    }

    // 이 앱이 저장한 파일만 보인다. 다른 앱(또는 다시 설치하기 전의 이 앱)이 만든 같은 이름은
    // 안드로이드가 알아서 번호를 붙인다.
    private fun taken(name: String): Boolean = resolver.query(
        collection, arrayOf(MediaStore.MediaColumns._ID),
        "${MediaStore.MediaColumns.RELATIVE_PATH} = ? AND ${MediaStore.MediaColumns.DISPLAY_NAME} = ?",
        arrayOf("$SAVE_DIR/", name), null,
    )?.use { it.count > 0 } ?: false
}
