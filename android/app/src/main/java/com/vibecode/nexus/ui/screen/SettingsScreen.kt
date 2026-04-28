package com.vibecode.nexus.ui.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.NexusApplication
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.diagnostics.DiagCheck
import com.vibecode.nexus.diagnostics.DiagReport
import com.vibecode.nexus.diagnostics.DiagStatus
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

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
    val nexusApp = context.applicationContext as? NexusApplication
    var isPaired by remember { mutableStateOf(connectionSettings.isPaired) }
    var coreUrl by remember { mutableStateOf(connectionSettings.coreUrl ?: "") }
    var isConnected by remember { mutableStateOf<Boolean?>(null) }
    var manualPairInput by remember { mutableStateOf("") }
    val latestDiag by (nexusApp?.latestDiag?.collectAsState()
        ?: remember { mutableStateOf<DiagReport?>(null) })
    var fallbackDiag by remember { mutableStateOf<DiagReport?>(null) }

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

    // Pull last server-side report once if we have nothing in the in-process flow yet.
    LaunchedEffect(isPaired, latestDiag) {
        if (isPaired && latestDiag == null && fallbackDiag == null) {
            apiClient.listDiagReports(
                limit = 1,
                source = "android",
                deviceId = connectionSettings.deviceId,
            ).getOrNull()?.firstOrNull()?.let { fallbackDiag = it }
        }
    }
    val effectiveDiag = latestDiag ?: fallbackDiag

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

            // Self-Diagnostics card
            DiagnosticsCard(
                report = effectiveDiag,
                onRefresh = if (isPaired) {
                    {
                        scope.launch {
                            nexusApp?.runDiagnostics()
                            if (nexusApp == null) {
                                apiClient.listDiagReports(
                                    limit = 1,
                                    source = "android",
                                    deviceId = connectionSettings.deviceId,
                                ).getOrNull()?.firstOrNull()?.let { fallbackDiag = it }
                            }
                        }
                        Unit
                    }
                } else null,
            )

            // QR Scan button — ML Kit Code Scanner (Play Services)
            Button(
                onClick = {
                    startQrPairingScan(
                        context = context,
                        onSuccess = { raw ->
                            applyPairing(raw, "Gepaart", "QR-Code hat kein gültiges Pairing-Format")
                        },
                        onCancel = {},
                        onFailure = { e ->
                            scope.launch {
                                snackbarHostState.showSnackbar(
                                    "Scanner-Fehler: ${e.message ?: e::class.java.simpleName}"
                                )
                            }
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

@Composable
private fun DiagnosticsCard(
    report: DiagReport?,
    onRefresh: (() -> Unit)?,
) {
    var expanded by remember { mutableStateOf(false) }
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = report != null) { expanded = !expanded },
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    "Selbst-Diagnose",
                    style = MaterialTheme.typography.titleMedium
                )
                Spacer(Modifier.weight(1f))
                if (report != null) {
                    Icon(
                        if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                        contentDescription = if (expanded) "Einklappen" else "Ausklappen"
                    )
                }
            }

            Spacer(Modifier.height(8.dp))

            if (report == null) {
                Text(
                    text = "Noch keine Diagnose verfügbar",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            } else {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    DiagBadge("✓ ${report.passCount}", Color(0xFF4CAF50))
                    DiagBadge("⚠ ${report.warnCount}", Color(0xFFFFA000))
                    DiagBadge("✗ ${report.failCount}", Color(0xFFF44336))
                }
                Spacer(Modifier.height(6.dp))
                Text(
                    text = "Stand: ${formatTimestamp(report.createdAt)}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                if (expanded) {
                    Spacer(Modifier.height(12.dp))
                    HorizontalDivider()
                    Spacer(Modifier.height(8.dp))
                    report.results.forEach { check ->
                        DiagCheckRow(check)
                    }
                }
            }

            if (onRefresh != null) {
                Spacer(Modifier.height(12.dp))
                OutlinedButton(onClick = onRefresh) {
                    Text("Erneut laufen lassen")
                }
            }
        }
    }
}

@Composable
private fun DiagBadge(text: String, color: Color) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodyMedium,
        color = color
    )
}

@Composable
private fun DiagCheckRow(check: DiagCheck) {
    val (icon: ImageVector, tint: Color) = when (check.status) {
        DiagStatus.PASS -> Icons.Default.CheckCircle to Color(0xFF4CAF50)
        DiagStatus.WARN -> Icons.Default.Warning to Color(0xFFFFA000)
        DiagStatus.FAIL -> Icons.Default.Error to Color(0xFFF44336)
    }
    Column(modifier = Modifier.padding(vertical = 4.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                icon,
                contentDescription = null,
                tint = tint,
                modifier = Modifier.size(16.dp)
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = check.name,
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier.weight(1f)
            )
            Text(
                text = "${check.durationMs} ms",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        check.message?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(start = 24.dp)
            )
        }
        check.error?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(start = 24.dp)
            )
        }
    }
}

private fun formatTimestamp(unixSeconds: Long?): String {
    if (unixSeconds == null) return "—"
    val fmt = SimpleDateFormat("dd.MM.yyyy HH:mm:ss", Locale.getDefault())
    return fmt.format(Date(unixSeconds * 1000))
}
