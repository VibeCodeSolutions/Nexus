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
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
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
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.SparkPhotoFrame
import com.vibecode.nexus.data.SparkPhotoSseClient
import kotlinx.coroutines.delay
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

    // DANIEL-POLISH DB-005: kurzer weißer Vollflächen-Fade als Shutter-Beat
    // beim Capture-Start. Self-resetting via LaunchedEffect.
    var flashing by remember { mutableStateOf(false) }
    LaunchedEffect(flashing) {
        if (flashing) {
            delay(150)
            flashing = false
        }
    }

    fun reset() {
        state = CaptureState.IDLE
        statusMessage = null
        ocrLines.clear()
        suggestedTags = emptyList()
        acceptedTags.clear()
        lastSparkId = null
    }

    fun captureAndStream() {
        // DANIEL-POLISH DB-005: Flash-Trigger vor dem eigentlichen Capture.
        flashing = true
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
      Box(modifier = Modifier.fillMaxWidth()) {
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
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(3f / 4f),
                ) {
                    CameraPreview(
                        imageCapture = imageCapture,
                        modifier = Modifier
                            .fillMaxSize()
                            .border(1.dp, MaterialTheme.colorScheme.outline, RoundedCornerShape(12.dp)),
                    )
                    // DANIEL-POLISH DB-001: Ecken-Marker + animierte Linie über
                    // der Live-Preview. Sichtbar nur im IDLE-Zustand (vor Capture).
                    if (state == CaptureState.IDLE) {
                        PreviewScanOverlay(
                            modifier = Modifier.fillMaxSize(),
                            cornerLength = 28.dp,
                            cornerThickness = 3.dp,
                            cornerInset = 12.dp,
                            accent = Color(0xFFA78BFA),
                        )
                    }
                }
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
                // DANIEL-POLISH DB-002: zentrierter Pill-Style-Chip mit Spinner
                // während Streaming, Check-Glyph nach Erfolg. Ersetzt den
                // bisherigen plain-Text-Status.
                statusMessage?.let { msg ->
                    AnalyzeStatusChip(state = state, text = msg)
                }

                // DANIEL-POLISH DB-003: blinkender Cursor am Ende der OCR-Liste,
                // nur während STREAMING aktiv ist.
                OcrStreamPanel(
                    lines = ocrLines,
                    streaming = state == CaptureState.STREAMING,
                )

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

        // DANIEL-POLISH DB-005: Vollflächen-Weiß-Fade als Shutter-Beat.
        // 40ms FadeIn (während flashing=true), 110ms FadeOut (nach LaunchedEffect-Reset).
        val flashAlpha by animateFloatAsState(
            targetValue = if (flashing) 0.85f else 0f,
            animationSpec = tween(if (flashing) 40 else 110),
            label = "shutter-flash",
        )
        if (flashAlpha > 0.01f) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.White.copy(alpha = flashAlpha)),
            )
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
private fun OcrStreamPanel(
    lines: SnapshotStateList<String>,
    streaming: Boolean = false,
) {
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
            Row {
                Text(
                    "Warte auf Eingabe …",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    fontFamily = FontFamily.Monospace,
                    style = MaterialTheme.typography.bodySmall,
                )
                if (streaming) BlinkingCursor()
            }
        } else {
            LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
                items(lines.size) { idx ->
                    val isLast = idx == lines.size - 1
                    Row {
                        Text(
                            lines[idx],
                            fontFamily = FontFamily.Monospace,
                            style = MaterialTheme.typography.bodySmall,
                        )
                        if (isLast && streaming) BlinkingCursor()
                    }
                }
            }
        }
    }
}

