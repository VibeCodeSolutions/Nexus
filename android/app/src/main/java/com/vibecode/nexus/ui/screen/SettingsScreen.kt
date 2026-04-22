package com.vibecode.nexus.ui.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    connectionSettings: ConnectionSettings,
    apiClient: NexusApiClient,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    var isPaired by remember { mutableStateOf(connectionSettings.isPaired) }
    var coreUrl by remember { mutableStateOf(connectionSettings.coreUrl ?: "") }
    var isConnected by remember { mutableStateOf<Boolean?>(null) }
    var manualPairInput by remember { mutableStateOf("") }

    // Google ML Kit Code Scanner (Play Services component, keine Kamera-Permission nötig)
    val codeScanner = remember {
        val options = GmsBarcodeScannerOptions.Builder()
            .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
            .build()
        GmsBarcodeScanning.getClient(context, options)
    }

    fun applyPairing(raw: String, successMsg: String, failureMsg: String) {
        val ok = connectionSettings.saveFromQr(raw)
        if (ok) {
            isPaired = true
            coreUrl = connectionSettings.coreUrl ?: ""
            scope.launch {
                isConnected = apiClient.checkHealth()
                snackbarHostState.showSnackbar(successMsg)
            }
        } else {
            scope.launch {
                snackbarHostState.showSnackbar(failureMsg)
            }
        }
    }

    // Check connection on enter
    LaunchedEffect(isPaired) {
        if (isPaired) {
            isConnected = apiClient.checkHealth()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Einstellungen") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Zurück")
                    }
                }
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
        modifier = modifier
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            // Connection status card
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant
                )
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            "Verbindung zum Core",
                            style = MaterialTheme.typography.titleMedium
                        )
                        Spacer(Modifier.width(8.dp))
                        ConnectionDot(isConnected)
                    }

                    Spacer(Modifier.height(8.dp))

                    if (isPaired) {
                        Text(
                            text = coreUrl,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text(
                            text = when (isConnected) {
                                true -> "Verbunden"
                                false -> "Nicht erreichbar"
                                null -> "Prüfe…"
                            },
                            style = MaterialTheme.typography.bodySmall,
                            color = when (isConnected) {
                                true -> Color(0xFF4CAF50)
                                false -> MaterialTheme.colorScheme.error
                                null -> MaterialTheme.colorScheme.onSurfaceVariant
                            }
                        )
                        if (isConnected == false) {
                            apiClient.lastHealthError?.let { err ->
                                Spacer(Modifier.height(4.dp))
                                Text(
                                    text = err,
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.error
                                )
                            }
                            Spacer(Modifier.height(8.dp))
                            OutlinedButton(
                                onClick = {
                                    scope.launch {
                                        isConnected = null
                                        isConnected = apiClient.checkHealth()
                                    }
                                }
                            ) {
                                Text("Erneut testen")
                            }
                        }
                    } else {
                        Text(
                            text = "Noch nicht gekoppelt",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            // QR Scan button — ML Kit Code Scanner (Play Services)
            Button(
                onClick = {
                    codeScanner.startScan()
                        .addOnSuccessListener { barcode ->
                            val raw = barcode.rawValue.orEmpty()
                            if (raw.isEmpty()) {
                                scope.launch { snackbarHostState.showSnackbar("Leerer Code") }
                                return@addOnSuccessListener
                            }
                            applyPairing(raw, "Gepaart", "QR-Code hat kein gültiges Pairing-Format")
                        }
                        .addOnCanceledListener {
                            // User brach den Scan ab — stumm ignorieren
                        }
                        .addOnFailureListener { e ->
                            scope.launch {
                                snackbarHostState.showSnackbar(
                                    "Scanner-Fehler: ${e.message ?: e::class.java.simpleName}"
                                )
                            }
                        }
                },
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(
                    Icons.Default.QrCodeScanner,
                    contentDescription = null,
                    modifier = Modifier.padding(end = 8.dp)
                )
                Text(if (isPaired) "Erneut koppeln" else "QR-Code scannen")
            }

            // Manual paste fallback — akzeptiert Deep-Link-URI und Legacy-JSON
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                OutlinedTextField(
                    value = manualPairInput,
                    onValueChange = { manualPairInput = it },
                    label = { Text("Pairing-Daten manuell einfügen") },
                    singleLine = false,
                    minLines = 2,
                    modifier = Modifier.weight(1f)
                )
                Button(
                    onClick = {
                        val raw = manualPairInput
                        applyPairing(raw, "Gepaart", "Ungültige Pairing-Daten")
                        if (connectionSettings.isPaired) manualPairInput = ""
                    }
                ) {
                    Text("Anwenden")
                }
            }

            // Unpair button
            if (isPaired) {
                OutlinedButton(
                    onClick = {
                        connectionSettings.clear()
                        isPaired = false
                        coreUrl = ""
                        isConnected = null
                        scope.launch {
                            snackbarHostState.showSnackbar("Kopplung aufgehoben")
                        }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(
                        Icons.Default.Delete,
                        contentDescription = null,
                        modifier = Modifier.padding(end = 8.dp)
                    )
                    Text("Kopplung aufheben")
                }
            }
        }
    }
}

@Composable
fun ConnectionDot(isConnected: Boolean?) {
    val color = when (isConnected) {
        true -> Color(0xFF4CAF50)
        false -> Color(0xFFF44336)
        null -> Color(0xFF9E9E9E)
    }
    Box(
        modifier = Modifier
            .size(12.dp)
            .clip(CircleShape)
            .background(color)
    )
}
