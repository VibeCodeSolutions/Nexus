package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class Link(
    val id: String,
    val source_type: String,
    val source_id: String,
    val target_type: String,
    val target_id: String,
    val relation: String,
    val confidence: Double,
    val reason: String? = null,
    val created_at: String,
    val created_by: String,
)

@Serializable
data class SparkLinks(
    val outgoing: List<Link> = emptyList(),
    val incoming: List<Link> = emptyList(),
)
