package com.vibecode.nexus.data

import android.content.Context
import android.content.SharedPreferences
import android.net.Uri
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

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

    fun clear() {
        prefs.edit().clear().apply()
    }

    companion object {
        private const val TAG = "ConnectionSettings"
        private const val PREFS_NAME = "nexus_connection"
        private const val KEY_URL = "core_url"
        private const val KEY_TOKEN = "core_token"

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
                Log.w(TAG, "Encrypted prefs still failing after reset — falling back to plain SharedPreferences", e)
                context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            }
        }
    }
}
