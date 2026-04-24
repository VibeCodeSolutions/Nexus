package com.vibecode.nexus.ui.screen

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
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
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.model.ProjectProgress
import com.vibecode.nexus.data.model.ProjectResponse
import kotlinx.coroutines.launch

data class ProjectWithProgress(
    val project: ProjectResponse,
    val progress: ProjectProgress?
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProjectsScreen(
    apiClient: NexusApiClient,
    isPaired: Boolean,
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    var items by remember { mutableStateOf<List<ProjectWithProgress>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var isRefreshing by remember { mutableStateOf(false) }

    suspend fun loadData() {
        apiClient.getProjects().onSuccess { projects ->
            val withProgress = projects.map { project ->
                val progress = apiClient.getProjectProgress(project.id).getOrNull()
                ProjectWithProgress(project, progress)
            }
            items = withProgress
        }
    }

    LaunchedEffect(isPaired) {
        if (!isPaired) return@LaunchedEffect
        isLoading = true
        loadData()
        isLoading = false
    }

    Scaffold(modifier = modifier) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(horizontal = 16.dp, vertical = 8.dp)
        ) {
            Text(
                text = "Projekte",
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(bottom = 16.dp, top = 8.dp)
            )

            if (!isPaired) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        "Zuerst mit Core koppeln",
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            } else if (isLoading) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else {
                PullToRefreshBox(
                    isRefreshing = isRefreshing,
                    onRefresh = {
                        scope.launch {
                            isRefreshing = true
                            loadData()
                            isRefreshing = false
                        }
                    },
                    modifier = Modifier.fillMaxSize()
                ) {
                    if (items.isEmpty()) {
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(
                                "Keine Projekte vorhanden",
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            items(items, key = { it.project.id }) { item ->
                                ProjectCard(item)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ProjectCard(item: ProjectWithProgress) {
    val progress = item.progress
    val percent = progress?.progress_percent ?: 0
    val fraction = percent / 100f

    val animatedProgress by animateFloatAsState(
        targetValue = fraction,
        animationSpec = tween(durationMillis = 800),
        label = "progressAnim"
    )

    val progressColor = progressGlowColor(percent)

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        ),
        shape = RoundedCornerShape(16.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = item.project.name,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface
            )

            Spacer(Modifier.height(12.dp))

            // ProgressGlow bar
            LinearProgressIndicator(
                progress = { animatedProgress },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(12.dp)
                    .clip(RoundedCornerShape(6.dp)),
                color = progressColor,
                trackColor = MaterialTheme.colorScheme.surfaceVariant,
                strokeCap = StrokeCap.Round,
            )

            Spacer(Modifier.height(8.dp))

            if (progress != null) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = "${progress.done_tasks} / ${progress.total_tasks} Tasks",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = "$percent%",
                        style = MaterialTheme.typography.bodySmall,
                        fontWeight = FontWeight.Bold,
                        color = progressColor
                    )
                }
            }
        }
    }
}

private fun progressGlowColor(percent: Int): Color {
    // Red (0%) → Yellow (50%) → Green (100%)
    return when {
        percent <= 50 -> {
            val t = percent / 50f
            Color(
                red = 1f,
                green = t * 0.8f,
                blue = 0f
            )
        }
        else -> {
            val t = (percent - 50) / 50f
            Color(
                red = 1f - t * 0.7f,
                green = 0.8f + t * 0.2f,
                blue = t * 0.2f
            )
        }
    }
}
