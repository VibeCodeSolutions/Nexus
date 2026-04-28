package com.vibecode.nexus

import android.app.Application
import android.util.Log
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.diagnostics.DiagReport
import com.vibecode.nexus.diagnostics.DiagnosticRunner
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class NexusApplication : Application() {

    val applicationScope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private val _latestDiag = MutableStateFlow<DiagReport?>(null)
    val latestDiag: StateFlow<DiagReport?> = _latestDiag

    override fun onCreate() {
        super.onCreate()
        if (!BuildConfig.DEBUG) return
        applicationScope.launch {
            try {
                runDiagnostics()
            } catch (e: Throwable) {
                Log.w("NexusApplication", "boot diagnostics failed", e)
            }
        }
    }

    suspend fun runDiagnostics() {
        val settings = ConnectionSettings(this)
        val apiClient = NexusApiClient(settings)
        try {
            val runner = DiagnosticRunner(this, settings, apiClient)
            val result = runner.runAndUpload()
            result.getOrNull()?.let { _latestDiag.value = it }
        } finally {
            apiClient.close()
        }
    }
}
