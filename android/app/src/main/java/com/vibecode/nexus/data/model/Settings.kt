package com.vibecode.nexus.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ProviderStatus(
    val name: String,
    @SerialName("has_key") val hasKey: Boolean,
    @SerialName("has_model") val hasModel: Boolean,
    @SerialName("is_default") val isDefault: Boolean,
)

@Serializable
data class ProvidersResponse(
    val providers: List<ProviderStatus>,
)

@Serializable
data class ModelsResponse(
    val provider: String,
    val models: List<String>,
    val current: String? = null,
)

@Serializable
data class SetProviderRequest(
    val provider: String,
    @SerialName("api_key") val apiKey: String? = null,
    val model: String? = null,
)
