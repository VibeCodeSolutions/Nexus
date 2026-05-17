package com.vibecode.nexus.ui.screen

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageCapture
import androidx.camera.core.ImageCaptureException
import androidx.camera.core.ImageProxy
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CameraAlt
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Replay
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.SparkPhotoFrame
import com.vibecode.nexus.data.SparkPhotoSseClient
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.launch

private enum class CaptureState { IDLE, STREAMING, SUCCESS, ERROR }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SparkPhotoSheet(
    settings: ConnectionSettings,
    apiClient: NexusApiClient,
    onDismiss: () -> Unit,
    onSparkSaved: (String) -> Unit,
) {
    val context = LocalContext.current
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    var hasCameraPermission by remember {
        mutableStateOf(
            ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA)
                == PackageManager.PERMISSION_GRANTED
        )
    }
    var permissionDenied by remember { mutableStateOf(false) }
    val permissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
    ) { granted ->
        hasCameraPermission = granted
        if (!granted) permissionDenied = true
    }

    LaunchedEffect(Unit) {
        if (!hasCameraPermission) permissionLauncher.launch(Manifest.permission.CAMERA)
    }

    var state by remember { mutableStateOf(CaptureState.IDLE) }
    var statusMessage by remember { mutableStateOf<String?>(null) }
    val ocrLines = remember { mutableStateListOf<String>() }
    val acceptedTags = remember { mutableStateMapOf<String, Boolean>() }
    var suggestedTags by remember { mutableStateOf<List<String>>(emptyList()) }
    var lastSparkId by remember { mutableStateOf<String?>(null) }
    val imageCapture = remember { ImageCapture.Builder().build() }
    val sseClient = remember(settings) { SparkPhotoSseClient(settings) }
    val captureExecutor = remember(context) { ContextCompat.getMainExecutor(context) }

    fun reset() {
        state = CaptureState.IDLE
        statusMessage = null
        ocrLines.clear()
        suggestedTags = emptyList()
        acceptedTags.clear()
        lastSparkId = null
    }

    fun captureAndStream() {
        imageCapture.takePicture(
            captureExecutor,
            object : ImageCapture.OnImageCapturedCallback() {
                override fun onCaptureSuccess(image: ImageProxy) {
                    val bytes = imageProxyToJpegBytes(image)
                    image.close()
                    scope.launch {
                        state = CaptureState.STREAMING
                        statusMessage = "Analysiere …"
                        ocrLines.clear()
                        suggestedTags = emptyList()
                        acceptedTags.clear()
                        sseClient.streamFromImage(bytes)
                            .catch { e ->
                                state = CaptureState.ERROR
                                statusMessage = "Verbindungsfehler: ${e.message}"
                            }
                            .onCompletion {
                                if (state == CaptureState.STREAMING) {
                                    state = CaptureState.ERROR
                                    statusMessage = "Stream unerwartet beendet"
                                }
                            }
                            .collect { frame ->
                                when (frame) {
                                    is SparkPhotoFrame.Line -> ocrLines.add(frame.text)
                                    is SparkPhotoFrame.Tags -> {
                                        suggestedTags = frame.tags
                                        frame.tags.forEach { acceptedTags[it] = false }
                                    }
                                    is SparkPhotoFrame.Done -> {
                                        lastSparkId = frame.sparkId
                                        state = CaptureState.SUCCESS
                                        statusMessage = "Gespeichert"
                                        onSparkSaved(frame.sparkId)
                                    }
                                    is SparkPhotoFrame.ErrorFrame -> {
                                        state = CaptureState.ERROR
                                        statusMessage = frame.message
                                    }
                                }
                            }
                    }
                }

                override fun onError(exc: ImageCaptureException) {
                    state = CaptureState.ERROR
                    statusMessage = "Capture fehlgeschlagen: ${exc.message}"
                }
            },
        )
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    "Foto-Spark",
                    style = MaterialTheme.typography.titleLarge,
                    modifier = Modifier.weight(1f),
                )
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Default.Close, contentDescription = "Schließen")
                }
            }

            if (!hasCameraPermission) {
                if (permissionDenied) {
                    Text(
                        "Kamera-Zugriff wurde abgelehnt. Bitte in den App-Einstellungen aktivieren.",
                        style = MaterialTheme.typography.bodyMedium,
                    )
                    Button(onClick = {
                        val intent = Intent(
                            Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                            Uri.fromParts("package", context.packageName, null),
                        ).apply { addFlags(Intent.FLAG_ACTIVITY_NEW_TASK) }
                        context.startActivity(intent)
                    }) {
                        Text("App-Einstellungen öffnen")
                    }
                } else {
                    Text("Kamera-Zugriff wird benötigt, um ein Foto-Spark anzulegen.")
                    Button(onClick = { permissionLauncher.launch(Manifest.permission.CAMERA) }) {
                        Text("Erlauben")
                    }
                }
            } else if (state == CaptureState.IDLE || state == CaptureState.ERROR) {
                CameraPreview(
                    imageCapture = imageCapture,
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(3f / 4f)
                        .border(1.dp, MaterialTheme.colorScheme.outline, RoundedCornerShape(12.dp)),
                )
                if (statusMessage != null && state == CaptureState.ERROR) {
                    Text(
                        statusMessage!!,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
                Button(
                    onClick = { captureAndStream() },
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Icon(Icons.Default.CameraAlt, contentDescription = null)
                    Spacer(Modifier.size(8.dp))
                    Text("Aufnehmen")
                }
            } else {
                statusMessage?.let { msg ->
                    val color = when (state) {
                        CaptureState.STREAMING -> MaterialTheme.colorScheme.primary
                        CaptureState.SUCCESS -> MaterialTheme.colorScheme.primary
                        CaptureState.ERROR -> MaterialTheme.colorScheme.error
                        else -> MaterialTheme.colorScheme.onSurface
                    }
                    Text(msg, color = color, style = MaterialTheme.typography.bodyMedium)
                }

                OcrStreamPanel(lines = ocrLines)

                if (suggestedTags.isNotEmpty()) {
                    Text("Tag-Vorschläge", style = MaterialTheme.typography.labelLarge)
                    TagPills(
                        tags = suggestedTags,
                        accepted = acceptedTags,
                        onToggle = { tag ->
                            val now = !(acceptedTags[tag] ?: false)
                            acceptedTags[tag] = now
                            val sparkId = lastSparkId
                            if (sparkId != null) {
                                scope.launch {
                                    apiClient.updateSparkTags(
                                        sparkId,
                                        acceptedTags.filterValues { it }.keys.toList(),
                                    )
                                }
                            }
                        },
                    )
                }

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedButton(
                        onClick = { reset() },
                        modifier = Modifier.weight(1f),
                    ) {
                        Icon(Icons.Default.Replay, contentDescription = null)
                        Spacer(Modifier.size(8.dp))
                        Text("Neu")
                    }
                    Button(
                        onClick = onDismiss,
                        modifier = Modifier.weight(1f),
                        enabled = state == CaptureState.SUCCESS,
                    ) {
                        Icon(Icons.Default.Check, contentDescription = null)
                        Spacer(Modifier.size(8.dp))
                        Text("Fertig")
                    }
                }
            }
        }
    }
}

