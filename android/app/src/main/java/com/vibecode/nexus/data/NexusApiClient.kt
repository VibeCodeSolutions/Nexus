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
            requestTimeoutMillis = 10_000
            connectTimeoutMillis = 5_000
        }
    }

    fun close() {
        client.close()
    }

    private val baseUrl get() = settings.coreUrl
    private val token get() = settings.token

    // Health

    suspend fun checkHealth(): Boolean {
        return try {
            val response: HealthResponse = client.get("${baseUrl ?: return false}/health").body()
            response.status == "ok"
        } catch (_: Exception) {
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
