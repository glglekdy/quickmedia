package com.glglekdy.quickmedia.download

import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** youtubedl-android ffmpeg 0.18.1 묶음에서 4KB 정렬로 빌드된 라이브러리 */
val WEBP_LIBS = listOf("libsharpyuv.so", "libwebp.so", "libwebpdecoder.so", "libwebpdemux.so", "libwebpmux.so")

private val ELF_MAGIC = byteArrayOf(0x7f, 'E'.code.toByte(), 'L'.code.toByte(), 'F'.code.toByte())
private const val PT_LOAD = 1

/** ELF64 의 PT_LOAD 정렬값 중 가장 작은 값. ELF64 가 아니거나 LOAD 가 없으면 null. */
fun minLoadAlign(elf: ByteArray): Long? {
    if (elf.size < 64 || !elf.copyOf(4).contentEquals(ELF_MAGIC) || elf[4].toInt() != 2) return null
    val b = ByteBuffer.wrap(elf).order(ByteOrder.LITTLE_ENDIAN)
    val phoff = b.getLong(0x20)
    val size = b.getShort(0x36).toInt() and 0xffff
    val count = b.getShort(0x38).toInt() and 0xffff
    return (0 until count).map { phoff + it.toLong() * size }
        .filter { it + 0x38 <= elf.size && b.getInt(it.toInt()) == PT_LOAD }
        .minOfOrNull { b.getLong(it.toInt() + 0x30) }
}

/**
 * ffmpeg 묶음의 libwebp 계열은 4KB 정렬이라 16KB 페이지 기기에서 ffmpeg 가 아예 뜨지 않는다.
 * 그러면 영상·음성 합치기, mp3 바꾸기, 구간 자르기가 모두 실패한다(yausername/youtubedl-android#365).
 * 앱에 넣어 둔 16KB 정렬 빌드([ours], android/third_party/libwebp-16k)로, 풀린 파일([ffmpegLib]) 중
 * 4KB 정렬인 것만 바꾼다. 라이브러리가 고쳐져 16KB 로 나오면 아무것도 하지 않는다. 바꾼 파일 이름을 돌려준다.
 */
fun fixPageSize(ours: File, ffmpegLib: File): List<String> = WEBP_LIBS.filter { name ->
    val target = File(ffmpegLib, name)
    val replacement = File(ours, name)
    val align = if (target.isFile && replacement.isFile) minLoadAlign(target.readBytes()) else null
    (align != null && align < 16384).also { if (it) replacement.copyTo(target, overwrite = true) }
}
