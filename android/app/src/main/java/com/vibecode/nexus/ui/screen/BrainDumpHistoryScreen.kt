package com.vibecode.nexus.ui.screen

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.model.BrainDumpResponse
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BrainDumpHistoryScreen(apiClient: NexusApiClient) {
    val scope = rememberCoroutineScope()
    var entries by remember { mutableStateOf<List<BrainDumpResponse>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var errorMsg by remember { mutableStateOf<String?>(null) }
    val snackbarHostState = remember { SnackbarHostState() }

    fun load() {
        scope.launch {
            isLoading = true
            apiClient.getBrainDumps()
                .onSuccess { entries = it; isLoading = false }
                .onFailure { errorMsg = it.message; isLoading = false }
        }
    }

    LaunchedEffect(Unit) { load() }

    Scaffold(snackbarHost = { SnackbarHost(snackbarHostState) }) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp)
        ) {
            Text(
                text = "BrainDumps",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 16.dp)
            )

            when {
                isLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
                errorMsg != null -> Text("Fehler: $errorMsg", color = MaterialTheme.colorScheme.error)
                entries.isEmpty() -> Text("Keine BrainDumps vorhanden.", color = MaterialTheme.colorScheme.onSurfaceVariant)
                else -> LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(entries, key = { it.id }) { entry ->
                        SwipeToDismissItem(
                            entry = entry,
                            onDelete = {
                                scope.launch {
                                    apiClient.deleteBrainDump(entry.id)
                                        .onSuccess {
                                            entries = entries.filter { it.id != entry.id }
                                            snackbarHostState.showSnackbar("Gelöscht")
                                        }
                                        .onFailure {
                                            snackbarHostState.showSnackbar("Löschen fehlgeschlagen: ${it.message}")
                                            load()
                                        }
                                }
                            }
                        )
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SwipeToDismissItem(entry: BrainDumpResponse, onDelete: () -> Unit) {
    val dismissState = rememberSwipeToDismissBoxState(
        confirmValueChange = { value ->
            if (value == SwipeToDismissBoxValue.EndToStart) {
                onDelete()
                true
            } else false
        }
    )

    SwipeToDismissBox(
        state = dismissState,
        enableDismissFromStartToEnd = false,
        backgroundContent = {
            val color by animateColorAsState(
                if (dismissState.targetValue == SwipeToDismissBoxValue.EndToStart)
                    MaterialTheme.colorScheme.errorContainer
                else Color.Transparent,
                label = "swipe_bg"
            )
            Box(
                modifier = Modifier.fillMaxSize().background(color).padding(end = 20.dp),
                contentAlignment = Alignment.CenterEnd
            ) {
                Icon(Icons.Default.Delete, contentDescription = "Löschen", tint = MaterialTheme.colorScheme.onErrorContainer)
            }
        }
    ) {
        Card(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(12.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = entry.category ?: "Unsorted",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary
                    )
                    Text(
                        text = entry.created_at.take(16).replace("T", " "),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                Spacer(Modifier.height(4.dp))
                Text(entry.raw_text, style = MaterialTheme.typography.bodyMedium)
                if (!entry.summary.isNullOrBlank()) {
                    Spacer(Modifier.height(4.dp))
                    Text(
                        entry.summary,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}
