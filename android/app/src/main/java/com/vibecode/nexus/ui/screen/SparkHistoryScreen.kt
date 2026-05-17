package com.vibecode.nexus.ui.screen

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
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
import com.vibecode.nexus.data.model.SparkLinks
import com.vibecode.nexus.data.model.SparkResponse
import com.vibecode.nexus.data.model.Link
import com.vibecode.nexus.data.model.ProjectResponse
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SparkHistoryScreen(apiClient: NexusApiClient) {
    val scope = rememberCoroutineScope()
    var entries by remember { mutableStateOf<List<SparkResponse>>(emptyList()) }
    var projects by remember { mutableStateOf<List<ProjectResponse>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var errorMsg by remember { mutableStateOf<String?>(null) }
    var detailEntry by remember { mutableStateOf<SparkResponse?>(null) }
    val snackbarHostState = remember { SnackbarHostState() }

    fun load() {
        scope.launch {
            isLoading = true
            apiClient.getSparks()
                .onSuccess { entries = it; isLoading = false }
                .onFailure { errorMsg = it.message; isLoading = false }
            apiClient.getProjects().onSuccess { projects = it }
        }
    }

    LaunchedEffect(Unit) { load() }

    val unsortedCount = entries.count { it.category.isNullOrBlank() || it.category == "Unsorted" }
    // Dynamische Kategorien aus den Daten (Spec §4.6 Filter Pills, single-select).
    val categories = remember(entries) {
        entries.mapNotNull { it.category?.takeIf { c -> c.isNotBlank() && c != "Unsorted" } }
            .distinct()
            .sorted()
    }
    var activeFilter by remember { mutableStateOf<String?>(null) } // null = "Alle"
    val visibleEntries = when (activeFilter) {
        null -> entries
        "__unsorted__" -> entries.filter { it.category.isNullOrBlank() || it.category == "Unsorted" }
        else -> entries.filter { it.category.equals(activeFilter, ignoreCase = true) }
    }

    Scaffold(snackbarHost = { SnackbarHost(snackbarHostState) }) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp)
        ) {
            Text(
                text = "Sparks",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 16.dp)
            )

            // Filter-Pills (Spec §4.6): Alle / <dynamische Kategorien> / Unsortiert
            val filterScroll = rememberScrollState()
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(filterScroll)
                    .padding(bottom = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                FilterChip(
                    selected = activeFilter == null,
                    onClick = { activeFilter = null },
                    label = { Text("Alle (${entries.size})") }
                )
                categories.forEach { cat ->
                    val n = entries.count { it.category.equals(cat, ignoreCase = true) }
                    FilterChip(
                        selected = activeFilter == cat,
                        onClick = { activeFilter = cat },
                        label = { Text("$cat ($n)") }
                    )
                }
                if (unsortedCount > 0) {
                    FilterChip(
                        selected = activeFilter == "__unsorted__",
                        onClick = { activeFilter = "__unsorted__" },
                        label = { Text("Unsortiert ($unsortedCount)") }
                    )
                }
            }

            when {
                isLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
                errorMsg != null -> Text("Fehler: $errorMsg", color = MaterialTheme.colorScheme.error)
                visibleEntries.isEmpty() -> Text(
                    when (activeFilter) {
                        null -> "Keine Sparks vorhanden."
                        "__unsorted__" -> "Keine unsortierten Einträge."
                        else -> "Keine Einträge in „$activeFilter\"."
                    },
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                else -> LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(visibleEntries, key = { it.id }) { entry ->
                        SwipeToDismissItem(
                            entry = entry,
                            onClick = { detailEntry = entry },
                            onDelete = {
                                scope.launch {
                                    apiClient.deleteSpark(entry.id)
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
        SparkDetailSheet(
            entry = entry,
            apiClient = apiClient,
            entries = entries,
            projects = projects,
            onNavigateToSpark = { newId ->
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
private fun SparkDetailSheet(
    entry: SparkResponse,
    apiClient: NexusApiClient,
    entries: List<SparkResponse>,
    projects: List<ProjectResponse>,
    onNavigateToSpark: (String) -> Unit,
    onNavigateToProject: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var links by remember(entry.id) { mutableStateOf<SparkLinks?>(null) }
    var linksError by remember(entry.id) { mutableStateOf<String?>(null) }
    var linksLoading by remember(entry.id) { mutableStateOf(true) }

    LaunchedEffect(entry.id) {
        linksLoading = true
        linksError = null
        apiClient.getSparkLinks(entry.id)
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
                text = "Spark",
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
                                onSpark = onNavigateToSpark, onProject = onNavigateToProject)
                        }
                        if (visibleIn.isNotEmpty()) {
                            Text(
                                "Rückverweise",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            WikiLinkFlow(visibleIn, dirIsOut = false, entries = entries, projects = projects,
                                onSpark = onNavigateToSpark, onProject = onNavigateToProject)
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
    entries: List<SparkResponse>,
    projects: List<ProjectResponse>,
    onSpark: (String) -> Unit,
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
                    if (type == "project") onProject(label) else onSpark(id)
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
    entries: List<SparkResponse>,
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
    entry: SparkResponse,
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
