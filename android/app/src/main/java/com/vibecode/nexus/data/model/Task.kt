package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class TaskCreateRequest(
    val title: String,
    val project_id: String? = null,
    val priority: String = "medium"
)

@Serializable
data class TaskUpdateRequest(
    val status: String? = null,
    val title: String? = null
)

@Serializable
data class TaskResponse(
    val id: Long,
    val title: String,
    val status: String,
    val priority: String,
    val project_id: String? = null,
    val created_at: String
)
