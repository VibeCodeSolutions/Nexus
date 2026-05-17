package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class SparkRequest(
    val text: String
)

@Serializable
data class SparkResponse(
    val id: String,
    val raw_text: String,
    val category: String? = null,
    val summary: String? = null,
    val tags: List<String> = emptyList(),
    val created_at: String
)

@Serializable
data class HealthResponse(
    val status: String
)

@Serializable
data class UpdateSparkTagsRequest(
    val tags: List<String>
)

@Serializable
data class UnsortedCountResponse(
    val count: Long
)

@Serializable
data class ExtractTasksResponse(
    val created: List<String> = emptyList(),
    val skipped: Long = 0,
    val count: Long = 0
)
