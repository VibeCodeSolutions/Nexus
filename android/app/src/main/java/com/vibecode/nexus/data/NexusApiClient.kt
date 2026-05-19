package com.vibecode.nexus.data

import com.vibecode.nexus.data.model.AcceptSuggestionResponse
import com.vibecode.nexus.data.model.SparkLinks
import com.vibecode.nexus.data.model.SparkRequest
import com.vibecode.nexus.data.model.SparkResponse
import com.vibecode.nexus.data.model.HealthResponse
import com.vibecode.nexus.data.model.ModelsResponse
import com.vibecode.nexus.data.model.ProjectCreateRequest
import com.vibecode.nexus.data.model.ProjectProgress
import com.vibecode.nexus.data.model.ProjectResponse
import com.vibecode.nexus.data.model.ProjectSuggestion
import com.vibecode.nexus.data.model.ProjectUpdateRequest
import com.vibecode.nexus.data.model.ProvidersResponse
import com.vibecode.nexus.data.model.SetProviderRequest
import com.vibecode.nexus.data.model.TaskCreateRequest
import com.vibecode.nexus.data.model.TaskResponse
import com.vibecode.nexus.data.model.TaskUpdateRequest
import com.vibecode.nexus.data.model.ExtractTasksResponse
import com.vibecode.nexus.data.model.UnsortedCountResponse
import com.vibecode.nexus.data.model.DashboardStatsResponse
import com.vibecode.nexus.data.model.DashboardNextFocusResponse
import com.vibecode.nexus.data.model.UpdateSparkTagsRequest
import com.vibecode.nexus.diagnostics.DiagReport
import com.vibecode.nexus.diagnostics.DiagReportAck
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json

class NexusApiClient(private val settings: ConnectionSettings) {

    private val client = HttpClient(OkHttp) {
        // Make non-2xx responses throw a ResponseException so authedRequest
        // surfaces them as Result.failure. Without this, Ktor 2.x silently
        // returns the error body and our DELETE handlers (which don't call
        // .body()) would treat 4xx/5xx as success.
        expectSuccess = true
        install(ContentNegotiation) {
            json(Json { ignoreUnknownKeys = true })
        }
        install(HttpTimeout) {
            requestTimeoutMillis = 60_000
            connectTimeoutMillis = 10_000
        }
    }

    fun close() {
        client.close()
    }

    private val baseUrl get() = settings.coreUrl
    private val token get() = settings.token

    // last health-check error, exposed to UI for debugging
    @Volatile var lastHealthError: String? = null
        private set

    // Pairing

    suspend fun pairHandshake(): Result<Unit> = authedRequest {
        val response = client.post("$baseUrl/api/pair/handshake") {
            bearerAuth(token!!)
        }
        if (!response.status.isSuccess()) {
            error("Handshake HTTP ${response.status.value}")
        }
        Unit
    }

    // Health

    suspend fun checkHealth(): Boolean {
        val url = baseUrl
        if (url == null) {
            lastHealthError = "Keine Core-URL gespeichert"
            return false
        }
        return try {
            val response: HealthResponse = client.get("$url/health").body()
            val ok = response.status == "ok"
            lastHealthError = if (ok) null else "status=${response.status}"
            ok
        } catch (e: Exception) {
            android.util.Log.w("NexusApiClient", "checkHealth failed for $url", e)
            lastHealthError = "${e::class.java.simpleName}: ${e.message}"
            false
        }
    }

    // Spark

