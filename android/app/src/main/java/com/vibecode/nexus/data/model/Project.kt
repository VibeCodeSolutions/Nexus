package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class ProjectResponse(
    val id: String,
    val name: String,
    val created_at: String,
    val description: String? = null,
    val status: String = "active",
    val nexus_external_id: String? = null
)

@Serializable
data class ProjectCreateRequest(
    val name: String,
    val description: String = "",
    val spark_ids: List<String> = emptyList()
)

@Serializable
data class ProjectUpdateRequest(
    val name: String,
    val description: String,
    val status: String
)

@Serializable
data class ProjectProgress(
    val project_id: String,
    val total_tasks: Int,
    val done_tasks: Int,
    val progress_percent: Int
)
