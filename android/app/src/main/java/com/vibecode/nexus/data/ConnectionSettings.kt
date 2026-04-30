package com.vibecode.nexus.data

import android.content.Context
import android.content.SharedPreferences
import android.net.Uri
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.util.UUID

@Serializable
data class QrPayload(
    val url: String,
    val token: String
)

class ConnectionSettings(context: Context) {

    private val prefs: SharedPreferences = openPrefs(context)

    var coreUrl: String?
        get() = prefs.getString(KEY_URL, null)?.stripWhitespace()?.trimEnd('/')?.takeIf { it.isNotEmpty() }
        set(value) = prefs.edit().putString(KEY_URL, value?.stripWhitespace()).apply()

    var token: String?
        get() = prefs.getString(KEY_TOKEN, null)?.stripWhitespace()?.takeIf { it.isNotEmpty() }
        set(value) = prefs.edit().putString(KEY_TOKEN, value?.stripWhitespace()).apply()

    val deviceId: String
        @Synchronized
        get() {
            val existing = prefs.getString(KEY_DEVICE_ID, null)?.takeIf { it.isNotBlank() }
            if (existing != null) return existing
            val fresh = UUID.randomUUID().toString()
            prefs.edit().putString(KEY_DEVICE_ID, fresh).apply()
            return fresh
        }

    private fun String.stripWhitespace(): String = filter { !it.isWhitespace() }

    val isPaired: Boolean
        get() = !coreUrl.isNullOrBlank() && !token.isNullOrBlank()

    /**
     * Accepts both the new deep-link URI (`nexus://pair?url=…&token=…`)
     * and the legacy JSON payload (`{"url":"…","token":"…"}`).
     */
    fun saveFromQr(qrContent: String): Boolean {
        val cleaned = qrContent.stripWhitespace()
        if (cleaned.startsWith("nexus://")) {
            return saveFromDeepLink(cleaned)
        }
        return try {
            val payload = Json.decodeFromString<QrPayload>(qrContent.trim())
            coreUrl = payload.url.stripWhitespace().trimEnd('/')
            token = payload.token.stripWhitespace()
            true
        } catch (_: Exception) {
            false
        }
    }

    fun saveFromDeepLink(uriString: String): Boolean {
        return try {
            val uri = Uri.parse(uriString.stripWhitespace())
            if (uri.scheme != "nexus" || uri.host != "pair") return false
            val url = uri.getQueryParameter("url")?.stripWhitespace()?.trimEnd('/').orEmpty()
            val tok = uri.getQueryParameter("token")?.stripWhitespace().orEmpty()
            if (url.isEmpty() || tok.isEmpty()) return false
            coreUrl = url
            token = tok
            true
        } catch (_: Exception) {
            false
        }
    }

    /**
     * Clears the connection state (URL + token) but keeps [deviceId] so
     * diagnostic reports correlate across re-pair cycles.
     */
    fun clear() {
        prefs.edit()
            .remove(KEY_URL)
            .remove(KEY_TOKEN)
            .apply()
    }

    companion object {
        private const val TAG = "ConnectionSettings"
        private const val PREFS_NAME = "nexus_connection"
        private const val KEY_URL = "core_url"
        private const val KEY_TOKEN = "core_token"
        private const val KEY_DEVICE_ID = "device_id"

        // Both the MasterKey build and the EncryptedSharedPreferences create
        // can throw when a restored-from-backup prefs file no longer matches
        // the device keystore. Treat the whole bring-up as one operation.
        private fun buildEncrypted(context: Context): SharedPreferences {
            val masterKey = MasterKey.Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()
            return EncryptedSharedPreferences.create(
                context,
                PREFS_NAME,
                masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
            )
        }

        private fun openPrefs(context: Context): SharedPreferences {
            try {
                return buildEncrypted(context)
            } catch (e: Exception) {
                Log.w(TAG, "Encrypted prefs unreadable — resetting and retrying", e)
            }

            // Reset attempt: clear any leftover prefs file before the retry, in
            // case a stale entry from a restored backup is what's tripping the
            // keystore.
            try {
                context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                    .edit().clear().commit()
            } catch (_: Exception) { /* best effort */ }
            try {
                context.deleteSharedPreferences(PREFS_NAME)
            } catch (_: Exception) { /* best effort */ }

            return try {
                buildEncrypted(context)
            } catch (e: Exception) {
                // Hard fail — we never write the bearer token to unencrypted
                // SharedPreferences. The earlier plain fallback was a silent
                // security regression; surface it instead. NexusApplication
                // and MainActivity must catch this and present a re-install
                // banner rather than crash.
                Log.e(TAG, "Encrypted prefs still failing after reset — refusing to fall back to plain", e)
                throw SecurityException(
                    "Encrypted storage unavailable. Reinstall NEXUS or check device security state.",
                    e
                )
            }
        }
    }
}
