package com.glglekdy.quickmedia.download

private val LINK = Regex("""https?://[^\s)\]>"']+""")

/** 공유받은 글에서 첫 링크. 유튜브 앱은 "제목 https://youtu.be/…" 처럼 글과 함께 보낸다. */
fun extractLink(text: String?): String? = text?.let { LINK.find(it)?.value }
