package com.vibecode.nexus.data.model

import kotlinx.serialization.Serializable

@Serializable
data class ProjectResponse(
    val id: String,
    val name: String,
    val created_at: String
)

@Serializable
data class ProjectProgress(
    val project_id: String,
    val total_tasks: Int,
    val done_tasks: Int,
    val progress_percent: Int
)
