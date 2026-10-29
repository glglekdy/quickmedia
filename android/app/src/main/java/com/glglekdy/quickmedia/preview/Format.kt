package com.glglekdy.quickmedia.preview

import java.util.Locale

/** `213` → `3:33`, `3723` → `1:02:03`. 길이를 모르면 null. */
fun formatDuration(seconds: Long?): String? {
    if (seconds == null || seconds <= 0) return null
    val h = seconds / 3600
    val m = seconds % 3600 / 60
    val s = seconds % 60
    return if (h > 0) String.format(Locale.ROOT, "%d:%02d:%02d", h, m, s) else String.format(Locale.ROOT, "%d:%02d", m, s)
}

/** `1234` → `1,234회`, `12345` → `1.2만 회`, `1500000000` → `15억 회`. 소수는 버린다. */
fun formatViews(count: Long?): String? {
    if (count == null) return null
    return when {
        count < 10_000 -> "${grouped(count)}회"
        count < 100_000_000 -> "${scaled(count, 10_000)}만 회"
        else -> "${scaled(count, 100_000_000)}억 회"
    }
}

// 10 단위 미만이면 소수 첫째 자리까지, `.0` 은 뺀다
private fun scaled(count: Long, unit: Long): String {
    val whole = count / unit
    if (whole >= 10) return grouped(whole)
    val tenth = count * 10 / unit % 10
    return if (tenth == 0L) "$whole" else "$whole.$tenth"
}

private fun grouped(n: Long) = String.format(Locale.ROOT, "%,d", n)
