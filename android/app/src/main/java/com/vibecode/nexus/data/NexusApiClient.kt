package com.vibecode.nexus.data

import com.vibecode.nexus.data.model.BrainDumpRequest
import com.vibecode.nexus.data.model.BrainDumpResponse
import com.vibecode.nexus.data.model.HealthResponse
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.get
import io.ktor.client.request.post
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
    }

    suspend fun checkHealth(): Boolean {
        return try {
            val url = settings.coreUrl ?: return false
            val response: HealthResponse = client.get("$url/health").body()
            response.status == "ok"
        } catch (_: Exception) {
            false
        }
    }

    suspend fun sendBrainDump(text: String): Result<BrainDumpResponse> {
        return try {
            val url = settings.coreUrl ?: return Result.failure(Exception("Nicht gekoppelt"))
            val token = settings.token ?: return Result.failure(Exception("Kein Token"))
            val response: BrainDumpResponse = client.post("$url/braindump") {
                contentType(ContentType.Application.Json)
                bearerAuth(token)
                setBody(BrainDumpRequest(text))
            }.body()
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
