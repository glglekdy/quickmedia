package com.glglekdy.quickmedia.update

import org.json.JSONObject

const val LATEST_URL = "https://api.github.com/repos/glglekdy/quickmedia/releases/latest"

// 데스크톱 설치 파일(QuickMedia-Setup-*.exe)과 같은 릴리즈에 올라간다
private val APK_NAME = Regex("""^QuickMedia-.+\.apk$""", RegexOption.IGNORE_CASE)

data class Release(
    val version: String,
    val notes: String,
    val apkUrl: String,
    val apkSize: Long,
    /** 소문자 hex. GitHub 가 주지 않았으면 "" */
    val sha256: String,
)

/** `v1.2.3`, `1.2`, `1.2.3-beta` 모두 숫자 부분만. 끝의 0 은 뗀다(1.1.0 == 1.1). 데스크톱 `parse_version` 과 같다. */
fun parseVersion(text: String): List<Int> {
    val match = Regex("""^\s*v?(\d+(?:\.\d+)*)""", RegexOption.IGNORE_CASE).find(text) ?: return emptyList()
    val parts = match.groupValues[1].split(".").map(String::toInt).toMutableList()
    while (parts.size > 1 && parts.last() == 0) parts.removeAt(parts.lastIndex)
    return parts
}

fun isNewer(latest: String, current: String): Boolean {
    val a = parseVersion(latest)
    val b = parseVersion(current)
    for (i in 0 until minOf(a.size, b.size)) {
        if (a[i] != b[i]) return a[i] > b[i]
    }
    return a.size > b.size
}

/** releases/latest 응답에서 지금보다 새 APK. 없으면 null. */
fun parseRelease(json: String, current: String): Release? {
    val data = JSONObject(json)
    val version = data.str("tag_name").trimStart('v', 'V')
    if (!isNewer(version, current)) return null
    val assets = data.optJSONArray("assets") ?: return null
    val apk = (0 until assets.length()).map(assets::getJSONObject)
        .firstOrNull { APK_NAME.matches(it.str("name")) } ?: return null
    val digest = apk.str("digest")
    return Release(
        version = version,
        notes = data.str("body"),
        apkUrl = apk.getString("browser_download_url"),
        apkSize = apk.optLong("size"),
        sha256 = if (digest.startsWith("sha256:")) digest.substringAfter(':').lowercase() else "",
    )
}

/** 받은 파일이 릴리즈와 맞으면 null, 아니면 화면 문구. */
fun verify(size: Long, sha256Hex: String, release: Release): String? = when {
    release.apkSize > 0 && size != release.apkSize -> "받은 파일 크기가 맞지 않아요. 다시 시도해 주세요."
    release.sha256.isNotEmpty() && !sha256Hex.equals(release.sha256, ignoreCase = true) -> "받은 파일이 손상되었어요. (SHA-256 불일치)"
    else -> null
}

// 안드로이드 org.json 의 optString 은 null 값을 "null" 로 돌려준다
private fun JSONObject.str(key: String): String = if (isNull(key)) "" else optString(key)
