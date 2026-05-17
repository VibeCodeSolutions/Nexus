package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class ProjectSuggestion(
    val id: String,
    val name: String,
    val description: String,
    val member_spark_ids: List<String> = emptyList(),
    val confidence: Double,
    val reason: String? = null,
    val created_at: String,
    val status: String,
)

@Serializable
data class AcceptSuggestionResponse(
    val project_id: String,
    val name: String,
    val linked_sparks: Int,
    val requested_sparks: Int,
    val partial: Boolean,
)