// DANIEL-POLISH DB-003: blinkender ▌-Cursor in Akzentfarbe, alpha toggelt
// alle 500ms zwischen voll und unsichtbar. Nur instantiieren wenn aktiv —
// LaunchedEffect kommt dann automatisch in den Compose-Tree und wieder raus.
@Composable
private fun BlinkingCursor() {
    var visible by remember { mutableStateOf(true) }
    LaunchedEffect(Unit) {
        while (true) {
            delay(500)
            visible = !visible
        }
    }
    Text(
        "▌",
        color = Color(0xFFA78BFA),
        fontFamily = FontFamily.Monospace,
        style = MaterialTheme.typography.bodySmall,
        modifier = Modifier.alpha(if (visible) 1f else 0f),
    )
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

// DANIEL-POLISH DB-001: Overlay über dem Live-Preview mit 4 Ecken-Markern
// (L-Schenkel, abgerundete StrokeCap) und einer animierten Linie, die mit
// Gradient-Pinsel zwischen oberer und unterer Marker-Kante hoch-/runterwandert.
// Wird vom Aufrufer nur im IDLE-Zustand sichtbar gehalten.
@Composable
private fun PreviewScanOverlay(
    modifier: Modifier = Modifier,
    cornerLength: Dp = 18.dp,
    cornerThickness: Dp = 3.dp,
    cornerInset: Dp = 12.dp,
    accent: Color = Color(0xFFA78BFA),
) {
    val transition = rememberInfiniteTransition(label = "preview-scan")
    val progress by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 1500, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse,
        ),
        label = "scan-progress",
    )
    Canvas(modifier = modifier) {
        val len = cornerLength.toPx()
        val strokeW = cornerThickness.toPx()
        val inset = cornerInset.toPx()
        val w = size.width
        val h = size.height

        // 4 Ecken — je 2 Linien-Segmente (horizontal + vertikal).
        val corners = listOf(
            Offset(inset, inset) to listOf(
                Offset(inset + len, inset),
                Offset(inset, inset + len),
            ),
            Offset(w - inset, inset) to listOf(
                Offset(w - inset - len, inset),
                Offset(w - inset, inset + len),
            ),
            Offset(inset, h - inset) to listOf(
                Offset(inset + len, h - inset),
                Offset(inset, h - inset - len),
            ),
            Offset(w - inset, h - inset) to listOf(
                Offset(w - inset - len, h - inset),
                Offset(w - inset, h - inset - len),
            ),
        )
        corners.forEach { (origin, ends) ->
            ends.forEach { end ->
                drawLine(
                    color = accent,
                    start = origin,
                    end = end,
                    strokeWidth = strokeW,
                    cap = StrokeCap.Round,
                )
            }
        }

        // Animierte horizontale Linie: läuft zwischen y = inset und y = h - inset.
        val travelTop = inset
        val travelBottom = h - inset
        val y = travelTop + (travelBottom - travelTop) * progress
        val gradient = Brush.horizontalGradient(
            colorStops = arrayOf(
                0.0f to Color.Transparent,
                0.5f to accent.copy(alpha = 0.85f),
                1.0f to Color.Transparent,
            ),
            startX = inset,
            endX = w - inset,
        )
        drawLine(
            brush = gradient,
            start = Offset(inset, y),
            end = Offset(w - inset, y),
            strokeWidth = strokeW * 0.8f,
            cap = StrokeCap.Round,
        )
        // Sanfter Glow-Sekundärstrich (breiter, schwächere Opazität).
        drawLine(
            brush = Brush.horizontalGradient(
                colorStops = arrayOf(
                    0.0f to Color.Transparent,
                    0.5f to accent.copy(alpha = 0.25f),
                    1.0f to Color.Transparent,
                ),
                startX = inset,
                endX = w - inset,
            ),
            start = Offset(inset, y),
            end = Offset(w - inset, y),
            strokeWidth = strokeW * 3f,
            cap = StrokeCap.Round,
        )
    }
}

// DANIEL-POLISH DB-002: zentrierter Pill-Style-Chip mit Spinner während
// Streaming bzw. Check-Glyph nach Erfolg. Tonal-dunkler Hintergrund mit
// Akzent-Variante bei Success. Bei ERROR fallback auf Theme-Error-Container.
@Composable
private fun AnalyzeStatusChip(state: CaptureState, text: String) {
    val (bg, fg) = when (state) {
        CaptureState.STREAMING -> Color.Black.copy(alpha = 0.72f) to Color.White
        CaptureState.SUCCESS -> Color(0xFF22573B).copy(alpha = 0.92f) to Color(0xFFD8F5DD)
        CaptureState.ERROR -> MaterialTheme.colorScheme.errorContainer to MaterialTheme.colorScheme.onErrorContainer
        else -> MaterialTheme.colorScheme.surfaceVariant to MaterialTheme.colorScheme.onSurfaceVariant
    }
    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
        Surface(
            shape = RoundedCornerShape(50),
            color = bg,
            contentColor = fg,
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            ) {
                when (state) {
                    CaptureState.STREAMING -> {
                        CircularProgressIndicator(
                            modifier = Modifier.size(14.dp),
                            strokeWidth = 2.dp,
                            color = fg,
                        )
                    }
                    CaptureState.SUCCESS -> {
                        Icon(
                            Icons.Default.Check,
                            contentDescription = null,
                            modifier = Modifier.size(16.dp),
                            tint = fg,
                        )
                    }
                    else -> {}
                }
                Text(text, color = fg, style = MaterialTheme.typography.bodyMedium)
            }
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

