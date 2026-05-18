package com.vibecode.nexus.ui.screen

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalContext
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.UiPreferences
import com.vibecode.nexus.data.model.SparkLinks
import com.vibecode.nexus.data.model.SparkResponse
import com.vibecode.nexus.data.model.Link
import com.vibecode.nexus.data.model.ProjectResponse
import com.vibecode.nexus.data.model.TaskResponse
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

// DANIEL-POLISH DA-002: Mapping Spark → "wurde extrahiert?" via
// `task.nexus_external_id`-Pattern. Regex extrahiert die spark_id.
private val SPARK_EXTRACT_RE = Regex("""^spark-extract:(.+?):\d+$""")

private fun buildSparkExtractedSet(tasks: List<TaskResponse>): Set<String> {
    val out = HashSet<String>()
    for (t in tasks) {
        val ext = t.nexus_external_id ?: continue
        val m = SPARK_EXTRACT_RE.matchEntire(ext) ?: continue
        out.add(m.groupValues[1])
    }
    return out
}

// DANIEL-POLISH DA-002: Kind-Badge-Farben (Idea = lila tonal,
// Task = grün tonal). Background mit niedriger Alpha, Text-Color
// gesättigter. Funktioniert in beiden Themes.
private val KIND_IDEA_BG = Color(0x26A78BFA)  // rgba(167,139,250,0.15)
private val KIND_IDEA_FG = Color(0xFFC4B5FD)
private val KIND_TASK_BG = Color(0x2634D399)  // rgba(52,211,153,0.15)
private val KIND_TASK_FG = Color(0xFF6EE7B7)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SparkHistoryScreen(apiClient: NexusApiClient) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    // DANIEL-FUNKTIONAL DA-001: Filter-Persistenz via UiPreferences.
    val uiPrefs = remember { UiPreferences(context) }
    var entries by remember { mutableStateOf<List<SparkResponse>>(emptyList()) }
    var projects by remember { mutableStateOf<List<ProjectResponse>>(emptyList()) }
    // DANIEL-POLISH DA-002: Set der Spark-IDs, aus denen schon Tasks
    // extrahiert wurden. Wird parallel zu Sparks geladen und für die
    // IDEA/TASK-Kind-Badge in den Karten genutzt.
    var extractedSparkIds by remember { mutableStateOf<Set<String>>(emptySet()) }
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
            // DANIEL-POLISH DA-002: Tasks-Pull für Spark→Task-Mapping.
            // Fehler nicht fatal — fällt auf "alle Idea" zurück.
            apiClient.getTasks().onSuccess { taskList ->
                extractedSparkIds = buildSparkExtractedSet(taskList)
            }
        }
    }

    LaunchedEffect(Unit) { load() }

    val unsortedCount = entries.count { it.category.isNullOrBlank() || it.category == "Unsorted" }
    // DANIEL-FUNKTIONAL DA-001 Hybrid:
    // Reihe 2 = Lebensbereich-Kategorien (alle aus DB außer Idea/Task,
    // die in Reihe 1 stehen). Konsistent zum Desktop-Verhalten
    // (populateCategoryFilter() filtert TYPE_VALUES raus).
    val typeValues = remember { setOf("Idea", "Task") }
    val categories = remember(entries) {
        entries.mapNotNull { it.category?.takeIf { c -> c.isNotBlank() && c != "Unsorted" && c !in typeValues } }
            .distinct()
            .sorted()
    }
    // Beide Filter werden aus UiPreferences vorbelegt — Persistenz über
    // App-Restart hinweg. `null`/empty = „Alle" für die jeweilige Reihe.
    var typeFilter by remember { mutableStateOf(uiPrefs.sparkTypeFilter) }
    var lifeFilter by remember { mutableStateOf(uiPrefs.sparkLifeFilter) }

    val visibleEntries = remember(entries, typeFilter, lifeFilter) {
        entries.filter { entry ->
            // Lebensbereich-Filter (Reihe 2)
            if (lifeFilter == "__unsorted__") {
                if (!(entry.category.isNullOrBlank() || entry.category == "Unsorted")) return@filter false
            } else if (lifeFilter != null) {
                if (!entry.category.equals(lifeFilter, ignoreCase = true)) return@filter false
            }
            // Type-Filter (Reihe 1) — case-insensitive auf category
            if (typeFilter != null) {
                if (!entry.category.orEmpty().equals(typeFilter, ignoreCase = true)) return@filter false
            }
            true
        }
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

            // DANIEL-FUNKTIONAL DA-001 Hybrid: zwei Filter-Reihen.
            // Reihe 1 oben: Type (Alle / 💡 Idea / ✅ Task).
            // Reihe 2 unten: Lebensbereich (Alle / <dynamische Kategorien
            // ohne Idea/Task> / Unsortiert). Beide unabhängig, Persistenz
            // via UiPreferences.
            val typeScroll = rememberScrollState()
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(typeScroll)
                    .padding(bottom = 6.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                FilterChip(
                    selected = typeFilter == null,
                    onClick = { typeFilter = null; uiPrefs.sparkTypeFilter = null },
                    label = { Text("Alle") }
                )
                FilterChip(
                    selected = typeFilter == "Idea",
                    onClick = { typeFilter = "Idea"; uiPrefs.sparkTypeFilter = "Idea" },
                    label = { Text("💡 Idea") }
                )
                FilterChip(
                    selected = typeFilter == "Task",
                    onClick = { typeFilter = "Task"; uiPrefs.sparkTypeFilter = "Task" },
                    label = { Text("✅ Task") }
                )
            }

            val lifeScroll = rememberScrollState()
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(lifeScroll)
                    .padding(bottom = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                FilterChip(
                    selected = lifeFilter == null,
                    onClick = { lifeFilter = null; uiPrefs.sparkLifeFilter = null },
                    label = { Text("Alle (${entries.size})") }
                )
                categories.forEach { cat ->
                    val n = entries.count { it.category.equals(cat, ignoreCase = true) }
                    FilterChip(
                        selected = lifeFilter == cat,
                        onClick = { lifeFilter = cat; uiPrefs.sparkLifeFilter = cat },
                        label = { Text("$cat ($n)") }
                    )
                }
                if (unsortedCount > 0) {
                    FilterChip(
                        selected = lifeFilter == "__unsorted__",
                        onClick = { lifeFilter = "__unsorted__"; uiPrefs.sparkLifeFilter = "__unsorted__" },
                        label = { Text("Unsortiert ($unsortedCount)") }
                    )
                }
            }

            when {
                isLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
                errorMsg != null -> Text("Fehler: $errorMsg", color = MaterialTheme.colorScheme.error)
                visibleEntries.isEmpty() -> {
                    val msg = when {
                        typeFilter == null && lifeFilter == null -> "Keine Sparks vorhanden."
                        lifeFilter == "__unsorted__" && typeFilter == null -> "Keine unsortierten Einträge."
                        else -> {
                            val parts = listOfNotNull(typeFilter, lifeFilter?.takeUnless { it == "__unsorted__" })
                            "Keine Einträge im aktuellen Filter (${parts.joinToString(" + ")})."
                        }
                    }
                    Text(msg, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                else -> {
                    // DANIEL-POLISH DA-005: Sheet-Dim — beim offenen
                    // Detail-Sheet wird die Liste hinter dem Scrim
                    // zusätzlich auf 0.4 alpha gedimmt.
                    val listAlpha by animateFloatAsState(
                        targetValue = if (detailEntry != null) 0.4f else 1f,
                        animationSpec = tween(durationMillis = 200),
                        label = "list_dim"
                    )
                    LazyColumn(
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                        modifier = Modifier.graphicsLayer { alpha = listAlpha }
                    ) {
                        itemsIndexed(visibleEntries, key = { _, e -> e.id }) { index, entry ->
                            // DANIEL-POLISH DA-003: Karten-Stagger-Animation
                            // (slide-up + fade + scale) beim ersten
                            // Erscheinen, index-basierter Delay (cap 320 ms).
                            StaggeredAppear(index = index) {
                                SwipeToDismissItem(
                                    entry = entry,
                                    isTask = extractedSparkIds.contains(entry.id),
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

    // FEAT-001 Schicht D: Tasks aus Spark extrahieren.
    val scope = rememberCoroutineScope()
    var extractBusy by remember(entry.id) { mutableStateOf(false) }
    var extractStatus by remember(entry.id) { mutableStateOf<String?>(null) }

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
                // DANIEL-POLISH DA-004: Tag-Pop-In gestaffelt beim Sheet-
                // Open. Key am Composable-Aufrufer (entry.id) sorgt für
                // Reset, wenn man zwischen Sparks navigiert.
                key(entry.id) {
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        entry.tags.forEachIndexed { idx, tag ->
                            PopInTagChip(label = tag, index = idx)
                        }
                    }
                }
            }

            // FEAT-001 Schicht D: Tasks extrahieren — idempotenter Endpoint,
            // Doppelklick erzeugt keine Duplikate.
            OutlinedButton(
                onClick = {
                    extractBusy = true
                    extractStatus = "Analysiere Spark …"
                    scope.launch {
                        apiClient.extractTasksFromSpark(entry.id)
                            .onSuccess { res ->
                                extractStatus = when {
                                    res.count == 0L && res.skipped == 0L ->
                                        "Keine umsetzbaren Tasks im Spark gefunden."
                                    res.count == 0L ->
                                        "Bereits ${res.skipped} Task${if (res.skipped == 1L) "" else "s"} aus diesem Spark extrahiert (keine neuen)."
                                    else -> "✓ ${res.count} Task${if (res.count == 1L) "" else "s"} angelegt" +
                                        if (res.skipped > 0) " (${res.skipped} bereits vorhanden)." else "."
                                }
                            }
                            .onFailure { extractStatus = "Fehler: ${it.message ?: "unbekannt"}" }
                        extractBusy = false
                    }
                },
                enabled = !extractBusy,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(if (extractBusy) "Extrahiere …" else "📋 Tasks extrahieren")
            }
            extractStatus?.let { msg ->
                Text(
                    msg,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
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

// DANIEL-POLISH DA-002: Kompakte Kind-Badge — Letter-Spacing-Uppercase
// Pill links oben auf jeder Spark-Karte. IDEA = lila tonal, TASK = grün
// tonal.
@Composable
private fun KindBadge(isTask: Boolean) {
    val (bg, fg, label) = if (isTask) {
        Triple(KIND_TASK_BG, KIND_TASK_FG, "Task")
    } else {
        Triple(KIND_IDEA_BG, KIND_IDEA_FG, "Idea")
    }
    Surface(
        shape = RoundedCornerShape(6.dp),
        color = bg
    ) {
        Text(
            text = label.uppercase(),
            style = MaterialTheme.typography.labelSmall,
            color = fg,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(horizontal = 7.dp, vertical = 2.dp)
        )
    }
}

// DANIEL-POLISH DA-004: Tag-Chip mit Pop-In-Animation (scale 0.7→1 +
// fade), per-Chip-Delay (60 ms versetzt). Reset beim Sheet-Open via
// Key-Param am Composable-Aufrufer.
@Composable
private fun PopInTagChip(label: String, index: Int) {
    val visible = remember { Animatable(0f) }
    val scale = remember { Animatable(0.7f) }
    LaunchedEffect(Unit) {
        delay((index * 60L).coerceAtMost(420L))
        kotlinx.coroutines.coroutineScope {
            launch { visible.animateTo(1f, tween(220)) }
            launch { scale.animateTo(1f, tween(220)) }
        }
    }
    AssistChip(
        onClick = {},
        enabled = false,
        label = { Text(label) },
        modifier = Modifier.graphicsLayer {
            alpha = visible.value
            scaleX = scale.value
            scaleY = scale.value
        }
    )
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

// DANIEL-POLISH DA-003: Wrapper-Composable für gestaffeltes Einblenden
// der Spark-Karten. Initial einmal beim Erscheinen (LaunchedEffect-Key
// Unit + Index als Delay-Quelle). Cap bei 320 ms — sonst warten lange
// Listen unnötig.
@Composable
private fun StaggeredAppear(index: Int, content: @Composable () -> Unit) {
    val offsetY = remember { Animatable(18f) }
    val alpha = remember { Animatable(0f) }
    val scale = remember { Animatable(0.98f) }
    LaunchedEffect(Unit) {
        delay((index * 40L).coerceAtMost(320L))
        kotlinx.coroutines.coroutineScope {
            launch { offsetY.animateTo(0f, tween(320)) }
            launch { alpha.animateTo(1f, tween(320)) }
            launch { scale.animateTo(1f, tween(320)) }
        }
    }
    Box(
        modifier = Modifier.graphicsLayer {
            this.alpha = alpha.value
            this.translationY = offsetY.value
            this.scaleX = scale.value
            this.scaleY = scale.value
        }
    ) {
        content()
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SwipeToDismissItem(
    entry: SparkResponse,
    isTask: Boolean,
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
                // DANIEL-POLISH DA-002: Kind-Badge oben links auf jeder
                // Spark-Card. TASK wenn aus diesem Spark schon Tasks
                // extrahiert wurden, sonst IDEA als Default.
                Row(
                    modifier = Modifier.fillMaxWidth().padding(bottom = 6.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    KindBadge(isTask = isTask)
                    Text(
                        text = entry.created_at.take(16).replace("T", " "),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = entry.category ?: "Unsorted",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary
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
