package com.vibecode.nexus.ui.screen

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
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
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.model.BrainDumpResponse
import com.vibecode.nexus.speech.RecognizerState
import com.vibecode.nexus.speech.SpeechRecognizerManager
import kotlinx.coroutines.launch

@Composable
fun BrainDumpScreen(
    speechManager: SpeechRecognizerManager,
    apiClient: NexusApiClient,
    isPaired: Boolean,
    isConnected: Boolean?,
    hasPermission: Boolean,
    onRequestPermission: () -> Unit,
    onNavigateToSettings: () -> Unit,
    modifier: Modifier = Modifier
) {
    val speechState by speechManager.state.collectAsState()
    val isListening = speechState.recognizerState == RecognizerState.LISTENING
    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()

    var editableText by remember { mutableStateOf("") }
    var isSending by remember { mutableStateOf(false) }
    var lastResult by remember { mutableStateOf<BrainDumpResponse?>(null) }

    // Sync finalText into editable field
    LaunchedEffect(speechState.finalText) {
        if (speechState.finalText.isNotBlank()) {
            editableText = speechState.finalText
            lastResult = null
        }
    }

    // Show errors via snackbar
    LaunchedEffect(speechState.errorMessage) {
        speechState.errorMessage?.let {
            snackbarHostState.showSnackbar(it)
        }
    }

    Scaffold(
        snackbarHost = { SnackbarHost(snackbarHostState) },
        modifier = modifier
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.SpaceBetween
        ) {
            // Header with connection status
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(
                        text = "NEXUS",
                        style = MaterialTheme.typography.headlineMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary
                    )
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = "BrainDump",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(Modifier.width(8.dp))
                        ConnectionDot(isConnected)
                    }
                }
                IconButton(onClick = onNavigateToSettings) {
                    Icon(
                        Icons.Default.Settings,
                        contentDescription = "Einstellungen",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            // Content area
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(vertical = 16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                // Result card
                AnimatedVisibility(
                    visible = lastResult != null,
                    enter = fadeIn(),
                    exit = fadeOut()
                ) {
                    lastResult?.let { result ->
                        ResultCard(result)
                    }
                }

                if (!speechState.isAvailable) {
                    Text(
                        text = "Spracherkennung nicht verfügbar.\nPrüfe deine Internetverbindung.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.error,
                        textAlign = TextAlign.Center
                    )
                } else if (isListening && speechState.partialText.isNotBlank()) {
                    Text(
                        text = speechState.partialText,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        textAlign = TextAlign.Center,
                        modifier = Modifier.padding(horizontal = 16.dp)
                    )
                } else if (isListening) {
                    Text(
                        text = "Sprich jetzt…",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                if (editableText.isNotBlank() && !isListening && lastResult == null) {
                    OutlinedTextField(
                        value = editableText,
                        onValueChange = { editableText = it },
                        label = { Text("Transkript bearbeiten") },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 16.dp),
                        minLines = 3,
                        maxLines = 8
                    )
                }
            }

            // Controls
            Column(
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                if (lastResult == null) {
                    RecordButton(
                        isListening = isListening,
                        onClick = {
                            if (!hasPermission) {
                                onRequestPermission()
                            } else if (isListening) {
                                speechManager.stopListening()
                            } else {
                                speechManager.startListening()
                            }
                        }
                    )
                }

                Spacer(modifier = Modifier.height(24.dp))

                // Confirm & send button
                if (editableText.isNotBlank() && !isListening && lastResult == null) {
                    Button(
                        onClick = {
                            if (!isPaired) {
                                scope.launch {
                                    snackbarHostState.showSnackbar("Zuerst mit Core koppeln → Einstellungen")
                                }
                                return@Button
                            }
                            isSending = true
                            scope.launch {
                                val result = apiClient.sendBrainDump(editableText)
                                isSending = false
                                result.onSuccess { response ->
                                    lastResult = response
                                    editableText = ""
                                    speechManager.clearText()
                                }.onFailure { e ->
                                    snackbarHostState.showSnackbar(
                                        "Senden fehlgeschlagen: ${e.message}"
                                    )
                                }
                            }
                        },
                        enabled = !isSending,
                        modifier = Modifier.fillMaxWidth(),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary
                        )
                    ) {
                        if (isSending) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(20.dp),
                                color = MaterialTheme.colorScheme.onPrimary,
                                strokeWidth = 2.dp
                            )
                            Spacer(Modifier.width(8.dp))
                            Text("Sende…")
                        } else {
                            Icon(
                                Icons.Default.Check,
                                contentDescription = null,
                                modifier = Modifier.padding(end = 8.dp)
                            )
                            Text("Bestätigen & Senden")
                        }
                    }
                }

                // New dump button after result
                if (lastResult != null) {
                    Button(
                        onClick = {
                            lastResult = null
                            editableText = ""
                        },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Icon(
                            Icons.Default.Mic,
                            contentDescription = null,
                            modifier = Modifier.padding(end = 8.dp)
                        )
                        Text("Neuer BrainDump")
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))
            }
        }
    }
}

@Composable
private fun ResultCard(result: BrainDumpResponse) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.primaryContainer
        ),
        shape = RoundedCornerShape(16.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                CategoryBadge(result.category)
                Spacer(Modifier.width(8.dp))
                Text(
                    text = result.category,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onPrimaryContainer
                )
            }
            Spacer(Modifier.height(8.dp))
            Text(
                text = result.summary,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer
            )
            if (result.tags.isNotEmpty()) {
                Spacer(Modifier.height(8.dp))
                Text(
                    text = result.tags.joinToString(" ") { "#$it" },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f)
                )
            }
        }
    }
}

@Composable
private fun CategoryBadge(category: String) {
    val emoji = when (category.lowercase()) {
        "idea" -> "💡"
        "task" -> "✅"
        "worry" -> "😟"
        "question" -> "❓"
        "random" -> "🎲"
        else -> "📝"
    }
    Text(text = emoji, style = MaterialTheme.typography.titleLarge)
}

@Composable
private fun RecordButton(
    isListening: Boolean,
    onClick: () -> Unit
) {
    val buttonColor by animateColorAsState(
        targetValue = if (isListening) Color(0xFFB71C1C) else Color(0xFFD32F2F),
        label = "recordButtonColor"
    )

    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val scale by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = if (isListening) 1.1f else 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(600),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulseScale"
    )

    Box(contentAlignment = Alignment.Center) {
        FilledIconButton(
            onClick = onClick,
            modifier = Modifier
                .size(96.dp)
                .scale(scale),
            shape = CircleShape,
            colors = IconButtonDefaults.filledIconButtonColors(
                containerColor = buttonColor,
                contentColor = Color.White
            )
        ) {
            Icon(
                imageVector = if (isListening) Icons.Default.Stop else Icons.Default.Mic,
                contentDescription = if (isListening) "Aufnahme stoppen" else "Aufnahme starten",
                modifier = Modifier.size(48.dp)
            )
        }
    }
}
