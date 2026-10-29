package com.glglekdy.quickmedia.preview

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class FormatTest {
    @Test fun duration() {
        assertNull(formatDuration(null))
        assertNull(formatDuration(0))
        assertNull(formatDuration(-5))
        assertEquals("0:59", formatDuration(59))
        assertEquals("3:33", formatDuration(213))
        assertEquals("1:02:03", formatDuration(3723))
    }

    @Test fun views() {
        assertNull(formatViews(null))
        assertEquals("0회", formatViews(0))
        assertEquals("1,234회", formatViews(1234))
        assertEquals("9,999회", formatViews(9999))
        assertEquals("1만 회", formatViews(10_000))
        assertEquals("1.2만 회", formatViews(12_345))
        assertEquals("9.9만 회", formatViews(99_999))
        assertEquals("10만 회", formatViews(100_000))
        assertEquals("12만 회", formatViews(123_456))
        assertEquals("9,999만 회", formatViews(99_999_999))
        assertEquals("1억 회", formatViews(100_000_000))
        assertEquals("1.5억 회", formatViews(150_000_000))
        assertEquals("15억 회", formatViews(1_500_000_000))
    }
}
