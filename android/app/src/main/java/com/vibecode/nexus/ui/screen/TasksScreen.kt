package com.vibecode.nexus.ui.screen

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.rememberSwipeToDismissBoxState
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
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathMeasure
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.model.ProjectResponse
import com.vibecode.nexus.ui.theme.NexusAccent
import com.vibecode.nexus.data.model.TaskCreateRequest
import com.vibecode.nexus.data.model.TaskResponse
import com.vibecode.nexus.data.model.TaskUpdateRequest
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TasksScreen(
    apiClient: NexusApiClient,
    isPaired: Boolean,
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    val snackbarHostState = remember { SnackbarHostState() }
    var tasks by remember { mutableStateOf<List<TaskResponse>>(emptyList()) }
    var projects by remember { mutableStateOf<List<ProjectResponse>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var isRefreshing by remember { mutableStateOf(false) }
    var showCreateDialog by remember { mutableStateOf(false) }

    suspend fun fetchData() {
        apiClient.getTasks().onSuccess { serverTasks ->
            if (serverTasks.isNotEmpty() || tasks.isEmpty()) {
                tasks = serverTasks
            }
        }
        apiClient.getProjects().onSuccess { projects = it }
    }

    fun loadData() {
        scope.launch {
            isLoading = true
            fetchData()
            isLoading = false
        }
    }

    LaunchedEffect(isPaired) {
        if (isPaired) loadData()
    }

    if (showCreateDialog) {
        TaskCreateDialog(
            projects = projects,
            onDismiss = { showCreateDialog = false },
            onCreate = { title, projectId, priority ->
                showCreateDialog = false
                scope.launch {
                    apiClient.createTask(
                        TaskCreateRequest(title, projectId, priority)
                    ).onSuccess { newTask ->
                        tasks = tasks + newTask
                        loadData()
                    }.onFailure { e ->
                        snackbarHostState.showSnackbar("Fehler: ${e.message}")
                    }
                }
            }
        )
    }

    Scaffold(
        floatingActionButton = {
            if (isPaired) {
                FloatingActionButton(
                    onClick = { showCreateDialog = true },
                    containerColor = MaterialTheme.colorScheme.primary
                ) {
                    Icon(Icons.Default.Add, contentDescription = "Neuer Task")
                }
            }
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
        modifier = modifier
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(horizontal = 16.dp, vertical = 8.dp)
        ) {
            Text(
                text = "Aufgaben",
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
                            fetchData()
                            isRefreshing = false
                        }
                    },
                    modifier = Modifier.fillMaxSize()
                ) {
                    if (tasks.isEmpty()) {
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(
                                "Keine Tasks vorhanden",
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            items(tasks, key = { it.id }) { task ->
                                SwipeableTaskCard(
                                    task = task,
                                    onMarkDone = {
                                        scope.launch {
                                            apiClient.updateTask(task.id, TaskUpdateRequest(status = "done"))
                                                .onSuccess { loadData() }
                                                .onFailure { e ->
                                                    snackbarHostState.showSnackbar("Fehler: ${e.message}")
                                                }
                                        }
                                    },
                                    onDelete = {
                                        scope.launch {
                                            apiClient.deleteTask(task.id)
                                                .onSuccess { loadData() }
                                                .onFailure { e ->
                                                    snackbarHostState.showSnackbar("Fehler: ${e.message}")
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
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SwipeableTaskCard(
    task: TaskResponse,
    onMarkDone: () -> Unit,
    onDelete: () -> Unit
) {
    val isDone = task.status == "done"
    val dismissState = rememberSwipeToDismissBoxState(
        confirmValueChange = { value ->
            when (value) {
                SwipeToDismissBoxValue.StartToEnd -> {
                    if (!isDone) onMarkDone()
                    false
                }
                SwipeToDismissBoxValue.EndToStart -> {
                    onDelete()
                    false
                }
                SwipeToDismissBoxValue.Settled -> false
            }
        }
    )

    SwipeToDismissBox(
        state = dismissState,
        backgroundContent = {
            val direction = dismissState.dismissDirection
            val color by animateColorAsState(
                when (direction) {
                    SwipeToDismissBoxValue.StartToEnd -> Color(0xFF4CAF50)
                    SwipeToDismissBoxValue.EndToStart -> Color(0xFFF44336)
                    else -> Color.Transparent
                },
                label = "swipeBg"
            )
            val icon = when (direction) {
                SwipeToDismissBoxValue.StartToEnd -> Icons.Default.Check
                SwipeToDismissBoxValue.EndToStart -> Icons.Default.Delete
                else -> Icons.Default.Check
            }
            val alignment = when (direction) {
                SwipeToDismissBoxValue.StartToEnd -> Alignment.CenterStart
                else -> Alignment.CenterEnd
            }
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(color, RoundedCornerShape(12.dp))
                    .padding(horizontal = 24.dp),
                contentAlignment = alignment
            ) {
                Icon(icon, contentDescription = null, tint = Color.White)
            }
        }
    ) {
        TaskCard(task, onMarkDone = onMarkDone)
    }
}

@Composable
private fun TaskCard(task: TaskResponse, onMarkDone: () -> Unit = {}) {
    val isDone = task.status == "done"
    val priorityColor = when (task.priority) {
        "high" -> NexusAccent.PriorityHigh
        "medium" -> NexusAccent.PriorityMedium
        "low" -> NexusAccent.PriorityLow
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (isDone)
                MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
            else
                MaterialTheme.colorScheme.surfaceVariant
        ),
        shape = RoundedCornerShape(12.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // DANIEL-POLISH DC-005: Animierte Checkbox mit Checkmark-
            // Pfad (Canvas drawPath + PathMeasure für „zeichnet sich"-
            // Effekt). Tap auf die Box markiert offene Tasks als done.
            AnimatedCheckbox(
                checked = isDone,
                onToggle = { if (!isDone) onMarkDone() }
            )

            Spacer(Modifier.width(12.dp))

            // Priority indicator
            Box(
                modifier = Modifier
                    .size(8.dp, 32.dp)
                    .background(priorityColor, RoundedCornerShape(4.dp))
            )

            Spacer(Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = task.title,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Medium,
                    textDecoration = if (isDone) TextDecoration.LineThrough else null,
                    color = if (isDone)
                        MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                    else
                        MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(Modifier.height(4.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = when (task.status) {
                            "open" -> "Offen"
                            "done" -> "Erledigt"
                            else -> task.status.replace("_", " ")
                        },
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = when (task.priority) {
                            "low" -> "Niedrig"
                            "medium" -> "Mittel"
                            "high" -> "Hoch"
                            else -> task.priority
                        },
                        style = MaterialTheme.typography.labelSmall,
                        color = priorityColor
                    )
                }
            }
        }
    }
}

// DANIEL-POLISH DC-005: Animierte Checkbox — Checkmark-Pfad
// (M 2 6 L 5 9 L 10 3) zeichnet sich beim Tick via PathMeasure +
// animateFloatAsState (Progress 0→1, 240 ms ease-out). Box selbst
// transitions Background von transparent zu success-grün.
@Composable
private fun AnimatedCheckbox(
    checked: Boolean,
    onToggle: () -> Unit,
) {
    val progress by animateFloatAsState(
        targetValue = if (checked) 1f else 0f,
        animationSpec = tween(durationMillis = 240),
        label = "check_progress"
    )
    val bg by animateColorAsState(
        targetValue = if (checked) Color(0xFF4CAF50) else Color.Transparent,
        animationSpec = tween(durationMillis = 200),
        label = "check_bg"
    )
    val borderColor by animateColorAsState(
        targetValue = if (checked) Color(0xFF4CAF50) else MaterialTheme.colorScheme.outline,
        animationSpec = tween(durationMillis = 200),
        label = "check_border"
    )
    Box(
        modifier = Modifier
            .size(24.dp)
            .clip(RoundedCornerShape(5.dp))
            .background(bg)
            .border(2.dp, borderColor, RoundedCornerShape(5.dp))
            .clickable { onToggle() },
        contentAlignment = Alignment.Center
    ) {
        Canvas(modifier = Modifier.size(16.dp)) {
            // Pfad M 2 6 L 5 9 L 10 3 in 12x12-Viewbox → auf Canvas-Größe skaliert.
            val w = size.width
            val h = size.height
            val sx = w / 12f
            val sy = h / 12f
            val path = Path().apply {
                moveTo(2f * sx, 6f * sy)
                lineTo(5f * sx, 9f * sy)
                lineTo(10f * sx, 3f * sy)
            }
            val measure = PathMeasure().apply { setPath(path, false) }
            val totalLen = measure.length
            val animated = Path()
            measure.getSegment(0f, totalLen * progress, animated, true)
            drawPath(
                path = animated,
                color = Color.White,
                style = Stroke(
                    width = 2.4f * sx,
                    cap = StrokeCap.Round,
                    join = StrokeJoin.Round
                )
            )
        }
    }
}
