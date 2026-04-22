package com.vibecode.nexus.data

import com.vibecode.nexus.data.model.BrainDumpRequest
import com.vibecode.nexus.data.model.BrainDumpResponse
import com.vibecode.nexus.data.model.HealthResponse
import com.vibecode.nexus.data.model.ProjectProgress
import com.vibecode.nexus.data.model.ProjectResponse
import com.vibecode.nexus.data.model.TaskCreateRequest
import com.vibecode.nexus.data.model.TaskResponse
import com.vibecode.nexus.data.model.TaskUpdateRequest
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
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json

class NexusApiClient(private val settings: ConnectionSettings) {

    private val client = HttpClient(OkHttp) {
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

    // BrainDump

    suspend fun sendBrainDump(text: String): Result<BrainDumpResponse> = authedRequest {
        client.post("$baseUrl/braindump") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(BrainDumpRequest(text))
        }.body()
    }

    suspend fun getBrainDumps(): Result<List<BrainDumpResponse>> = authedRequest {
        client.get("$baseUrl/braindump") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun deleteBrainDump(id: String): Result<Unit> = authedRequest {
        client.delete("$baseUrl/braindump/$id") {
            bearerAuth(token!!)
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

    suspend fun updateTask(id: Long, request: TaskUpdateRequest): Result<TaskResponse> = authedRequest {
        client.put("$baseUrl/tasks/$id") {
            contentType(ContentType.Application.Json)
            bearerAuth(token!!)
            setBody(request)
        }.body()
    }

    suspend fun deleteTask(id: Long): Result<Unit> = authedRequest {
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

    suspend fun getProjectProgress(projectId: String): Result<ProjectProgress> = authedRequest {
        client.get("$baseUrl/projects/$projectId/progress") {
            bearerAuth(token!!)
        }.body()
    }

    suspend fun getProjectBrainDumps(projectId: String): Result<List<BrainDumpResponse>> = authedRequest {
        client.get("$baseUrl/projects/$projectId/braindumps") {
            bearerAuth(token!!)
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
