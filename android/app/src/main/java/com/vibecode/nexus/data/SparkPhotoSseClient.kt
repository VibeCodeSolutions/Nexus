package com.vibecode.nexus.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.job
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.concurrent.TimeUnit

sealed class SparkPhotoFrame {
    data class Line(val text: String) : SparkPhotoFrame()
    data class Tags(val tags: List<String>) : SparkPhotoFrame()
    data class Done(val sparkId: String, val imageUrl: String?) : SparkPhotoFrame()
    data class ErrorFrame(val message: String) : SparkPhotoFrame()
}

/**
 * Konsumiert den SSE-Stream von `POST /spark/from_image` als Flow.
 * Ktor hat in 3.0 noch keinen stabilen Multipart-+-SSE-Combo; wir nutzen OkHttp
 * direkt für den Streaming-Body und parsen die SSE-Frames manuell — analog
 * zum Desktop-Consumer in `desktop/src/index.html`.
 */
class SparkPhotoSseClient(private val settings: ConnectionSettings) {

    private val okhttp: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(10, TimeUnit.SECONDS)
        .readTimeout(2, TimeUnit.MINUTES)
        .build()

    private val json = Json { ignoreUnknownKeys = true; isLenient = true }

    fun streamFromImage(jpegBytes: ByteArray, filename: String = "photo.jpg"): Flow<SparkPhotoFrame> = flow {
        val token = settings.token
            ?: throw IllegalStateException("Kein Pairing-Token — bitte erst koppeln.")

        val body = MultipartBody.Builder()
            .setType(MultipartBody.FORM)
            .addFormDataPart(
                "image", filename,
                jpegBytes.toRequestBody("image/jpeg".toMediaTypeOrNull()),
            )
            .build()

        val req = Request.Builder()
            .url("${settings.coreUrl}/spark/from_image")
            .header("Authorization", "Bearer $token")
            .header("Accept", "text/event-stream")
            .post(body)
            .build()

        // Coroutine-Cancel propagiert nicht automatisch zu OkHttp — Call-Referenz
        // explizit an Job binden, damit `flow.cancel()` (z.B. beim Sheet-Close)
        // den blockierenden `execute()` sofort abbricht statt bis Read-Timeout zu hängen.
        val call = okhttp.newCall(req)
        currentCoroutineContext().job.invokeOnCompletion { call.cancel() }

        call.execute().use { response ->
            if (!response.isSuccessful) {
                val detail = response.body?.string()?.take(200) ?: response.message
                emit(SparkPhotoFrame.ErrorFrame("HTTP ${response.code}: $detail"))
                return@flow
            }
            val source = response.body?.source()
                ?: run {
                    emit(SparkPhotoFrame.ErrorFrame("Leerer Response-Body"))
                    return@flow
                }
            val frameBuffer = StringBuilder()
            while (!source.exhausted()) {
                val line = source.readUtf8Line() ?: break
                if (line.isEmpty()) {
                    parseFrame(frameBuffer.toString())?.let { emit(it) }
                    frameBuffer.clear()
                } else {
                    frameBuffer.append(line).append('\n')
                }
            }
            if (frameBuffer.isNotEmpty()) {
                parseFrame(frameBuffer.toString())?.let { emit(it) }
            }
        }
    }.flowOn(Dispatchers.IO)

    private fun parseFrame(raw: String): SparkPhotoFrame? {
        var event = "message"
        val dataLines = mutableListOf<String>()
        for (line in raw.split('\n')) {
            if (line.isBlank() || line.startsWith(":")) continue
            val colon = line.indexOf(':')
            if (colon < 0) continue
            val field = line.substring(0, colon)
            val value = line.substring(colon + 1).removePrefix(" ")
            when (field) {
                "event" -> event = value
                "data" -> dataLines.add(value)
            }
        }
        if (dataLines.isEmpty()) return null
        val payload = runCatching { json.parseToJsonElement(dataLines.joinToString("\n")).jsonObject }
            .getOrNull() ?: JsonObject(emptyMap())
        return when (event) {
            "line" -> SparkPhotoFrame.Line(payload["text"]?.jsonPrimitive?.contentOrNull.orEmpty())
            "tags" -> SparkPhotoFrame.Tags(
                payload["tags"]?.jsonArray
                    ?.mapNotNull { it.jsonPrimitive.contentOrNull }
                    ?: emptyList()
            )
            "done" -> SparkPhotoFrame.Done(
                sparkId = payload["spark_id"]?.jsonPrimitive?.contentOrNull
                    ?: payload["braindump_id"]?.jsonPrimitive?.contentOrNull.orEmpty(),
                imageUrl = payload["image_url"]?.jsonPrimitive?.contentOrNull,
            )
            "error" -> SparkPhotoFrame.ErrorFrame(
                payload["message"]?.jsonPrimitive?.contentOrNull ?: "Unbekannter Fehler"
            )
            else -> null
        }
    }
}
