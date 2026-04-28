package com.vibecode.nexus.diagnostics

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

@Serializable
enum class DiagStatus {
    @SerialName("pass") PASS,
    @SerialName("warn") WARN,
    @SerialName("fail") FAIL,
}

@Serializable
data class DiagCheck(
    val name: String,
    val status: DiagStatus,
    @SerialName("duration_ms") val durationMs: Long,
    val message: String? = null,
    val error: String? = null,
)

@Serializable
data class DiagReport(
    val source: String,
    @SerialName("device_id") val deviceId: String?,
    @SerialName("app_version") val appVersion: String,
    @SerialName("device_info") val deviceInfo: JsonObject,
    val results: List<DiagCheck>,
    @SerialName("pass_count") val passCount: Int,
    @SerialName("warn_count") val warnCount: Int,
    @SerialName("fail_count") val failCount: Int,
    @SerialName("created_at") val createdAt: Long? = null,
)

@Serializable
data class DiagReportAck(
    val id: String,
    @SerialName("created_at") val createdAt: Long,
)
