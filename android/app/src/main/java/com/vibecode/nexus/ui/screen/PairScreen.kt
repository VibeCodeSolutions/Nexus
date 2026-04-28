package com.vibecode.nexus.ui.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import kotlinx.coroutines.launch

@Composable
fun PairScreen(
    connectionSettings: ConnectionSettings,
    onPaired: () -> Unit,
    onBack: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var scanTriggered by remember { mutableStateOf(false) }

    fun completePairing(raw: String) {
        if (!connectionSettings.saveFromQr(raw)) {
            errorMessage = "QR-Code hat kein gültiges Pairing-Format"
            return
        }
        scope.launch {
            val client = NexusApiClient(connectionSettings)
            val result = client.pairHandshake()
            client.close()
            if (result.isSuccess) {
                onPaired()
            } else {
                connectionSettings.clear()
                val cause = result.exceptionOrNull()
                errorMessage = "Pairing fehlgeschlagen: ${cause?.message ?: cause?.let { it::class.java.simpleName } ?: "unbekannt"}"
            }
        }
    }

    // Auto-open scanner on first composition
    LaunchedEffect(Unit) {
        if (!scanTriggered) {
            scanTriggered = true
            startQrPairingScan(
                context = context,
                onSuccess = { raw -> completePairing(raw) },
                onCancel = { /* stay on screen, show retry */ },
                onFailure = { e ->
                    errorMessage = "Scanner-Fehler: ${e.message ?: e::class.java.simpleName}"
                }
            )
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = "QR-Code scannen",
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center
        )

        Spacer(Modifier.height(16.dp))

        if (errorMessage != null) {
            Text(
                text = errorMessage!!,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center
            )
            Spacer(Modifier.height(24.dp))
        } else {
            Text(
                text = "Halte die Kamera auf den QR-Code im NEXUS Desktop-Client.",
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.height(24.dp))
        }

        Button(
            onClick = {
                errorMessage = null
                startQrPairingScan(
                    context = context,
                    onSuccess = { raw -> completePairing(raw) },
                    onCancel = {},
                    onFailure = { e ->
                        errorMessage = "Scanner-Fehler: ${e.message ?: e::class.java.simpleName}"
                    }
                )
            },
            modifier = Modifier.fillMaxWidth()
        ) {
            Icon(
                Icons.Default.QrCodeScanner,
                contentDescription = null,
                modifier = Modifier.padding(end = 8.dp)
            )
            Text("Erneut scannen")
        }

        Spacer(Modifier.height(12.dp))

        OutlinedButton(
            onClick = onBack,
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("Zurück")
        }
    }
}
