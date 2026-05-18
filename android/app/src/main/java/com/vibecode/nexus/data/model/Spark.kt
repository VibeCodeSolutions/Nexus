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

/**
 * DANIEL-FUNKTIONAL DC-001..004: Dashboard-Stat-Aggregat. Speist die vier
 * Stat-Cards Heute / Offen / Projekte / Diese-Woche-Erledigt.
 */
@Serializable
data class DashboardStatsResponse(
    val today_sparks: Long,
    val done_this_week: Long,
    val total_open_tasks: Long,
    val active_projects: Long
)

/**
 * DANIEL-FUNKTIONAL DC-003: Nächster offener Task mit Fälligkeit für die
 * Hero-Card unter dem Stat-Grid. `task` ist explizit `null` wenn nichts
 * passt — kein 404, sondern nullable Body-Feld.
 */
@Serializable
data class DashboardNextFocusResponse(
    val task: com.vibecode.nexus.data.model.TaskResponse? = null
)
