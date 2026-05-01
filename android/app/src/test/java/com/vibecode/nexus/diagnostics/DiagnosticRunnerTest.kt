package com.vibecode.nexus.diagnostics

import kotlinx.serialization.json.JsonObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DiagnosticRunnerTest {

    private fun report(createdAt: Long? = null) = DiagReport(
        source = "android",
        deviceId = "test-device",
        appVersion = "0.1.0-test",
        deviceInfo = JsonObject(emptyMap()),
        results = emptyList(),
        passCount = 0,
        warnCount = 0,
        failCount = 0,
        createdAt = createdAt,
    )

    @Test
    fun applyAck_writesServerTimestampIntoReport() {
        val before = report(createdAt = null)
        val ack = DiagReportAck(id = "rep-123", createdAt = 1730000000L)

        val after = applyAck(before, ack)

        assertEquals(1730000000L, after.createdAt)
        assertEquals("android", after.source)
        assertEquals("test-device", after.deviceId)
    }

    @Test
    fun applyAck_doesNotMutateOriginalReport() {
        val before = report(createdAt = null)
        val ack = DiagReportAck(id = "rep-x", createdAt = 999L)

        applyAck(before, ack)

        assertNull("Original report stays immutable", before.createdAt)
    }

    @Test
    fun applyAck_overwritesExistingCreatedAt() {
        val before = report(createdAt = 100L)
        val ack = DiagReportAck(id = "rep-y", createdAt = 200L)

        val after = applyAck(before, ack)

        assertEquals(200L, after.createdAt)
    }
}
