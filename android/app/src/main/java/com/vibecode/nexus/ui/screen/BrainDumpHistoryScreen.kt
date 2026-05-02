package com.vibecode.nexus.ui.screen

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.model.BrainDumpLinks
import com.vibecode.nexus.data.model.BrainDumpResponse
import com.vibecode.nexus.data.model.Link
import com.vibecode.nexus.data.model.ProjectResponse
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BrainDumpHistoryScreen(apiClient: NexusApiClient) {
    val scope = rememberCoroutineScope()
    var entries by remember { mutableStateOf<List<BrainDumpResponse>>(emptyList()) }
    var projects by remember { mutableStateOf<List<ProjectResponse>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var errorMsg by remember { mutableStateOf<String?>(null) }
    var showOnlyUnsorted by remember { mutableStateOf(false) }
    var detailEntry by remember { mutableStateOf<BrainDumpResponse?>(null) }
    val snackbarHostState = remember { SnackbarHostState() }

    fun load() {
        scope.launch {
            isLoading = true
            apiClient.getBrainDumps()
                .onSuccess { entries = it; isLoading = false }
                .onFailure { errorMsg = it.message; isLoading = false }
            apiClient.getProjects().onSuccess { projects = it }
        }
    }

    LaunchedEffect(Unit) { load() }

    val unsortedCount = entries.count { it.category.isNullOrBlank() || it.category == "Unsorted" }
    val visibleEntries = if (showOnlyUnsorted) {
        entries.filter { it.category.isNullOrBlank() || it.category == "Unsorted" }
    } else entries

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

            if (unsortedCount > 0) {
                FilterChip(
                    selected = showOnlyUnsorted,
                    onClick = { showOnlyUnsorted = !showOnlyUnsorted },
                    label = { Text("Unsortiert ($unsortedCount)") },
                    modifier = Modifier.padding(bottom = 8.dp)
                )
            }

            when {
                isLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
                errorMsg != null -> Text("Fehler: $errorMsg", color = MaterialTheme.colorScheme.error)
                visibleEntries.isEmpty() -> Text(
                    if (showOnlyUnsorted) "Keine unsortierten Einträge."
                    else "Keine BrainDumps vorhanden.",
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                else -> LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(visibleEntries, key = { it.id }) { entry ->
                        SwipeToDismissItem(
                            entry = entry,
                            onClick = { detailEntry = entry },
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

    detailEntry?.let { entry ->
        BrainDumpDetailSheet(
            entry = entry,
            apiClient = apiClient,
            entries = entries,
            projects = projects,
            onNavigateToBraindump = { newId ->
                entries.firstOrNull { it.id == newId }?.let { detailEntry = it }
            },
            onNavigateToProject = { name ->
                scope.launch { snackbarHostState.showSnackbar("Projekt im Projekte-Tab: $name") }
                detailEntry = null
            },
            onDismiss = { detailEntry = null }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun BrainDumpDetailSheet(
    entry: BrainDumpResponse,
    apiClient: NexusApiClient,
    entries: List<BrainDumpResponse>,
    projects: List<ProjectResponse>,
    onNavigateToBraindump: (String) -> Unit,
    onNavigateToProject: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var links by remember(entry.id) { mutableStateOf<BrainDumpLinks?>(null) }
    var linksError by remember(entry.id) { mutableStateOf<String?>(null) }
    var linksLoading by remember(entry.id) { mutableStateOf(true) }

    LaunchedEffect(entry.id) {
        linksLoading = true
        linksError = null
        apiClient.getBrainDumpLinks(entry.id)
            .onSuccess { links = it; linksLoading = false }
            .onFailure { linksError = it.message; linksLoading = false }
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp)
                .padding(bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = "BrainDump",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold
            )

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                AssistChip(
                    onClick = {},
                    enabled = false,
                    label = { Text(entry.category ?: "Unsorted") }
                )
                Text(
                    text = entry.created_at.take(16).replace("T", " "),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.align(Alignment.CenterVertically)
                )
            }

            Text(entry.raw_text, style = MaterialTheme.typography.bodyMedium)

            if (!entry.summary.isNullOrBlank()) {
                Surface(
                    shape = RoundedCornerShape(8.dp),
                    color = MaterialTheme.colorScheme.surfaceVariant,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(12.dp)) {
                        Text(
                            "Zusammenfassung",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(Modifier.height(4.dp))
                        Text(entry.summary, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }

            if (entry.tags.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    entry.tags.forEach { tag ->
                        AssistChip(onClick = {}, enabled = false, label = { Text(tag) })
                    }
                }
            }

            HorizontalDivider()

            Text(
                "Verknüpft mit",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold
            )

            when {
                linksLoading -> Row(
                    modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                    horizontalArrangement = Arrangement.Center
                ) { CircularProgressIndicator(strokeWidth = 2.dp) }
                linksError != null -> Text(
                    "Verknüpfungen konnten nicht geladen werden.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                else -> {
                    val data = links
                    val visibleOut = data?.outgoing.orEmpty().filterNot { it.isSentinel() }
                    val visibleIn = data?.incoming.orEmpty().filterNot { it.isSentinel() }
                    if (visibleOut.isEmpty() && visibleIn.isEmpty()) {
                        Text(
                            "Keine Verknüpfungen — werden im Background extrahiert, sobald genug Kontext vorhanden ist.",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    } else {
                        if (visibleOut.isNotEmpty()) {
                            WikiLinkFlow(visibleOut, dirIsOut = true, entries = entries, projects = projects,
                                onBraindump = onNavigateToBraindump, onProject = onNavigateToProject)
                        }
                        if (visibleIn.isNotEmpty()) {
                            Text(
                                "Rückverweise",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            WikiLinkFlow(visibleIn, dirIsOut = false, entries = entries, projects = projects,
                                onBraindump = onNavigateToBraindump, onProject = onNavigateToProject)
                        }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun WikiLinkFlow(
    links: List<Link>,
    dirIsOut: Boolean,
    entries: List<BrainDumpResponse>,
    projects: List<ProjectResponse>,
    onBraindump: (String) -> Unit,
    onProject: (String) -> Unit,
) {
    FlowRow(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        links.forEach { link ->
            val type = if (dirIsOut) link.target_type else link.source_type
            val id = if (dirIsOut) link.target_id else link.source_id
            val label = wikiLabelFor(type, id, entries, projects)
            val confPct = (link.confidence * 100).toInt()
            Surface(
                shape = RoundedCornerShape(8.dp),
                color = MaterialTheme.colorScheme.secondaryContainer,
                modifier = Modifier.clickable {
                    if (type == "project") onProject(label) else onBraindump(id)
                }
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(label, style = MaterialTheme.typography.bodySmall)
                    Text(
                        "$confPct%",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

private fun Link.isSentinel(): Boolean = relation == "noop-marker" && created_by == "llm"

private fun wikiLabelFor(
    type: String,
    id: String,
    entries: List<BrainDumpResponse>,
    projects: List<ProjectResponse>,
): String {
    if (type == "project") {
        val p = projects.firstOrNull { it.id == id }
        return "📁 " + (p?.name ?: id)
    }
    val bd = entries.firstOrNull { it.id == id }
    if (bd != null) {
        val title = bd.summary?.takeIf { it.isNotBlank() } ?: bd.raw_text.take(60)
        return "📝 $title"
    }
    return "📝 $id"
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SwipeToDismissItem(
    entry: BrainDumpResponse,
    onClick: () -> Unit,
    onDelete: () -> Unit,
) {
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
        Card(modifier = Modifier.fillMaxWidth().clickable { onClick() }) {
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
