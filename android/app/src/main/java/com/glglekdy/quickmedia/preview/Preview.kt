package com.glglekdy.quickmedia.preview

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

/** 받기 전에 보여 줄 영상·재생목록 정보. yt-dlp `-J --flat-playlist` 의 JSON 에서 읽는다. */
data class Preview(
    val title: String,
    val channel: String?,
    val thumbnail: String?,
    /** 초 */
    val duration: Long?,
    val views: Long?,
    val isPlaylist: Boolean,
    /** 재생목록의 영상 수. 영상이면 null */
    val count: Int?,
) {
    companion object {
        /** 제목이 없거나 JSON 이 깨졌으면 null */
        fun parse(json: String): Preview? {
            val o = try {
                JSONObject(json)
            } catch (e: JSONException) {
                return null
            }
            val title = o.text("title") ?: return null
            val channel = o.text("uploader") ?: o.text("channel")
            if (o.optString("_type") == "playlist") {
                val entries = o.optJSONArray("entries")
                val count = if (o.has("playlist_count")) o.optInt("playlist_count") else entries?.length()
                val thumbnail = o.optJSONArray("thumbnails").lastUrl()
                    ?: entries?.optJSONObject(0)?.optJSONArray("thumbnails").lastUrl()
                return Preview(title, channel, thumbnail, null, null, true, count)
            }
            val thumbnail = o.text("thumbnail") ?: o.optJSONArray("thumbnails").lastUrl()
            val duration = if (o.has("duration") && !o.isNull("duration")) o.optDouble("duration").toLong() else null
            val views = if (o.has("view_count") && !o.isNull("view_count")) o.optLong("view_count") else null
            return Preview(title, channel, thumbnail, duration, views, false, null)
        }

        private fun JSONObject.text(key: String): String? =
            if (isNull(key)) null else optString(key).ifEmpty { null }

        private fun JSONArray?.lastUrl(): String? =
            if (this == null || length() == 0) null else optJSONObject(length() - 1)?.text("url")
    }
}
