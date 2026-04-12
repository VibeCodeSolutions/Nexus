package com.vibecode.nexus.data

import android.content.Context
import android.content.SharedPreferences
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

    private val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    private val prefs: SharedPreferences = EncryptedSharedPreferences.create(
        context,
        "nexus_connection",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    var coreUrl: String?
        get() = prefs.getString(KEY_URL, null)
        set(value) = prefs.edit().putString(KEY_URL, value).apply()

    var token: String?
        get() = prefs.getString(KEY_TOKEN, null)
        set(value) = prefs.edit().putString(KEY_TOKEN, value).apply()

    val isPaired: Boolean
        get() = !coreUrl.isNullOrBlank() && !token.isNullOrBlank()

    fun saveFromQr(qrContent: String): Boolean {
        return try {
            val payload = Json.decodeFromString<QrPayload>(qrContent)
            coreUrl = payload.url.trimEnd('/')
            token = payload.token
            true
        } catch (_: Exception) {
            false
        }
    }

    fun clear() {
        prefs.edit().clear().apply()
    }

    companion object {
        private const val KEY_URL = "core_url"
        private const val KEY_TOKEN = "core_token"
    }
}
