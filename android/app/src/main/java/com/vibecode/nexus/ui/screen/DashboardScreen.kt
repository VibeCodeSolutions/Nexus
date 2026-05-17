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
    var sparkCount by remember { mutableStateOf<Int?>(null) }
    var unsortedCount by remember { mutableStateOf(0) }
    var openTaskCount by remember { mutableStateOf<Int?>(null) }
    var projectCount by remember { mutableStateOf<Int?>(null) }

    LaunchedEffect(isPaired) {
        if (!isPaired) return@LaunchedEffect
        scope.launch {
            coroutineScope {
                val bd = async { apiClient.getSparks() }
                val ts = async { apiClient.getTasks() }
                val ps = async { apiClient.getProjects() }
                bd.await().onSuccess { entries ->
                    sparkCount = entries.size
                    unsortedCount = entries.count {
                        it.category.isNullOrBlank() || it.category.equals("Unsorted", ignoreCase = true)
                    }
                }
                ts.await().onSuccess { tasks ->
                    openTaskCount = tasks.count { it.status != "done" && it.status != "completed" }
                }
                ps.await().onSuccess { p -> projectCount = p.size }
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
        if (unsortedCount > 0) {
            AlertCard(
                count = unsortedCount,
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

        // 2x2 Overview-Grid
        LazyVerticalGrid(
            columns = GridCells.Fixed(2),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
            contentPadding = PaddingValues(0.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            item {
                OverviewCard(
                    count = sparkCount?.toString() ?: "—",
                    icon = "🧠",
                    label = "SPARKS",
                    onClick = { onNavigate("history") }
                )
            }
            item {
                OverviewCard(
                    count = openTaskCount?.toString() ?: "—",
                    icon = "✅",
                    label = "AUFGABEN",
                    onClick = { onNavigate("tasks") }
                )
            }
            item {
                OverviewCard(
                    count = projectCount?.toString() ?: "—",
                    icon = "📁",
                    label = "PROJEKTE",
                    onClick = { onNavigate("projects") }
                )
            }
            item {
                OverviewCard(
                    count = unsortedCount.toString(),
                    icon = "⚠",
                    label = "UNSORTIERT",
                    onClick = { onNavigate("history") }
                )
            }
        }
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
