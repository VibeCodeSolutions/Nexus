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
    val id: String,
    val title: String,
    val status: String,
    val priority: String,
    val project_id: String? = null,
    val created_at: String,
    val updated_at: String? = null,
    // FEAT-001 (Backend) → DANIEL-FUNKTIONAL (Android-Pull, 2026-05-18):
    // Backend liefert `due_date` als ISO-YYYY-MM-DD seit Migration
    // 20260520_001. Android-Model zog hinterher — jetzt nachgezogen,
    // damit die Nächster-Fokus-Card das Datum anzeigen kann.
    val due_date: String? = null,
    // DANIEL-POLISH DA-002 (2026-05-18): Tasks aus Sparks tragen
    // `nexus_external_id = "spark-extract:<spark_id>:<idx>"`. Wird
    // clientseitig in SparkHistoryScreen genutzt, um IDEA/TASK-Badge
    // pro Spark-Karte zu bestimmen.
    val nexus_external_id: String? = null
)