@Composable
private fun CameraPreview(
    imageCapture: ImageCapture,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val previewView = remember { PreviewView(context) }

    DisposableEffect(Unit) {
        val providerFuture = ProcessCameraProvider.getInstance(context)
        providerFuture.addListener({
            val provider = providerFuture.get()
            val preview = androidx.camera.core.Preview.Builder().build().apply {
                setSurfaceProvider(previewView.surfaceProvider)
            }
            val selector = CameraSelector.DEFAULT_BACK_CAMERA
            runCatching {
                provider.unbindAll()
                provider.bindToLifecycle(lifecycleOwner, selector, preview, imageCapture)
            }
        }, ContextCompat.getMainExecutor(context))
        onDispose {
            runCatching {
                ProcessCameraProvider.getInstance(context).get().unbindAll()
            }
        }
    }

    AndroidView(factory = { previewView }, modifier = modifier)
}

@Composable
private fun OcrStreamPanel(lines: SnapshotStateList<String>) {
    val listState = rememberLazyListState()
    LaunchedEffect(lines.size) {
        if (lines.isNotEmpty()) listState.animateScrollToItem(lines.size - 1)
    }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 100.dp, max = 220.dp)
            .background(
                MaterialTheme.colorScheme.surfaceVariant,
                RoundedCornerShape(8.dp),
            )
            .padding(12.dp),
    ) {
        if (lines.isEmpty()) {
            Text(
                "Warte auf Eingabe …",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontFamily = FontFamily.Monospace,
                style = MaterialTheme.typography.bodySmall,
            )
        } else {
            LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
                items(lines) { line ->
                    Text(
                        line,
                        fontFamily = FontFamily.Monospace,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        }
    }
}

@OptIn(androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
private fun TagPills(
    tags: List<String>,
    accepted: Map<String, Boolean>,
    onToggle: (String) -> Unit,
) {
    androidx.compose.foundation.layout.FlowRow(
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
        modifier = Modifier.fillMaxWidth(),
    ) {
        tags.forEach { tag ->
            FilterChip(
                selected = accepted[tag] == true,
                onClick = { onToggle(tag) },
                label = { Text(tag) },
            )
        }
    }
}

private fun imageProxyToJpegBytes(image: ImageProxy): ByteArray {
    // CameraX standard `takePicture` already delivers JPEG via ImageCapture.OUTPUT_FORMAT_JPEG
    // → Plane[0] enthält den fertigen JPEG-Stream.
    val buffer = image.planes[0].buffer
    val bytes = ByteArray(buffer.remaining())
    buffer.get(bytes)
    return bytes
}

