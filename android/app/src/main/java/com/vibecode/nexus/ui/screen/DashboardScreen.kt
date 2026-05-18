package com.vibecode.nexus.ui.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.model.TaskResponse
import com.vibecode.nexus.ui.theme.NexusAccent
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch

@Composable
fun DashboardScreen(
    apiClient: NexusApiClient,
    isPaired: Boolean,
    onNavigate: (String) -> Unit,
    onCreateTask: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val scope = rememberCoroutineScope()
    // DANIEL-FUNKTIONAL DC-001..004: vier Stat-Counter aus /api/dashboard/stats.
    // Vorher liefen drei Einzelaufrufe (getSparks/getTasks/getProjects); der
    // Aggregat-Endpoint spart Roundtrips und liefert HEUTE/DIESE-WOCHE-Werte,
    // die clientseitig nicht ableitbar waren.
    var todaySparks by remember { mutableStateOf<Long?>(null) }
    var openTaskCount by remember { mutableStateOf<Long?>(null) }
    var activeProjects by remember { mutableStateOf<Long?>(null) }
    var doneThisWeek by remember { mutableStateOf<Long?>(null) }
    var unsortedCount by remember { mutableStateOf(0L) }
    // DANIEL-FUNKTIONAL DC-003: Nächster offener Task mit Fälligkeit. `null`
    // = noch nicht geladen ODER kein passender Task ODER Netzwerk-Fehler.
    // Wir tracken den Lade-/Fehler-Zustand separat via `nextFocusLoaded` +
    // `nextFocusError`, damit die UI Empty-Hint vs. Network-Fail
    // differenzieren kann (analog Desktop DC-P2-N03-Fix).
    var nextFocus by remember { mutableStateOf<TaskResponse?>(null) }
    var nextFocusLoaded by remember { mutableStateOf(false) }
    var nextFocusError by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(isPaired) {
        if (!isPaired) return@LaunchedEffect
        scope.launch {
            coroutineScope {
                val stats = async { apiClient.getDashboardStats() }
                val focus = async { apiClient.getDashboardNextFocus() }
                val unsort = async { apiClient.getUnsortedCount() }
                stats.await().onSuccess { s ->
                    todaySparks    = s.today_sparks
                    openTaskCount  = s.total_open_tasks
                    activeProjects = s.active_projects
                    doneThisWeek   = s.done_this_week
                }
                focus.await()
                    .onSuccess { resp ->
                        nextFocus = resp.task
                        nextFocusLoaded = true
                        nextFocusError = null
                    }
                    .onFailure { err ->
                        nextFocusLoaded = true
                        nextFocusError = "Nächster Fokus konnte nicht geladen werden (Verbindung prüfen)."
                    }
                unsort.await().onSuccess { c -> unsortedCount = c }
            }
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(20.dp)
    ) {
        // Greeting
        Column {
            Text(
                text = "NEXUS",
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.8.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = "Was steht heute an?",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onBackground
            )
        }

        if (!isPaired) {
            Surface(
                color = MaterialTheme.colorScheme.surfaceVariant,
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = "Zuerst mit Core koppeln (Settings → Pairing).",
                    modifier = Modifier.padding(16.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            return@Column
        }

        // Quick-Actions Row
        Row(
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            QuickActionButton("🧠", "Sparks") { onNavigate("history") }
            QuickActionButton("✏️", "Neue Aufgabe") { onCreateTask() }
            QuickActionButton("📁", "Projekte") { onNavigate("projects") }
            QuickActionButton("🎙️", "Aufnehmen") { onNavigate("spark") }
        }

        // Alert-Card (only when unsorted > 0)
        if (unsortedCount > 0L) {
            AlertCard(
                count = unsortedCount.toInt(),
                onClick = { onNavigate("history") }
            )
        }

        // Section Label
        Text(
            text = "ÜBERSICHT",
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.6.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        // DANIEL-FUNKTIONAL: 2x2 Overview-Grid mit vier Stat-Cards
        // (HEUTE / OFFEN / PROJEKTE / DIESE WOCHE). UNSORTIERT-Card
        // entfällt — Unsortiert-Hinweis kommt weiter über die
        // AlertCard oben (sticht stärker hervor als ein Stat-Tile).
        LazyVerticalGrid(
            columns = GridCells.Fixed(2),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
            contentPadding = PaddingValues(0.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            item {
                OverviewCard(
                    count = todaySparks?.toString() ?: "—",
                    icon = "🧠",
                    label = "HEUTE",
                    onClick = { onNavigate("history") }
                )
            }
            item {
                OverviewCard(
                    count = openTaskCount?.toString() ?: "—",
                    icon = "✅",
                    label = "OFFEN",
                    onClick = { onNavigate("tasks") }
                )
            }
            item {
                OverviewCard(
                    count = activeProjects?.toString() ?: "—",
                    icon = "📁",
                    label = "PROJEKTE",
                    onClick = { onNavigate("projects") }
                )
            }
            item {
                OverviewCard(
                    count = doneThisWeek?.toString() ?: "—",
                    icon = "🏆",
                    label = "DIESE WOCHE",
                    onClick = { onNavigate("tasks") }
                )
            }
        }

        // DANIEL-FUNKTIONAL DC-003: Nächster-Fokus-Card unterhalb des Stat-Grids.
        // Drei Zustände: Task vorhanden / `task: null` / Netzwerkfehler.
        NextFocusCard(
            task = nextFocus,
            loaded = nextFocusLoaded,
            error = nextFocusError,
            onClick = { onNavigate("tasks") }
        )
    }
}

@Composable
private fun NextFocusCard(
    task: TaskResponse?,
    loaded: Boolean,
    error: String?,
    onClick: () -> Unit,
) {
    when {
        task != null -> {
            Surface(
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surface,
                border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.primary),
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onClick)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    Text(
                        text = "NÄCHSTER FOKUS",
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 1.6.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = task.title.ifBlank { "Ohne Titel" },
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onBackground
                    )
                    val meta = buildString {
                        task.due_date?.takeIf { it.isNotBlank() }?.let { append("Fällig: $it") }
                        if (task.priority.isNotBlank()) {
                            if (isNotEmpty()) append(" · ")
                            append("Priorität ${task.priority.uppercase()}")
                        }
                    }
                    if (meta.isNotEmpty()) {
                        Text(
                            text = meta,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        }
        loaded && error != null -> {
            Text(
                text = error,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 4.dp)
            )
        }
        loaded -> {
            Text(
                text = "Kein Fokus-Task — alle Termine erledigt oder ohne Datum.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 4.dp)
            )
        }
        // !loaded → kein Render (LaunchedEffect läuft noch)
        else -> Unit
    }
}

@Composable
private fun QuickActionButton(
    emoji: String,
    contentDescription: String,
    onClick: () -> Unit,
) {
    Surface(
        shape = CircleShape,
        color = MaterialTheme.colorScheme.surfaceVariant,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
        modifier = Modifier
            .size(48.dp)
            .clickable(onClick = onClick)
    ) {
        Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
            Text(text = emoji, fontSize = 20.sp)
        }
    }
}

@Composable
private fun AlertCard(count: Int, onClick: () -> Unit) {
    Surface(
        shape = RoundedCornerShape(16.dp),
        color = NexusAccent.Warning.copy(alpha = 0.10f),
        border = androidx.compose.foundation.BorderStroke(1.dp, NexusAccent.Warning.copy(alpha = 0.30f)),
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(NexusAccent.Warning.copy(alpha = 0.15f)),
                contentAlignment = Alignment.Center
            ) {
                Text("⚠", color = NexusAccent.Warning, fontSize = 18.sp)
            }
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "$count Spark${if (count == 1) "" else "s"} unsortiert",
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onBackground
                )
                Text(
                    text = "Jetzt sortieren",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Text("›", color = NexusAccent.Warning, fontSize = 22.sp, fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
private fun OverviewCard(
    count: String,
    icon: String,
    label: String,
    onClick: () -> Unit,
) {
    Surface(
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surface,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
        modifier = Modifier
            .fillMaxWidth()
            .height(112.dp)
            .clickable(onClick = onClick)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top
            ) {
                Text(
                    text = count,
                    fontSize = 28.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary
                )
                Text(text = icon, fontSize = 20.sp)
            }
            Text(
                text = label,
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.4.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Start
            )
        }
    }
}
