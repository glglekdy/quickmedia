package com.glglekdy.quickmedia.download

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

class PageSizeFixTest {
    @get:Rule val tmp = TemporaryFolder()

    /** 프로그램 헤더만 있는 ELF64. [loads] 는 PT_LOAD 정렬값, [others] 는 그 밖의 헤더(PT_DYNAMIC) 정렬값. */
    private fun elf(loads: List<Long>, others: List<Long> = emptyList(), tag: Byte = 0): ByteArray {
        val headers = loads.map { 1 to it } + others.map { 2 to it }
        val b = ByteBuffer.allocate(64 + headers.size * 56 + 1).order(ByteOrder.LITTLE_ENDIAN)
        b.put(byteArrayOf(0x7f, 'E'.code.toByte(), 'L'.code.toByte(), 'F'.code.toByte(), 2, 1, 1))
        b.putLong(0x20, 64)
        b.putShort(0x36, 56)
        b.putShort(0x38, headers.size.toShort())
        headers.forEachIndexed { i, (type, align) ->
            b.putInt(64 + i * 56, type)
            b.putLong(64 + i * 56 + 0x30, align)
        }
        b.put(64 + headers.size * 56, tag) // 파일끼리 내용이 다르게
        return b.array()
    }

    @Test fun smallestLoadAlignment() =
        assertEquals(0x1000L, minLoadAlign(elf(loads = listOf(0x4000, 0x1000), others = listOf(8))))

    @Test fun notElf() = assertNull(minLoadAlign("hello".toByteArray()))

    // 4KB 정렬인 것만 앱에 넣어 둔 16KB 빌드로 바꾼다. 이미 16KB 이거나 없는 파일은 그대로.
    @Test fun replacesOnly4kAlignedLibraries() {
        val ours = tmp.newFolder("ours")
        val ffmpeg = tmp.newFolder("ffmpeg")
        WEBP_LIBS.forEach { File(ours, it).writeBytes(elf(listOf(0x4000), tag = 1)) }
        File(ffmpeg, "libwebp.so").writeBytes(elf(listOf(0x1000)))
        val already = elf(listOf(0x4000), tag = 2)
        File(ffmpeg, "libsharpyuv.so").writeBytes(already)

        assertEquals(listOf("libwebp.so"), fixPageSize(ours, ffmpeg))
        assertArrayEquals(File(ours, "libwebp.so").readBytes(), File(ffmpeg, "libwebp.so").readBytes())
        assertArrayEquals(already, File(ffmpeg, "libsharpyuv.so").readBytes())
        assertFalse(File(ffmpeg, "libwebpmux.so").exists())
    }

    // 한 번 바꾼 뒤 다시 켜면 할 일이 없다
    @Test fun secondRunDoesNothing() {
        val ours = tmp.newFolder("ours")
        val ffmpeg = tmp.newFolder("ffmpeg")
        WEBP_LIBS.forEach {
            File(ours, it).writeBytes(elf(listOf(0x4000)))
            File(ffmpeg, it).writeBytes(elf(listOf(0x1000)))
        }
        assertEquals(WEBP_LIBS, fixPageSize(ours, ffmpeg))
        assertEquals(emptyList<String>(), fixPageSize(ours, ffmpeg))
    }
}
