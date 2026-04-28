package com.vibecode.nexus.diagnostics

import android.content.Context
import android.content.SharedPreferences
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.vibecode.nexus.BuildConfig
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlin.system.measureTimeMillis

private const val TAG = "DiagnosticRunner"
private const val LOG_TAG_JSON = "NEXUS_DIAG_JSON"
private const val DIAG_PREFS_NAME = "nexus_diag"
private const val SENTINEL_KEY = "roundtrip_sentinel"

class DiagnosticRunner(
    private val context: Context,
    private val settings: ConnectionSettings,
    private val apiClient: NexusApiClient,
) {

    private val json = Json {
        encodeDefaults = true
        explicitNulls = true
    }

    suspend fun run(): DiagReport {
        val results = mutableListOf<DiagCheck>()

        results += runCheck("settings.paired") {
            if (settings.isPaired) {
                CheckOutcome(DiagStatus.PASS, "url+token present")
            } else {
                CheckOutcome(DiagStatus.FAIL, "not paired")
            }
        }

        results += runCheck("network.online") {
            when (networkType()) {
                "none" -> CheckOutcome(DiagStatus.FAIL, "no active network")
                "unknown" -> CheckOutcome(DiagStatus.WARN, "ACCESS_NETWORK_STATE not granted")
                else -> CheckOutcome(DiagStatus.PASS, networkType())
            }
        }

        results += runCheck("core.health") {
            val ok = apiClient.checkHealth()
            if (ok) {
                CheckOutcome(DiagStatus.PASS, "200 ok")
            } else {
                CheckOutcome(
                    DiagStatus.FAIL,
                    apiClient.lastHealthError ?: "health probe failed",
                )
            }
        }

        results += runCheck("core.bearer") {
            if (!settings.isPaired) {
                CheckOutcome(DiagStatus.WARN, "skipped — not paired")
            } else {
                val res = apiClient.listDiagReports(limit = 1, deviceId = settings.deviceId)
                if (res.isSuccess) {
                    CheckOutcome(DiagStatus.PASS, "GET /api/diag/reports → 200")
                } else {
                    val msg = res.exceptionOrNull()?.message ?: "unknown error"
                    CheckOutcome(DiagStatus.FAIL, msg)
                }
            }
        }

        results += runCheck("prefs.roundtrip") {
            val prefs = openDiagPrefs()
            val sentinel = "ok-" + System.currentTimeMillis()
            prefs.edit().putString(SENTINEL_KEY, sentinel).commit()
            val readBack = prefs.getString(SENTINEL_KEY, null)
            if (readBack == sentinel) {
                CheckOutcome(DiagStatus.PASS, "encrypted prefs r/w ok")
            } else {
                CheckOutcome(DiagStatus.FAIL, "sentinel mismatch")
            }
        }

        results += runCheck("app.version") {
            CheckOutcome(DiagStatus.PASS, BuildConfig.VERSION_NAME)
        }

        results += runCheck("device.info") {
            CheckOutcome(DiagStatus.PASS, "${Build.MANUFACTURER} ${Build.MODEL} sdk${Build.VERSION.SDK_INT}")
        }

        return aggregate(results)
    }

    suspend fun runAndUpload(): Result<DiagReport> {
        val report = run()
        val payload = json.encodeToString(report)
        Log.i(LOG_TAG_JSON, payload)
        if (!settings.isPaired) {
            return Result.success(report)
        }
        return apiClient.submitDiagReport(report).map { report }
    }

    private suspend inline fun runCheck(
        name: String,
        crossinline block: suspend () -> CheckOutcome,
    ): DiagCheck {
        var outcome: CheckOutcome? = null
        var thrown: Throwable? = null
        val ms = measureTimeMillis {
            try {
                outcome = block()
            } catch (t: Throwable) {
                thrown = t
            }
        }
        return when {
            thrown != null -> DiagCheck(
                name = name,
                status = DiagStatus.FAIL,
                durationMs = ms,
                message = null,
                error = thrown!!::class.java.simpleName + ": " + (thrown!!.message ?: ""),
            )
            else -> DiagCheck(
                name = name,
                status = outcome!!.status,
                durationMs = ms,
                message = outcome!!.message,
                error = null,
            )
        }
    }

    private fun aggregate(results: List<DiagCheck>): DiagReport {
        val pass = results.count { it.status == DiagStatus.PASS }
        val warn = results.count { it.status == DiagStatus.WARN }
        val fail = results.count { it.status == DiagStatus.FAIL }
        return DiagReport(
            source = "android",
            deviceId = settings.deviceId,
            appVersion = BuildConfig.VERSION_NAME,
            deviceInfo = buildDeviceInfo(),
            results = results,
            passCount = pass,
            warnCount = warn,
            failCount = fail,
            createdAt = null,
        )
    }

    private fun buildDeviceInfo(): JsonObject = buildJsonObject {
        put("model", Build.MODEL)
        put("manufacturer", Build.MANUFACTURER)
        put("sdk", Build.VERSION.SDK_INT)
        put("network", networkType())
    }

    private fun networkType(): String {
        return try {
            val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
                ?: return "unknown"
            val active = cm.activeNetwork ?: return "none"
            val caps = cm.getNetworkCapabilities(active) ?: return "none"
            when {
                caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> "wifi"
                caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> "cellular"
                caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> "ethernet"
                else -> "other"
            }
        } catch (_: SecurityException) {
            "unknown"
        } catch (e: Exception) {
            Log.w(TAG, "networkType failed", e)
            "unknown"
        }
    }

    private fun openDiagPrefs(): SharedPreferences {
        return try {
            val masterKey = MasterKey.Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()
            EncryptedSharedPreferences.create(
                context,
                DIAG_PREFS_NAME,
                masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
            )
        } catch (e: Exception) {
            Log.w(TAG, "encrypted diag prefs failed, falling back to plain", e)
            context.getSharedPreferences(DIAG_PREFS_NAME, Context.MODE_PRIVATE)
        }
    }

    private data class CheckOutcome(val status: DiagStatus, val message: String?)
}
