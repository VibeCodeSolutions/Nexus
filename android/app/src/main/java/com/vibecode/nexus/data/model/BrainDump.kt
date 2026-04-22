package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class BrainDumpRequest(
    val text: String
)

@Serializable
data class BrainDumpResponse(
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