    suspend fun sendSpark(text: String): Result<SparkResponse> = authedRequest {
        client.post("$baseUrl/spark") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(SparkRequest(text))
        }.body()
    }

    suspend fun getSparks(): Result<List<SparkResponse>> = authedRequest {
        client.get("$baseUrl/spark") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun deleteSpark(id: String): Result<Unit> = authedRequest {
        client.delete("$baseUrl/spark/$id") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun getUnsortedCount(): Result<Long> = authedRequest {
        val response: UnsortedCountResponse = client.get("$baseUrl/spark/unsorted/count") {
            bearerAuth(token!!)
        }.body()
        response.count
    }

    /**
     * DANIEL-FUNKTIONAL DC-001..004: Aggregat-Endpoint für die vier
     * Dashboard-Stat-Cards. Ein Roundtrip statt drei Einzelaufrufe für
     * Sparks/Tasks/Projects-Counts.
     */
    suspend fun getDashboardStats(): Result<DashboardStatsResponse> = authedRequest {
        client.get("$baseUrl/api/dashboard/stats") {
            bearerAuth(token!!)
        }.body()
    }

    /**
     * DANIEL-FUNKTIONAL DC-003: Nächster offener Task mit Fälligkeit für
     * die Hero-Card. `result.task == null` bedeutet keine passende Aufgabe
     * (alle erledigt oder ohne Datum).
     */
    suspend fun getDashboardNextFocus(): Result<DashboardNextFocusResponse> = authedRequest {
        client.get("$baseUrl/api/dashboard/next-focus") {
            bearerAuth(token!!)
        }.body()
    }

    /**
     * FEAT-001: Extrahiert Action-Items aus dem Spark via LLM und legt sie als
     * Tasks an. Idempotent — wiederholter Aufruf erzeugt keine Duplikate.
     */
    suspend fun extractTasksFromSpark(sparkId: String): Result<ExtractTasksResponse> = authedRequest {
        client.post("$baseUrl/spark/$sparkId/extract-tasks") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun updateSparkTags(id: String, tags: List<String>): Result<Unit> = authedRequest {
        client.post("$baseUrl/spark/$id/tags") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(UpdateSparkTagsRequest(tags))
        }.body()
    }

    // Tasks

    suspend fun getTasks(
        projectId: String? = null,
        status: String? = null
    ): Result<List<TaskResponse>> = authedRequest {
        client.get("$baseUrl/tasks") {
            bearerAuth(token!!)
            projectId?.let { parameter("project_id", it) }
            status?.let { parameter("status", it) }
        }.body()
    }

    suspend fun createTask(request: TaskCreateRequest): Result<TaskResponse> = authedRequest {
        client.post("$baseUrl/tasks") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(request)
        }.body()
    }

    suspend fun updateTask(id: String, request: TaskUpdateRequest): Result<TaskResponse> = authedRequest {
        client.put("$baseUrl/tasks/$id") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(request)
        }.body()
    }

    suspend fun deleteTask(id: String): Result<Unit> = authedRequest {
        client.delete("$baseUrl/tasks/$id") {
            bearerAuth(token!!)
        }
        Unit
    }

    // Projects

    suspend fun getProjects(): Result<List<ProjectResponse>> = authedRequest {
        client.get("$baseUrl/projects") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun createProject(request: ProjectCreateRequest): Result<ProjectResponse> = authedRequest {
        client.post("$baseUrl/projects") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(request)
        }.body()
    }

    suspend fun getProject(id: String): Result<ProjectResponse> = authedRequest {
        client.get("$baseUrl/projects/$id") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun updateProject(id: String, request: ProjectUpdateRequest): Result<ProjectResponse> = authedRequest {
        client.put("$baseUrl/projects/$id") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(request)
        }.body()
    }

    suspend fun deleteProject(id: String): Result<Unit> = authedRequest {
        client.delete("$baseUrl/projects/$id") {
            bearerAuth(token!!)
        }
        Unit
    }

    suspend fun getProjectProgress(projectId: String): Result<ProjectProgress> = authedRequest {
        client.get("$baseUrl/projects/$projectId/progress") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun getProjectSparks(projectId: String): Result<List<SparkResponse>> = authedRequest {
        client.get("$baseUrl/projects/$projectId/sparks") {
            bearerAuth(token!!)
        }.body()
    }

    // Synaptic Mosaic — Links + Project-Suggestions

    suspend fun getSparkLinks(sparkId: String): Result<SparkLinks> = authedRequest {
        client.get("$baseUrl/spark/$sparkId/links") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun listProjectSuggestions(): Result<List<ProjectSuggestion>> = authedRequest {
        client.get("$baseUrl/projects/suggestions") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun acceptProjectSuggestion(id: String): Result<AcceptSuggestionResponse> = authedRequest {
        client.post("$baseUrl/projects/suggestions/$id/accept") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun dismissProjectSuggestion(id: String): Result<Unit> = authedRequest {
        client.post("$baseUrl/projects/suggestions/$id/dismiss") {
            bearerAuth(token!!)
        }
        Unit
    }

    // Diagnostics

    suspend fun submitDiagReport(report: DiagReport): Result<DiagReportAck> = authedRequest {
        client.post("$baseUrl/api/diag/report") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(report)
        }.body()
    }

    // Settings (Phase C)
    suspend fun getProviders(): Result<ProvidersResponse> = authedRequest {
        client.get("$baseUrl/api/settings/providers") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun getModels(provider: String): Result<ModelsResponse> = authedRequest {
        client.get("$baseUrl/api/settings/models") {
            bearerAuth(token!!)
            parameter("provider", provider)
        }.body()
    }

    suspend fun setProvider(request: SetProviderRequest): Result<Unit> = authedRequest {
        client.post("$baseUrl/api/settings/provider") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(request)
        }
        Unit
    }

    suspend fun getUserPrefs(): Result<Map<String, String>> = authedRequest {
        client.get("$baseUrl/api/user_prefs") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun setUserPref(key: String, value: String): Result<Unit> = authedRequest {
        client.post("$baseUrl/api/user_prefs/$key") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(mapOf("value" to value))
        }
        Unit
    }

    suspend fun listDiagReports(
        limit: Int = 5,
        source: String? = null,
        deviceId: String? = null,
    ): Result<List<DiagReport>> = authedRequest {
        client.get("$baseUrl/api/diag/reports") {
            bearerAuth(token!!)
            parameter("limit", limit)
            source?.let { parameter("source", it) }
            deviceId?.let { parameter("device_id", it) }
        }.body()
    }

    private inline fun <T> authedRequest(block: () -> T): Result<T> {
        if (baseUrl == null) return Result.failure(Exception("Nicht gekoppelt"))
        if (token == null) return Result.failure(Exception("Kein Token"))
        return try {
            Result.success(block())
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
